//! Token-based parser for JCL
//!
//! This module implements a recursive descent parser that consumes
//! tokens from the lexer to build an AST. This approach correctly
//! handles keyword/identifier distinction.

use crate::ast::{
    BinaryOperator, Expression, ImportItem, Module, Parameter, Pattern, SourceSpan, Statement,
    StringPart, Type, UnaryOperator, Value, WhenArm,
};
use crate::lexer::{StringValue, Token, TokenKind};
use anyhow::{anyhow, Result};

/// Parser that consumes tokens to produce an AST
pub struct TokenParser {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenParser {
    /// Create a new parser from a token stream
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Parse a complete module
    pub fn parse_module(&mut self) -> Result<Module> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Collect doc comments
            let mut doc_comments = Vec::new();
            while self.check(&TokenKind::DocComment(String::new())) {
                if let TokenKind::DocComment(text) = &self.current().kind {
                    doc_comments.push(text.clone());
                }
                self.advance();
            }

            if self.is_at_end() {
                break;
            }

            let stmt = self.parse_statement(doc_comments)?;
            statements.push(stmt);
        }

        Ok(Module { statements })
    }

    /// Parse a single statement
    fn parse_statement(&mut self, doc_comments: Vec<String>) -> Result<Statement> {
        let start = self.mark_position();

        // Check for import
        if self.check(&TokenKind::Import) {
            return self.parse_import();
        }

        // Check for function definition
        if self.check(&TokenKind::Fn) {
            return self.parse_function_def(doc_comments);
        }

        // Check for for loop
        if self.check(&TokenKind::For) {
            return self.parse_for_loop();
        }

        // Check for mutable assignment
        if self.check(&TokenKind::Mut) {
            return self.parse_assignment(true, doc_comments);
        }

        // Check for module patterns (module.interface, module.outputs, module.<type>.<instance>)
        if self.check_identifier() {
            // Peek at the current identifier without consuming
            if let TokenKind::Identifier(name) = &self.current().kind {
                if name == "module" {
                    let next_pos = self.position + 1;
                    if next_pos < self.tokens.len() {
                        if matches!(self.tokens[next_pos].kind, TokenKind::Dot) {
                            return self.parse_assignment(false, doc_comments);
                        }
                    }
                }
            }
        }

        // Check for assignment (identifier followed by = or :)
        if self.check_identifier() {
            let next_pos = self.position + 1;
            if next_pos < self.tokens.len() {
                let next = &self.tokens[next_pos].kind;
                if matches!(next, TokenKind::Equal | TokenKind::Colon) {
                    return self.parse_assignment(false, doc_comments);
                }
            }
        }

        // Otherwise it's an expression statement
        let expr = self.parse_expression()?;
        Ok(Statement::Expression {
            expr,
            span: self.span_from(start),
        })
    }

    /// Parse an import statement
    /// Supports two patterns:
    /// 1. Path-based: `import "path"` or `import "path" as alias`
    /// 2. Selective: `import (items) from "path"` or `import * from "path"`
    fn parse_import(&mut self) -> Result<Statement> {
        use crate::ast::ImportKind;

        let start = self.mark_position();
        self.expect(&TokenKind::Import)?;

        // Check if this is a path-based import (starts with string)
        if self.check_string() {
            // Pattern 1: import "path" [as alias]
            let path = self.parse_string_literal()?;
            let alias = if self.check(&TokenKind::As) {
                self.advance();
                Some(self.parse_identifier()?)
            } else {
                None
            };

            return Ok(Statement::Import {
                path,
                kind: ImportKind::Full { alias },
                doc_comments: None,
                span: self.span_from(start),
            });
        }

        // Pattern 2: Selective import (existing syntax)
        let items = self.parse_import_items()?;
        self.expect(&TokenKind::From)?;
        let path = self.parse_string_literal()?;

        let kind = if items.len() == 1 && items[0].name == "*" {
            ImportKind::Wildcard
        } else {
            ImportKind::Selective { items }
        };

        Ok(Statement::Import {
            path,
            kind,
            doc_comments: None,
            span: self.span_from(start),
        })
    }

    /// Parse import items with optional aliases
    fn parse_import_items(&mut self) -> Result<Vec<ImportItem>> {
        use crate::ast::ImportItem;

        if self.check(&TokenKind::Star) {
            self.advance();
            return Ok(vec![ImportItem {
                name: "*".to_string(),
                alias: None,
            }]);
        }

        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let mut items = vec![self.parse_import_item()?];
            while self.check(&TokenKind::Comma) {
                self.advance();
                items.push(self.parse_import_item()?);
            }
            self.expect(&TokenKind::RightParen)?;
            return Ok(items);
        }

        Ok(vec![self.parse_import_item()?])
    }

    /// Parse a single import item with optional alias
    fn parse_import_item(&mut self) -> Result<ImportItem> {
        use crate::ast::ImportItem;

        let name = self.parse_identifier()?;
        let alias = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.parse_identifier()?)
        } else {
            None
        };

        Ok(ImportItem { name, alias })
    }

    /// Check if current token is a string literal
    fn check_string(&self) -> bool {
        if self.position < self.tokens.len() {
            matches!(self.tokens[self.position].kind, TokenKind::String(_))
        } else {
            false
        }
    }

    /// Parse a function definition
    fn parse_function_def(&mut self, doc_comments: Vec<String>) -> Result<Statement> {
        let start = self.mark_position();

        self.expect(&TokenKind::Fn)?;
        let name = self.parse_identifier()?;

        self.expect(&TokenKind::LeftParen)?;
        let params = if !self.check(&TokenKind::RightParen) {
            self.parse_param_list()?
        } else {
            Vec::new()
        };
        self.expect(&TokenKind::RightParen)?;

        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Equal)?;
        let body = self.parse_expression()?;

        let doc = if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments)
        };

        Ok(Statement::FunctionDef {
            name,
            params,
            return_type,
            body,
            doc_comments: doc,
            span: self.span_from(start),
        })
    }

    /// Parse parameter list
    fn parse_param_list(&mut self) -> Result<Vec<Parameter>> {
        let mut params = vec![self.parse_param()?];
        while self.check(&TokenKind::Comma) {
            self.advance();
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    /// Parse a single parameter
    fn parse_param(&mut self) -> Result<Parameter> {
        let name = self.parse_identifier()?;
        let param_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        Ok(Parameter {
            name,
            param_type,
            default: None,
        })
    }

    /// Parse a type expression
    fn parse_type(&mut self) -> Result<Type> {
        if let Some(TokenKind::Identifier(name)) = self.current_identifier() {
            match name.as_str() {
                "string" => {
                    self.advance();
                    Ok(Type::String)
                }
                "int" => {
                    self.advance();
                    Ok(Type::Int)
                }
                "float" => {
                    self.advance();
                    Ok(Type::Float)
                }
                "bool" => {
                    self.advance();
                    Ok(Type::Bool)
                }
                "any" => {
                    self.advance();
                    Ok(Type::Any)
                }
                "list" => {
                    self.advance();
                    self.expect(&TokenKind::Less)?;
                    let inner = self.parse_type()?;
                    self.expect(&TokenKind::Greater)?;
                    Ok(Type::List(Box::new(inner)))
                }
                "map" => {
                    self.advance();
                    self.expect(&TokenKind::Less)?;
                    let key = self.parse_type()?;
                    self.expect(&TokenKind::Comma)?;
                    let value = self.parse_type()?;
                    self.expect(&TokenKind::Greater)?;
                    Ok(Type::Map(Box::new(key), Box::new(value)))
                }
                _ => Err(anyhow!("Unknown type: {}", name)),
            }
        } else {
            Err(anyhow!("Expected type name"))
        }
    }

    /// Parse an assignment
    fn parse_assignment(&mut self, mutable: bool, doc_comments: Vec<String>) -> Result<Statement> {
        let start = self.mark_position();

        if mutable {
            self.expect(&TokenKind::Mut)?;
        }

        let name = self.parse_identifier()?;

        // Check for module-specific patterns
        if name == "module" && self.check(&TokenKind::Dot) {
            self.advance(); // consume the dot
            let member = self.parse_identifier()?;

            // module.interface = (inputs = (...), outputs = (...))
            if member == "interface" {
                return self.parse_module_interface(doc_comments, start);
            }

            // module.outputs = (...)
            if member == "outputs" {
                return self.parse_module_outputs(doc_comments, start);
            }

            // module.<type>.<instance> = (source = "...", ...)
            // member is the type, need to parse instance name
            if self.check(&TokenKind::Dot) {
                self.advance(); // consume the dot
                let instance_name = self.parse_identifier()?;
                return self.parse_module_instance(member, instance_name, doc_comments, start);
            }

            return Err(anyhow!(
                "Invalid module pattern: module.{}, expected 'interface', 'outputs', or '<type>.<instance>'",
                member
            ));
        }

        let type_annotation = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Equal)?;
        let value = self.parse_expression()?;

        let doc = if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments)
        };

        Ok(Statement::Assignment {
            name,
            value,
            type_annotation,
            mutable,
            doc_comments: doc,
            span: self.span_from(start),
        })
    }

    /// Parse module.interface = (inputs = (...), outputs = (...))
    fn parse_module_interface(
        &mut self,
        doc_comments: Vec<String>,
        start: usize,
    ) -> Result<Statement> {
        use std::collections::HashMap;

        self.expect(&TokenKind::Equal)?;
        self.expect(&TokenKind::LeftParen)?;

        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();

        // Parse interface fields: inputs and outputs
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let field_name = self.parse_identifier()?;
            self.expect(&TokenKind::Equal)?;

            if field_name == "inputs" {
                // Parse inputs map
                inputs = self.parse_module_inputs_map()?;
            } else if field_name == "outputs" {
                // Parse outputs map
                outputs = self.parse_module_outputs_map()?;
            } else {
                return Err(anyhow!(
                    "Invalid module interface field: {}, expected 'inputs' or 'outputs'",
                    field_name
                ));
            }

            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(Statement::ModuleInterface {
            inputs,
            outputs,
            doc_comments: if doc_comments.is_empty() {
                None
            } else {
                Some(doc_comments)
            },
            span: self.span_from(start),
        })
    }

    /// Parse inputs map: (input_name = (type = "String", required = true, ...), ...)
    fn parse_module_inputs_map(
        &mut self,
    ) -> Result<std::collections::HashMap<String, crate::ast::ModuleInput>> {
        use crate::ast::ModuleInput;
        use std::collections::HashMap;

        self.expect(&TokenKind::LeftParen)?;
        let mut inputs = HashMap::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let input_name = self.parse_identifier()?;
            self.expect(&TokenKind::Equal)?;
            self.expect(&TokenKind::LeftParen)?;

            let mut input_type = Type::Any;
            let mut required = true;
            let mut default = None;
            let mut description = None;

            // Parse input parameters
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                let param_name = self.parse_identifier()?;
                self.expect(&TokenKind::Equal)?;

                match param_name.as_str() {
                    "type" => {
                        input_type = self.parse_type()?;
                    }
                    "required" => {
                        let expr = self.parse_expression()?;
                        if let Expression::Literal {
                            value: Value::Bool(b),
                            ..
                        } = expr
                        {
                            required = b;
                        } else {
                            return Err(anyhow!("Expected boolean for 'required' field"));
                        }
                    }
                    "default" => {
                        default = Some(self.parse_expression()?);
                    }
                    "description" => {
                        let expr = self.parse_expression()?;
                        if let Expression::Literal {
                            value: Value::String(s),
                            ..
                        } = expr
                        {
                            description = Some(s);
                        } else {
                            return Err(anyhow!("Expected string for 'description' field"));
                        }
                    }
                    _ => {
                        return Err(anyhow!("Invalid input parameter: {}", param_name));
                    }
                }

                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }

            self.expect(&TokenKind::RightParen)?;

            inputs.insert(
                input_name,
                ModuleInput {
                    input_type,
                    required,
                    default,
                    description,
                },
            );

            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RightParen)?;
        Ok(inputs)
    }

    /// Parse outputs map: (output_name = (type = "String", description = "..."), ...)
    fn parse_module_outputs_map(
        &mut self,
    ) -> Result<std::collections::HashMap<String, crate::ast::ModuleOutput>> {
        use crate::ast::ModuleOutput;
        use std::collections::HashMap;

        self.expect(&TokenKind::LeftParen)?;
        let mut outputs = HashMap::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let output_name = self.parse_identifier()?;
            self.expect(&TokenKind::Equal)?;
            self.expect(&TokenKind::LeftParen)?;

            let mut output_type = Type::Any;
            let mut description = None;

            // Parse output parameters
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                let param_name = self.parse_identifier()?;
                self.expect(&TokenKind::Equal)?;

                match param_name.as_str() {
                    "type" => {
                        output_type = self.parse_type()?;
                    }
                    "description" => {
                        let expr = self.parse_expression()?;
                        if let Expression::Literal {
                            value: Value::String(s),
                            ..
                        } = expr
                        {
                            description = Some(s);
                        } else {
                            return Err(anyhow!("Expected string for 'description' field"));
                        }
                    }
                    _ => {
                        return Err(anyhow!("Invalid output parameter: {}", param_name));
                    }
                }

                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }

            self.expect(&TokenKind::RightParen)?;

            outputs.insert(
                output_name,
                ModuleOutput {
                    output_type,
                    description,
                },
            );

            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RightParen)?;
        Ok(outputs)
    }

    /// Parse module.outputs = (output_name = expression, ...)
    fn parse_module_outputs(
        &mut self,
        doc_comments: Vec<String>,
        start: usize,
    ) -> Result<Statement> {
        use std::collections::HashMap;

        self.expect(&TokenKind::Equal)?;
        self.expect(&TokenKind::LeftParen)?;

        let mut outputs = HashMap::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let output_name = self.parse_identifier()?;
            self.expect(&TokenKind::Equal)?;
            let output_expr = self.parse_expression()?;

            outputs.insert(output_name, output_expr);

            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(Statement::ModuleOutputs {
            outputs,
            doc_comments: if doc_comments.is_empty() {
                None
            } else {
                Some(doc_comments)
            },
            span: self.span_from(start),
        })
    }

    /// Parse module.<type>.<instance> = (source = "...", input1 = value1, ...)
    fn parse_module_instance(
        &mut self,
        module_type: String,
        instance_name: String,
        doc_comments: Vec<String>,
        start: usize,
    ) -> Result<Statement> {
        use std::collections::HashMap;

        self.expect(&TokenKind::Equal)?;
        self.expect(&TokenKind::LeftParen)?;

        let mut source = None;
        let mut inputs = HashMap::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let field_name = self.parse_identifier()?;
            self.expect(&TokenKind::Equal)?;

            if field_name == "source" {
                // Parse source path (must be a string)
                let expr = self.parse_expression()?;
                if let Expression::Literal {
                    value: Value::String(s),
                    ..
                } = expr
                {
                    source = Some(s);
                } else {
                    return Err(anyhow!("Module source must be a string literal"));
                }
            } else {
                // Parse input parameter
                let input_expr = self.parse_expression()?;
                inputs.insert(field_name, input_expr);
            }

            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        self.expect(&TokenKind::RightParen)?;

        let source = source.ok_or_else(|| anyhow!("Module instance missing 'source' field"))?;

        Ok(Statement::ModuleInstance {
            module_type,
            instance_name,
            source,
            inputs,
            doc_comments: if doc_comments.is_empty() {
                None
            } else {
                Some(doc_comments)
            },
            span: self.span_from(start),
        })
    }

    /// Parse a for loop
    fn parse_for_loop(&mut self) -> Result<Statement> {
        let start = self.mark_position();

        self.expect(&TokenKind::For)?;

        let mut variables = vec![self.parse_identifier()?];
        while self.check(&TokenKind::Comma) {
            self.advance();
            variables.push(self.parse_identifier()?);
        }

        self.expect(&TokenKind::In)?;

        let mut iterables = vec![self.parse_expression()?];
        while self.check(&TokenKind::Comma) {
            self.advance();
            iterables.push(self.parse_expression()?);
        }

        self.expect(&TokenKind::LeftParen)?;
        let mut body = Vec::new();
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            let stmt = self.parse_statement(Vec::new())?;
            body.push(stmt);
        }
        self.expect(&TokenKind::RightParen)?;

        Ok(Statement::ForLoop {
            variables,
            iterables,
            body,
            condition: None,
            doc_comments: None,
            span: self.span_from(start),
        })
    }

    /// Parse an expression (entry point for Pratt parsing)
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_ternary()
    }

    /// Parse ternary expression: expr ? expr : expr
    fn parse_ternary(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut expr = self.parse_or()?;

        if self.check(&TokenKind::Question) {
            self.advance();
            let then_expr = self.parse_expression()?;
            self.expect(&TokenKind::Colon)?;
            let else_expr = self.parse_expression()?;
            expr = Expression::If {
                condition: Box::new(expr),
                then_expr: Box::new(then_expr),
                else_expr: Some(Box::new(else_expr)),
                span: self.span_from(start),
            };
        }

        Ok(expr)
    }

    /// Parse or expression
    fn parse_or(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut left = self.parse_and()?;

        while self.check(&TokenKind::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        Ok(left)
    }

    /// Parse and expression
    fn parse_and(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        Ok(left)
    }

    /// Parse equality expression
    fn parse_equality(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut left = self.parse_comparison()?;

        loop {
            let op = if self.check(&TokenKind::EqualEqual) {
                BinaryOperator::Equal
            } else if self.check(&TokenKind::NotEqual) {
                BinaryOperator::NotEqual
            } else {
                break;
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        Ok(left)
    }

    /// Parse comparison expression
    fn parse_comparison(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut left = self.parse_null_coalesce()?;

        loop {
            let op = if self.check(&TokenKind::Less) {
                BinaryOperator::LessThan
            } else if self.check(&TokenKind::LessEqual) {
                BinaryOperator::LessThanOrEqual
            } else if self.check(&TokenKind::Greater) {
                BinaryOperator::GreaterThan
            } else if self.check(&TokenKind::GreaterEqual) {
                BinaryOperator::GreaterThanOrEqual
            } else {
                break;
            };
            self.advance();
            let right = self.parse_null_coalesce()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        Ok(left)
    }

    /// Parse null coalesce expression
    fn parse_null_coalesce(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut left = self.parse_addition()?;

        while self.check(&TokenKind::QuestionQuestion) {
            self.advance();
            let right = self.parse_addition()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::NullCoalesce,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        Ok(left)
    }

    /// Parse addition/subtraction
    fn parse_addition(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut left = self.parse_multiplication()?;

        loop {
            let op = if self.check(&TokenKind::Plus) {
                BinaryOperator::Add
            } else if self.check(&TokenKind::Minus) {
                BinaryOperator::Subtract
            } else {
                break;
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        Ok(left)
    }

    /// Parse multiplication/division
    fn parse_multiplication(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut left = self.parse_pipe()?;

        loop {
            let op = if self.check(&TokenKind::Star) {
                BinaryOperator::Multiply
            } else if self.check(&TokenKind::Slash) {
                BinaryOperator::Divide
            } else if self.check(&TokenKind::Percent) {
                BinaryOperator::Modulo
            } else {
                break;
            };
            self.advance();
            let right = self.parse_pipe()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }

        Ok(left)
    }

    /// Parse pipe expression
    fn parse_pipe(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut stages = vec![self.parse_unary()?];

        while self.check(&TokenKind::Pipe) {
            self.advance();
            stages.push(self.parse_unary()?);
        }

        if stages.len() == 1 {
            Ok(stages.remove(0))
        } else {
            Ok(Expression::Pipeline {
                stages,
                span: self.span_from(start),
            })
        }
    }

    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expression> {
        let start = self.mark_position();

        if self.check(&TokenKind::Minus) {
            let minus_span = self.current_span();
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expression::BinaryOp {
                left: Box::new(Expression::Literal {
                    value: Value::Int(0),
                    span: minus_span,
                }),
                op: BinaryOperator::Subtract,
                right: Box::new(expr),
                span: self.span_from(start),
            });
        }

        if self.check(&TokenKind::Not) || self.check(&TokenKind::Bang) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expression::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(expr),
                span: self.span_from(start),
            });
        }

        self.parse_postfix()
    }

    /// Parse postfix expressions (calls, member access, indexing)
    fn parse_postfix(&mut self) -> Result<Expression> {
        let start_pos = self.mark_position();
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LeftParen) && matches!(expr, Expression::Variable { .. }) {
                // Function call - only if expression is a variable
                self.advance();
                let args = if !self.check(&TokenKind::RightParen) {
                    self.parse_argument_list()?
                } else {
                    Vec::new()
                };
                self.expect(&TokenKind::RightParen)?;

                if let Expression::Variable { name, .. } = expr {
                    expr = Expression::FunctionCall {
                        name,
                        args,
                        span: self.span_from(start_pos),
                    };
                } else {
                    unreachable!()
                }
            } else if self.check(&TokenKind::Dot) {
                // Member access
                self.advance();
                let field = self.parse_identifier()?;
                expr = Expression::MemberAccess {
                    object: Box::new(expr),
                    field,
                    span: self.span_from(start_pos),
                };
            } else if self.check(&TokenKind::QuestionDot) {
                // Optional chaining
                self.advance();
                let field = self.parse_identifier()?;
                expr = Expression::OptionalChain {
                    object: Box::new(expr),
                    field,
                    span: self.span_from(start_pos),
                };
            } else if self.check(&TokenKind::LeftBracket) {
                // Index, slice, or splat access
                self.advance();

                // Check for splat operator [*]
                if self.check(&TokenKind::Star) {
                    self.advance(); // consume *
                    self.expect(&TokenKind::RightBracket)?;

                    expr = Expression::Splat {
                        object: Box::new(expr),
                        span: self.span_from(start_pos),
                    };
                    continue; // Continue postfix parsing for chained access
                }

                // Check if this is a slice (starts with colon or has colon after first expr)
                let mut slice_start = None;
                let mut slice_end = None;
                let mut slice_step = None;

                if !self.check(&TokenKind::Colon) && !self.check(&TokenKind::RightBracket) {
                    // Parse first expression (could be index or slice start)
                    let first_expr = self.parse_expression()?;

                    if self.check(&TokenKind::Colon) {
                        // This is a slice: list[start:...]
                        slice_start = Some(Box::new(first_expr));
                    } else {
                        // This is an index: list[index]
                        self.expect(&TokenKind::RightBracket)?;
                        expr = Expression::Index {
                            object: Box::new(expr),
                            index: Box::new(first_expr),
                            span: self.span_from(start_pos),
                        };
                        continue;
                    }
                }

                // At this point, we're parsing a slice
                // Handle first colon (between start and end)
                if self.check(&TokenKind::Colon) {
                    self.advance();

                    // Parse end if present
                    if !self.check(&TokenKind::Colon) && !self.check(&TokenKind::RightBracket) {
                        slice_end = Some(Box::new(self.parse_expression()?));
                    }
                }

                // Handle second colon (between end and step)
                if self.check(&TokenKind::Colon) {
                    self.advance();

                    // Parse step if present
                    if !self.check(&TokenKind::RightBracket) {
                        slice_step = Some(Box::new(self.parse_expression()?));
                    }
                }

                self.expect(&TokenKind::RightBracket)?;
                expr = Expression::Slice {
                    object: Box::new(expr),
                    start: slice_start,
                    end: slice_end,
                    step: slice_step,
                    span: self.span_from(start_pos),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse argument list
    fn parse_argument_list(&mut self) -> Result<Vec<Expression>> {
        let mut args = vec![self.parse_expression()?];
        while self.check(&TokenKind::Comma) {
            self.advance();
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }

    /// Parse primary expression
    fn parse_primary(&mut self) -> Result<Expression> {
        // Parenthesized expression or map
        if self.check(&TokenKind::LeftParen) {
            return self.parse_paren_or_map();
        }

        // List or list comprehension
        if self.check(&TokenKind::LeftBracket) {
            return self.parse_list_or_comprehension();
        }

        // If expression
        if self.check(&TokenKind::If) {
            return self.parse_if_expr();
        }

        // When expression
        if self.check(&TokenKind::When) {
            return self.parse_when_expr();
        }

        // Try expression
        if self.check(&TokenKind::Try) {
            return self.parse_try_expr();
        }

        // Lambda (identifier => expr)
        if self.check_identifier() {
            let next_pos = self.position + 1;
            if next_pos < self.tokens.len()
                && matches!(self.tokens[next_pos].kind, TokenKind::Arrow)
            {
                return self.parse_lambda();
            }
        }

        // Check for interpolated strings before other literals
        if let TokenKind::String(StringValue::Interpolated(_)) = &self.current().kind {
            return self.parse_interpolated_string();
        }

        // Check for heredoc strings
        if let TokenKind::String(StringValue::Heredoc { .. }) = &self.current().kind {
            return self.parse_heredoc_string();
        }

        // Literals
        let literal_span = self.current_span();
        if let Some(value) = self.parse_literal()? {
            return Ok(Expression::Literal {
                value,
                span: literal_span,
            });
        }

        // Identifier (variable reference)
        if self.check_identifier() {
            let span = self.current_span();
            let name = self.parse_identifier()?;
            return Ok(Expression::Variable { name, span });
        }

        Err(anyhow!("Unexpected token: {:?}", self.current().kind))
    }

    /// Parse parenthesized expression or map literal
    fn parse_paren_or_map(&mut self) -> Result<Expression> {
        let start = self.mark_position();

        // Check if this is a multi-parameter lambda: (a, b, ...) =>
        if self.is_lambda_params() {
            return self.parse_lambda();
        }

        self.expect(&TokenKind::LeftParen)?;

        // Empty map
        if self.check(&TokenKind::RightParen) {
            self.advance();
            return Ok(Expression::Map {
                entries: Vec::new(),
                span: self.span_from(start),
            });
        }

        // Check if this is a map (key = value or key : value)
        if self.check_identifier() {
            let next_pos = self.position + 1;
            if next_pos < self.tokens.len() {
                let next = &self.tokens[next_pos].kind;
                if matches!(next, TokenKind::Equal | TokenKind::Colon) {
                    // It's a map
                    let entries = self.parse_map_entries()?;
                    self.expect(&TokenKind::RightParen)?;
                    return Ok(Expression::Map {
                        entries,
                        span: self.span_from(start),
                    });
                }
            }
        }

        // Parenthesized expression
        let expr = self.parse_expression()?;
        self.expect(&TokenKind::RightParen)?;
        Ok(expr)
    }

    /// Check if current position is the start of lambda parameters: (a, b) =>
    fn is_lambda_params(&self) -> bool {
        if !self.check(&TokenKind::LeftParen) {
            return false;
        }

        // Scan ahead to find matching ) and check if => follows
        let mut pos = self.position + 1;
        let mut depth = 1;

        while pos < self.tokens.len() && depth > 0 {
            match &self.tokens[pos].kind {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => depth -= 1,
                TokenKind::Equal | TokenKind::Colon => {
                    // This looks like a map entry, not lambda params
                    if depth == 1 {
                        return false;
                    }
                }
                TokenKind::Eof => return false,
                _ => {}
            }
            pos += 1;
        }

        // Check if => follows the closing )
        pos < self.tokens.len() && matches!(self.tokens[pos].kind, TokenKind::Arrow)
    }

    /// Parse map entries
    fn parse_map_entries(&mut self) -> Result<Vec<(String, Expression)>> {
        let mut entries = vec![self.parse_map_entry()?];
        while self.check(&TokenKind::Comma) {
            self.advance();
            if self.check(&TokenKind::RightParen) {
                break; // Trailing comma
            }
            entries.push(self.parse_map_entry()?);
        }
        Ok(entries)
    }

    /// Parse a single map entry
    fn parse_map_entry(&mut self) -> Result<(String, Expression)> {
        let key = self.parse_identifier()?;
        if self.check(&TokenKind::Equal) || self.check(&TokenKind::Colon) {
            self.advance();
        } else {
            return Err(anyhow!("Expected '=' or ':' in map entry"));
        }
        let value = self.parse_expression()?;
        Ok((key, value))
    }

    /// Parse list or list comprehension
    fn parse_list_or_comprehension(&mut self) -> Result<Expression> {
        let start = self.mark_position();

        self.expect(&TokenKind::LeftBracket)?;

        // Empty list
        if self.check(&TokenKind::RightBracket) {
            self.advance();
            return Ok(Expression::List {
                elements: Vec::new(),
                span: self.span_from(start),
            });
        }

        // Parse first expression - use parse_or to stop at 'for' keyword
        let first = self.parse_or()?;

        // Check for range syntax: [start..end] or [start..<end] or [start..end:step]
        if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotLess) {
            let inclusive = self.check(&TokenKind::DotDot);
            self.advance(); // consume .. or ..<

            let end = self.parse_or()?;

            // Check for optional step parameter: :step
            let step = if self.check(&TokenKind::Colon) {
                self.advance(); // consume :
                Some(Box::new(self.parse_or()?))
            } else {
                None
            };

            self.expect(&TokenKind::RightBracket)?;

            return Ok(Expression::Range {
                start: Box::new(first),
                end: Box::new(end),
                step,
                inclusive,
                span: self.span_from(start),
            });
        }

        // Check for list comprehension
        if self.check(&TokenKind::For) {
            // Parse multiple for clauses: for x in list1 for y in list2 ...
            let mut iterators = Vec::new();

            while self.check(&TokenKind::For) {
                self.advance();
                let var = self.parse_identifier()?;
                self.expect(&TokenKind::In)?;
                // Use parse_or instead of parse_expression to stop at keywords like 'for' and 'if'
                let iter = self.parse_or()?;
                iterators.push((var, iter));
            }

            let condition = if self.check(&TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            self.expect(&TokenKind::RightBracket)?;

            return Ok(Expression::ListComprehension {
                expr: Box::new(first),
                iterators,
                condition,
                span: self.span_from(start),
            });
        }

        // Regular list
        let mut items = vec![first];
        while self.check(&TokenKind::Comma) {
            self.advance();
            if self.check(&TokenKind::RightBracket) {
                break; // Trailing comma
            }
            items.push(self.parse_expression()?);
        }
        self.expect(&TokenKind::RightBracket)?;

        Ok(Expression::List {
            elements: items,
            span: self.span_from(start),
        })
    }

    /// Parse if expression
    fn parse_if_expr(&mut self) -> Result<Expression> {
        let start = self.mark_position();

        self.expect(&TokenKind::If)?;
        let condition = self.parse_expression()?;
        self.expect(&TokenKind::Then)?;
        let then_expr = self.parse_expression()?;

        let else_expr = if self.check(&TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        Ok(Expression::If {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr,
            span: self.span_from(start),
        })
    }

    /// Parse when expression (pattern matching)
    fn parse_when_expr(&mut self) -> Result<Expression> {
        let start = self.mark_position();

        self.expect(&TokenKind::When)?;
        let value = self.parse_expression()?;
        self.expect(&TokenKind::LeftParen)?;

        let mut arms = vec![self.parse_when_arm()?];
        while self.check(&TokenKind::Comma) {
            self.advance();
            if self.check(&TokenKind::RightParen) {
                break; // Trailing comma
            }
            arms.push(self.parse_when_arm()?);
        }
        self.expect(&TokenKind::RightParen)?;

        Ok(Expression::When {
            value: Box::new(value),
            arms,
            span: self.span_from(start),
        })
    }

    /// Parse a when arm
    fn parse_when_arm(&mut self) -> Result<WhenArm> {
        let pattern = self.parse_pattern()?;

        let guard = if self.check(&TokenKind::If) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(&TokenKind::Arrow)?;
        let expr = self.parse_expression()?;

        Ok(WhenArm {
            pattern,
            guard,
            expr,
        })
    }

    /// Parse a pattern for when expressions
    fn parse_pattern(&mut self) -> Result<Pattern> {
        // Wildcard pattern: * or _
        if self.check(&TokenKind::Star) {
            self.advance();
            return Ok(Pattern::Wildcard);
        }

        // Check for underscore as identifier
        if let TokenKind::Identifier(name) = &self.current().kind {
            if name == "_" {
                self.advance();
                return Ok(Pattern::Wildcard);
            }
        }

        // Tuple pattern: (pattern, pattern, ...)
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            if self.check(&TokenKind::RightParen) {
                self.advance();
                return Ok(Pattern::Tuple(Vec::new()));
            }

            let mut patterns = vec![self.parse_pattern()?];
            while self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::RightParen) {
                    break;
                }
                patterns.push(self.parse_pattern()?);
            }
            self.expect(&TokenKind::RightParen)?;
            return Ok(Pattern::Tuple(patterns));
        }

        // Literal patterns
        match &self.current().kind {
            TokenKind::Integer(i) => {
                let val = *i;
                self.advance();
                Ok(Pattern::Literal(Value::Int(val)))
            }
            TokenKind::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Pattern::Literal(Value::Float(val)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Value::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Value::Bool(false)))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Pattern::Literal(Value::Null))
            }
            TokenKind::String(StringValue::Simple(s)) => {
                let val = s.clone();
                self.advance();
                Ok(Pattern::Literal(Value::String(val)))
            }
            TokenKind::Identifier(name) => {
                let n = name.clone();
                self.advance();
                Ok(Pattern::Variable(n))
            }
            _ => Err(anyhow!("Expected pattern, got {:?}", self.current().kind)),
        }
    }

    /// Parse try expression
    fn parse_try_expr(&mut self) -> Result<Expression> {
        let start = self.mark_position();

        self.expect(&TokenKind::Try)?;
        self.expect(&TokenKind::LeftParen)?;
        let expr = self.parse_expression()?;

        let default = if self.check(&TokenKind::Comma) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(&TokenKind::RightParen)?;

        Ok(Expression::Try {
            expr: Box::new(expr),
            default,
            span: self.span_from(start),
        })
    }

    /// Parse lambda expression
    fn parse_lambda(&mut self) -> Result<Expression> {
        let start = self.mark_position();
        let mut params = Vec::new();

        if self.check(&TokenKind::LeftParen) {
            self.advance();
            if !self.check(&TokenKind::RightParen) {
                params.push(Parameter {
                    name: self.parse_identifier()?,
                    param_type: None,
                    default: None,
                });
                while self.check(&TokenKind::Comma) {
                    self.advance();
                    params.push(Parameter {
                        name: self.parse_identifier()?,
                        param_type: None,
                        default: None,
                    });
                }
            }
            self.expect(&TokenKind::RightParen)?;
        } else {
            params.push(Parameter {
                name: self.parse_identifier()?,
                param_type: None,
                default: None,
            });
        }

        self.expect(&TokenKind::Arrow)?;
        let body = self.parse_expression()?;

        Ok(Expression::Lambda {
            params,
            body: Box::new(body),
            span: self.span_from(start),
        })
    }

    /// Parse an interpolated string
    fn parse_interpolated_string(&mut self) -> Result<Expression> {
        let span = self.current_span();

        if let TokenKind::String(StringValue::Interpolated(parts)) = &self.current().kind {
            let mut result_parts = Vec::new();

            for part in parts {
                match part {
                    crate::lexer::StringPart::Literal(s) => {
                        result_parts.push(StringPart::Literal(s.clone()));
                    }
                    crate::lexer::StringPart::Interpolation(expr_str) => {
                        // Parse the interpolated expression
                        // We need to tokenize and parse the expression text
                        let mut lexer = crate::lexer::Lexer::new(expr_str);
                        let tokens = lexer.tokenize()?;
                        let mut parser = TokenParser::new(tokens);
                        let expr = parser.parse_expression()?;
                        result_parts.push(StringPart::Interpolation(Box::new(expr)));
                    }
                }
            }

            self.advance();
            Ok(Expression::InterpolatedString {
                parts: result_parts,
                span,
            })
        } else {
            Err(anyhow!("Expected interpolated string"))
        }
    }

    /// Parse a heredoc string (identical logic to interpolated strings)
    fn parse_heredoc_string(&mut self) -> Result<Expression> {
        let span = self.current_span();

        if let TokenKind::String(StringValue::Heredoc { parts, .. }) = &self.current().kind {
            let mut result_parts = Vec::new();

            for part in parts {
                match part {
                    crate::lexer::StringPart::Literal(s) => {
                        result_parts.push(StringPart::Literal(s.clone()));
                    }
                    crate::lexer::StringPart::Interpolation(expr_str) => {
                        // Parse the interpolated expression
                        // We need to tokenize and parse the expression text
                        let mut lexer = crate::lexer::Lexer::new(expr_str);
                        let tokens = lexer.tokenize()?;
                        let mut parser = TokenParser::new(tokens);
                        let expr = parser.parse_expression()?;
                        result_parts.push(StringPart::Interpolation(Box::new(expr)));
                    }
                }
            }

            self.advance();
            Ok(Expression::InterpolatedString {
                parts: result_parts,
                span,
            })
        } else {
            Err(anyhow!("Expected heredoc string"))
        }
    }

    /// Try to parse a literal value
    fn parse_literal(&mut self) -> Result<Option<Value>> {
        match &self.current().kind {
            TokenKind::Integer(i) => {
                let val = *i;
                self.advance();
                Ok(Some(Value::Int(val)))
            }
            TokenKind::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Some(Value::Float(val)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Some(Value::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Some(Value::Bool(false)))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Some(Value::Null))
            }
            TokenKind::String(sv) => {
                match sv {
                    StringValue::Simple(s) => {
                        let val = s.clone();
                        self.advance();
                        Ok(Some(Value::String(val)))
                    }
                    StringValue::Multiline(s) => {
                        let val = s.clone();
                        self.advance();
                        Ok(Some(Value::String(val)))
                    }
                    StringValue::Interpolated(_) => {
                        // Return None to signal this needs special handling
                        Ok(None)
                    }
                    StringValue::Heredoc { .. } => {
                        // Heredocs need special handling like interpolated strings
                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// Parse a string literal
    fn parse_string_literal(&mut self) -> Result<String> {
        match &self.current().kind {
            TokenKind::String(StringValue::Simple(s)) => {
                let val = s.clone();
                self.advance();
                Ok(val)
            }
            TokenKind::String(StringValue::Multiline(s)) => {
                let val = s.clone();
                self.advance();
                Ok(val)
            }
            _ => Err(anyhow!("Expected string literal")),
        }
    }

    /// Parse an identifier
    fn parse_identifier(&mut self) -> Result<String> {
        match &self.current().kind {
            TokenKind::Identifier(name) => {
                let n = name.clone();
                self.advance();
                Ok(n)
            }
            _ => Err(anyhow!(
                "Expected identifier, got {:?}",
                self.current().kind
            )),
        }
    }

    // Helper methods

    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    fn check_identifier(&self) -> bool {
        matches!(self.current().kind, TokenKind::Identifier(_))
    }

    fn current_identifier(&self) -> Option<TokenKind> {
        if self.check_identifier() {
            Some(self.current().kind.clone())
        } else {
            None
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<()> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(anyhow!(
                "Expected {:?}, got {:?}",
                kind,
                self.current().kind
            ))
        }
    }

    /// Mark the current position to start tracking a span
    fn mark_position(&self) -> usize {
        self.position
    }

    /// Create a SourceSpan from a marked position to the previous token
    /// (the last token that was consumed before the current position)
    fn span_from(&self, start_pos: usize) -> Option<SourceSpan> {
        if start_pos >= self.tokens.len() {
            return None;
        }

        let start_token = &self.tokens[start_pos];

        // The end is the previous token (last consumed token)
        let end_pos = if self.position > 0 {
            self.position - 1
        } else {
            0
        };

        if end_pos >= self.tokens.len() {
            return None;
        }

        let end_token = &self.tokens[end_pos];

        Some(SourceSpan {
            line: start_token.span.start.line,
            column: start_token.span.start.column,
            offset: start_token.span.start.offset,
            length: (end_token.span.end.offset - start_token.span.start.offset),
        })
    }

    /// Get the span of the current token
    fn current_span(&self) -> Option<SourceSpan> {
        if self.position >= self.tokens.len() {
            return None;
        }

        let token = &self.tokens[self.position];
        Some(SourceSpan {
            line: token.span.start.line,
            column: token.span.start.column,
            offset: token.span.start.offset,
            length: token.span.text.len(),
        })
    }
}

/// Parse source code using the token-based parser
pub fn parse(source: &str) -> Result<Module> {
    let tokens = crate::lexer::tokenize(source)?;
    let mut parser = TokenParser::new(tokens);
    parser.parse_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyword_prefixed_identifier() {
        let result = parse("format = 42");
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);
        if let Statement::Assignment { name, .. } = &module.statements[0] {
            assert_eq!(name, "format");
        } else {
            panic!("Expected assignment");
        }
    }

    #[test]
    fn test_parse_info_identifier() {
        let result = parse("info = \"test\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_order_identifier() {
        let result = parse("order = 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function_call() {
        let result = parse("result = fmt(\"Hello, %s!\", name)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_for_loop() {
        // Use [1,2,3] instead of bare identifier to avoid function call ambiguity
        let input = "for x in [1, 2, 3] (x)";
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_basic_example() {
        let content = r#"
name = "JCL"
version = 1
is_stable = true
config_path = null
port = 8000 + 80
timeout = 30 * 1000
half = 100 / 2
remainder = 10 % 3
is_production = version > 0.5
needs_update = version < 2 and !is_stable
status = if is_production then "live" else "dev"
effective_port = config_path ?? port
fallback_name = null ?? "default" ?? "backup"
"#;
        let result = parse(content);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 13);
    }

    #[test]
    fn test_parse_string_interpolation() {
        let input = r#"name = "World"
greeting = "Hello, ${name}!"
complex = "Value: ${1 + 2 * 3}"
"#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 3);

        // Check that the second statement is an interpolated string
        if let Statement::Assignment { value, .. } = &module.statements[1] {
            assert!(matches!(value, Expression::InterpolatedString { .. }));
        } else {
            panic!("Expected interpolated string assignment");
        }
    }

    #[test]
    fn test_parse_functions_example() {
        let content = r#"
fn double(x: int): int = x * 2
fn add(a: int, b: int) = a + b
fn quadruple(x: int) = double(double(x))

square = x => x * x
multiply = (a, b) => a * b

result1 = double(5)
result2 = add(10, 20)
result3 = quadruple(3)
"#;
        let result = parse(content);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_collections_example() {
        let content = r#"
numbers = [1, 2, 3, 4, 5]
empty_list = []
nested = [[1, 2], [3, 4]]

server = (
    host = "localhost",
    port = 8080,
    debug = true
)

config = (
    database = (host = "db.local", port = 5432),
    cache = (enabled = true, ttl = 3600)
)
"#;
        let result = parse(content);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_list_comprehension() {
        let content = r#"
doubled = [x * 2 for x in numbers]
evens = [x for x in numbers if x % 2 == 0]
"#;
        let result = parse(content);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_import_path_based() {
        let input = r#"import "./config.jcl""#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::Import { path, kind, .. } = &module.statements[0] {
            assert_eq!(path, "./config.jcl");
            assert!(matches!(kind, crate::ast::ImportKind::Full { alias: None }));
        } else {
            panic!("Expected import statement");
        }
    }

    #[test]
    fn test_parse_import_path_based_with_alias() {
        let input = r#"import "./config.jcl" as cfg"#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::Import { path, kind, .. } = &module.statements[0] {
            assert_eq!(path, "./config.jcl");
            if let crate::ast::ImportKind::Full { alias } = kind {
                assert_eq!(alias.as_ref().unwrap(), "cfg");
            } else {
                panic!("Expected Full import kind");
            }
        } else {
            panic!("Expected import statement");
        }
    }

    #[test]
    fn test_parse_import_selective() {
        let input = r#"import (database, server) from "./config.jcl""#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::Import { path, kind, .. } = &module.statements[0] {
            assert_eq!(path, "./config.jcl");
            if let crate::ast::ImportKind::Selective { items } = kind {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name, "database");
                assert_eq!(items[0].alias, None);
                assert_eq!(items[1].name, "server");
                assert_eq!(items[1].alias, None);
            } else {
                panic!("Expected Selective import kind");
            }
        } else {
            panic!("Expected import statement");
        }
    }

    #[test]
    fn test_parse_import_selective_with_aliases() {
        let input = r#"import (database as db, server as srv) from "./config.jcl""#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::Import { path, kind, .. } = &module.statements[0] {
            assert_eq!(path, "./config.jcl");
            if let crate::ast::ImportKind::Selective { items } = kind {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name, "database");
                assert_eq!(items[0].alias.as_ref().unwrap(), "db");
                assert_eq!(items[1].name, "server");
                assert_eq!(items[1].alias.as_ref().unwrap(), "srv");
            } else {
                panic!("Expected Selective import kind");
            }
        } else {
            panic!("Expected import statement");
        }
    }

    #[test]
    fn test_parse_import_wildcard() {
        let input = r#"import * from "./config.jcl""#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::Import { path, kind, .. } = &module.statements[0] {
            assert_eq!(path, "./config.jcl");
            assert!(matches!(kind, crate::ast::ImportKind::Wildcard));
        } else {
            panic!("Expected import statement");
        }
    }

    #[test]
    fn test_parse_module_interface() {
        let input = r#"
module.interface = (
    inputs = (
        name = (type = string, required = true, description = "The name"),
        count = (type = int, required = false, default = 10)
    ),
    outputs = (
        result = (type = string, description = "The result")
    )
)
"#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::ModuleInterface {
            inputs, outputs, ..
        } = &module.statements[0]
        {
            assert_eq!(inputs.len(), 2);
            assert!(inputs.contains_key("name"));
            assert!(inputs.contains_key("count"));
            assert_eq!(outputs.len(), 1);
            assert!(outputs.contains_key("result"));
        } else {
            panic!("Expected module interface statement");
        }
    }

    #[test]
    fn test_parse_module_outputs() {
        let input = r#"
module.outputs = (
    result = "hello",
    count = 42
)
"#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::ModuleOutputs { outputs, .. } = &module.statements[0] {
            assert_eq!(outputs.len(), 2);
            assert!(outputs.contains_key("result"));
            assert!(outputs.contains_key("count"));
        } else {
            panic!("Expected module outputs statement");
        }
    }

    #[test]
    fn test_parse_module_instance() {
        let input = r#"
module.server.web = (
    source = "./modules/server.jcl",
    port = 8080,
    host = "localhost"
)
"#;
        let result = parse(input);
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.statements.len(), 1);

        if let Statement::ModuleInstance {
            module_type,
            instance_name,
            source,
            inputs,
            ..
        } = &module.statements[0]
        {
            assert_eq!(module_type, "server");
            assert_eq!(instance_name, "web");
            assert_eq!(source, "./modules/server.jcl");
            assert_eq!(inputs.len(), 2);
            assert!(inputs.contains_key("port"));
            assert!(inputs.contains_key("host"));
        } else {
            panic!("Expected module instance statement");
        }
    }
}
