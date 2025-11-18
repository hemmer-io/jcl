//! Type system for JCL with advanced static type inference

use crate::ast::{
    BinaryOperator, Expression, Module, SourceSpan, Statement, Type, UnaryOperator, Value,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Type error with source location
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl TypeError {
    pub fn new(message: String, span: Option<SourceSpan>) -> Self {
        Self { message, span }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeError {}

/// Type environment for tracking variable and function types
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    variables: HashMap<String, Type>,
    functions: HashMap<String, Type>,
    parent: Option<Box<TypeEnvironment>>,
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnvironment {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: None,
        }
    }

    /// Create a child environment (for nested scopes)
    pub fn child(&self) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Define a variable type
    pub fn define_variable(&mut self, name: String, var_type: Type) {
        self.variables.insert(name, var_type);
    }

    /// Define a function type
    pub fn define_function(&mut self, name: String, func_type: Type) {
        self.functions.insert(name, func_type);
    }

    /// Look up a variable type
    pub fn lookup_variable(&self, name: &str) -> Option<Type> {
        if let Some(t) = self.variables.get(name) {
            Some(t.clone())
        } else if let Some(ref parent) = self.parent {
            parent.lookup_variable(name)
        } else {
            None
        }
    }

    /// Look up a function type
    pub fn lookup_function(&self, name: &str) -> Option<Type> {
        if let Some(t) = self.functions.get(name) {
            Some(t.clone())
        } else if let Some(ref parent) = self.parent {
            parent.lookup_function(name)
        } else {
            None
        }
    }

    /// Register built-in function signatures
    pub fn register_builtins(&mut self) {
        // String functions
        self.define_function(
            "upper".to_string(),
            Type::Function {
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        self.define_function(
            "lower".to_string(),
            Type::Function {
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        self.define_function(
            "trim".to_string(),
            Type::Function {
                params: vec![Type::String],
                return_type: Box::new(Type::String),
            },
        );
        self.define_function(
            "split".to_string(),
            Type::Function {
                params: vec![Type::String, Type::String],
                return_type: Box::new(Type::List(Box::new(Type::String))),
            },
        );
        self.define_function(
            "join".to_string(),
            Type::Function {
                params: vec![Type::List(Box::new(Type::String)), Type::String],
                return_type: Box::new(Type::String),
            },
        );

        // Collection functions
        self.define_function(
            "length".to_string(),
            Type::Function {
                params: vec![Type::Any],
                return_type: Box::new(Type::Int),
            },
        );
        self.define_function(
            "reverse".to_string(),
            Type::Function {
                params: vec![Type::List(Box::new(Type::Any))],
                return_type: Box::new(Type::List(Box::new(Type::Any))),
            },
        );

        // Math functions
        self.define_function(
            "abs".to_string(),
            Type::Function {
                params: vec![Type::Int],
                return_type: Box::new(Type::Int),
            },
        );
        self.define_function(
            "min".to_string(),
            Type::Function {
                params: vec![Type::List(Box::new(Type::Int))],
                return_type: Box::new(Type::Int),
            },
        );
        self.define_function(
            "max".to_string(),
            Type::Function {
                params: vec![Type::List(Box::new(Type::Int))],
                return_type: Box::new(Type::Int),
            },
        );
        self.define_function(
            "sum".to_string(),
            Type::Function {
                params: vec![Type::List(Box::new(Type::Int))],
                return_type: Box::new(Type::Int),
            },
        );

        // Type conversion
        self.define_function(
            "str".to_string(),
            Type::Function {
                params: vec![Type::Any],
                return_type: Box::new(Type::String),
            },
        );
        self.define_function(
            "int".to_string(),
            Type::Function {
                params: vec![Type::Any],
                return_type: Box::new(Type::Int),
            },
        );
        self.define_function(
            "float".to_string(),
            Type::Function {
                params: vec![Type::Any],
                return_type: Box::new(Type::Float),
            },
        );
    }
}

/// Type checker with advanced static inference
pub struct TypeChecker {
    env: TypeEnvironment,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        let mut env = TypeEnvironment::new();
        env.register_builtins();
        Self {
            env,
            errors: Vec::new(),
        }
    }

    /// Get all type errors collected during checking
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// Check a module and infer all types
    pub fn check_module(&mut self, module: &Module) -> Result<(), Vec<TypeError>> {
        self.errors.clear();

        for statement in &module.statements {
            if let Err(e) = self.check_statement(statement) {
                self.errors.push(e);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Check a statement
    fn check_statement(&mut self, stmt: &Statement) -> Result<(), TypeError> {
        match stmt {
            Statement::Assignment {
                name,
                value,
                type_annotation,
                span,
                ..
            } => {
                let inferred_type = self.infer_expression(value)?;

                // If type annotation exists, verify it matches
                if let Some(expected_type) = type_annotation {
                    if !self.is_compatible(&inferred_type, expected_type) {
                        return Err(TypeError::new(
                            format!(
                                "Type mismatch for variable '{}': expected {:?}, got {:?}",
                                name, expected_type, inferred_type
                            ),
                            span.clone(),
                        ));
                    }
                }

                self.env.define_variable(name.clone(), inferred_type);
                Ok(())
            }

            Statement::FunctionDef {
                name,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                // Create function type
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| p.param_type.clone().unwrap_or(Type::Any))
                    .collect();

                let func_type = Type::Function {
                    params: param_types.clone(),
                    return_type: Box::new(return_type.clone().unwrap_or(Type::Any)),
                };

                self.env.define_function(name.clone(), func_type);

                // Check function body in new scope
                let mut child_checker = TypeChecker {
                    env: self.env.child(),
                    errors: Vec::new(),
                };

                // Add parameters to scope
                for param in params {
                    let param_type = param.param_type.clone().unwrap_or(Type::Any);
                    child_checker
                        .env
                        .define_variable(param.name.clone(), param_type);
                }

                let body_type = child_checker.infer_expression(body)?;

                // Verify return type if specified
                if let Some(expected_return) = return_type {
                    if !self.is_compatible(&body_type, expected_return) {
                        return Err(TypeError::new(
                            format!(
                                "Function '{}' return type mismatch: expected {:?}, got {:?}",
                                name, expected_return, body_type
                            ),
                            span.clone(),
                        ));
                    }
                }

                Ok(())
            }

            Statement::Expression { expr, .. } => {
                self.infer_expression(expr)?;
                Ok(())
            }

            _ => Ok(()),
        }
    }

    /// Infer the type of an expression
    pub fn infer_expression(&self, expr: &Expression) -> Result<Type, TypeError> {
        match expr {
            Expression::Literal { value, .. } => Ok(self.infer_value(value)),

            Expression::Variable { name, span } => {
                // Check variables first, then functions
                if let Some(var_type) = self.env.lookup_variable(name) {
                    Ok(var_type)
                } else if let Some(func_type) = self.env.lookup_function(name) {
                    Ok(func_type)
                } else {
                    Err(TypeError::new(
                        format!("Undefined variable: {}", name),
                        span.clone(),
                    ))
                }
            }

            Expression::List { elements, .. } => {
                if elements.is_empty() {
                    Ok(Type::List(Box::new(Type::Any)))
                } else {
                    let elem_type = self.infer_expression(&elements[0])?;
                    // TODO: Check all elements have compatible types
                    Ok(Type::List(Box::new(elem_type)))
                }
            }

            Expression::Map { entries, .. } => {
                if entries.is_empty() {
                    Ok(Type::Map(Box::new(Type::String), Box::new(Type::Any)))
                } else {
                    let value_type = self.infer_expression(&entries[0].1)?;
                    Ok(Type::Map(Box::new(Type::String), Box::new(value_type)))
                }
            }

            Expression::BinaryOp {
                op,
                left,
                right,
                span,
            } => self.infer_binary_op(*op, left, right, span.as_ref()),

            Expression::UnaryOp {
                op, operand, span, ..
            } => self.infer_unary_op(*op, operand, span.as_ref()),

            Expression::FunctionCall {
                name, args, span, ..
            } => self.infer_function_call(name, args, span.as_ref()),

            Expression::If {
                condition,
                then_expr,
                else_expr,
                span,
            } => {
                let cond_type = self.infer_expression(condition)?;
                if !self.is_compatible(&cond_type, &Type::Bool) {
                    return Err(TypeError::new(
                        format!("Condition must be boolean, got {:?}", cond_type),
                        span.clone(),
                    ));
                }

                let then_type = self.infer_expression(then_expr)?;
                if let Some(else_e) = else_expr {
                    let else_type = self.infer_expression(else_e)?;
                    Ok(self.unify_types(&then_type, &else_type))
                } else {
                    Ok(then_type)
                }
            }

            Expression::MemberAccess {
                object,
                field,
                span,
            } => {
                let obj_type = self.infer_expression(object)?;
                match obj_type {
                    Type::Map(_, value_type) => Ok(*value_type),
                    _ => Err(TypeError::new(
                        format!(
                            "Cannot access field '{}' on non-map type {:?}",
                            field, obj_type
                        ),
                        span.clone(),
                    )),
                }
            }

            Expression::Lambda { params, body, .. } => {
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| p.param_type.clone().unwrap_or(Type::Any))
                    .collect();

                // Infer return type in new scope
                let mut child_checker = TypeChecker {
                    env: self.env.child(),
                    errors: Vec::new(),
                };

                for param in params {
                    let param_type = param.param_type.clone().unwrap_or(Type::Any);
                    child_checker
                        .env
                        .define_variable(param.name.clone(), param_type);
                }

                let return_type = child_checker.infer_expression(body)?;

                Ok(Type::Function {
                    params: param_types,
                    return_type: Box::new(return_type),
                })
            }

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
                span,
            } => {
                let cond_type = self.infer_expression(condition)?;
                if !self.is_compatible(&cond_type, &Type::Bool) {
                    return Err(TypeError::new(
                        format!("Ternary condition must be boolean, got {:?}", cond_type),
                        span.clone(),
                    ));
                }

                let then_type = self.infer_expression(then_expr)?;
                let else_type = self.infer_expression(else_expr)?;
                Ok(self.unify_types(&then_type, &else_type))
            }

            Expression::Index {
                object,
                index,
                span,
            } => {
                let obj_type = self.infer_expression(object)?;
                let idx_type = self.infer_expression(index)?;

                match obj_type {
                    Type::List(elem_type) => {
                        if !self.is_compatible(&idx_type, &Type::Int) {
                            return Err(TypeError::new(
                                format!("List index must be Int, got {:?}", idx_type),
                                span.clone(),
                            ));
                        }
                        Ok(*elem_type)
                    }
                    Type::Map(key_type, value_type) => {
                        if !self.is_compatible(&idx_type, &key_type) {
                            return Err(TypeError::new(
                                format!(
                                    "Map key type mismatch: expected {:?}, got {:?}",
                                    key_type, idx_type
                                ),
                                span.clone(),
                            ));
                        }
                        Ok(*value_type)
                    }
                    _ => Err(TypeError::new(
                        format!("Cannot index into type {:?}", obj_type),
                        span.clone(),
                    )),
                }
            }

            // Default to Any for unimplemented expressions
            _ => Ok(Type::Any),
        }
    }

    /// Infer type of binary operation
    fn infer_binary_op(
        &self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
        span: Option<&SourceSpan>,
    ) -> Result<Type, TypeError> {
        let left_type = self.infer_expression(left)?;
        let right_type = self.infer_expression(right)?;

        use BinaryOperator::*;
        match op {
            Add | Subtract | Multiply | Divide | Modulo | Power => {
                // Arithmetic operations
                if self.is_compatible(&left_type, &Type::Int)
                    && self.is_compatible(&right_type, &Type::Int)
                {
                    Ok(Type::Int)
                } else if self.is_compatible(&left_type, &Type::Float)
                    || self.is_compatible(&right_type, &Type::Float)
                {
                    Ok(Type::Float)
                } else {
                    Err(TypeError::new(
                        format!(
                            "Arithmetic operation requires numeric types, got {:?} and {:?}",
                            left_type, right_type
                        ),
                        span.cloned(),
                    ))
                }
            }

            Equal | NotEqual | LessThan | LessThanOrEqual | GreaterThan | GreaterThanOrEqual => {
                // Comparison operations always return bool
                Ok(Type::Bool)
            }

            And | Or => {
                // Logical operations require bool operands
                if !self.is_compatible(&left_type, &Type::Bool)
                    || !self.is_compatible(&right_type, &Type::Bool)
                {
                    return Err(TypeError::new(
                        format!(
                            "Logical operation requires boolean operands, got {:?} and {:?}",
                            left_type, right_type
                        ),
                        span.cloned(),
                    ));
                }
                Ok(Type::Bool)
            }

            NullCoalesce => {
                // Null coalescing returns non-null type
                Ok(right_type)
            }

            Concat => {
                // String concatenation
                if !self.is_compatible(&left_type, &Type::String)
                    || !self.is_compatible(&right_type, &Type::String)
                {
                    return Err(TypeError::new(
                        format!(
                            "String concatenation requires string operands, got {:?} and {:?}",
                            left_type, right_type
                        ),
                        span.cloned(),
                    ));
                }
                Ok(Type::String)
            }
        }
    }

    /// Infer type of unary operation
    fn infer_unary_op(
        &self,
        op: UnaryOperator,
        operand: &Expression,
        span: Option<&SourceSpan>,
    ) -> Result<Type, TypeError> {
        let operand_type = self.infer_expression(operand)?;

        use UnaryOperator::*;
        match op {
            Not => {
                if !self.is_compatible(&operand_type, &Type::Bool) {
                    return Err(TypeError::new(
                        format!(
                            "Logical NOT requires boolean operand, got {:?}",
                            operand_type
                        ),
                        span.cloned(),
                    ));
                }
                Ok(Type::Bool)
            }
            Negate => {
                if self.is_compatible(&operand_type, &Type::Int) {
                    Ok(Type::Int)
                } else if self.is_compatible(&operand_type, &Type::Float) {
                    Ok(Type::Float)
                } else {
                    Err(TypeError::new(
                        format!("Negation requires numeric type, got {:?}", operand_type),
                        span.cloned(),
                    ))
                }
            }
        }
    }

    /// Infer type of function call
    fn infer_function_call(
        &self,
        name: &str,
        args: &[Expression],
        span: Option<&SourceSpan>,
    ) -> Result<Type, TypeError> {
        if let Some(func_type) = self.env.lookup_function(name) {
            match func_type {
                Type::Function {
                    params,
                    return_type,
                } => {
                    // Check argument count
                    if args.len() != params.len() {
                        return Err(TypeError::new(
                            format!(
                                "Function '{}' expects {} arguments, got {}",
                                name,
                                params.len(),
                                args.len()
                            ),
                            span.cloned(),
                        ));
                    }

                    // Check argument types
                    for (i, (arg, expected_type)) in args.iter().zip(params.iter()).enumerate() {
                        let arg_type = self.infer_expression(arg)?;
                        if !self.is_compatible(&arg_type, expected_type) {
                            return Err(TypeError::new(
                                format!(
                                    "Function '{}' argument {} type mismatch: expected {:?}, got {:?}",
                                    name,
                                    i + 1,
                                    expected_type,
                                    arg_type
                                ),
                                span.cloned(),
                            ));
                        }
                    }

                    Ok(*return_type)
                }
                _ => Err(TypeError::new(
                    format!("'{}' is not a function", name),
                    span.cloned(),
                )),
            }
        } else {
            // Unknown function - could be user-defined or built-in we didn't register
            Ok(Type::Any)
        }
    }

    /// Unify two types (find common type)
    fn unify_types(&self, t1: &Type, t2: &Type) -> Type {
        if t1 == t2 {
            t1.clone()
        } else if matches!(t1, Type::Any) {
            t2.clone()
        } else if matches!(t2, Type::Any) {
            t1.clone()
        } else if matches!(
            (t1, t2),
            (Type::Int, Type::Float) | (Type::Float, Type::Int)
        ) {
            Type::Float
        } else {
            Type::Any
        }
    }

    /// Infer type from a value
    fn infer_value(&self, value: &Value) -> Type {
        match value {
            Value::String(_) => Type::String,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Bool(_) => Type::Bool,
            Value::Null => Type::Null,
            Value::List(items) => {
                if items.is_empty() {
                    Type::List(Box::new(Type::Any))
                } else {
                    Type::List(Box::new(self.infer_value(&items[0])))
                }
            }
            Value::Map(map) => {
                if map.is_empty() {
                    Type::Map(Box::new(Type::String), Box::new(Type::Any))
                } else {
                    let first_value = map.values().next().unwrap();
                    Type::Map(
                        Box::new(Type::String),
                        Box::new(self.infer_value(first_value)),
                    )
                }
            }
            Value::Function { params, .. } => {
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| p.param_type.clone().unwrap_or(Type::Any))
                    .collect();
                Type::Function {
                    params: param_types,
                    return_type: Box::new(Type::Any),
                }
            }
        }
    }

    /// Register a variable type (for external use)
    pub fn register_variable(&mut self, name: String, var_type: Type) {
        self.env.define_variable(name, var_type);
    }

    /// Check if a value matches a type
    pub fn check(&self, value: &Value, expected: &Type) -> Result<()> {
        match (value, expected) {
            (Value::String(_), Type::String) => Ok(()),
            (Value::Int(_), Type::Int) => Ok(()),
            (Value::Float(_), Type::Float) => Ok(()),
            (Value::Bool(_), Type::Bool) => Ok(()),
            (Value::Null, _) => Ok(()), // Null is compatible with any type
            (_, Type::Any) => Ok(()),   // Any accepts anything

            (Value::List(items), Type::List(item_type)) => {
                for item in items {
                    self.check(item, item_type)?;
                }
                Ok(())
            }

            (Value::Map(map), Type::Map(key_type, value_type)) => {
                for (k, v) in map {
                    self.check(&Value::String(k.clone()), key_type)?;
                    self.check(v, value_type)?;
                }
                Ok(())
            }

            // Object type checking removed - use Map instead
            _ => Err(anyhow!(
                "Type mismatch: expected {:?}, got {:?}",
                expected,
                value
            )),
        }
    }

    /// Infer the type of a value
    pub fn infer(&self, value: &Value) -> Type {
        match value {
            Value::String(_) => Type::String,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Bool(_) => Type::Bool,
            Value::Null => Type::Any,
            Value::List(items) => {
                if items.is_empty() {
                    Type::List(Box::new(Type::Any))
                } else {
                    Type::List(Box::new(self.infer(&items[0])))
                }
            }
            Value::Map(map) => {
                if map.is_empty() {
                    Type::Map(Box::new(Type::String), Box::new(Type::Any))
                } else {
                    let first_value = map.values().next().unwrap();
                    Type::Map(Box::new(Type::String), Box::new(self.infer(first_value)))
                }
            }
            Value::Function { .. } => Type::Any, // Functions have complex types
        }
    }

    /// Check type compatibility (can assign from -> to)
    pub fn is_compatible(&self, from: &Type, to: &Type) -> bool {
        match (from, to) {
            (_, Type::Any) => true,
            (Type::Any, _) => true,
            (a, b) if a == b => true,
            (Type::Int, Type::Float) => true, // Allow int -> float coercion
            (Type::List(a), Type::List(b)) => self.is_compatible(a, b),
            (Type::Map(k1, v1), Type::Map(k2, v2)) => {
                self.is_compatible(k1, k2) && self.is_compatible(v1, v2)
            }
            _ => false,
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_string() {
        let checker = TypeChecker::new();
        let value = Value::String("hello".to_string());
        assert!(checker.check(&value, &Type::String).is_ok());
        assert!(checker.check(&value, &Type::Int).is_err());
    }

    #[test]
    fn test_check_list() {
        let checker = TypeChecker::new();
        let value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert!(checker
            .check(&value, &Type::List(Box::new(Type::Int)))
            .is_ok());
        assert!(checker
            .check(&value, &Type::List(Box::new(Type::String)))
            .is_err());
    }

    #[test]
    fn test_infer_type() {
        let checker = TypeChecker::new();
        assert_eq!(
            checker.infer(&Value::String("test".to_string())),
            Type::String
        );
        assert_eq!(checker.infer(&Value::Int(42)), Type::Int);
        assert_eq!(checker.infer(&Value::Bool(true)), Type::Bool);
    }

    #[test]
    fn test_type_compatibility() {
        let checker = TypeChecker::new();
        assert!(checker.is_compatible(&Type::Int, &Type::Float));
        assert!(checker.is_compatible(&Type::String, &Type::Any));
        assert!(!checker.is_compatible(&Type::String, &Type::Int));
    }

    // Advanced type inference tests

    #[test]
    fn test_infer_binary_arithmetic() {
        let checker = TypeChecker::new();

        // 10 + 20
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(20),
                span: None,
            }),
            span: None,
        };

        assert_eq!(checker.infer_expression(&expr).unwrap(), Type::Int);
    }

    #[test]
    fn test_infer_binary_comparison() {
        let checker = TypeChecker::new();

        // 10 > 5
        let expr = Expression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(5),
                span: None,
            }),
            span: None,
        };

        assert_eq!(checker.infer_expression(&expr).unwrap(), Type::Bool);
    }

    #[test]
    fn test_infer_logical_and() {
        let checker = TypeChecker::new();

        // true and false
        let expr = Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Bool(false),
                span: None,
            }),
            span: None,
        };

        assert_eq!(checker.infer_expression(&expr).unwrap(), Type::Bool);
    }

    #[test]
    fn test_type_error_logical_with_non_bool() {
        let checker = TypeChecker::new();

        // 10 and 20 (should fail)
        let expr = Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Int(20),
                span: None,
            }),
            span: None,
        };

        assert!(checker.infer_expression(&expr).is_err());
    }

    #[test]
    fn test_infer_if_expression() {
        let checker = TypeChecker::new();

        // if true then 10 else 20
        let expr = Expression::If {
            condition: Box::new(Expression::Literal {
                value: Value::Bool(true),
                span: None,
            }),
            then_expr: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            else_expr: Some(Box::new(Expression::Literal {
                value: Value::Int(20),
                span: None,
            })),
            span: None,
        };

        assert_eq!(checker.infer_expression(&expr).unwrap(), Type::Int);
    }

    #[test]
    fn test_type_error_if_non_bool_condition() {
        let checker = TypeChecker::new();

        // if 10 then 20 else 30 (should fail)
        let expr = Expression::If {
            condition: Box::new(Expression::Literal {
                value: Value::Int(10),
                span: None,
            }),
            then_expr: Box::new(Expression::Literal {
                value: Value::Int(20),
                span: None,
            }),
            else_expr: Some(Box::new(Expression::Literal {
                value: Value::Int(30),
                span: None,
            })),
            span: None,
        };

        assert!(checker.infer_expression(&expr).is_err());
    }

    #[test]
    fn test_infer_list_index() {
        let checker = TypeChecker::new();

        // [10, 20, 30][0]
        let expr = Expression::Index {
            object: Box::new(Expression::List {
                elements: vec![
                    Expression::Literal {
                        value: Value::Int(10),
                        span: None,
                    },
                    Expression::Literal {
                        value: Value::Int(20),
                        span: None,
                    },
                    Expression::Literal {
                        value: Value::Int(30),
                        span: None,
                    },
                ],
                span: None,
            }),
            index: Box::new(Expression::Literal {
                value: Value::Int(0),
                span: None,
            }),
            span: None,
        };

        assert_eq!(checker.infer_expression(&expr).unwrap(), Type::Int);
    }

    #[test]
    fn test_type_error_list_index_with_string() {
        let checker = TypeChecker::new();

        // [10, 20]["invalid"] (should fail)
        let expr = Expression::Index {
            object: Box::new(Expression::List {
                elements: vec![Expression::Literal {
                    value: Value::Int(10),
                    span: None,
                }],
                span: None,
            }),
            index: Box::new(Expression::Literal {
                value: Value::String("invalid".to_string()),
                span: None,
            }),
            span: None,
        };

        assert!(checker.infer_expression(&expr).is_err());
    }

    #[test]
    fn test_check_module_simple_assignment() {
        // Test module with simple assignments (without type annotations)
        let module = Module {
            statements: vec![
                Statement::Assignment {
                    name: "x".to_string(),
                    mutable: false,
                    value: Expression::Literal {
                        value: Value::Int(42),
                        span: None,
                    },
                    type_annotation: None,
                    doc_comments: None,
                    span: None,
                },
                Statement::Assignment {
                    name: "y".to_string(),
                    mutable: false,
                    value: Expression::Literal {
                        value: Value::String("hello".to_string()),
                        span: None,
                    },
                    type_annotation: None,
                    doc_comments: None,
                    span: None,
                },
            ],
        };

        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_check_module_type_annotation_mismatch() {
        // Test type annotation mismatch
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "x".to_string(),
                mutable: false,
                value: Expression::Literal {
                    value: Value::Int(42),
                    span: None,
                },
                type_annotation: Some(Type::String), // Mismatch: expected String, got Int
                doc_comments: None,
                span: None,
            }],
        };

        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_err());
    }

    #[test]
    fn test_function_type_checking() {
        // Test function definition with type checking
        let module = Module {
            statements: vec![Statement::FunctionDef {
                name: "double".to_string(),
                params: vec![crate::ast::Parameter {
                    name: "x".to_string(),
                    param_type: Some(Type::Int),
                    default: None,
                }],
                return_type: Some(Type::Int),
                body: Expression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(Expression::Variable {
                        name: "x".to_string(),
                        span: None,
                    }),
                    right: Box::new(Expression::Literal {
                        value: Value::Int(2),
                        span: None,
                    }),
                    span: None,
                },
                doc_comments: None,
                span: None,
            }],
        };

        let mut checker = TypeChecker::new();
        assert!(checker.check_module(&module).is_ok());
    }

    #[test]
    fn test_builtin_function_type_checking() {
        let checker = TypeChecker::new();

        // upper("hello")
        let expr = Expression::FunctionCall {
            name: "upper".to_string(),
            args: vec![Expression::Literal {
                value: Value::String("hello".to_string()),
                span: None,
            }],
            span: None,
        };

        let result_type = checker.infer_expression(&expr).unwrap();
        assert_eq!(result_type, Type::String);
    }

    #[test]
    fn test_builtin_function_wrong_arg_type() {
        let checker = TypeChecker::new();

        // upper(42) - should fail
        let expr = Expression::FunctionCall {
            name: "upper".to_string(),
            args: vec![Expression::Literal {
                value: Value::Int(42),
                span: None,
            }],
            span: None,
        };

        assert!(checker.infer_expression(&expr).is_err());
    }
}
