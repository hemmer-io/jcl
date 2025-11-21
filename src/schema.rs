//! Schema validation for JCL configurations
//!
//! This module provides schema definition and validation capabilities for JCL.
//! Schemas define the expected structure, types, and constraints for configuration values.

use crate::ast::{Module, Value};
use crate::evaluator::Evaluator;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema definition for JCL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema version
    #[serde(default = "default_version")]
    pub version: String,

    /// Schema title
    pub title: Option<String>,

    /// Schema description
    pub description: Option<String>,

    /// Root type definition
    #[serde(rename = "type")]
    pub type_def: TypeDef,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// Type definition for schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TypeDef {
    /// String type
    String {
        #[serde(default)]
        min_length: Option<usize>,
        #[serde(default)]
        max_length: Option<usize>,
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default)]
        enum_values: Option<Vec<String>>,
    },

    /// Number type (int or float)
    Number {
        #[serde(default)]
        minimum: Option<f64>,
        #[serde(default)]
        maximum: Option<f64>,
        #[serde(default)]
        integer_only: bool,
    },

    /// Boolean type
    Boolean,

    /// Null type
    Null,

    /// List type
    List {
        items: Box<TypeDef>,
        #[serde(default)]
        min_items: Option<usize>,
        #[serde(default)]
        max_items: Option<usize>,
    },

    /// Map type (object with string keys)
    Map {
        properties: HashMap<String, Property>,
        #[serde(default)]
        required: Vec<String>,
        #[serde(default)]
        additional_properties: bool,
    },

    /// Any type (no validation)
    Any,

    /// Union of multiple types
    Union { types: Vec<TypeDef> },
}

/// Property definition for map types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "type")]
    pub type_def: TypeDef,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub default: Option<Value>,
}

/// Error type categorization for validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    /// Required field is missing
    Required,
    /// Type mismatch (expected vs found)
    TypeMismatch,
    /// Constraint violation (min/max, pattern, enum)
    ConstraintViolation,
    /// Custom validation error
    Custom,
}

