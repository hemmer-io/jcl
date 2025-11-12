//! Abstract Syntax Tree definitions for JCL v1.0
//!
//! This module defines the AST nodes for the Jack-of-All Configuration Language.
//! JCL is a general-purpose configuration language, not IaC-specific.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A JCL module (file)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub statements: Vec<Statement>,
}

/// Top-level statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    /// Variable assignment: `name = value` or `mut name = value`
    Assignment {
        name: String,
        mutable: bool,
        value: Expression,
        type_annotation: Option<Type>,
    },

    /// Function definition: `fn name(params) = expr`
    FunctionDef {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Expression,
    },

    /// Import statement: `import (items) from "path"`
    Import {
        items: Vec<String>,
        path: String,
        wildcard: bool, // true for `import * from "path"`
    },

    /// For loop: `for x in list (body)`
    ForLoop {
        variables: Vec<String>, // Can have multiple for Cartesian product
        iterables: Vec<Expression>,
        body: Vec<Statement>,
        condition: Option<Expression>, // Optional filter condition
    },

    /// Expression statement (for side effects)
    Expression(Expression),
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>,
    pub default: Option<Expression>,
}

/// Expression (produces a value)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// Literal value
    Literal(Value),

    /// Variable reference
    Variable(String),

    /// Member access: `obj.field`
    MemberAccess {
        object: Box<Expression>,
        field: String,
    },

    /// Optional chaining: `obj?.field`
    OptionalChain {
        object: Box<Expression>,
        field: String,
    },

    /// Index access: `list[0]` or `map["key"]`
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    /// Function call: `func(args)`
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },

    /// Method call: `obj.method(args)` or pipeline: `obj | func`
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },

    /// Binary operation: `a + b`, `a == b`, etc.
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Unary operation: `!a`, `-a`
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },

    /// Ternary conditional: `condition ? then_expr : else_expr`
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },

    /// If expression: `if condition then expr else expr`
    If {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Option<Box<Expression>>,
    },

    /// When expression (pattern matching): `when value (pattern => expr, ...)`
    When {
        value: Box<Expression>,
        arms: Vec<WhenArm>,
    },

    /// Lambda function: `x => x * 2` or `(x, y) => x + y`
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expression>,
    },

    /// List comprehension: `[expr for x in list if condition]`
    ListComprehension {
        expr: Box<Expression>,
        variable: String,
        iterable: Box<Expression>,
        condition: Option<Box<Expression>>,
    },

    /// Pipeline: `value | func1 | func2`
    Pipeline {
        stages: Vec<Expression>,
    },

    /// Try expression: `try(expr)`
    Try {
        expr: Box<Expression>,
        default: Option<Box<Expression>>,
    },

    /// String with interpolation: `"Hello, ${name}!"`
    InterpolatedString {
        parts: Vec<StringPart>,
    },

    /// List literal: `[1, 2, 3]`
    List(Vec<Expression>),

    /// Map literal: `(key = value, ...)`
    Map(Vec<(String, Expression)>),

    /// Spread operator: `...list`
    Spread(Box<Expression>),
}

/// String part (literal or interpolation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    Literal(String),
    Interpolation(Box<Expression>),
}

/// When arm for pattern matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhenArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub expr: Expression,
}

/// Pattern for when expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Literal match: `"prod"`
    Literal(Value),

    /// Tuple match: `("prod", "us-west-2")`
    Tuple(Vec<Pattern>),

    /// Variable binding: `x` (binds to variable)
    Variable(String),

    /// Wildcard: `*`
    Wildcard,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // **

    // Comparison
    Equal,              // ==
    NotEqual,           // !=
    LessThan,           // <
    LessThanOrEqual,    // <=
    GreaterThan,        // >
    GreaterThanOrEqual, // >=

    // Logical
    And, // and
    Or,  // or

    // Null coalescing
    NullCoalesce, // ??

    // String concatenation
    Concat, // ++
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,    // !
    Negate, // -
}

/// Value representation (runtime values)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Function {
        params: Vec<Parameter>,
        body: Expression,
    },
    Null,
}

impl Value {
    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Convert to string representation
    pub fn to_string_repr(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::List(items) => {
                let strs: Vec<_> = items.iter().map(|v| v.to_string_repr()).collect();
                format!("[{}]", strs.join(", "))
            }
            Value::Map(m) => {
                let pairs: Vec<_> = m
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, v.to_string_repr()))
                    .collect();
                format!("({})", pairs.join(", "))
            }
            Value::Function { .. } => "<function>".to_string(),
        }
    }

    /// Get type of this value
    pub fn get_type(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::Bool(_) => Type::Bool,
            Value::List(items) => {
                if items.is_empty() {
                    Type::List(Box::new(Type::Any))
                } else {
                    Type::List(Box::new(items[0].get_type()))
                }
            }
            Value::Map(_) => Type::Map(Box::new(Type::String), Box::new(Type::Any)),
            Value::Function { params, .. } => {
                let param_types = params
                    .iter()
                    .map(|p| p.param_type.clone().unwrap_or(Type::Any))
                    .collect();
                Type::Function {
                    params: param_types,
                    return_type: Box::new(Type::Any),
                }
            }
            Value::Null => Type::Null,
        }
    }
}

/// Type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    String,
    Int,
    Float,
    Bool,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Null,
    Any,
}

impl Type {
    /// Check if this type is nullable (can be null)
    pub fn is_nullable(&self) -> bool {
        matches!(self, Type::Any | Type::Null)
    }

    /// Create a nullable version of this type
    pub fn nullable(self) -> Type {
        // For now, we'll use Any to represent nullable types
        // A more sophisticated approach would be Type::Optional(Box<Type>)
        Type::Any
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "string"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::List(inner) => write!(f, "list<{}>", inner),
            Type::Map(k, v) => write!(f, "map<{}, {}>", k, v),
            Type::Function { params, return_type } => {
                let param_strs: Vec<_> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "fn({}) -> {}", param_strs.join(", "), return_type)
            }
            Type::Null => write!(f, "null"),
            Type::Any => write!(f, "any"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::String("hello".to_string()).to_string_repr(), "hello");
        assert_eq!(Value::Int(42).to_string_repr(), "42");
        assert_eq!(Value::Bool(true).to_string_repr(), "true");
        assert_eq!(Value::Null.to_string_repr(), "null");
    }

    #[test]
    fn test_value_get_type() {
        assert_eq!(Value::String("hello".to_string()).get_type(), Type::String);
        assert_eq!(Value::Int(42).get_type(), Type::Int);
        assert_eq!(Value::Bool(true).get_type(), Type::Bool);
        assert_eq!(Value::Null.get_type(), Type::Null);
    }

    #[test]
    fn test_value_is_null() {
        assert!(Value::Null.is_null());
        assert!(!Value::Int(0).is_null());
    }
}
