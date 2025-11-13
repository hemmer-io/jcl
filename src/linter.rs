//! Linter for JCL - checks code for style issues and best practices

use crate::ast::{Expression, Module, Statement, Value};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Severity level for lint issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A lint issue found in the code
#[derive(Debug, Clone)]
pub struct LintIssue {
    pub severity: Severity,
    pub message: String,
    pub rule: String,
    pub suggestion: Option<String>,
}

/// Linter for JCL code
pub struct Linter {
    issues: Vec<LintIssue>,
    variables: HashMap<String, bool>, // name -> used
    functions: HashMap<String, bool>, // name -> used
}

impl Linter {
    /// Create a new linter
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Lint a module and return all issues
    pub fn lint(&mut self, module: &Module) -> Result<Vec<LintIssue>> {
        self.issues.clear();
        self.variables.clear();
        self.functions.clear();

        // First pass: collect all definitions
        for statement in &module.statements {
            match statement {
                Statement::Assignment { name, .. } => {
                    self.variables.insert(name.clone(), false);
                }
                Statement::FunctionDef { name, .. } => {
                    self.functions.insert(name.clone(), false);
                }
                _ => {}
            }
        }

        // Second pass: analyze statements
        for statement in &module.statements {
            self.check_statement(statement);
        }

        // Check for unused variables and functions
        self.check_unused();

        Ok(self.issues.clone())
    }

    /// Check a statement
    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assignment {
                name,
                value,
                type_annotation,
                mutable,
                doc_comments: _,
            } => {
                // Check naming convention
                if !Self::is_snake_case(name) {
                    self.add_issue(
                        Severity::Warning,
                        format!("Variable '{}' should use snake_case naming", name),
                        "naming-convention",
                        Some(format!("Consider renaming to '{}'", Self::to_snake_case(name))),
                    );
                }

                // Check for missing type annotation
                if type_annotation.is_none() && Self::should_have_type_annotation(value) {
                    self.add_issue(
                        Severity::Info,
                        format!("Variable '{}' could benefit from a type annotation", name),
                        "missing-type-annotation",
                        Some(format!("Add type annotation like: {}: type = ...", name)),
                    );
                }

                // Check if mutable variable is actually mutated
                if *mutable {
                    self.add_issue(
                        Severity::Info,
                        format!("Variable '{}' is declared mutable but JCL is immutable by default", name),
                        "unnecessary-mut",
                        Some("Consider removing 'mut' keyword".to_string()),
                    );
                }

                // Check the expression
                self.check_expression(value);

                // Check for constant expressions
                if Self::is_constant_expression(value) {
                    self.add_issue(
                        Severity::Info,
                        format!("Variable '{}' is assigned a constant value that could be inlined", name),
                        "constant-variable",
                        None,
                    );
                }
            }

            Statement::FunctionDef {
                name,
                params,
                body,
                ..
            } => {
                // Check naming convention
                if !Self::is_snake_case(name) {
                    self.add_issue(
                        Severity::Warning,
                        format!("Function '{}' should use snake_case naming", name),
                        "naming-convention",
                        Some(format!("Consider renaming to '{}'", Self::to_snake_case(name))),
                    );
                }

                // Check for unused parameters
                let mut used_params = HashSet::new();
                Self::collect_used_variables(body, &mut used_params);

                for param in params {
                    if !used_params.contains(&param.name) && !param.name.starts_with('_') {
                        self.add_issue(
                            Severity::Warning,
                            format!("Parameter '{}' is never used", param.name),
                            "unused-parameter",
                            Some(format!("Consider renaming to '_{}'", param.name)),
                        );
                    }
                }

                self.check_expression(body);
            }

            Statement::Expression(expr) => {
                self.check_expression(expr);
            }

