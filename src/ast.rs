//! Abstract Syntax Tree definitions for JCL v1.0
//!
//! This module defines the AST nodes for the Jack-of-All Configuration Language.
//! JCL is a general-purpose configuration language, not IaC-specific.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "cli")]
use crate::lexer::Span;

/// Source location information (always available)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

#[cfg(feature = "cli")]
impl From<&Span> for SourceSpan {
    fn from(span: &Span) -> Self {
        SourceSpan {
            line: span.start.line,
            column: span.start.column,
            offset: span.start.offset,
            length: span.text.len(),
        }
    }
}

/// A JCL module (file)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub statements: Vec<Statement>,
}

/// Import kind - distinguishes between path-based and selective imports
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportKind {
    /// Import entire file: `import "path"` or `import "path" as alias`
    Full { alias: Option<String> },
    /// Import specific items: `import (item1, item2) from "path"`
    Selective { items: Vec<ImportItem> },
    /// Import everything: `import * from "path"`
    Wildcard,
}

/// Individual import item with optional alias
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
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
        doc_comments: Option<Vec<String>>,
        span: Option<SourceSpan>,
    },

    /// Function definition: `fn name(params) = expr`
    FunctionDef {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Expression,
        doc_comments: Option<Vec<String>>,
        span: Option<SourceSpan>,
    },

    /// Import statement: supports both patterns:
    /// - Path-based: `import "path" as alias` or `import "path"`
    /// - Selective: `import (items) from "path"` or `import * from "path"`
    Import {
        path: String,
        kind: ImportKind,
        doc_comments: Option<Vec<String>>,
        span: Option<SourceSpan>,
    },

    /// For loop: `for x in list (body)`
    ForLoop {
        variables: Vec<String>, // Can have multiple for Cartesian product
        iterables: Vec<Expression>,
        body: Vec<Statement>,
        condition: Option<Expression>, // Optional filter condition
        doc_comments: Option<Vec<String>>,
        span: Option<SourceSpan>,
    },

    /// Expression statement (for side effects)
    Expression {
        expr: Expression,
        span: Option<SourceSpan>,
    },

    /// Module interface declaration: `module.interface = (...)`
    ModuleInterface {
        inputs: HashMap<String, ModuleInput>,
        outputs: HashMap<String, ModuleOutput>,
        doc_comments: Option<Vec<String>>,
        span: Option<SourceSpan>,
    },

    /// Module outputs declaration: `module.outputs = (...)`
    ModuleOutputs {
        outputs: HashMap<String, Expression>,
        doc_comments: Option<Vec<String>>,
        span: Option<SourceSpan>,
    },

    /// Module instantiation: `module.<type>.<instance> = (source = "...", ...)`
    ModuleInstance {
        module_type: String,
        instance_name: String,
        source: String,
        inputs: HashMap<String, Expression>,
        doc_comments: Option<Vec<String>>,
        span: Option<SourceSpan>,
    },
}

/// Module input parameter definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleInput {
    pub input_type: Type,
    pub required: bool,
    pub default: Option<Expression>,
    pub description: Option<String>,
}

