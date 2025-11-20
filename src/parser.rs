//! Parser for JCL configuration files using Pest PEG parser with Pratt parsing
//!
//! This module implements the parser for JCL v1.0 specification using Pratt parsing
//! for expressions to handle operator precedence cleanly.

use crate::ast::*;
use crate::error;
use anyhow::{anyhow, Context, Result};
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use pest_derive::Parser;
use std::path::Path;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct JCLParser;

// Pratt parser for handling operator precedence
lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;

        PrattParser::new()
            // Lowest precedence - ternary
            .op(Op::infix(ternary_op, Assoc::Right))
            // Pipeline
            .op(Op::infix(pipe_op, Assoc::Left))
            // Logical OR
            .op(Op::infix(or_op, Assoc::Left))
            // Logical AND
            .op(Op::infix(and_op, Assoc::Left))
            // Comparison
            .op(Op::infix(eq_op, Assoc::Left)
                | Op::infix(ne_op, Assoc::Left)
                | Op::infix(le_op, Assoc::Left)
                | Op::infix(ge_op, Assoc::Left)
                | Op::infix(lt_op, Assoc::Left)
                | Op::infix(gt_op, Assoc::Left))
            // Null coalescing
            .op(Op::infix(null_coalesce_op, Assoc::Right))
            // Addition and subtraction
            .op(Op::infix(add_op, Assoc::Left) | Op::infix(sub_op, Assoc::Left))
            // Multiplication, division, modulo
            .op(Op::infix(mul_op, Assoc::Left) | Op::infix(div_op, Assoc::Left) | Op::infix(mod_op, Assoc::Left))
            // Prefix operators
            .op(Op::prefix(neg) | Op::prefix(not))
            // Postfix operators (highest precedence - member access, calls, indexing)
            .op(Op::postfix(call_args) | Op::postfix(member_access) | Op::postfix(index_access) | Op::postfix(optional_chain))
    };
}

/// Parse a JCL file from a path
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Module> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    parse_str(&content)
}

/// Parse JCL from a string
pub fn parse_str(input: &str) -> Result<Module> {
    let pairs = match JCLParser::parse(Rule::program, input) {
        Ok(pairs) => pairs,
        Err(pest_error) => {
            // Use our improved error formatting
            return Err(error::create_parse_error(pest_error, input));
        }
    };

    let mut statements = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::statement => {
                        statements.push(parse_statement(inner_pair)?);
                    }
                    Rule::EOI => break,
                    _ => {}
                }
            }
        }
    }

    Ok(Module { statements })
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement> {
    let mut doc_comments = None;
    let mut stmt_pair = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::doccomments => {
                let comments: Vec<String> = inner
                    .into_inner()
                    .map(|dc| {
                        let text = dc.as_str();
                        // Remove "///" prefix and trim
                        text.strip_prefix("///").unwrap_or(text).trim().to_string()
                    })
                    .collect();
                doc_comments = Some(comments);
            }
            Rule::stmtbody => {
                stmt_pair = Some(inner);
            }
            _ => {}
        }
    }

    let stmt_inner = stmt_pair.ok_or_else(|| anyhow!("Empty statement"))?;

    // Get the actual statement from stmtbody
    let inner = stmt_inner
        .into_inner()
        .next()
        .ok_or_else(|| anyhow!("Empty stmtbody"))?;

    match inner.as_rule() {
        Rule::assignment => parse_assignment(inner, doc_comments),
        Rule::function_def => parse_function_def(inner, doc_comments),
        Rule::import_stmt => parse_import(inner, doc_comments),
        Rule::for_loop => parse_for_loop(inner, doc_comments),
        Rule::expression => Ok(Statement::Expression {
            expr: parse_expression(inner)?,
            span: None,
        }),
        _ => Err(anyhow!("Unknown statement type: {:?}", inner.as_rule())),
    }
}