            _ => {}
        }
    }

    /// Check an expression
    fn check_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Variable(name) => {
                // Mark variable as used
                if let Some(used) = self.variables.get_mut(name) {
                    *used = true;
                }
                if let Some(used) = self.functions.get_mut(name) {
                    *used = true;
                }
            }

            Expression::BinaryOp { left, right, op } => {
                self.check_expression(left);
                self.check_expression(right);

                // Check for redundant operations
                if Self::is_redundant_operation(left, right, op) {
                    self.add_issue(
                        Severity::Info,
                        "Redundant operation detected".to_string(),
                        "redundant-operation",
                        Some("This operation can be simplified".to_string()),
                    );
                }
            }

            Expression::FunctionCall { name, args } => {
                // Mark function as used
                if let Some(used) = self.functions.get_mut(name) {
                    *used = true;
                }

                for arg in args {
                    self.check_expression(arg);
                }
            }

            Expression::Lambda { params, body } => {
                // Check for unused lambda parameters
                let mut used_params = HashSet::new();
                Self::collect_used_variables(body, &mut used_params);

                for param in params {
                    if !used_params.contains(&param.name) && !param.name.starts_with('_') {
                        self.add_issue(
                            Severity::Warning,
                            format!("Lambda parameter '{}' is never used", param.name),
                            "unused-parameter",
                            Some(format!("Consider renaming to '_{}'", param.name)),
                        );
                    }
                }

                self.check_expression(body);
            }

            Expression::If {
                condition,
                then_expr,
                else_expr,
            } => {
                self.check_expression(condition);
                self.check_expression(then_expr);
                if let Some(else_e) = else_expr {
                    self.check_expression(else_e);
                }

                // Check for constant conditions
                if Self::is_constant_expression(condition) {
                    self.add_issue(
                        Severity::Warning,
                        "Condition is always constant".to_string(),
                        "constant-condition",
                        Some("Consider removing the if expression".to_string()),
                    );
                }
            }

            Expression::List(items) => {
                for item in items {
                    self.check_expression(item);
                }
            }

            Expression::Map(entries) => {
                for (_, value) in entries {
                    self.check_expression(value);
                }
            }

            Expression::MemberAccess { object, .. } => {
                self.check_expression(object);
            }

            Expression::Index { object, index } => {
                self.check_expression(object);
                self.check_expression(index);
            }

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.check_expression(condition);
                self.check_expression(then_expr);
                self.check_expression(else_expr);
            }

            Expression::UnaryOp { operand, .. } => {
                self.check_expression(operand);
            }

            Expression::MethodCall { object, args, .. } => {
                self.check_expression(object);
                for arg in args {
                    self.check_expression(arg);
                }
            }

            Expression::OptionalChain { object, .. } => {
                self.check_expression(object);
            }

            _ => {}
        }
    }

    /// Check for unused variables and functions
    fn check_unused(&mut self) {
        // Collect unused variables first to avoid borrow checker issues
        let unused_vars: Vec<String> = self.variables.iter()
            .filter(|(name, used)| !**used && !name.starts_with('_'))
            .map(|(name, _)| name.clone())
            .collect();

        // Add issues for unused variables
        for name in unused_vars {
            self.add_issue(
                Severity::Warning,
                format!("Variable '{}' is never used", name),
                "unused-variable",
                Some(format!("Consider removing or renaming to '_{}'", name)),
            );
        }

        // Collect unused functions
        let unused_funcs: Vec<String> = self.functions.iter()
            .filter(|(name, used)| !**used && !name.starts_with('_'))
            .map(|(name, _)| name.clone())
            .collect();

        // Add issues for unused functions
        for name in unused_funcs {
            self.add_issue(
                Severity::Warning,
                format!("Function '{}' is never used", name),
                "unused-function",
                Some(format!("Consider removing or renaming to '_{}'", name)),
            );
        }
    }

    /// Add a lint issue
    fn add_issue(&mut self, severity: Severity, message: String, rule: &str, suggestion: Option<String>) {
        self.issues.push(LintIssue {
            severity,
            message,
            rule: rule.to_string(),
            suggestion,
        });
    }

    /// Check if a name is in snake_case
    fn is_snake_case(name: &str) -> bool {
        name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
    }

    /// Convert a name to snake_case
    fn to_snake_case(name: &str) -> String {
        let mut result = String::new();
        let mut prev_lower = false;

        for (i, c) in name.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 && prev_lower {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
                prev_lower = false;
            } else {
                result.push(c);
                prev_lower = c.is_lowercase();
            }
        }

        result
    }

    /// Check if an expression should have a type annotation
    fn should_have_type_annotation(expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::List(_) | Expression::Map(_) | Expression::Lambda { .. }
        )
    }

    /// Check if an expression is constant
    fn is_constant_expression(expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::Literal(_)
        )
    }

    /// Check for redundant operations
    fn is_redundant_operation(left: &Expression, right: &Expression, op: &crate::ast::BinaryOperator) -> bool {
        use crate::ast::BinaryOperator;

        match (left, right, op) {
            // x + 0 or 0 + x
            (_, Expression::Literal(Value::Int(0)), BinaryOperator::Add) => true,
            (Expression::Literal(Value::Int(0)), _, BinaryOperator::Add) => true,
            // x * 1 or 1 * x
            (_, Expression::Literal(Value::Int(1)), BinaryOperator::Multiply) => true,
            (Expression::Literal(Value::Int(1)), _, BinaryOperator::Multiply) => true,
            // x * 0 or 0 * x
            (_, Expression::Literal(Value::Int(0)), BinaryOperator::Multiply) => true,
            (Expression::Literal(Value::Int(0)), _, BinaryOperator::Multiply) => true,
            _ => false,
        }
    }

    /// Collect used variables in an expression
    fn collect_used_variables(expr: &Expression, used: &mut HashSet<String>) {
        match expr {
            Expression::Variable(name) => {
                used.insert(name.clone());
            }
            Expression::BinaryOp { left, right, .. } => {
                Self::collect_used_variables(left, used);
                Self::collect_used_variables(right, used);
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    Self::collect_used_variables(arg, used);
                }
            }
            Expression::Lambda { body, .. } => {
                Self::collect_used_variables(body, used);
            }
            Expression::List(items) => {
                for item in items {
                    Self::collect_used_variables(item, used);
                }
            }
            Expression::Map(entries) => {
                for (_, value) in entries {
                    Self::collect_used_variables(value, used);
                }
            }
            Expression::If {
                condition,
                then_expr,
                else_expr,
            } => {
                Self::collect_used_variables(condition, used);
                Self::collect_used_variables(then_expr, used);
                if let Some(else_e) = else_expr {
                    Self::collect_used_variables(else_e, used);
                }
            }
            Expression::MemberAccess { object, .. } => {
                Self::collect_used_variables(object, used);
            }
            Expression::Index { object, index } => {
                Self::collect_used_variables(object, used);
                Self::collect_used_variables(index, used);
            }
            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                Self::collect_used_variables(condition, used);
                Self::collect_used_variables(then_expr, used);
                Self::collect_used_variables(else_expr, used);
            }
            Expression::UnaryOp { operand, .. } => {
                Self::collect_used_variables(operand, used);
            }
            _ => {}
        }
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