/// Module output definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleOutput {
    pub output_type: Type,
    pub description: Option<String>,
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
    Literal {
        value: Value,
        span: Option<SourceSpan>,
    },

    /// Variable reference
    Variable {
        name: String,
        span: Option<SourceSpan>,
    },

    /// Member access: `obj.field`
    MemberAccess {
        object: Box<Expression>,
        field: String,
        span: Option<SourceSpan>,
    },

    /// Optional chaining: `obj?.field`
    OptionalChain {
        object: Box<Expression>,
        field: String,
        span: Option<SourceSpan>,
    },

    /// Index access: `list[0]` or `map["key"]`
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        span: Option<SourceSpan>,
    },

    /// Slice access: `list[start:end]` or `list[start:end:step]`
    Slice {
        object: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        step: Option<Box<Expression>>,
        span: Option<SourceSpan>,
    },

    /// Range expression: `[start..end]`, `[start..end:step]`, `[start..<end]`
    /// Generates a sequence of values from start to end
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        step: Option<Box<Expression>>,
        inclusive: bool, // true for `..`, false for `..<`
        span: Option<SourceSpan>,
    },

    /// Function call: `func(args)`
    FunctionCall {
        name: String,
        args: Vec<Expression>,
        span: Option<SourceSpan>,
    },

    /// Method call: `obj.method(args)` or pipeline: `obj | func`
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
        span: Option<SourceSpan>,
    },

    /// Binary operation: `a + b`, `a == b`, etc.
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Option<SourceSpan>,
    },

    /// Unary operation: `!a`, `-a`
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
        span: Option<SourceSpan>,
    },

    /// Ternary conditional: `condition ? then_expr : else_expr`
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
        span: Option<SourceSpan>,
    },

    /// If expression: `if condition then expr else expr`
    If {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Option<Box<Expression>>,
        span: Option<SourceSpan>,
    },

    /// When expression (pattern matching): `when value (pattern => expr, ...)`
    When {
        value: Box<Expression>,
        arms: Vec<WhenArm>,
        span: Option<SourceSpan>,
    },

    /// Lambda function: `x => x * 2` or `(x, y) => x + y`
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expression>,
        span: Option<SourceSpan>,
    },

    /// List comprehension: `[expr for x in list for y in list2 if condition]`
    /// Supports multiple for clauses for flattening/nested iteration
    ListComprehension {
        expr: Box<Expression>,
        /// Vector of (variable, iterable) pairs for each for clause
        iterators: Vec<(String, Expression)>,
        condition: Option<Box<Expression>>,
        span: Option<SourceSpan>,
    },

    /// Pipeline: `value | func1 | func2`
    Pipeline {
        stages: Vec<Expression>,
        span: Option<SourceSpan>,
    },

    /// Try expression: `try(expr)`
    Try {
        expr: Box<Expression>,
        default: Option<Box<Expression>>,
        span: Option<SourceSpan>,
    },

    /// String with interpolation: `"Hello, ${name}!"`
    InterpolatedString {
        parts: Vec<StringPart>,
        span: Option<SourceSpan>,
    },

    /// List literal: `[1, 2, 3]`
    List {
        elements: Vec<Expression>,
        span: Option<SourceSpan>,
    },

    /// Map literal: `(key = value, ...)`
    Map {
        entries: Vec<(String, Expression)>,
        span: Option<SourceSpan>,
    },

    /// Spread operator: `...list`
    Spread {
        expr: Box<Expression>,
        span: Option<SourceSpan>,
    },

    /// Splat operator: `list[*]` for extracting attributes from all elements
    Splat {
        object: Box<Expression>,
        span: Option<SourceSpan>,
    },
}

impl Expression {
    /// Get the span of this expression, if available
    pub fn span(&self) -> Option<&SourceSpan> {
        match self {
            Expression::Literal { span, .. } => span.as_ref(),
            Expression::Variable { span, .. } => span.as_ref(),
            Expression::MemberAccess { span, .. } => span.as_ref(),
            Expression::OptionalChain { span, .. } => span.as_ref(),
            Expression::Index { span, .. } => span.as_ref(),
            Expression::Slice { span, .. } => span.as_ref(),
            Expression::Range { span, .. } => span.as_ref(),
            Expression::FunctionCall { span, .. } => span.as_ref(),
            Expression::MethodCall { span, .. } => span.as_ref(),
            Expression::BinaryOp { span, .. } => span.as_ref(),
            Expression::UnaryOp { span, .. } => span.as_ref(),
            Expression::Ternary { span, .. } => span.as_ref(),
            Expression::If { span, .. } => span.as_ref(),
            Expression::When { span, .. } => span.as_ref(),
            Expression::Lambda { span, .. } => span.as_ref(),
            Expression::ListComprehension { span, .. } => span.as_ref(),
            Expression::Pipeline { span, .. } => span.as_ref(),
            Expression::Try { span, .. } => span.as_ref(),
            Expression::InterpolatedString { span, .. } => span.as_ref(),
            Expression::List { span, .. } => span.as_ref(),
            Expression::Map { span, .. } => span.as_ref(),
            Expression::Spread { span, .. } => span.as_ref(),
            Expression::Splat { span, .. } => span.as_ref(),
        }
    }
}

impl Statement {
    /// Get the span of this statement, if available
    pub fn span(&self) -> Option<&SourceSpan> {
        match self {
            Statement::Assignment { span, .. } => span.as_ref(),
            Statement::FunctionDef { span, .. } => span.as_ref(),
            Statement::Import { span, .. } => span.as_ref(),
            Statement::ForLoop { span, .. } => span.as_ref(),
            Statement::Expression { span, .. } => span.as_ref(),
            Statement::ModuleInterface { span, .. } => span.as_ref(),
            Statement::ModuleOutputs { span, .. } => span.as_ref(),
            Statement::ModuleInstance { span, .. } => span.as_ref(),
        }
    }
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
        body: Box<Expression>,
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
            Type::Function {
                params,
                return_type,
            } => {
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