fn parse_assignment(pair: Pair<Rule>, doc_comments: Option<Vec<String>>) -> Result<Statement> {
    let mut mutable = false;
    let mut name = String::new();
    let mut type_annotation = None;
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                if name.is_empty() {
                    name = inner.as_str().to_string();
                }
            }
            Rule::type_annotation => {
                type_annotation = Some(parse_type_annotation(inner)?);
            }
            Rule::expression => {
                value = Some(parse_expression(inner)?);
            }
            _ => {
                if inner.as_str() == "mut" {
                    mutable = true;
                }
            }
        }
    }

    Ok(Statement::Assignment {
        name,
        mutable,
        value: value.ok_or_else(|| anyhow!("Missing value in assignment"))?,
        type_annotation,
        doc_comments,
        span: None,
    })
}

fn parse_function_def(pair: Pair<Rule>, doc_comments: Option<Vec<String>>) -> Result<Statement> {
    let mut name = String::new();
    let mut params = Vec::new();
    let mut return_type = None;
    let mut body = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                name = inner.as_str().to_string();
            }
            Rule::param_list => {
                params = parse_param_list(inner)?;
            }
            Rule::type_annotation => {
                return_type = Some(parse_type_annotation(inner)?);
            }
            Rule::expression => {
                body = Some(parse_expression(inner)?);
            }
            _ => {}
        }
    }

    Ok(Statement::FunctionDef {
        name,
        params,
        return_type,
        body: body.ok_or_else(|| anyhow!("Missing body in function definition"))?,
        doc_comments,
        span: None,
    })
}

fn parse_import(pair: Pair<Rule>, doc_comments: Option<Vec<String>>) -> Result<Statement> {
    let mut items = Vec::new();
    let mut path = String::new();
    let mut wildcard = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::import_items => {
                for item in inner.into_inner() {
                    match item.as_rule() {
                        Rule::import_item => {
                            let id = item
                                .into_inner()
                                .next()
                                .ok_or_else(|| anyhow!("Missing import item"))?;
                            items.push(id.as_str().to_string());
                        }
                        _ if item.as_str() == "*" => {
                            wildcard = true;
                        }
                        _ => {}
                    }
                }
            }
            Rule::string => {
                path = parse_string_literal(inner)?;
            }
            _ => {}
        }
    }

    Ok(Statement::Import {
        items,
        path,
        wildcard,
        doc_comments,
        span: None,
    })
}

fn parse_for_loop(pair: Pair<Rule>, doc_comments: Option<Vec<String>>) -> Result<Statement> {
    let mut variables = Vec::new();
    let mut iterables = Vec::new();
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::for_variables => {
                for var in inner.into_inner() {
                    variables.push(var.as_str().to_string());
                }
            }
            Rule::for_iterables => {
                for iter in inner.into_inner() {
                    iterables.push(parse_expression(iter)?);
                }
            }
            Rule::statement => {
                body.push(parse_statement(inner)?);
            }
            _ => {}
        }
    }

    Ok(Statement::ForLoop {
        variables,
        iterables,
        body,
        condition: None,
        doc_comments,
        span: None,
    })
}

fn parse_param_list(pair: Pair<Rule>) -> Result<Vec<Parameter>> {
    let mut params = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::param {
            let mut name = String::new();
            let mut param_type = None;

            for param_inner in inner.into_inner() {
                match param_inner.as_rule() {
                    Rule::identifier => {
                        name = param_inner.as_str().to_string();
                    }
                    Rule::type_annotation => {
                        param_type = Some(parse_type_annotation(param_inner)?);
                    }
                    _ => {}
                }
            }

            params.push(Parameter {
                name,
                param_type,
                default: None,
            });
        }
    }

    Ok(params)
}

fn parse_type_annotation(pair: Pair<Rule>) -> Result<Type> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| anyhow!("Empty type annotation"))?;
    parse_type_expr(inner)
}