/// Lint a module and return issues
pub fn lint(module: &Module) -> Result<Vec<LintIssue>> {
    let mut linter = Linter::new();
    linter.lint(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_unused_variable() {
        let input = "unused = 42\nused = 10\nresult = used + 5";
        let module = parser::parse_str(input).unwrap();
        let issues = lint(&module).unwrap();

        assert!(issues.iter().any(|i| i.rule == "unused-variable" && i.message.contains("unused")));
    }

    #[test]
    fn test_naming_convention() {
        let input = "myVariable = 42";
        let module = parser::parse_str(input).unwrap();
        let issues = lint(&module).unwrap();

        assert!(issues.iter().any(|i| i.rule == "naming-convention"));
    }

    #[test]
    fn test_unused_parameter() {
        let input = "fn test(x, y) = x + 1";
        let module = parser::parse_str(input).unwrap();
        let issues = lint(&module).unwrap();

        assert!(issues.iter().any(|i| i.rule == "unused-parameter" && i.message.contains("'y'")));
    }

    #[test]
    fn test_redundant_operation() {
        let input = "result = x + 0";
        let module = parser::parse_str(input).unwrap();
        let issues = lint(&module).unwrap();

        assert!(issues.iter().any(|i| i.rule == "redundant-operation"));
    }

    #[test]
    fn test_no_warnings_for_used_code() {
        let input = r#"
            fn double(x) = x * 2
            value = 10
            result = double(value)
            _output = result
        "#;
        let module = parser::parse_str(input).unwrap();
        let issues = lint(&module).unwrap();

        // Should have no errors or warnings (may have info messages)
        let has_errors = issues.iter().any(|i| i.severity == Severity::Error);
        let has_warnings = issues.iter().any(|i| i.severity == Severity::Warning);
        assert!(!has_errors && !has_warnings, "Expected no errors or warnings, got: {:?}", issues);
    }
}
