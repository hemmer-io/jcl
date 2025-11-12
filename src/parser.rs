//! Parser for JCL configuration files using Pest PEG parser
//!
//! This module implements the parser for JCL v1.0 specification.

use crate::ast::*;
use anyhow::{anyhow, Context, Result};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::path::Path;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct JCLParser;

/// Parse a JCL file from a path
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Module> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    parse_str(&content)
}

/// Parse JCL from a string
pub fn parse_str(input: &str) -> Result<Module> {
    let pairs = JCLParser::parse(Rule::program, input)
        .with_context(|| "Failed to parse JCL input")?;

    let mut statements = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
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
            _ => {}
        }
    }

    Ok(Module { statements })
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement> {
    let inner = pair.into_inner().next().ok_or_else(|| anyhow!("Empty statement"))?;

    match inner.as_rule() {
        Rule::assignment => parse_assignment(inner),
        Rule::function_def => parse_function_def(inner),
        Rule::import_stmt => parse_import(inner),
        Rule::for_loop => parse_for_loop(inner),
        Rule::expression => Ok(Statement::Expression(parse_expression(inner)?)),
        _ => Err(anyhow!("Unknown statement type: {:?}", inner.as_rule())),
    }
}

fn parse_assignment(pair: Pair<Rule>) -> Result<Statement> {
    let mut mutable = false;
    let mut name = String::new();
    let mut type_annotation = None;
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                if inner.as_str() == "mut" {
                    mutable = true;
                } else {
                    name = inner.as_str().to_string();
                }
            }
            Rule::type_annotation => {
                type_annotation = Some(parse_type_annotation(inner)?);
            }
            Rule::expression => {
                value = Some(parse_expression(inner)?);
            }
            _ => {}
        }
    }

    Ok(Statement::Assignment {
        name,
        mutable,
        value: value.ok_or_else(|| anyhow!("Missing value in assignment"))?,
        type_annotation,
    })
}

fn parse_function_def(pair: Pair<Rule>) -> Result<Statement> {
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
    })
}

fn parse_import(pair: Pair<Rule>) -> Result<Statement> {
    let mut items = Vec::new();
    let mut path = String::new();
    let mut wildcard = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::import_items => {
                for item in inner.into_inner() {
                    match item.as_rule() {
                        Rule::import_item => {
                            let id = item.into_inner().next()
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

    Ok(Statement::Import { items, path, wildcard })
}

fn parse_for_loop(pair: Pair<Rule>) -> Result<Statement> {
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
    let inner = pair.into_inner().next()
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
                        let inner_type = inner.next()
                            .ok_or_else(|| anyhow!("Missing list inner type"))?;
                        Ok(Type::List(Box::new(parse_type_expr(inner_type)?)))
                    }
                    "map" => {
                        let key_type = inner.next()
                            .ok_or_else(|| anyhow!("Missing map key type"))?;
                        let val_type = inner.next()
                            .ok_or_else(|| anyhow!("Missing map value type"))?;
                        Ok(Type::Map(
                            Box::new(parse_type_expr(key_type)?),
                            Box::new(parse_type_expr(val_type)?)
                        ))
                    }
                    _ => Err(anyhow!("Unknown type: {}", type_str))
                }
            } else {
                Err(anyhow!("Invalid type expression: {}", type_str))
            }
        }
    }
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expression> {
    let inner = pair.into_inner().next()
        .ok_or_else(|| anyhow!("Empty expression"))?;

    match inner.as_rule() {
        Rule::pipeline_expr => parse_pipeline_expr(inner),
        Rule::or_expr => parse_or_expr(inner),
        Rule::primary_expr => parse_primary_expr(inner),
        _ => parse_primary_expr(inner),
    }
}

fn parse_pipeline_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut stages = Vec::new();

    for inner in pair.into_inner() {
        stages.push(parse_or_expr(inner)?);
    }

    if stages.len() == 1 {
        Ok(stages.into_iter().next().unwrap())
    } else {
        Ok(Expression::Pipeline { stages })
    }
}