fn parse_type_expr(pair: Pair<Rule>) -> Result<Type> {
    let type_str = pair.as_str();

    match type_str {
        "string" => Ok(Type::String),
        "int" => Ok(Type::Int),
        "float" => Ok(Type::Float),
        "bool" => Ok(Type::Bool),
        "any" => Ok(Type::Any),
        _ => {
            // Complex types like list<T> or map<K,V>
            let mut inner = pair.into_inner();
            if let Some(first) = inner.next() {
                match first.as_str() {
                    "list" => {
                        let inner_type = inner
                            .next()
                            .ok_or_else(|| anyhow!("Missing list inner type"))?;
                        Ok(Type::List(Box::new(parse_type_expr(inner_type)?)))
                    }
                    "map" => {
                        let key_type = inner
                            .next()
                            .ok_or_else(|| anyhow!("Missing map key type"))?;
                        let val_type = inner
                            .next()
                            .ok_or_else(|| anyhow!("Missing map value type"))?;
                        Ok(Type::Map(
                            Box::new(parse_type_expr(key_type)?),
                            Box::new(parse_type_expr(val_type)?),
                        ))
                    }
                    _ => Err(anyhow!("Unknown type: {}", type_str)),
                }
            } else {
                Err(anyhow!("Invalid type expression: {}", type_str))
            }
        }
    }
}

// Parse expression using Pratt parser
fn parse_expression(pair: Pair<Rule>) -> Result<Expression> {
    PRATT_PARSER
        .map_primary(|primary| parse_primary(primary))
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg => Ok(Expression::UnaryOp {
                op: UnaryOperator::Negate,
                operand: Box::new(rhs?),
                span: None,
            }),
            Rule::not => Ok(Expression::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(rhs?),
                span: None,
            }),
            _ => Err(anyhow!("Unknown prefix operator: {:?}", op.as_rule())),
        })
        .map_postfix(|lhs, op| {
            let lhs = lhs?;

            match op.as_rule() {
                Rule::optional_chain => {
                    let field = op
                        .into_inner()
                        .next()
                        .ok_or_else(|| anyhow!("Missing field in optional chain"))?
                        .as_str()
                        .to_string();
                    Ok(Expression::OptionalChain {
                        object: Box::new(lhs),
                        field,
                        span: None,
                    })
                }
                Rule::member_access => {
                    let field = op
                        .into_inner()
                        .next()
                        .ok_or_else(|| anyhow!("Missing field in member access"))?
                        .as_str()
                        .to_string();
                    Ok(Expression::MemberAccess {
                        object: Box::new(lhs),
                        field,
                        span: None,
                    })
                }
                Rule::index_access => {
                    let index = op
                        .into_inner()
                        .next()
                        .ok_or_else(|| anyhow!("Missing index"))?;
                    Ok(Expression::Index {
                        object: Box::new(lhs),
                        index: Box::new(parse_expression(index)?),
                        span: None,
                    })
                }
                Rule::call_args => {
                    let args = if let Some(arg_list) = op.into_inner().next() {
                        parse_argument_list(arg_list)?
                    } else {
                        Vec::new()
                    };

                    // Distinguish between method calls and function calls
                    if let Expression::MemberAccess { object, field, .. } = lhs {
                        Ok(Expression::MethodCall {
                            object,
                            method: field,
                            args,
                            span: None,
                        })
                    } else if let Expression::Variable { name, .. } = lhs {
                        Ok(Expression::FunctionCall {
                            name,
                            args,
                            span: None,
                        })
                    } else {
                        Err(anyhow!("Invalid function call target"))
                    }
                }
                _ => Err(anyhow!("Unknown postfix operator: {:?}", op.as_rule())),
            }
        })
        .map_infix(|lhs, op, rhs| {
            let lhs = lhs?;

            let binary_op = match op.as_rule() {
                Rule::add_op => BinaryOperator::Add,
                Rule::sub_op => BinaryOperator::Subtract,
                Rule::mul_op => BinaryOperator::Multiply,
                Rule::div_op => BinaryOperator::Divide,
                Rule::mod_op => BinaryOperator::Modulo,
                Rule::eq_op => BinaryOperator::Equal,
                Rule::ne_op => BinaryOperator::NotEqual,
                Rule::lt_op => BinaryOperator::LessThan,
                Rule::le_op => BinaryOperator::LessThanOrEqual,
                Rule::gt_op => BinaryOperator::GreaterThan,
                Rule::ge_op => BinaryOperator::GreaterThanOrEqual,
                Rule::and_op => BinaryOperator::And,
                Rule::or_op => BinaryOperator::Or,
                Rule::null_coalesce_op => BinaryOperator::NullCoalesce,
                Rule::pipe_op => {
                    // Pipeline operator creates a Pipeline expression
                    // If lhs is already a pipeline, extend it; otherwise create a new one
                    let mut stages = if let Expression::Pipeline {
                        stages: existing, ..
                    } = lhs
                    {
                        existing
                    } else {
                        vec![lhs]
                    };
                    stages.push(rhs?);
                    return Ok(Expression::Pipeline { stages, span: None });
                }
                Rule::ternary_op => {
                    // Ternary: condition ? then_expr : else_expr
                    // op contains "? then_expr :"
                    let mut inner = op.into_inner();
                    let then_expr = inner
                        .next()
                        .ok_or_else(|| anyhow!("Missing then expression in ternary"))?;
                    return Ok(Expression::Ternary {
                        condition: Box::new(lhs),
                        then_expr: Box::new(parse_expression(then_expr)?),
                        else_expr: Box::new(rhs?),
                        span: None,
                    });
                }
                _ => return Err(anyhow!("Unknown infix operator: {:?}", op.as_rule())),
            };

            Ok(Expression::BinaryOp {
                op: binary_op,
                left: Box::new(lhs),
                right: Box::new(rhs?),
                span: None,
            })
        })
        .parse(pair.into_inner())
}

