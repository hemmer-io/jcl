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

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Schema validator
pub struct Validator {
    schema: Schema,
}

impl Validator {
    /// Create a new validator from a schema
    pub fn new(schema: Schema) -> Self {
        Self { schema }
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
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: format!(
                                    "String length {} is less than minimum {}",
                                    s.len(),
                                    min
                                ),
                            });
                        }
                    }
                    if let Some(max) = max_length {
                        if s.len() > *max {
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: format!(
                                    "String length {} exceeds maximum {}",
                                    s.len(),
                                    max
                                ),
                            });
                        }
                    }
                    if let Some(pat) = pattern {
                        if let Ok(regex) = regex::Regex::new(pat) {
                            if !regex.is_match(s) {
                                errors.push(ValidationError {
                                    path: path.to_string(),
                                    message: format!(
                                        "String '{}' does not match pattern '{}'",
                                        s, pat
                                    ),
                                });
                            }
                        }
                    }
                    if let Some(enum_vals) = enum_values {
                        if !enum_vals.contains(s) {
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: format!(
                                    "String '{}' is not one of allowed values: {:?}",
                                    s, enum_vals
                                ),
                            });
                        }
                    }
                }
                _ => {
                    errors.push(ValidationError {
                        path: path.to_string(),
                        message: format!("Expected string, got {:?}", value),
                    });
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
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: "Expected integer, got float".to_string(),
                            });
                        }
                    }
                    if let Some(min) = minimum {
                        if n < *min {
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: format!("Number {} is less than minimum {}", n, min),
                            });
                        }
                    }
                    if let Some(max) = maximum {
                        if n > *max {
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: format!("Number {} exceeds maximum {}", n, max),
                            });
                        }
                    }
                } else {
                    errors.push(ValidationError {
                        path: path.to_string(),
                        message: format!("Expected number, got {:?}", value),
                    });
                }
            }

            TypeDef::Boolean => {
                if !matches!(value, Value::Bool(_)) {
                    errors.push(ValidationError {
                        path: path.to_string(),
                        message: format!("Expected boolean, got {:?}", value),
                    });
                }
            }

            TypeDef::Null => {
                if !matches!(value, Value::Null) {
                    errors.push(ValidationError {
                        path: path.to_string(),
                        message: format!("Expected null, got {:?}", value),
                    });
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
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: format!(
                                    "List length {} is less than minimum {}",
                                    list.len(),
                                    min
                                ),
                            });
                        }
                    }
                    if let Some(max) = max_items {
                        if list.len() > *max {
                            errors.push(ValidationError {
                                path: path.to_string(),
                                message: format!(
                                    "List length {} exceeds maximum {}",
                                    list.len(),
                                    max
                                ),
                            });
                        }
                    }

                    for (i, item) in list.iter().enumerate() {
                        let item_path = format!("{}[{}]", path, i);
                        self.validate_value(item, items, &item_path, errors);
                    }
                }
                _ => {
                    errors.push(ValidationError {
                        path: path.to_string(),
                        message: format!("Expected list, got {:?}", value),
                    });
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
                                errors.push(ValidationError {
                                    path: format!("{}.{}", path, req),
                                    message: format!("Required property '{}' is missing", req),
                                });
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
                                errors.push(ValidationError {
                                    path: prop_path,
                                    message: format!("Unknown property '{}'", key),
                                });
                            }
                        }
                    }
                    _ => {
                        errors.push(ValidationError {
                            path: path.to_string(),
                            message: format!("Expected map, got {:?}", value),
                        });
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
                    errors.push(ValidationError {
                        path: path.to_string(),
                        message: format!("Value does not match any type in union"),
                    });
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
}