fn parse_or_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();
    let mut left = parse_and_expr(parts.next()
        .ok_or_else(|| anyhow!("Empty or expression"))?)?;

    for right_pair in parts {
        let right = parse_and_expr(right_pair)?;
        left = Expression::BinaryOp {
            op: BinaryOperator::Or,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_and_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();
    let mut left = parse_comparison_expr(parts.next()
        .ok_or_else(|| anyhow!("Empty and expression"))?)?;

    for right_pair in parts {
        let right = parse_comparison_expr(right_pair)?;
        left = Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_comparison_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();
    let mut left = parse_null_coalesce_expr(parts.next()
        .ok_or_else(|| anyhow!("Empty comparison"))?)?;

    while let Some(op_pair) = parts.next() {
        if op_pair.as_rule() == Rule::comparison_op {
            let op = match op_pair.as_str() {
                "==" => BinaryOperator::Equal,
                "!=" => BinaryOperator::NotEqual,
                "<" => BinaryOperator::LessThan,
                "<=" => BinaryOperator::LessThanOrEqual,
                ">" => BinaryOperator::GreaterThan,
                ">=" => BinaryOperator::GreaterThanOrEqual,
                _ => return Err(anyhow!("Unknown comparison operator")),
            };

            let right = parse_null_coalesce_expr(parts.next()
                .ok_or_else(|| anyhow!("Missing right operand"))?)?;

            left = Expression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    Ok(left)
}

fn parse_null_coalesce_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();
    let mut left = parse_additive_expr(parts.next()
        .ok_or_else(|| anyhow!("Empty null coalesce expression"))?)?;

    for right_pair in parts {
        let right = parse_additive_expr(right_pair)?;
        left = Expression::BinaryOp {
            op: BinaryOperator::NullCoalesce,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_additive_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner().peekable();
    let mut left = parse_multiplicative_expr(parts.next()
        .ok_or_else(|| anyhow!("Empty additive expression"))?)?;

    while let Some(next) = parts.peek() {
        let op = match next.as_str() {
            "+" => BinaryOperator::Add,
            "-" => BinaryOperator::Subtract,
            _ => break,
        };
        parts.next(); // consume operator

        let right = parse_multiplicative_expr(parts.next()
            .ok_or_else(|| anyhow!("Missing right operand"))?)?;

        left = Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_multiplicative_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner().peekable();
    let mut left = parse_unary_expr(parts.next()
        .ok_or_else(|| anyhow!("Empty multiplicative expression"))?)?;

    while let Some(next) = parts.peek() {
        let op = match next.as_str() {
            "*" => BinaryOperator::Multiply,
            "/" => BinaryOperator::Divide,
            "%" => BinaryOperator::Modulo,
            _ => break,
        };
        parts.next(); // consume operator

        let right = parse_unary_expr(parts.next()
            .ok_or_else(|| anyhow!("Missing right operand"))?)?;

        left = Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_unary_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();
    let first = parts.next()
        .ok_or_else(|| anyhow!("Empty unary expression"))?;

    match first.as_str() {
        "not" | "-" => {
            let op = if first.as_str() == "not" {
                UnaryOperator::Not
            } else {
                UnaryOperator::Negate
            };

            let operand = parse_unary_expr(parts.next()
                .ok_or_else(|| anyhow!("Missing operand"))?)?;

            Ok(Expression::UnaryOp {
                op,
                operand: Box::new(operand),
            })
        }
        _ => parse_postfix_expr(first),
    }
}

fn parse_postfix_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();
    let mut expr = parse_primary_expr(parts.next()
        .ok_or_else(|| anyhow!("Empty postfix expression"))?)?;

    for postfix_op in parts {
        expr = match postfix_op.as_rule() {
            Rule::member_access => {
                let field = postfix_op.into_inner().next()
                    .ok_or_else(|| anyhow!("Missing field name"))?
                    .as_str()
                    .to_string();
                Expression::MemberAccess {
                    object: Box::new(expr),
                    field,
                }
            }
            Rule::optional_chain => {
                let field = postfix_op.into_inner().next()
                    .ok_or_else(|| anyhow!("Missing field name"))?
                    .as_str()
                    .to_string();
                Expression::OptionalChain {
                    object: Box::new(expr),
                    field,
                }
            }
            Rule::index_access => {
                let index = parse_expression(postfix_op.into_inner().next()
                    .ok_or_else(|| anyhow!("Missing index"))?)?;
                Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                }
            }
            Rule::function_call_op => {
                // This handles method calls like obj.method(args)
                // For now, convert to function call
                if let Expression::MemberAccess { object, field } = expr {
                    let args = parse_argument_list(postfix_op)?;
                    Expression::MethodCall {
                        object,
                        method: field,
                        args,
                    }
                } else {
                    return Err(anyhow!("Invalid function call"));
                }
            }
            _ => return Err(anyhow!("Unknown postfix operator")),
        };
    }

    Ok(expr)
}

fn parse_primary_expr(pair: Pair<Rule>) -> Result<Expression> {
    let inner = pair.into_inner().next()
        .ok_or_else(|| anyhow!("Empty primary expression"))?;

    match inner.as_rule() {
        Rule::number => parse_number(inner),
        Rule::boolean => Ok(Expression::Literal(Value::Bool(inner.as_str() == "true"))),
        Rule::null => Ok(Expression::Literal(Value::Null)),
        Rule::quoted_string | Rule::multiline_string => {
            Ok(Expression::Literal(Value::String(parse_string_literal(inner)?)))
        }
        Rule::interpolated_string => parse_interpolated_string(inner),
        Rule::identifier => Ok(Expression::Variable(inner.as_str().to_string())),
        Rule::list => parse_list(inner),
        Rule::map => parse_map(inner),
        Rule::function_call => parse_function_call(inner),
        Rule::lambda => parse_lambda(inner),
        Rule::if_expr => parse_if_expr(inner),
        Rule::when_expr => parse_when_expr(inner),
        Rule::ternary => parse_ternary(inner),
        Rule::list_comprehension => parse_list_comprehension(inner),
        Rule::expression => parse_expression(inner),
        _ => Err(anyhow!("Unknown primary expression: {:?}", inner.as_rule())),
    }
}

fn parse_number(pair: Pair<Rule>) -> Result<Expression> {
    let s = pair.as_str();
    if s.contains('.') {
        Ok(Expression::Literal(Value::Float(s.parse()?)))
    } else {
        Ok(Expression::Literal(Value::Int(s.parse()?)))
    }
}

fn parse_string_literal(pair: Pair<Rule>) -> Result<String> {
    let s = pair.as_str();

    // Remove quotes
    let s = if s.starts_with("\"\"\"") && s.ends_with("\"\"\"") {
        &s[3..s.len()-3]
    } else if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len()-1]
    } else {
        s
    };

    // Handle escape sequences
    let s = s.replace("\\n", "\n")
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
                let expr = parse_expression(inner.into_inner().next()
                    .ok_or_else(|| anyhow!("Empty interpolation"))?)?;
                parts.push(StringPart::Interpolation(Box::new(expr)));
            }
            _ => {}
        }
    }

    Ok(Expression::InterpolatedString { parts })
}