fn parse_primary(pair: Pair<Rule>) -> Result<Expression> {
    match pair.as_rule() {
        Rule::primary => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| anyhow!("Empty primary expression"))?;
            parse_primary(inner)
        }
        Rule::number => parse_number(pair),
        Rule::boolean => Ok(Expression::Literal {
            value: Value::Bool(pair.as_str() == "true"),
            span: None,
        }),
        Rule::null => Ok(Expression::Literal {
            value: Value::Null,
            span: None,
        }),
        Rule::quoted_string | Rule::multiline_string => Ok(Expression::Literal {
            value: Value::String(parse_string_literal(pair)?),
            span: None,
        }),
        Rule::interpolated_string => parse_interpolated_string(pair),
        Rule::identifier => Ok(Expression::Variable {
            name: pair.as_str().to_string(),
            span: None,
        }),
        Rule::list => parse_list(pair),
        Rule::map => parse_map(pair),
        Rule::lambda => parse_lambda(pair),
        Rule::if_expr => parse_if_expr(pair),
        Rule::when_expr => parse_when_expr(pair),
        Rule::list_comprehension => parse_list_comprehension(pair),
        Rule::try_expr => parse_try_expr(pair),
        Rule::expression => parse_expression(pair),
        _ => Err(anyhow!("Unknown primary expression: {:?}", pair.as_rule())),
    }
}

fn parse_number(pair: Pair<Rule>) -> Result<Expression> {
    let s = pair.as_str();
    if s.contains('.') {
        Ok(Expression::Literal {
            value: Value::Float(s.parse()?),
            span: None,
        })
    } else {
        Ok(Expression::Literal {
            value: Value::Int(s.parse()?),
            span: None,
        })
    }
}

fn parse_string_literal(pair: Pair<Rule>) -> Result<String> {
    let s = pair.as_str();

    // Remove quotes
    let s = if s.starts_with("\"\"\"") && s.ends_with("\"\"\"") {
        &s[3..s.len() - 3]
    } else if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    };

    // Handle escape sequences
    let s = s
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\");

    Ok(s)
}

