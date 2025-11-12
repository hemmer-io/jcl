//! Code formatter for JCL
//!
//! Formats JCL code with consistent style rules

use crate::ast::{Expression, Module, Statement, StringPart, Value, BinaryOperator, UnaryOperator};
use anyhow::Result;

/// Formatting options
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Number of spaces per indent level
    pub indent_size: usize,
    /// Maximum line length before wrapping
    pub max_line_length: usize,
    /// Whether to use trailing commas in lists/maps
    pub trailing_commas: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_size: 2,
            max_line_length: 100,
            trailing_commas: false,
        }
    }
}

/// Formatter for JCL code
pub struct Formatter {
    options: FormatOptions,
    indent_level: usize,
}

impl Formatter {
    /// Create a new formatter with default options
    pub fn new() -> Self {
        Self {
            options: FormatOptions::default(),
            indent_level: 0,
        }
    }

    /// Create a new formatter with custom options
    pub fn with_options(options: FormatOptions) -> Self {
        Self {
            options,
            indent_level: 0,
        }
    }

    /// Format a module
    pub fn format_module(&mut self, module: &Module) -> Result<String> {
        let mut output = String::new();
        let mut first = true;

        for statement in &module.statements {
            if !first {
                output.push('\n');
            }
            first = false;
            output.push_str(&self.format_statement(statement)?);
        }

        Ok(output)
    }

    /// Format a statement
    fn format_statement(&mut self, stmt: &Statement) -> Result<String> {
        match stmt {
            Statement::Assignment { name, mutable, value, type_annotation } => {
                let mut result = self.indent();
                if *mutable {
                    result.push_str("mut ");
                }
                result.push_str(name);

                if let Some(type_ann) = type_annotation {
                    result.push_str(&format!(": {}", self.format_type(type_ann)));
                }

                result.push_str(" = ");
                result.push_str(&self.format_expression(value)?);
                Ok(result)
            }

            Statement::FunctionDef { name, params, body, return_type } => {
                let mut result = self.indent();
                result.push_str("fn ");
                result.push_str(name);
                result.push('(');

                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&param.name);
                    if let Some(param_type) = &param.param_type {
                        result.push_str(&format!(": {}", self.format_type(param_type)));
                    }
                }

                result.push(')');

                if let Some(ret_type) = return_type {
                    result.push_str(&format!(": {}", self.format_type(ret_type)));
                }

                result.push_str(" = ");
                result.push_str(&self.format_expression(body)?);
                Ok(result)
            }

            Statement::ForLoop { .. } => {
                Ok(format!("{}# For loop (formatting not yet implemented)", self.indent()))
            }

            Statement::Import { .. } => {
                Ok(format!("{}# Import (formatting not yet implemented)", self.indent()))
            }

            Statement::Expression(expr) => {
                let mut result = self.indent();
                result.push_str(&self.format_expression(expr)?);
                Ok(result)
            }
        }
    }

    /// Format an expression
    fn format_expression(&mut self, expr: &Expression) -> Result<String> {
        match expr {
            Expression::Literal(value) => Ok(self.format_value(value)),

            Expression::Variable(name) => Ok(name.clone()),

            Expression::List(items) => {
                if items.is_empty() {
                    return Ok("[]".to_string());
                }

                let mut result = String::from("[");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(item)?);
                }
                result.push(']');
                Ok(result)
            }

            Expression::Map(entries) => {
                if entries.is_empty() {
                    return Ok("()".to_string());
                }

                let mut result = String::from("(");
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(key);
                    result.push_str(" = ");
                    result.push_str(&self.format_expression(value)?);
                }
                result.push(')');
                Ok(result)
            }

            Expression::BinaryOp { op, left, right } => {
                Ok(format!(
                    "{} {} {}",
                    self.format_expression(left)?,
                    self.format_binary_op(*op),
                    self.format_expression(right)?
                ))
            }

            Expression::UnaryOp { op, operand } => {
                Ok(format!(
                    "{}{}",
                    self.format_unary_op(*op),
                    self.format_expression(operand)?
                ))
            }

            Expression::FunctionCall { name, args } => {
                let mut result = name.clone();
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(arg)?);
                }
                result.push(')');
                Ok(result)
            }

            Expression::MethodCall { object, method, args } => {
                let mut result = self.format_expression(object)?;
                result.push('.');
                result.push_str(method);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.format_expression(arg)?);
                }
                result.push(')');
                Ok(result)
            }

            Expression::MemberAccess { object, field } => {
                Ok(format!("{}.{}", self.format_expression(object)?, field))
            }

            Expression::OptionalChain { object, field } => {
                Ok(format!("{}?.{}", self.format_expression(object)?, field))
            }

            Expression::Index { object, index } => {
                Ok(format!(
                    "{}[{}]",
                    self.format_expression(object)?,
                    self.format_expression(index)?
                ))
            }

            Expression::Ternary { condition, then_expr, else_expr } => {
                Ok(format!(
                    "{} ? {} : {}",
                    self.format_expression(condition)?,
                    self.format_expression(then_expr)?,
                    self.format_expression(else_expr)?
                ))
            }

            Expression::If { condition, then_expr, else_expr } => {
                let mut result = format!(
                    "if {} then {}",
                    self.format_expression(condition)?,
                    self.format_expression(then_expr)?
                );
                if let Some(else_e) = else_expr {
                    result.push_str(&format!(" else {}", self.format_expression(else_e)?));
                }
                Ok(result)
            }

            Expression::Lambda { params, body } => {
                let params_str = if params.len() == 1 {
                    params[0].name.clone()
                } else {
                    let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                    format!("({})", param_names.join(", "))
                };
                Ok(format!("{} => {}", params_str, self.format_expression(body)?))
            }

            Expression::InterpolatedString { parts } => {
                let mut result = String::from("\"");
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(&Self::escape_string(s)),
                        StringPart::Interpolation(expr) => {
                            result.push_str("${");
                            result.push_str(&self.format_expression(expr)?);
                            result.push('}');
                        }
                    }
                }
                result.push('"');
                Ok(result)
            }

            _ => Ok("# Complex expression (formatting not yet implemented)".to_string()),
        }
    }

    /// Format a value
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::String(s) => format!("\"{}\"", Self::escape_string(s)),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::List(items) => {
                let formatted: Vec<String> = items.iter().map(|v| self.format_value(v)).collect();
                format!("[{}]", formatted.join(", "))
            }
            Value::Map(map) => {
                let entries: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, self.format_value(v)))
                    .collect();
                format!("({})", entries.join(", "))
            }
            Value::Function { .. } => "<function>".to_string(),
        }
    }

    /// Format a type
    fn format_type(&self, ty: &crate::ast::Type) -> String {
        use crate::ast::Type;
        match ty {
            Type::String => "string".to_string(),
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Null => "null".to_string(),
            Type::Any => "any".to_string(),
            Type::List(inner) => format!("list<{}>", self.format_type(inner)),
            Type::Map(k, v) => format!("map<{}, {}>", self.format_type(k), self.format_type(v)),
            Type::Function { params, return_type } => {
                let param_types: Vec<String> = params.iter().map(|t| self.format_type(t)).collect();
                format!("({}) -> {}", param_types.join(", "), self.format_type(return_type))
            }
        }
    }

    /// Format a binary operator
    fn format_binary_op(&self, op: BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Power => "**",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::LessThanOrEqual => "<=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::GreaterThanOrEqual => ">=",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
            BinaryOperator::NullCoalesce => "??",
            BinaryOperator::Concat => "++",
        }
    }

    /// Format a unary operator
    fn format_unary_op(&self, op: UnaryOperator) -> &'static str {
        match op {
            UnaryOperator::Not => "!",
            UnaryOperator::Negate => "-",
        }
    }

    /// Escape string for output
    fn escape_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Get current indentation string
    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.options.indent_size)
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a JCL module with default options
pub fn format(module: &Module) -> Result<String> {
    let mut formatter = Formatter::new();
    formatter.format_module(module)
}