fn parse_list(pair: Pair<Rule>) -> Result<Expression> {
    let mut items = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            items.push(parse_expression(inner)?);
        }
    }

    Ok(Expression::List(items))
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

    Ok(Expression::Map(entries))
}

fn parse_function_call(pair: Pair<Rule>) -> Result<Expression> {
    let mut name = String::new();
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                name = inner.as_str().to_string();
            }
            Rule::argument_list => {
                args = parse_argument_list(inner)?;
            }
            _ => {}
        }
    }

    Ok(Expression::FunctionCall { name, args })
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
    })
}

fn parse_if_expr(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();

    let condition = parse_expression(parts.next()
        .ok_or_else(|| anyhow!("Missing if condition"))?)?;
    let then_expr = parse_expression(parts.next()
        .ok_or_else(|| anyhow!("Missing then expression"))?)?;
    let else_expr = parts.next().map(parse_expression).transpose()?;

    Ok(Expression::If {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: else_expr.map(Box::new),
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
    })
}

fn parse_when_arm(pair: Pair<Rule>) -> Result<WhenArm> {
    let mut pattern = None;
    let mut expr = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::when_pattern => {
                pattern = Some(parse_when_pattern(inner)?);
            }
            Rule::expression => {
                expr = Some(parse_expression(inner)?);
            }
            _ => {}
        }
    }

    Ok(WhenArm {
        pattern: pattern.ok_or_else(|| anyhow!("Missing pattern"))?,
        guard: None,
        expr: expr.ok_or_else(|| anyhow!("Missing expression"))?,
    })
}

fn parse_when_pattern(pair: Pair<Rule>) -> Result<Pattern> {
    let s = pair.as_str();

    if s == "_" || s == "*" {
        return Ok(Pattern::Wildcard);
    }

    // For now, just parse as literal expression
    // TODO: Implement proper pattern matching
    let inner = pair.into_inner().next()
        .ok_or_else(|| anyhow!("Empty pattern"))?;

    match parse_expression(inner)? {
        Expression::Literal(val) => Ok(Pattern::Literal(val)),
        Expression::Variable(name) => Ok(Pattern::Variable(name)),
        _ => Err(anyhow!("Invalid pattern")),
    }
}

fn parse_ternary(pair: Pair<Rule>) -> Result<Expression> {
    let mut parts = pair.into_inner();

    let condition = parse_expression(parts.next()
        .ok_or_else(|| anyhow!("Missing condition"))?)?;
    let then_expr = parse_expression(parts.next()
        .ok_or_else(|| anyhow!("Missing then expression"))?)?;
    let else_expr = parse_expression(parts.next()
        .ok_or_else(|| anyhow!("Missing else expression"))?)?;

    Ok(Expression::Ternary {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
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
                } else {
                    iterable = Some(parse_expression(inner)?);
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
                let filter_expr = inner.into_inner().next()
                    .ok_or_else(|| anyhow!("Empty filter"))?;
                condition = Some(parse_expression(filter_expr)?);
            }
            _ => {}
        }
    }

    Ok(Expression::ListComprehension {
        expr: Box::new(expr.ok_or_else(|| anyhow!("Missing comprehension expression"))?),
        variable,
        iterable: Box::new(iterable.ok_or_else(|| anyhow!("Missing iterable"))?),
        condition: condition.map(Box::new),
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
    fn test_parse_function_call() {
        let result = parse_str("result = upper(\"hello\")");
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
    fn test_parse_list_comprehension() {
        let result = parse_str("evens = [x for x in numbers if x % 2 == 0]");
        assert!(result.is_ok());
    }
}