fn parse_interpolated_string(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string_literal_part => {
                parts.push(StringPart::Literal(inner.as_str().to_string()));
            }
            Rule::interpolation => {
                let expr = parse_expression(
                    inner
                        .into_inner()
                        .next()
                        .ok_or_else(|| anyhow!("Empty interpolation"))?,
                )?;
                parts.push(StringPart::Interpolation(Box::new(expr)));
            }
            _ => {}
        }
    }

    Ok(Expression::InterpolatedString { parts, span: None })
}

fn parse_list(pair: Pair<Rule>) -> Result<Expression> {
    let mut items = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            items.push(parse_expression(inner)?);
        }
    }

    Ok(Expression::List {
        elements: items,
        span: None,
    })
}

fn parse_map(pair: Pair<Rule>) -> Result<Expression> {
    let mut entries = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::map_entry {
            let mut key = String::new();
            let mut value = None;

            for entry_part in inner.into_inner() {
                match entry_part.as_rule() {
                    Rule::identifier => {
                        key = entry_part.as_str().to_string();
                    }
                    Rule::expression => {
                        value = Some(parse_expression(entry_part)?);
                    }
                    _ => {}
                }
            }

            if let Some(val) = value {
                entries.push((key, val));
            }
        }
    }

    Ok(Expression::Map {
        entries,
        span: None,
    })
}

fn parse_argument_list(pair: Pair<Rule>) -> Result<Vec<Expression>> {
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            args.push(parse_expression(inner)?);
        }
    }

    Ok(args)
}

fn parse_lambda(pair: Pair<Rule>) -> Result<Expression> {
    let mut params = Vec::new();
    let mut body = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::lambda_params => {
                for param in inner.into_inner() {
                    if param.as_rule() == Rule::identifier {
                        params.push(Parameter {
                            name: param.as_str().to_string(),
                            param_type: None,
                            default: None,
                        });
                    }
                }
            }
            Rule::expression => {
                body = Some(parse_expression(inner)?);
            }
            _ => {}
        }
    }

    Ok(Expression::Lambda {
        params,
        body: Box::new(body.ok_or_else(|| anyhow!("Missing lambda body"))?),
        span: None,
    })
}

fn parse_if_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();

    let condition = parse_expression(
        parts
            .next()
            .ok_or_else(|| anyhow!("Missing if condition"))?,
    )?;
    let then_expr = parse_expression(
        parts
            .next()
            .ok_or_else(|| anyhow!("Missing then expression"))?,
    )?;
    let else_expr = parts.next().map(parse_expression).transpose()?;

    Ok(Expression::If {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: else_expr.map(Box::new),
        span: None,
    })
}

fn parse_when_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut value = None;
    let mut arms = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => {
                if value.is_none() {
                    value = Some(parse_expression(inner)?);
                }
            }
            Rule::when_arm => {
                arms.push(parse_when_arm(inner)?);
            }
            _ => {}
        }
    }

    Ok(Expression::When {
        value: Box::new(value.ok_or_else(|| anyhow!("Missing when value"))?),
        arms,
        span: None,
    })
}

fn parse_when_arm(pair: Pair<Rule>) -> Result<WhenArm> {
    let mut pattern = None;
    let mut guard = None;
    let mut expr = None;

    let mut inner = pair.into_inner();

    // First is always the pattern
    if let Some(pattern_pair) = inner.next() {
        pattern = Some(parse_when_pattern(pattern_pair)?);
    }

    // Check remaining pairs for guard and expression
    for remaining in inner {
        if remaining.as_rule() == Rule::expression {
            if guard.is_none() && expr.is_none() {
                // This could be a guard condition or the result expression
                // We need to look ahead - for now, assume it's the result
                expr = Some(parse_expression(remaining)?);
            } else if guard.is_some() {
                // We have a guard, so this must be the expression
                expr = Some(parse_expression(remaining)?);
            } else {
                // First expression after pattern - could be guard
                // Check if there's another expression coming
                guard = Some(parse_expression(remaining)?);
            }
        }
    }

    Ok(WhenArm {
        pattern: pattern.ok_or_else(|| anyhow!("Missing pattern"))?,
        guard: guard.or(None),
        expr: expr.ok_or_else(|| anyhow!("Missing expression"))?,
    })
}