/// Validation error with rich context
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Path to the field (e.g., "database.port")
    pub path: String,
    /// Field name (last component of path)
    pub field: String,
    /// Error type category
    pub error_type: ErrorType,
    /// Human-readable error message
    pub message: String,
    /// Optional suggestion for fixing the error
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(path: String, error_type: ErrorType, message: String) -> Self {
        let field = path.split('.').last().unwrap_or(&path).to_string();
        Self {
            path,
            field,
            error_type,
            message,
            suggestion: None,
        }
    }

    /// Create a required field error
    pub fn required(path: String, field: String) -> Self {
        Self {
            path: path.clone(),
            field: field.clone(),
            error_type: ErrorType::Required,
            message: format!("Required property '{}' is missing", field),
            suggestion: None,
        }
    }

    /// Create a type mismatch error
    pub fn type_mismatch(path: String, expected: &str, found: &str) -> Self {
        let field = path.split('.').last().unwrap_or(&path).to_string();
        Self {
            path: path.clone(),
            field,
            error_type: ErrorType::TypeMismatch,
            message: format!("Expected {}, got {}", expected, found),
            suggestion: None,
        }
    }

    /// Create a constraint violation error
    pub fn constraint_violation(path: String, message: String) -> Self {
        let field = path.split('.').last().unwrap_or(&path).to_string();
        Self {
            path,
            field,
            error_type: ErrorType::ConstraintViolation,
            message,
            suggestion: None,
        }
    }

    /// Create a custom validation error
    pub fn custom(message: String) -> Self {
        Self {
            path: String::new(),
            field: String::new(),
            error_type: ErrorType::Custom,
            message,
            suggestion: None,
        }
    }

    /// Add a suggestion to help fix the error
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Set the path for this error
    pub fn with_path(mut self, path: String) -> Self {
        self.field = path.split('.').last().unwrap_or(&path).to_string();
        self.path = path;
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " (Suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Builder for constructing Property definitions programmatically
#[derive(Debug, Clone)]
pub struct PropertyBuilder {
    type_def: TypeDef,
    description: Option<String>,
    default: Option<Value>,
}

impl PropertyBuilder {
    /// Create a new property builder with a type definition
    pub fn new(type_def: TypeDef) -> Self {
        Self {
            type_def,
            description: None,
            default: None,
        }
    }

    /// Set the description for this property
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set a default value for this property
    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Build the Property
    pub fn build(self) -> Property {
        Property {
            type_def: self.type_def,
            description: self.description,
            default: self.default,
        }
    }
}

/// Builder for constructing Schema definitions programmatically
#[derive(Debug, Clone)]
pub struct SchemaBuilder {
    title: Option<String>,
    description: Option<String>,
    properties: HashMap<String, PropertyBuilder>,
    required: Vec<String>,
    additional_properties: bool,
}

impl SchemaBuilder {
    /// Create a new schema builder with a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            description: None,
            properties: HashMap::new(),
            required: Vec::new(),
            additional_properties: false,
        }
    }

    /// Set the schema description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a required field to the schema
    pub fn required_field(mut self, name: impl Into<String>, type_def: TypeDef) -> Self {
        let field_name = name.into();
        self.required.push(field_name.clone());
        self.properties
            .insert(field_name, PropertyBuilder::new(type_def));
        self
    }

    /// Add an optional field to the schema
    pub fn optional_field(mut self, name: impl Into<String>, type_def: TypeDef) -> Self {
        let field_name = name.into();
        self.properties
            .insert(field_name, PropertyBuilder::new(type_def));
        self
    }

    /// Add a field with a PropertyBuilder for more complex configuration
    pub fn field(mut self, name: impl Into<String>, property: PropertyBuilder) -> Self {
        self.properties.insert(name.into(), property);
        self
    }

    /// Mark a field as required
    pub fn mark_required(mut self, name: impl Into<String>) -> Self {
        let field_name = name.into();
        if !self.required.contains(&field_name) {
            self.required.push(field_name);
        }
        self
    }

    /// Allow additional properties not defined in the schema
    pub fn allow_additional_properties(mut self, allow: bool) -> Self {
        self.additional_properties = allow;
        self
    }

    /// Build the Schema
    pub fn build(self) -> Schema {
        Schema {
            version: "1.0".to_string(),
            title: self.title,
            description: self.description,
            type_def: TypeDef::Map {
                properties: self
                    .properties
                    .into_iter()
                    .map(|(k, v)| (k, v.build()))
                    .collect(),
                required: self.required,
                additional_properties: self.additional_properties,
            },
        }
    }
}

/// Schema validator
pub struct Validator {
    schema: Schema,
}

impl Validator {
    /// Create a new validator from a schema
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    /// Create a validator from a SchemaBuilder (programmatic API)
    pub fn from_builder(builder: SchemaBuilder) -> Self {
        Self::new(builder.build())
    }