/// Format a JCL module with custom options
pub fn format_with_options(module: &Module, options: FormatOptions) -> Result<String> {
    let mut formatter = Formatter::with_options(options);
    formatter.format_module(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_format_simple_assignment() {
        let input = "name=\"John\"";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "name = \"John\"");
    }

    #[test]
    fn test_format_type_annotation() {
        let input = "age:int=25";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "age: int = 25");
    }

    #[test]
    fn test_format_list() {
        let input = "numbers=[1,2,3,4,5]";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "numbers = [1, 2, 3, 4, 5]");
    }

    #[test]
    fn test_format_map() {
        let input = "config=(name=\"test\", port=8080)";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "config = (name = \"test\", port = 8080)");
    }

    #[test]
    fn test_format_lambda() {
        let input = "double=x=>x*2";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "double = x => x * 2");
    }

    #[test]
    fn test_format_multi_param_lambda() {
        let input = "add=(x,y)=>x+y";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "add = (x, y) => x + y");
    }

    #[test]
    fn test_format_function_call() {
        let input = "result=map(x=>x*2,[1,2,3])";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "result = map(x => x * 2, [1, 2, 3])");
    }

    #[test]
    fn test_format_binary_operations() {
        let input = "result=1+2*3";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "result = 1 + 2 * 3");
    }

    #[test]
    fn test_format_string() {
        let input = r#"greeting="Hello, World!""#;
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, r#"greeting = "Hello, World!""#);
    }

    #[test]
    fn test_format_if_expression() {
        let input = "result=if x>0 then \"positive\" else \"negative\"";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "result = if x > 0 then \"positive\" else \"negative\"");
    }

    #[test]
    fn test_format_ternary() {
        let input = "result=x>0?\"pos\":\"neg\"";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "result = x > 0 ? \"pos\" : \"neg\"");
    }

    #[test]
    fn test_format_multiple_statements() {
        let input = r#"
            name="John"
            age=25
            active=true
        "#;
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "name = \"John\"\nage = 25\nactive = true");
    }

    #[test]
    fn test_format_function_definition() {
        let input = "fn double(x)=x*2";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "fn double(x) = x * 2");
    }

    #[test]
    fn test_format_function_with_type_annotation() {
        let input = "fn add(x:int,y:int):int=x+y";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "fn add(x: int, y: int): int = x + y");
    }

    #[test]
    fn test_format_member_access() {
        let input = "value=config.server.port";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "value = config.server.port");
    }

    #[test]
    fn test_format_optional_chain() {
        let input = "value=config?.server?.port";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "value = config?.server?.port");
    }

    #[test]
    fn test_format_index_access() {
        let input = "item=list[0]";
        let module = parser::parse_str(input).unwrap();
        let formatted = format(&module).unwrap();
        assert_eq!(formatted, "item = list[0]");
    }
}