fn parse_when_pattern(pair: Pair<Rule>) -> Result<Pattern> {
    let s = pair.as_str();

    if s == "_" || s == "*" {
        return Ok(Pattern::Wildcard);
    }

    // Check inner rules
    let inner = pair.into_inner().next();

    if let Some(inner_pair) = inner {
        match inner_pair.as_rule() {
            Rule::literal_value => {
                // Parse as literal
                let lit_inner = inner_pair
                    .into_inner()
                    .next()
                    .ok_or_else(|| anyhow!("Empty literal"))?;
                match parse_primary(lit_inner)? {
                    Expression::Literal { value, .. } => Ok(Pattern::Literal(value)),
                    _ => Err(anyhow!("Invalid literal pattern")),
                }
            }
            Rule::identifier => Ok(Pattern::Variable(inner_pair.as_str().to_string())),
            Rule::expression => {
                // Tuple pattern
                let exprs: Result<Vec<_>> = inner_pair
                    .into_inner()
                    .map(|e| match parse_expression(e)? {
                        Expression::Literal { value, .. } => Ok(Pattern::Literal(value)),
                        Expression::Variable { name, .. } => Ok(Pattern::Variable(name)),
                        _ => Err(anyhow!("Invalid pattern expression")),
                    })
                    .collect();
                Ok(Pattern::Tuple(exprs?))
            }
            _ => Err(anyhow!("Invalid pattern")),
        }
    } else {
        Ok(Pattern::Wildcard)
    }
}

fn parse_try_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut expr = None;
    let mut default = None;

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            if expr.is_none() {
                expr = Some(parse_expression(inner)?);
            } else {
                default = Some(parse_expression(inner)?);
            }
        }
    }

    Ok(Expression::Try {
        expr: Box::new(expr.ok_or_else(|| anyhow!("Missing try expression"))?),
        default: default.map(Box::new),
        span: None,
    })
}

fn parse_list_comprehension(pair: Pair<Rule>) -> Result<Expression> {
    let mut expr = None;
    let mut variable = String::new();
    let mut iterable = None;
    let mut condition = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => {
                if expr.is_none() {
                    expr = Some(parse_expression(inner)?);
                }
            }
            Rule::comprehension_clause => {
                for clause_part in inner.into_inner() {
                    match clause_part.as_rule() {
                        Rule::identifier => {
                            variable = clause_part.as_str().to_string();
                        }
                        Rule::expression => {
                            iterable = Some(parse_expression(clause_part)?);
                        }
                        _ => {}
                    }
                }
            }
            Rule::comprehension_filter => {
                let filter_expr = inner
                    .into_inner()
                    .next()
                    .ok_or_else(|| anyhow!("Empty filter"))?;
                condition = Some(parse_expression(filter_expr)?);
            }
            _ => {}
        }
    }

    Ok(Expression::ListComprehension {
        expr: Box::new(expr.ok_or_else(|| anyhow!("Missing comprehension expression"))?),
        iterators: vec![(variable, iterable.ok_or_else(|| anyhow!("Missing iterable"))?)],
        condition: condition.map(Box::new),
        span: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse_str("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().statements.len(), 0);
    }

    #[test]
    fn test_parse_assignment() {
        let result = parse_str("x = 42");
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);
    }

    #[test]
    fn test_parse_list() {
        let result = parse_str("x = [1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_map() {
        let result = parse_str("config = (host = localhost, port = 8080)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_arithmetic() {
        let result = parse_str("x = 1 + 2 * 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda() {
        let result = parse_str("double = x => x * 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_expr() {
        let result = parse_str("x = if y > 0 then \"positive\" else \"negative\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pipeline() {
        let result = parse_str("x = data | filter | map");
        assert!(result.is_ok());
    }
}