    /// Load schema from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        let schema: Schema = serde_json::from_str(json).context("Failed to parse schema JSON")?;
        Ok(Self::new(schema))
    }

    /// Load schema from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let schema: Schema = serde_yaml::from_str(yaml).context("Failed to parse schema YAML")?;
        Ok(Self::new(schema))
    }

    /// Validate a JCL module against the schema
    pub fn validate_module(&self, module: &Module) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Evaluate the module to get variable bindings
        let mut evaluator = Evaluator::new();
        let evaluated = evaluator.evaluate(module.clone())?;

        // Use the bindings from the evaluated module
        let root_value = Value::Map(evaluated.bindings.clone());

        self.validate_value(&root_value, &self.schema.type_def, "", &mut errors);

        Ok(errors)
    }

    /// Validate a single value against a type definition
    fn validate_value(
        &self,
        value: &Value,
        type_def: &TypeDef,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        match type_def {
            TypeDef::String {
                min_length,
                max_length,
                pattern,
                enum_values,
            } => match value {
                Value::String(s) => {
                    if let Some(min) = min_length {
                        if s.len() < *min {
                            errors.push(ValidationError::constraint_violation(
                                path.to_string(),
                                format!("String length {} is less than minimum {}", s.len(), min),
                            ));
                        }
                    }
                    if let Some(max) = max_length {
                        if s.len() > *max {
                            errors.push(ValidationError::constraint_violation(
                                path.to_string(),
                                format!("String length {} exceeds maximum {}", s.len(), max),
                            ));
                        }
                    }
                    if let Some(pat) = pattern {
                        if let Ok(regex) = regex::Regex::new(pat) {
                            if !regex.is_match(s) {
                                errors.push(ValidationError::constraint_violation(
                                    path.to_string(),
                                    format!("String '{}' does not match pattern '{}'", s, pat),
                                ));
                            }
                        }
                    }
                    if let Some(enum_vals) = enum_values {
                        if !enum_vals.contains(s) {
                            errors.push(ValidationError::constraint_violation(
                                path.to_string(),
                                format!(
                                    "String '{}' is not one of allowed values: {:?}",
                                    s, enum_vals
                                ),
                            ));
                        }
                    }
                }
                _ => {
                    let found = match value {
                        Value::Int(_) => "integer",
                        Value::Float(_) => "float",
                        Value::Bool(_) => "boolean",
                        Value::List(_) => "list",
                        Value::Map(_) => "map",
                        Value::Null => "null",
                        _ => "unknown",
                    };
                    errors.push(ValidationError::type_mismatch(
                        path.to_string(),
                        "string",
                        found,
                    ));
                }
            },

            TypeDef::Number {
                minimum,
                maximum,
                integer_only,
            } => {
                let num = match value {
                    Value::Int(i) => Some(*i as f64),
                    Value::Float(f) => Some(*f),
                    _ => None,
                };

                if let Some(n) = num {
                    if *integer_only {
                        if let Value::Float(_) = value {
                            errors.push(ValidationError::type_mismatch(
                                path.to_string(),
                                "integer",
                                "float",
                            ));
                        }
                    }
                    if let Some(min) = minimum {
                        if n < *min {
                            errors.push(ValidationError::constraint_violation(
                                path.to_string(),
                                format!("Number {} is less than minimum {}", n, min),
                            ));
                        }
                    }
                    if let Some(max) = maximum {
                        if n > *max {
                            errors.push(ValidationError::constraint_violation(
                                path.to_string(),
                                format!("Number {} exceeds maximum {}", n, max),
                            ));
                        }
                    }
                } else {
                    let found = match value {
                        Value::String(_) => "string",
                        Value::Bool(_) => "boolean",
                        Value::List(_) => "list",
                        Value::Map(_) => "map",
                        Value::Null => "null",
                        _ => "unknown",
                    };
                    errors.push(ValidationError::type_mismatch(
                        path.to_string(),
                        "number",
                        found,
                    ));
                }
            }

            TypeDef::Boolean => {
                if !matches!(value, Value::Bool(_)) {
                    let found = match value {
                        Value::String(_) => "string",
                        Value::Int(_) => "integer",
                        Value::Float(_) => "float",
                        Value::List(_) => "list",
                        Value::Map(_) => "map",
                        Value::Null => "null",
                        _ => "unknown",
                    };
                    errors.push(ValidationError::type_mismatch(
                        path.to_string(),
                        "boolean",
                        found,
                    ));
                }
            }

            TypeDef::Null => {
                if !matches!(value, Value::Null) {
                    let found = match value {
                        Value::String(_) => "string",
                        Value::Int(_) => "integer",
                        Value::Float(_) => "float",
                        Value::Bool(_) => "boolean",
                        Value::List(_) => "list",
                        Value::Map(_) => "map",
                        _ => "unknown",
                    };
                    errors.push(ValidationError::type_mismatch(
                        path.to_string(),
                        "null",
                        found,
                    ));
                }
            }

            TypeDef::List {
                items,
                min_items,
                max_items,
            } => match value {
                Value::List(list) => {
                    if let Some(min) = min_items {
                        if list.len() < *min {
                            errors.push(ValidationError::constraint_violation(
                                path.to_string(),
                                format!("List length {} is less than minimum {}", list.len(), min),
                            ));
                        }
                    }
                    if let Some(max) = max_items {
                        if list.len() > *max {
                            errors.push(ValidationError::constraint_violation(
                                path.to_string(),
                                format!("List length {} exceeds maximum {}", list.len(), max),
                            ));
                        }
                    }

                    for (i, item) in list.iter().enumerate() {
                        let item_path = format!("{}[{}]", path, i);
                        self.validate_value(item, items, &item_path, errors);
                    }
                }
                _ => {
                    let found = match value {
                        Value::String(_) => "string",
                        Value::Int(_) => "integer",
                        Value::Float(_) => "float",
                        Value::Bool(_) => "boolean",
                        Value::Map(_) => "map",
                        Value::Null => "null",
                        _ => "unknown",
                    };
                    errors.push(ValidationError::type_mismatch(
                        path.to_string(),
                        "list",
                        found,
                    ));
                }
            },

            TypeDef::Map {
                properties,
                required,
                additional_properties,
            } => {
                match value {
                    Value::Map(map) => {
                        // Check required properties
                        for req in required {
                            if !map.contains_key(req) {
                                let prop_path = if path.is_empty() {
                                    req.clone()
                                } else {
                                    format!("{}.{}", path, req)
                                };
                                errors.push(ValidationError::required(prop_path, req.clone()));
                            }
                        }

                        // Validate each property
                        for (key, val) in map {
                            let prop_path = if path.is_empty() {
                                key.clone()
                            } else {
                                format!("{}.{}", path, key)
                            };

                            if let Some(prop) = properties.get(key) {
                                self.validate_value(val, &prop.type_def, &prop_path, errors);
                            } else if !additional_properties {
                                errors.push(ValidationError::constraint_violation(
                                    prop_path,
                                    format!("Unknown property '{}'", key),
                                ));
                            }
                        }
                    }
                    _ => {
                        let found = match value {
                            Value::String(_) => "string",
                            Value::Int(_) => "integer",
                            Value::Float(_) => "float",
                            Value::Bool(_) => "boolean",
                            Value::List(_) => "list",
                            Value::Null => "null",
                            _ => "unknown",
                        };
                        errors.push(ValidationError::type_mismatch(
                            path.to_string(),
                            "map",
                            found,
                        ));
                    }
                }
            }

            TypeDef::Any => {
                // Any type - no validation needed
            }

            TypeDef::Union { types } => {
                // Try to validate against each type in the union
                let mut all_errors = Vec::new();
                let mut valid = false;

                for ty in types {
                    let mut type_errors = Vec::new();
                    self.validate_value(value, ty, path, &mut type_errors);

                    if type_errors.is_empty() {
                        valid = true;
                        break;
                    }
                    all_errors.push(type_errors);
                }

                if !valid {
                    errors.push(ValidationError::constraint_violation(
                        path.to_string(),
                        "Value does not match any type in union".to_string(),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expression, Statement};

    #[test]
    fn test_string_validation() {
        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "name".to_string(),
                        Property {
                            type_def: TypeDef::String {
                                min_length: Some(3),
                                max_length: Some(10),
                                pattern: None,
                                enum_values: None,
                            },
                            description: None,
                            default: None,
                        },
                    );
                    props
                },
                required: vec![],
                additional_properties: true,
            },
        };

        let validator = Validator::new(schema);
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "name".to_string(),
                value: Expression::Literal {
                    value: Value::String("hello".to_string()),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };

        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_number_validation() {
        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "port".to_string(),
                        Property {
                            type_def: TypeDef::Number {
                                minimum: Some(1.0),
                                maximum: Some(65535.0),
                                integer_only: true,
                            },
                            description: None,
                            default: None,
                        },
                    );
                    props
                },
                required: vec!["port".to_string()],
                additional_properties: false,
            },
        };

        let validator = Validator::new(schema);
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "port".to_string(),
                value: Expression::Literal {
                    value: Value::Int(8080),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };

        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_required_property_missing() {
        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: HashMap::new(),
                required: vec!["name".to_string()],
                additional_properties: true,
            },
        };

        let validator = Validator::new(schema);
        let module = Module { statements: vec![] };

        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .message
            .contains("Required property 'name' is missing"));
    }

    // ========== Builder API Tests ==========

    #[test]
    fn test_schema_builder_basic() {
        let schema = SchemaBuilder::new("test_schema")
            .description("A test schema")
            .required_field(
                "name",
                TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                },
            )
            .optional_field(
                "age",
                TypeDef::Number {
                    minimum: Some(0.0),
                    maximum: Some(150.0),
                    integer_only: true,
                },
            )
            .build();

        assert_eq!(schema.title, Some("test_schema".to_string()));
        assert_eq!(schema.description, Some("A test schema".to_string()));

        match schema.type_def {
            TypeDef::Map {
                properties,
                required,
                ..
            } => {
                assert_eq!(required.len(), 1);
                assert!(required.contains(&"name".to_string()));
                assert_eq!(properties.len(), 2);
                assert!(properties.contains_key("name"));
                assert!(properties.contains_key("age"));
            }
            _ => panic!("Expected Map type_def"),
        }
    }

    #[test]
    fn test_validator_from_builder() {
        let builder = SchemaBuilder::new("app_config")
            .required_field(
                "app_name",
                TypeDef::String {
                    min_length: Some(1),
                    max_length: Some(50),
                    pattern: None,
                    enum_values: None,
                },
            )
            .required_field(
                "port",
                TypeDef::Number {
                    minimum: Some(1.0),
                    maximum: Some(65535.0),
                    integer_only: true,
                },
            )
            .allow_additional_properties(false);

        let validator = Validator::from_builder(builder);

        // Valid configuration
        let valid_module = Module {
            statements: vec![
                Statement::Assignment {
                    name: "app_name".to_string(),
                    value: Expression::Literal {
                        value: Value::String("my-app".to_string()),
                        span: None,
                    },
                    type_annotation: None,
                    mutable: false,
                    doc_comments: Some(vec![]),
                    span: None,
                },
                Statement::Assignment {
                    name: "port".to_string(),
                    value: Expression::Literal {
                        value: Value::Int(8080),
                        span: None,
                    },
                    type_annotation: None,
                    mutable: false,
                    doc_comments: Some(vec![]),
                    span: None,
                },
            ],
        };

        let errors = validator.validate_module(&valid_module).unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_enhanced_error_types() {
        let builder = SchemaBuilder::new("test").required_field(
            "name",
            TypeDef::String {
                min_length: Some(3),
                max_length: None,
                pattern: None,
                enum_values: None,
            },
        );

        let validator = Validator::from_builder(builder);

        // Missing required field
        let module = Module { statements: vec![] };
        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, ErrorType::Required);
        assert_eq!(errors[0].field, "name");

        // Type mismatch
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "name".to_string(),
                value: Expression::Literal {
                    value: Value::Int(42),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };
        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, ErrorType::TypeMismatch);
        assert!(errors[0].message.contains("Expected string"));

        // Constraint violation
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "name".to_string(),
                value: Expression::Literal {
                    value: Value::String("ab".to_string()), // Too short
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };
        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, ErrorType::ConstraintViolation);
        assert!(errors[0].message.contains("less than minimum"));
    }

    #[test]
    fn test_property_builder_with_description() {
        let property = PropertyBuilder::new(TypeDef::String {
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
        })
        .description("User's full name")
        .default_value(Value::String("Anonymous".to_string()))
        .build();

        assert_eq!(property.description, Some("User's full name".to_string()));
        assert_eq!(
            property.default,
            Some(Value::String("Anonymous".to_string()))
        );
    }

    #[test]
    fn test_builder_with_nested_map() {
        let builder = SchemaBuilder::new("config").required_field(
            "database",
            TypeDef::Map {
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "host".to_string(),
                        Property {
                            type_def: TypeDef::String {
                                min_length: None,
                                max_length: None,
                                pattern: None,
                                enum_values: None,
                            },
                            description: None,
                            default: None,
                        },
                    );
                    props.insert(
                        "port".to_string(),
                        Property {
                            type_def: TypeDef::Number {
                                minimum: Some(1.0),
                                maximum: Some(65535.0),
                                integer_only: true,
                            },
                            description: None,
                            default: None,
                        },
                    );
                    props
                },
                required: vec!["host".to_string(), "port".to_string()],
                additional_properties: false,
            },
        );

        let validator = Validator::from_builder(builder);

        // Valid nested config
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "database".to_string(),
                value: Expression::Literal {
                    value: Value::Map({
                        let mut map = HashMap::new();
                        map.insert("host".to_string(), Value::String("localhost".to_string()));
                        map.insert("port".to_string(), Value::Int(5432));
                        map
                    }),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };

        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_error_with_suggestion() {
        let error = ValidationError::required("config.port".to_string(), "port".to_string())
            .with_suggestion("Add: port = 8080".to_string());

        assert_eq!(error.error_type, ErrorType::Required);
        assert_eq!(error.suggestion, Some("Add: port = 8080".to_string()));
        assert!(error.to_string().contains("Suggestion:"));
    }
}
