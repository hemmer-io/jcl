//! Abstract Syntax Tree definitions for JCL

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A JCL module (file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub environments: Vec<Environment>,
    pub stacks: Vec<Stack>,
    pub resources: Vec<Resource>,
    pub data_sources: Vec<DataSource>,
    pub variables: Vec<Variable>,
    pub outputs: Vec<Output>,
}

/// Environment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub region: Option<String>,
    pub variables: HashMap<String, Value>,
    pub tags: HashMap<String, String>,
    pub providers: Vec<Provider>,
}

/// Stack definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub name: String,
    pub environment: Option<String>,
    pub depends_on: Vec<String>,
    pub variables: HashMap<String, Value>,
    pub resources: Vec<Resource>,
}

/// Resource definition (managed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: String,
    pub name: String,
    pub attributes: HashMap<String, Value>,
    pub configuration: Option<Configuration>,
    pub lifecycle: Lifecycle,
    pub depends_on: Vec<String>,
}

/// Data source (read-only reference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub data_type: String,
    pub name: String,
    pub filters: HashMap<String, Value>,
}

/// Configuration block for config management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub tasks: Vec<ConfigTask>,
}

/// Configuration task (Ansible-like)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigTask {
    Package {
        name: String,
        state: PackageState,
        version: Option<String>,
    },
    File {
        path: String,
        content: Option<String>,
        source: Option<String>,
        mode: Option<String>,
        owner: Option<String>,
        group: Option<String>,
    },
    Service {
        name: String,
        state: ServiceState,
        enabled: bool,
    },
    Command {
        command: String,
        creates: Option<String>,
        unless: Option<String>,
    },
    GitClone {
        repo: String,
        dest: String,
        version: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageState {
    Present,
    Absent,
    Latest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceState {
    Running,
    Stopped,
    Reloaded,
    Restarted,
}

/// Variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value_type: Option<Type>,
    pub default: Option<Value>,
    pub description: Option<String>,
    pub validation: Option<Validation>,
}

/// Output definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub name: String,
    pub value: Expression,
    pub description: Option<String>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub config: HashMap<String, Value>,
}

/// Resource lifecycle settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub managed: bool,
    pub create_before_destroy: bool,
    pub prevent_destroy: bool,
    pub ignore_changes: Vec<String>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            managed: true,
            create_before_destroy: false,
            prevent_destroy: false,
            ignore_changes: vec![],
        }
    }
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub condition: Expression,
    pub error_message: String,
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
    Object(HashMap<String, Type>),
    Any,
}

/// Value representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Null,
    Expression(Expression),
}

/// Expression (for computed values)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Reference(String),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    ForEach {
        variable: String,
        collection: Box<Expression>,
        body: Box<Expression>,
    },
    Pipeline {
        stages: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
}
