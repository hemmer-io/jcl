//! Schema validation for JCL configurations
//!
//! This module provides schema definition and validation capabilities for JCL.
//! Schemas define the expected structure, types, and constraints for configuration values.
//!
//! # Features
//!
//! ## Basic Schema Validation
//! Define schemas with type definitions, constraints, and descriptions:
//! ```rust
//! use jcl::schema::{SchemaBuilder, PropertyBuilder, TypeDef};
//!
//! let schema = SchemaBuilder::new("config")
//!     .field("host", PropertyBuilder::new(TypeDef::String {
//!         min_length: None,
//!         max_length: Some(255),
//!         pattern: None,
//!         enum_values: None,
//!     }))
//!     .field("port", PropertyBuilder::new(TypeDef::Number {
//!         minimum: Some(1.0),
//!         maximum: Some(65535.0),
//!         integer_only: true,
//!     }))
//!     .build();
//! ```
//!
//! ## Custom Validators (Phase 2)
//! Register custom validation functions for domain-specific logic:
//! ```rust
//! use jcl::schema::{SchemaBuilder, PropertyBuilder, TypeDef, Validator, ValidatorFn};
//! use jcl::ast::Value;
//!
//! let email_validator: ValidatorFn = Box::new(|value| {
//!     match value {
//!         Value::String(s) if s.contains('@') && s.contains('.') => Ok(()),
//!         Value::String(_) => Err(jcl::schema::ValidationError::custom("Invalid email format".to_string())),
//!         _ => Err(jcl::schema::ValidationError::custom("Expected string".to_string())),
//!     }
//! });
//!
//! let builder = SchemaBuilder::new("user")
//!     .field("email", PropertyBuilder::new(TypeDef::String {
//!         min_length: None,
//!         max_length: None,
//!         pattern: None,
//!         enum_values: None,
//!     }).with_validator("email"));
//!
//! let mut validator = Validator::from_builder(builder);
//! validator.register_validator("email", email_validator);
//! ```
//!
//! ## Conditional Validation Rules (Phase 2)
//! Define field dependencies and mutual exclusions:
//! ```rust
//! use jcl::schema::{SchemaBuilder, PropertyBuilder, TypeDef};
//!
//! // Field that requires another field to be present
//! let builder = SchemaBuilder::new("config")
//!     .field("use_ssl", PropertyBuilder::new(TypeDef::Boolean))
//!     .field("ssl_cert", PropertyBuilder::new(TypeDef::String {
//!         min_length: None,
//!         max_length: None,
//!         pattern: None,
//!         enum_values: None,
//!     }).requires("use_ssl"));
//!
//! // Mutually exclusive fields
//! let builder = SchemaBuilder::new("config")
//!     .field("local_path", PropertyBuilder::new(TypeDef::String {
//!         min_length: None,
//!         max_length: None,
//!         pattern: None,
//!         enum_values: None,
//!     }))
//!     .field("remote_url", PropertyBuilder::new(TypeDef::String {
//!         min_length: None,
//!         max_length: None,
//!         pattern: None,
//!         enum_values: None,
//!     }).requires_absence_of("local_path"))
//!     .mutually_exclusive(vec!["local_path".to_string(), "remote_url".to_string()]);
//! ```
//!
//! ## Type Handling and Coercion
//!
//! JCL's schema validation follows the language's design principle of **explicit types** with
//! no surprising type coercions. This differs from YAML, JSON Schema, and some other formats.
//!
//! ### Number Type Flexibility
//!
//! The `TypeDef::Number` type accepts both integers and floats, providing natural numeric flexibility:
//! ```rust
//! use jcl::schema::{SchemaBuilder, PropertyBuilder, TypeDef};
//!
//! let schema = SchemaBuilder::new("config")
//!     .field("value", PropertyBuilder::new(TypeDef::Number {
//!         minimum: None,
//!         maximum: None,
//!         integer_only: false,
//!     }));
//!
//! // Both integer and float values are accepted:
//! // value = 42      ✅ Valid
//! // value = 42.5    ✅ Valid
//! ```
//!
//! ### No Implicit String Coercion
//!
//! Unlike some schema systems, JCL does **not** coerce strings to other types:
//! - `"42"` (string) is NOT coerced to `42` (number)
//! - `"true"` (string) is NOT coerced to `true` (boolean)
//! - `"null"` (string) is NOT coerced to `null`
//!
//! This explicit approach prevents common configuration errors and aligns with JCL's
//! philosophy of being predictable and safe.

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

impl Schema {
    /// Export schema to JSON Schema format (Draft 7)
    ///
    /// Converts the JCL schema to a JSON Schema Draft 7 compatible format,
    /// which can be used with standard JSON Schema validators and tooling.
    ///
    /// # Example
    ///
    /// ```
    /// use jcl::schema::{SchemaBuilder, PropertyBuilder, TypeDef};
    ///
    /// let schema = SchemaBuilder::new("User")
    ///     .version("1.0.0")
    ///     .description("User account schema")
    ///     .field("email", PropertyBuilder::new(TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: Some("^[^@]+@[^@]+$".to_string()),
    ///         enum_values: None,
    ///     }))
    ///     .mark_required("email")
    ///     .build();
    ///
    /// let json_schema = schema.to_json_schema();
    /// assert!(json_schema.contains("\"$schema\": \"http://json-schema.org/draft-07/schema#\""));
    /// assert!(json_schema.contains("\"email\""));
    /// ```
    pub fn to_json_schema(&self) -> String {
        let mut schema_obj = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": self.title.as_ref().unwrap_or(&"Schema".to_string()),
            "type": "object"
        });

        if let Some(desc) = &self.description {
            schema_obj["description"] = serde_json::json!(desc);
        }

        // Convert TypeDef to JSON Schema format
        if let TypeDef::Map {
            properties,
            required,
            additional_properties,
        } = &self.type_def
        {
            let mut props = serde_json::Map::new();
            for (name, prop) in properties {
                props.insert(name.clone(), type_def_to_json_schema(&prop.type_def));
            }
            schema_obj["properties"] = serde_json::json!(props);

            if !required.is_empty() {
                schema_obj["required"] = serde_json::json!(required);
            }

            schema_obj["additionalProperties"] = serde_json::json!(additional_properties);
        }

        serde_json::to_string_pretty(&schema_obj).unwrap_or_default()
    }

    /// Export schema to OpenAPI 3.0 format
    ///
    /// Converts the JCL schema to an OpenAPI 3.0 schema object,
    /// which can be embedded in OpenAPI specifications for API documentation.
    ///
    /// # Example
    ///
    /// ```
    /// use jcl::schema::{SchemaBuilder, PropertyBuilder, TypeDef};
    ///
    /// let schema = SchemaBuilder::new("User")
    ///     .description("User resource")
    ///     .field("username", PropertyBuilder::new(TypeDef::String {
    ///         min_length: Some(3),
    ///         max_length: Some(20),
    ///         pattern: None,
    ///         enum_values: None,
    ///     }))
    ///     .build();
    ///
    /// let openapi_schema = schema.to_openapi();
    /// assert!(openapi_schema.contains("\"type\": \"object\""));
    /// assert!(openapi_schema.contains("\"username\""));
    /// ```
    pub fn to_openapi(&self) -> String {
        let mut schema_obj = serde_json::json!({
            "type": "object"
        });

        if let Some(title) = &self.title {
            schema_obj["title"] = serde_json::json!(title);
        }

        if let Some(desc) = &self.description {
            schema_obj["description"] = serde_json::json!(desc);
        }

        // Convert TypeDef to OpenAPI format (very similar to JSON Schema)
        if let TypeDef::Map {
            properties,
            required,
            additional_properties,
        } = &self.type_def
        {
            let mut props = serde_json::Map::new();
            for (name, prop) in properties {
                let mut prop_obj = type_def_to_json_schema(&prop.type_def);

                // Add description if available
                if let Some(desc) = &prop.description {
                    if let Some(obj) = prop_obj.as_object_mut() {
                        obj.insert("description".to_string(), serde_json::json!(desc));
                    }
                }

                props.insert(name.clone(), prop_obj);
            }
            schema_obj["properties"] = serde_json::json!(props);

            if !required.is_empty() {
                schema_obj["required"] = serde_json::json!(required);
            }

            schema_obj["additionalProperties"] = serde_json::json!(additional_properties);
        }

        serde_json::to_string_pretty(&schema_obj).unwrap_or_default()
    }
}

/// Helper function to convert TypeDef to JSON Schema representation
fn type_def_to_json_schema(type_def: &TypeDef) -> serde_json::Value {
    match type_def {
        TypeDef::String {
            min_length,
            max_length,
            pattern,
            enum_values,
        } => {
            let mut obj = serde_json::json!({"type": "string"});
            if let Some(min) = min_length {
                obj["minLength"] = serde_json::json!(min);
            }
            if let Some(max) = max_length {
                obj["maxLength"] = serde_json::json!(max);
            }
            if let Some(pat) = pattern {
                obj["pattern"] = serde_json::json!(pat);
            }
            if let Some(enums) = enum_values {
                obj["enum"] = serde_json::json!(enums);
            }
            obj
        }
        TypeDef::Number {
            minimum,
            maximum,
            integer_only,
        } => {
            let mut obj = if *integer_only {
                serde_json::json!({"type": "integer"})
            } else {
                serde_json::json!({"type": "number"})
            };
            if let Some(min) = minimum {
                obj["minimum"] = serde_json::json!(min);
            }
            if let Some(max) = maximum {
                obj["maximum"] = serde_json::json!(max);
            }
            obj
        }
        TypeDef::Boolean => serde_json::json!({"type": "boolean"}),
        TypeDef::Null => serde_json::json!({"type": "null"}),
        TypeDef::List {
            items,
            min_items,
            max_items,
        } => {
            let mut obj = serde_json::json!({
                "type": "array",
                "items": type_def_to_json_schema(items)
            });
            if let Some(min) = min_items {
                obj["minItems"] = serde_json::json!(min);
            }
            if let Some(max) = max_items {
                obj["maxItems"] = serde_json::json!(max);
            }
            obj
        }
        TypeDef::Map {
            properties,
            required,
            additional_properties,
        } => {
            let mut props = serde_json::Map::new();
            for (name, prop) in properties {
                props.insert(name.clone(), type_def_to_json_schema(&prop.type_def));
            }
            let mut obj = serde_json::json!({
                "type": "object",
                "properties": props,
                "additionalProperties": additional_properties
            });
            if !required.is_empty() {
                obj["required"] = serde_json::json!(required);
            }
            obj
        }
        TypeDef::Union { types } => {
            let schemas: Vec<serde_json::Value> =
                types.iter().map(type_def_to_json_schema).collect();
            serde_json::json!({"anyOf": schemas})
        }
        TypeDef::DiscriminatedUnion {
            discriminator,
            variants,
        } => {
            // JSON Schema doesn't have native discriminated unions,
            // so we use oneOf with const discriminator
            let schemas: Vec<serde_json::Value> = variants
                .iter()
                .map(|(variant_name, variant_type)| {
                    let mut variant_schema = type_def_to_json_schema(variant_type);
                    // Add discriminator constraint
                    if let Some(obj) = variant_schema.as_object_mut() {
                        if let Some(props) =
                            obj.get_mut("properties").and_then(|p| p.as_object_mut())
                        {
                            props.insert(
                                discriminator.clone(),
                                serde_json::json!({"const": variant_name}),
                            );
                        }
                    }
                    variant_schema
                })
                .collect();
            serde_json::json!({
                "oneOf": schemas,
                "discriminator": {
                    "propertyName": discriminator
                }
            })
        }
        TypeDef::Ref { name } => {
            // JSON Schema reference
            serde_json::json!({"$ref": format!("#/definitions/{}", name)})
        }
        TypeDef::Any => serde_json::json!({}),
    }
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

    /// Union of multiple types (try each type until one matches)
    Union { types: Vec<TypeDef> },

    /// Discriminated union (tagged union) - uses a discriminator field to determine the type
    ///
    /// # Example
    ///
    /// ```jcl
    /// // Storage configuration with discriminated union
    /// storage = (
    ///     type = "s3",      // discriminator field
    ///     bucket = "my-bucket",
    ///     region = "us-west-2"
    /// )
    ///
    /// // OR
    ///
    /// storage = (
    ///     type = "local",   // discriminator field
    ///     path = "/var/data"
    /// )
    /// ```
    ///
    /// The discriminator field determines which variant schema to validate against.
    DiscriminatedUnion {
        /// Field name to use as discriminator (e.g., "type", "kind")
        discriminator: String,
        /// Map from discriminator value to schema
        /// Example: {"s3" -> S3Schema, "local" -> LocalSchema}
        variants: HashMap<String, Box<TypeDef>>,
    },

    /// Recursive type reference (for self-referential types like trees)
    ///
    /// # Example
    ///
    /// ```jcl
    /// // Tree node that can contain child nodes
    /// node = (
    ///     value = 42,
    ///     children = [
    ///         (value = 10, children = []),
    ///         (value = 20, children = [])
    ///     ]
    /// )
    /// ```
    Ref {
        /// Name of the type being referenced (must be defined elsewhere)
        name: String,
    },
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
        let field = path.split('.').next_back().unwrap_or(&path).to_string();
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
        let field = path.split('.').next_back().unwrap_or(&path).to_string();
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
        let field = path.split('.').next_back().unwrap_or(&path).to_string();
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
        self.field = path.split('.').next_back().unwrap_or(&path).to_string();
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

/// Custom validator function type (Phase 2)
///
/// Custom validators allow you to define domain-specific validation logic that goes beyond
/// basic type checking. Validators are registered by name and can be applied to individual fields.
///
/// # Examples
///
/// ```rust
/// use jcl::schema::{ValidatorFn, ValidationError};
/// use jcl::ast::Value;
///
/// // Email validator
/// let email_validator: ValidatorFn = Box::new(|value| {
///     match value {
///         Value::String(s) if s.contains('@') => Ok(()),
///         _ => Err(ValidationError::custom("Invalid email".to_string())),
///     }
/// });
/// ```
pub type ValidatorFn = Box<dyn Fn(&Value) -> Result<(), ValidationError> + Send + Sync>;

/// Builder for constructing Property definitions programmatically
#[derive(Clone)]
pub struct PropertyBuilder {
    type_def: TypeDef,
    description: Option<String>,
    default: Option<Value>,
    validator_names: Vec<String>,
    requires: Vec<String>,
    requires_absence_of: Vec<String>,
}

impl std::fmt::Debug for PropertyBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyBuilder")
            .field("type_def", &self.type_def)
            .field("description", &self.description)
            .field("default", &self.default)
            .field("validator_names", &self.validator_names)
            .field("requires", &self.requires)
            .field("requires_absence_of", &self.requires_absence_of)
            .finish()
    }
}

impl PropertyBuilder {
    /// Create a new property builder with a type definition
    pub fn new(type_def: TypeDef) -> Self {
        Self {
            type_def,
            description: None,
            default: None,
            validator_names: Vec::new(),
            requires: Vec::new(),
            requires_absence_of: Vec::new(),
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

    /// Register a custom validator by name (Phase 2)
    ///
    /// The validator must be registered with the `Validator` using `register_validator()`.
    /// Multiple validators can be applied to a single field by calling this method multiple times.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::PropertyBuilder;
    /// use jcl::schema::TypeDef;
    ///
    /// let email_field = PropertyBuilder::new(TypeDef::String {
    ///     min_length: None,
    ///     max_length: None,
    ///     pattern: None,
    ///     enum_values: None,
    /// }).with_validator("email");
    /// ```
    pub fn with_validator(mut self, validator_name: impl Into<String>) -> Self {
        self.validator_names.push(validator_name.into());
        self
    }

    /// Specify that this field requires another field to be present (Phase 2)
    ///
    /// When this field is provided, the specified required field must also be present.
    /// This creates a one-way dependency relationship.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::PropertyBuilder;
    /// use jcl::schema::TypeDef;
    ///
    /// // ssl_cert requires use_ssl to be present
    /// let ssl_cert = PropertyBuilder::new(TypeDef::String {
    ///     min_length: None,
    ///     max_length: None,
    ///     pattern: None,
    ///     enum_values: None,
    /// }).requires("use_ssl");
    /// ```
    pub fn requires(mut self, field: impl Into<String>) -> Self {
        self.requires.push(field.into());
        self
    }

    /// Specify that this field requires another field to be absent (Phase 2)
    ///
    /// When this field is provided, the specified field must NOT be present.
    /// This creates a mutual exclusion relationship between two fields.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::PropertyBuilder;
    /// use jcl::schema::TypeDef;
    ///
    /// // remote_url and local_path are mutually exclusive
    /// let remote_url = PropertyBuilder::new(TypeDef::String {
    ///     min_length: None,
    ///     max_length: None,
    ///     pattern: None,
    ///     enum_values: None,
    /// }).requires_absence_of("local_path");
    /// ```
    pub fn requires_absence_of(mut self, field: impl Into<String>) -> Self {
        self.requires_absence_of.push(field.into());
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

    /// Get validator names (internal use)
    pub(crate) fn validator_names(&self) -> &[String] {
        &self.validator_names
    }

    /// Get required fields (internal use)
    pub(crate) fn required_fields(&self) -> &[String] {
        &self.requires
    }

    /// Get mutually exclusive fields (internal use)
    pub(crate) fn mutually_exclusive_fields(&self) -> &[String] {
        &self.requires_absence_of
    }
}

/// Metadata about field relationships and validation rules
#[derive(Debug, Clone)]
pub(crate) struct FieldMetadata {
    validator_names: Vec<String>,
    requires: Vec<String>,
    requires_absence_of: Vec<String>,
}

/// Builder for constructing Schema definitions programmatically
#[derive(Debug, Clone)]
pub struct SchemaBuilder {
    title: Option<String>,
    description: Option<String>,
    properties: HashMap<String, PropertyBuilder>,
    required: Vec<String>,
    additional_properties: bool,
    field_metadata: HashMap<String, FieldMetadata>,
    mutually_exclusive_groups: Vec<Vec<String>>,
    version: Option<String>,
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
            field_metadata: HashMap::new(),
            mutually_exclusive_groups: Vec::new(),
            version: None,
        }
    }

    /// Create a SchemaBuilder from an existing Schema (for re-building/modifying)
    pub fn from_schema(schema: Schema) -> Self {
        let mut builder = Self {
            title: schema.title,
            description: schema.description,
            properties: HashMap::new(),
            required: Vec::new(),
            additional_properties: false,
            field_metadata: HashMap::new(),
            mutually_exclusive_groups: Vec::new(),
            version: Some(schema.version),
        };

        // Extract properties from the schema's type_def if it's a Map
        if let TypeDef::Map {
            properties,
            required,
            additional_properties,
        } = schema.type_def
        {
            // Convert Properties back to PropertyBuilders
            for (name, prop) in properties {
                builder.properties.insert(
                    name,
                    PropertyBuilder {
                        type_def: prop.type_def,
                        description: prop.description,
                        default: prop.default,
                        validator_names: Vec::new(),
                        requires: Vec::new(),
                        requires_absence_of: Vec::new(),
                    },
                );
            }
            builder.required = required;
            builder.additional_properties = additional_properties;
        }

        builder
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
        let field_name = name.into();

        // Extract metadata from PropertyBuilder
        let metadata = FieldMetadata {
            validator_names: property.validator_names().to_vec(),
            requires: property.required_fields().to_vec(),
            requires_absence_of: property.mutually_exclusive_fields().to_vec(),
        };

        self.field_metadata.insert(field_name.clone(), metadata);
        self.properties.insert(field_name, property);
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

    /// Define a group of mutually exclusive fields (Phase 2)
    ///
    /// Specifies that only one field from the given group can be present in the configuration.
    /// If multiple fields from the group are present, validation will fail.
    ///
    /// This is useful for cases where you have alternative configuration options that cannot
    /// coexist (e.g., local vs remote storage, different authentication methods, etc.).
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::SchemaBuilder;
    /// use jcl::schema::PropertyBuilder;
    /// use jcl::schema::TypeDef;
    ///
    /// let builder = SchemaBuilder::new("storage")
    ///     .field("local_path", PropertyBuilder::new(TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     }))
    ///     .field("s3_bucket", PropertyBuilder::new(TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     }))
    ///     .field("azure_container", PropertyBuilder::new(TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     }))
    ///     .mutually_exclusive(vec![
    ///         "local_path".to_string(),
    ///         "s3_bucket".to_string(),
    ///         "azure_container".to_string(),
    ///     ]);
    /// ```
    pub fn mutually_exclusive(mut self, fields: Vec<String>) -> Self {
        self.mutually_exclusive_groups.push(fields);
        self
    }

    /// Get field metadata (internal use for Validator)
    pub(crate) fn field_metadata(&self) -> &HashMap<String, FieldMetadata> {
        &self.field_metadata
    }

    /// Get mutually exclusive groups (internal use for Validator)
    pub(crate) fn mutually_exclusive_groups(&self) -> &[Vec<String>] {
        &self.mutually_exclusive_groups
    }

    // ========== Phase 4: Schema Composition Methods ==========

    /// Set schema version (Phase 4)
    ///
    /// Version string for tracking schema changes and compatibility.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::SchemaBuilder;
    ///
    /// let schema = SchemaBuilder::new("aws_instance")
    ///     .version("2.0.0")
    ///     .build();
    /// ```
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Extend this schema with fields from another schema (Phase 4)
    ///
    /// Schema inheritance allows reusing common field definitions. Fields from the
    /// parent schema are merged into this schema. If a field exists in both schemas,
    /// the child schema's definition takes precedence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::{SchemaBuilder, TypeDef};
    ///
    /// // Base schema with common fields
    /// let base = SchemaBuilder::new("base_resource")
    ///     .required_field("id", TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     })
    ///     .required_field("name", TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     });
    ///
    /// // Child schema extends base
    /// let instance_schema = SchemaBuilder::new("aws_instance")
    ///     .extends(base)
    ///     .required_field("ami", TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     })
    ///     .build();
    /// ```
    pub fn extends(mut self, parent: SchemaBuilder) -> Self {
        // Merge properties from parent (child overrides parent)
        for (name, prop) in parent.properties {
            self.properties.entry(name.clone()).or_insert(prop);
        }

        // Merge required fields
        for req in parent.required {
            if !self.required.contains(&req) {
                self.required.push(req);
            }
        }

        // Merge field metadata
        for (name, metadata) in parent.field_metadata {
            self.field_metadata.entry(name).or_insert(metadata);
        }

        // Merge mutually exclusive groups
        self.mutually_exclusive_groups
            .extend(parent.mutually_exclusive_groups);

        // Inherit additional_properties setting if not explicitly set
        if !self.additional_properties && parent.additional_properties {
            self.additional_properties = true;
        }

        self
    }

    /// Merge multiple schemas into this one (Phase 4)
    ///
    /// Schema composition allows combining multiple schemas. All fields from
    /// all schemas are merged. If a field exists in multiple schemas, the
    /// last one wins.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::{SchemaBuilder, TypeDef};
    ///
    /// let network_fields = SchemaBuilder::new("network")
    ///     .optional_field("vpc_id", TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     })
    ///     .optional_field("subnet_id", TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     });
    ///
    /// let tags_fields = SchemaBuilder::new("tags")
    ///     .optional_field("tags", TypeDef::Map {
    ///         properties: std::collections::HashMap::new(),
    ///         required: vec![],
    ///         additional_properties: true,
    ///     });
    ///
    /// let combined = SchemaBuilder::new("aws_instance")
    ///     .merge(vec![network_fields, tags_fields])
    ///     .build();
    /// ```
    pub fn merge(mut self, others: Vec<SchemaBuilder>) -> Self {
        for other in others {
            // Merge all properties (last wins)
            self.properties.extend(other.properties);

            // Merge required fields (no duplicates)
            for req in other.required {
                if !self.required.contains(&req) {
                    self.required.push(req);
                }
            }

            // Merge field metadata
            self.field_metadata.extend(other.field_metadata);

            // Merge mutually exclusive groups
            self.mutually_exclusive_groups
                .extend(other.mutually_exclusive_groups);
        }

        self
    }

    /// Generate documentation for this schema (Phase 4)
    ///
    /// Returns a markdown-formatted documentation string describing the schema,
    /// its fields, types, constraints, and validation rules.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::{SchemaBuilder, PropertyBuilder, TypeDef};
    ///
    /// let schema = SchemaBuilder::new("aws_instance")
    ///     .description("AWS EC2 instance configuration")
    ///     .version("1.0.0")
    ///     .required_field("ami", TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: Some("^ami-[0-9a-f]{17}$".to_string()),
    ///         enum_values: None,
    ///     })
    ///     .required_field("instance_type", TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: Some(vec!["t2.micro".to_string(), "t3.micro".to_string()]),
    ///     });
    ///
    /// let docs = schema.generate_docs();
    /// println!("{}", docs);
    /// ```
    pub fn generate_docs(&self) -> String {
        let mut doc = String::new();

        // Title and version
        if let Some(title) = &self.title {
            doc.push_str(&format!("# {}\n\n", title));
        }

        if let Some(version) = &self.version {
            doc.push_str(&format!("**Version**: {}\n\n", version));
        }

        // Description
        if let Some(desc) = &self.description {
            doc.push_str(&format!("{}\n\n", desc));
        }

        // Fields
        doc.push_str("## Fields\n\n");

        // Collect all field names sorted
        let mut field_names: Vec<_> = self.properties.keys().collect();
        field_names.sort();

        for field_name in field_names {
            let prop = &self.properties[field_name];
            let is_required = self.required.contains(field_name);

            doc.push_str(&format!(
                "### `{}`{}\n\n",
                field_name,
                if is_required { " (required)" } else { "" }
            ));

            // Type information
            doc.push_str(&format!(
                "**Type**: {}\n\n",
                Self::type_to_string(&prop.type_def)
            ));

            // Description if available
            if let Some(desc) = &prop.description {
                doc.push_str(&format!("{}\n\n", desc));
            }

            // Validation rules
            if let Some(metadata) = self.field_metadata.get(field_name) {
                if !metadata.validator_names.is_empty() {
                    doc.push_str(&format!(
                        "**Validators**: {}\n\n",
                        metadata.validator_names.join(", ")
                    ));
                }

                if !metadata.requires.is_empty() {
                    doc.push_str(&format!(
                        "**Requires**: {}\n\n",
                        metadata.requires.join(", ")
                    ));
                }

                if !metadata.requires_absence_of.is_empty() {
                    doc.push_str(&format!(
                        "**Mutually exclusive with**: {}\n\n",
                        metadata.requires_absence_of.join(", ")
                    ));
                }
            }
        }

        // Mutually exclusive groups
        if !self.mutually_exclusive_groups.is_empty() {
            doc.push_str("## Mutually Exclusive Groups\n\n");
            for group in &self.mutually_exclusive_groups {
                doc.push_str(&format!("- Only one of: {}\n", group.join(", ")));
            }
            doc.push('\n');
        }

        doc
    }

    /// Helper to convert TypeDef to string for documentation
    fn type_to_string(type_def: &TypeDef) -> String {
        match type_def {
            TypeDef::String { enum_values, .. } => {
                if let Some(values) = enum_values {
                    format!("String (one of: {})", values.join(", "))
                } else {
                    "String".to_string()
                }
            }
            TypeDef::Number { integer_only, .. } => {
                if *integer_only {
                    "Integer".to_string()
                } else {
                    "Number".to_string()
                }
            }
            TypeDef::Boolean => "Boolean".to_string(),
            TypeDef::Null => "Null".to_string(),
            TypeDef::List { .. } => "List".to_string(),
            TypeDef::Map { .. } => "Map".to_string(),
            TypeDef::Any => "Any".to_string(),
            TypeDef::Union { types } => {
                let type_strs: Vec<String> = types.iter().map(Self::type_to_string).collect();
                format!("Union ({})", type_strs.join(" | "))
            }
            TypeDef::DiscriminatedUnion { discriminator, .. } => {
                format!("Discriminated Union (discriminator: {})", discriminator)
            }
            TypeDef::Ref { name } => format!("Reference to {}", name),
        }
    }

    /// Build the Schema
    pub fn build(self) -> Schema {
        Schema {
            version: self.version.unwrap_or_else(|| "1.0".to_string()),
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

/// Schema validator with custom validation support
pub struct Validator {
    schema: Schema,
    custom_validators: HashMap<String, ValidatorFn>,
    field_metadata: HashMap<String, FieldMetadata>,
    mutually_exclusive_groups: Vec<Vec<String>>,
}

impl Validator {
    /// Create a new validator from a schema
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            custom_validators: HashMap::new(),
            field_metadata: HashMap::new(),
            mutually_exclusive_groups: Vec::new(),
        }
    }

    /// Create a validator from a SchemaBuilder (programmatic API)
    pub fn from_builder(builder: SchemaBuilder) -> Self {
        let field_metadata = builder.field_metadata().clone();
        let mutually_exclusive_groups = builder.mutually_exclusive_groups().to_vec();
        let schema = builder.build();

        Self {
            schema,
            custom_validators: HashMap::new(),
            field_metadata,
            mutually_exclusive_groups,
        }
    }

    /// Register a custom validator function (Phase 2)
    ///
    /// Custom validators are referenced by name in `PropertyBuilder::with_validator()`.
    /// You must register the validator implementation before validation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::{Validator, SchemaBuilder, PropertyBuilder, TypeDef, ValidatorFn};
    /// use jcl::ast::Value;
    ///
    /// let email_validator: ValidatorFn = Box::new(|value| {
    ///     match value {
    ///         Value::String(s) if s.contains('@') => Ok(()),
    ///         _ => Err(jcl::schema::ValidationError::custom("Invalid email".to_string())),
    ///     }
    /// });
    ///
    /// let builder = SchemaBuilder::new("user")
    ///     .field("email", PropertyBuilder::new(TypeDef::String {
    ///         min_length: None,
    ///         max_length: None,
    ///         pattern: None,
    ///         enum_values: None,
    ///     }).with_validator("email"));
    ///
    /// let mut validator = Validator::from_builder(builder);
    /// validator.register_validator("email", email_validator);
    /// ```
    pub fn register_validator(
        &mut self,
        name: impl Into<String>,
        validator: ValidatorFn,
    ) -> &mut Self {
        self.custom_validators.insert(name.into(), validator);
        self
    }

    /// Register multiple validators at once (Phase 2)
    ///
    /// Convenience method for bulk validator registration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jcl::schema::{Validator, ValidatorFn};
    /// use jcl::ast::Value;
    /// use std::collections::HashMap;
    ///
    /// let mut validators: HashMap<String, ValidatorFn> = HashMap::new();
    ///
    /// validators.insert("email".to_string(), Box::new(|value| {
    ///     match value {
    ///         Value::String(s) if s.contains('@') => Ok(()),
    ///         _ => Err(jcl::schema::ValidationError::custom("Invalid email".to_string())),
    ///     }
    /// }));
    ///
    /// validators.insert("port".to_string(), Box::new(|value| {
    ///     match value {
    ///         Value::Int(p) if (1..=65535).contains(p) => Ok(()),
    ///         _ => Err(jcl::schema::ValidationError::custom("Invalid port".to_string())),
    ///     }
    /// }));
    ///
    /// let schema = jcl::schema::Schema {
    ///     version: "1.0".to_string(),
    ///     title: None,
    ///     description: None,
    ///     type_def: jcl::schema::TypeDef::Map {
    ///         properties: HashMap::new(),
    ///         required: Vec::new(),
    ///         additional_properties: false,
    ///     },
    /// };
    ///
    /// let mut validator = Validator::new(schema);
    /// validator.register_validators(validators);
    /// ```
    pub fn register_validators(&mut self, validators: HashMap<String, ValidatorFn>) -> &mut Self {
        self.custom_validators.extend(validators);
        self
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

        // First, validate against the schema's type definitions
        self.validate_value(&root_value, &self.schema.type_def, "", &mut errors);

        // Then, apply conditional validation rules and custom validators
        if let Value::Map(map) = &root_value {
            self.validate_conditional_rules(map, &mut errors);
            self.validate_custom_validators(map, "", &mut errors);
        }

        Ok(errors)
    }

    /// Validate conditional rules (requires, requires_absence_of, mutually_exclusive)
    fn validate_conditional_rules(
        &self,
        map: &HashMap<String, Value>,
        errors: &mut Vec<ValidationError>,
    ) {
        // Check field dependencies (requires)
        for (field_name, metadata) in &self.field_metadata {
            if map.contains_key(field_name) {
                // Field is present, check if it requires other fields
                for required_field in &metadata.requires {
                    if !map.contains_key(required_field) {
                        errors.push(
                            ValidationError::constraint_violation(
                                required_field.clone(),
                                format!(
                                    "Field '{}' requires '{}' to be present",
                                    field_name, required_field
                                ),
                            )
                            .with_suggestion(format!("Add field: {} = ...", required_field)),
                        );
                    }
                }

                // Check if it requires absence of other fields (mutually exclusive)
                for exclusive_field in &metadata.requires_absence_of {
                    if map.contains_key(exclusive_field) {
                        errors.push(ValidationError::constraint_violation(
                            field_name.clone(),
                            format!(
                                "Fields '{}' and '{}' are mutually exclusive",
                                field_name, exclusive_field
                            ),
                        ));
                    }
                }
            }
        }

        // Check mutually exclusive groups
        for group in &self.mutually_exclusive_groups {
            let present_fields: Vec<&String> =
                group.iter().filter(|f| map.contains_key(*f)).collect();

            if present_fields.len() > 1 {
                errors.push(ValidationError::constraint_violation(
                    present_fields[0].clone(),
                    format!(
                        "Only one of these fields can be present: [{}]",
                        group.join(", ")
                    ),
                ));
            }
        }
    }

    /// Apply custom validators to fields
    fn validate_custom_validators(
        &self,
        map: &HashMap<String, Value>,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        for (field_name, metadata) in &self.field_metadata {
            if let Some(value) = map.get(field_name) {
                let field_path = if path.is_empty() {
                    field_name.clone()
                } else {
                    format!("{}.{}", path, field_name)
                };

                // Apply all registered validators for this field
                for validator_name in &metadata.validator_names {
                    if let Some(validator) = self.custom_validators.get(validator_name) {
                        if let Err(err) = validator(value) {
                            errors.push(err.with_path(field_path.clone()));
                        }
                    } else {
                        // Validator not registered - this is a warning but not a validation error
                        eprintln!(
                            "Warning: Validator '{}' referenced but not registered",
                            validator_name
                        );
                    }
                }

                // Recursively validate nested maps
                if let Value::Map(nested_map) = value {
                    self.validate_custom_validators(nested_map, &field_path, errors);
                }
            }
        }
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

            TypeDef::DiscriminatedUnion {
                discriminator,
                variants,
            } => {
                // Discriminated unions require a map with a discriminator field
                match value {
                    Value::Map(map) => {
                        // Check if discriminator field exists
                        match map.get(discriminator) {
                            Some(Value::String(discriminator_value)) => {
                                // Find the matching variant
                                match variants.get(discriminator_value) {
                                    Some(variant_type) => {
                                        // Validate against the variant's schema
                                        self.validate_value(value, variant_type, path, errors);
                                    }
                                    None => {
                                        let valid_variants: Vec<&str> =
                                            variants.keys().map(|s| s.as_str()).collect();
                                        errors.push(
                                            ValidationError::constraint_violation(
                                                path.to_string(),
                                                format!(
                                                    "Unknown variant '{}' for discriminator '{}'. Valid variants: [{}]",
                                                    discriminator_value,
                                                    discriminator,
                                                    valid_variants.join(", ")
                                                ),
                                            )
                                            .with_suggestion(format!(
                                                "Use one of: {}",
                                                valid_variants.join(", ")
                                            )),
                                        );
                                    }
                                }
                            }
                            Some(_) => {
                                errors.push(
                                    ValidationError::type_mismatch(
                                        format!("{}.{}", path, discriminator),
                                        "string",
                                        "non-string",
                                    )
                                    .with_suggestion(format!(
                                        "Discriminator field '{}' must be a string",
                                        discriminator
                                    )),
                                );
                            }
                            None => {
                                errors.push(
                                    ValidationError::required(
                                        path.to_string(),
                                        discriminator.clone(),
                                    )
                                    .with_suggestion(format!(
                                        "Add discriminator field: {} = \"variant_name\"",
                                        discriminator
                                    )),
                                );
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
                        errors.push(
                            ValidationError::type_mismatch(path.to_string(), "map", found)
                                .with_suggestion(
                                    "Discriminated unions require a map with a discriminator field"
                                        .to_string(),
                                ),
                        );
                    }
                }
            }

            TypeDef::Ref { name } => {
                // Recursive type references need to be resolved from a type registry
                // For now, we'll add a placeholder error indicating the feature needs type registry support
                errors.push(
                    ValidationError::custom(format!(
                        "Recursive type reference '{}' at path '{}' requires type registry (not yet implemented)",
                        name, path
                    ))
                    .with_suggestion(
                        "Type registry support will be added in a future update".to_string(),
                    ),
                );
            }
        }
    }
}

// ========== Schema Generation from Examples ==========

/// Options for schema generation
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    /// Infer specific types from values (vs. using Any)
    pub infer_types: bool,
    /// Infer constraints like min/max lengths, ranges
    pub infer_constraints: bool,
    /// Mark all fields as optional (permissive schema)
    pub mark_all_optional: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            infer_types: true,
            infer_constraints: true,
            mark_all_optional: false,
        }
    }
}

/// Generate a schema from example JCL modules
///
/// Analyzes one or more JCL modules to infer a schema definition.
/// The generated schema can be used as a starting point and refined manually.
///
/// # Examples
///
/// ```
/// use jcl::schema::{generate_from_examples, GenerateOptions};
/// use jcl::parser::Parser;
///
/// let example = r#"
/// server = (
///     host = "localhost",
///     port = 8080
/// )
/// "#;
///
/// let module = Parser::new(example).parse().unwrap();
/// let schema = generate_from_examples(&[module], GenerateOptions::default()).unwrap();
/// ```
pub fn generate_from_examples(
    modules: &[crate::ast::Module],
    options: GenerateOptions,
) -> anyhow::Result<Schema> {
    let mut builder = SchemaBuilder::new("Generated");
    let mut field_occurrences: HashMap<String, usize> = HashMap::new();
    let total_modules = modules.len();

    // First pass: collect all fields and count occurrences
    for module in modules {
        for statement in &module.statements {
            if let crate::ast::Statement::Assignment { name, value, .. } = statement {
                *field_occurrences.entry(name.clone()).or_insert(0) += 1;

                // If field already exists, we might need to merge types
                if !builder.properties.contains_key(name) {
                    let type_def = if options.infer_types {
                        infer_type_from_value(value, &options)
                    } else {
                        TypeDef::Any
                    };
                    builder = builder.field(name.clone(), PropertyBuilder::new(type_def));
                }
            }
        }
    }

    // Second pass: mark required fields (present in all examples)
    if !options.mark_all_optional {
        for (field_name, occurrences) in &field_occurrences {
            if *occurrences == total_modules {
                builder = builder.mark_required(field_name.clone());
            }
        }
    }

    Ok(builder.build())
}

/// Infer TypeDef from a JCL expression
fn infer_type_from_value(value: &crate::ast::Expression, options: &GenerateOptions) -> TypeDef {
    use crate::ast::Expression;

    match value {
        Expression::Literal { value: val, .. } => infer_type_from_literal(val, options),
        Expression::List { elements, .. } => {
            if elements.is_empty() {
                TypeDef::List {
                    items: Box::new(TypeDef::Any),
                    min_items: if options.infer_constraints {
                        Some(0)
                    } else {
                        None
                    },
                    max_items: None,
                }
            } else {
                // Infer type from first element
                let item_type = infer_type_from_value(&elements[0], options);
                TypeDef::List {
                    items: Box::new(item_type),
                    min_items: if options.infer_constraints {
                        Some(elements.len())
                    } else {
                        None
                    },
                    max_items: if options.infer_constraints {
                        Some(elements.len())
                    } else {
                        None
                    },
                }
            }
        }
        Expression::Map { entries, .. } => {
            let mut properties = HashMap::new();
            let mut required = Vec::new();

            for (key_name, value_expr) in entries {
                let prop_type = infer_type_from_value(value_expr, options);
                properties.insert(
                    key_name.clone(),
                    Property {
                        type_def: prop_type,
                        description: None,
                        default: None,
                    },
                );

                if !options.mark_all_optional {
                    required.push(key_name.clone());
                }
            }

            TypeDef::Map {
                properties,
                required,
                additional_properties: false,
            }
        }
        _ => TypeDef::Any, // For other expression types, use Any
    }
}

/// Infer TypeDef from a literal value
fn infer_type_from_literal(value: &crate::ast::Value, options: &GenerateOptions) -> TypeDef {
    use crate::ast::Value;

    match value {
        Value::String(s) => {
            let mut pattern = None;

            // Pattern detection for common formats
            if options.infer_constraints {
                // Email pattern
                if s.contains('@') && s.contains('.') {
                    pattern = Some(r"^[^@]+@[^@]+\.[^@]+$".to_string());
                }
                // URL pattern
                else if s.starts_with("http://") || s.starts_with("https://") {
                    pattern = Some(r"^https?://".to_string());
                }
                // File path pattern
                else if s.starts_with('/') && s.contains('.') {
                    pattern = Some(r"^/.*\..+$".to_string());
                }
            }

            TypeDef::String {
                min_length: if options.infer_constraints {
                    Some(s.len())
                } else {
                    None
                },
                max_length: if options.infer_constraints {
                    Some(s.len())
                } else {
                    None
                },
                pattern,
                enum_values: None,
            }
        }
        Value::Int(n) => TypeDef::Number {
            minimum: if options.infer_constraints {
                Some(*n as f64)
            } else {
                None
            },
            maximum: if options.infer_constraints {
                Some(*n as f64)
            } else {
                None
            },
            integer_only: true,
        },
        Value::Float(f) => TypeDef::Number {
            minimum: if options.infer_constraints {
                Some(*f)
            } else {
                None
            },
            maximum: if options.infer_constraints {
                Some(*f)
            } else {
                None
            },
            integer_only: false,
        },
        Value::Bool(_) => TypeDef::Boolean,
        Value::Null => TypeDef::Null,
        Value::List(items) => {
            if items.is_empty() {
                TypeDef::List {
                    items: Box::new(TypeDef::Any),
                    min_items: if options.infer_constraints {
                        Some(0)
                    } else {
                        None
                    },
                    max_items: None,
                }
            } else {
                // Infer type from first item
                let item_type = infer_type_from_literal(&items[0], options);
                TypeDef::List {
                    items: Box::new(item_type),
                    min_items: if options.infer_constraints {
                        Some(items.len())
                    } else {
                        None
                    },
                    max_items: if options.infer_constraints {
                        Some(items.len())
                    } else {
                        None
                    },
                }
            }
        }
        Value::Map(entries) => {
            let mut properties = HashMap::new();
            let mut required = Vec::new();

            for (key, val) in entries {
                let prop_type = infer_type_from_literal(val, options);
                properties.insert(
                    key.clone(),
                    Property {
                        type_def: prop_type,
                        description: None,
                        default: None,
                    },
                );

                if !options.infer_constraints {
                    required.push(key.clone());
                }
            }

            TypeDef::Map {
                properties,
                required,
                additional_properties: false,
            }
        }
        Value::Function { .. } => TypeDef::Any,
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

    // ========== Phase 2: Custom Validators & Conditional Rules Tests ==========

    #[test]
    fn test_custom_validator() {
        // Create a custom validator for email addresses
        let email_validator: ValidatorFn = Box::new(|value| match value {
            Value::String(s) => {
                if s.contains('@') && s.contains('.') {
                    Ok(())
                } else {
                    Err(ValidationError::custom("Invalid email format".to_string()))
                }
            }
            _ => Err(ValidationError::custom(
                "Expected string for email".to_string(),
            )),
        });

        let property = PropertyBuilder::new(TypeDef::String {
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
        })
        .with_validator("email");

        let builder = SchemaBuilder::new("user_schema").field("email", property);

        let mut validator = Validator::from_builder(builder);
        validator.register_validator("email", email_validator);

        // Valid email
        let valid_module = Module {
            statements: vec![Statement::Assignment {
                name: "email".to_string(),
                value: Expression::Literal {
                    value: Value::String("user@example.com".to_string()),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };
        let errors = validator.validate_module(&valid_module).unwrap();
        assert_eq!(errors.len(), 0);

        // Invalid email
        let invalid_module = Module {
            statements: vec![Statement::Assignment {
                name: "email".to_string(),
                value: Expression::Literal {
                    value: Value::String("not-an-email".to_string()),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };
        let errors = validator.validate_module(&invalid_module).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Invalid email format"));
    }

    #[test]
    fn test_field_requires() {
        let subnet_prop = PropertyBuilder::new(TypeDef::String {
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
        })
        .requires("vpc_id");

        let builder = SchemaBuilder::new("network_config")
            .field("subnet_id", subnet_prop)
            .optional_field(
                "vpc_id",
                TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                },
            );

        let validator = Validator::from_builder(builder);

        // Missing required field
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "subnet_id".to_string(),
                value: Expression::Literal {
                    value: Value::String("subnet-123".to_string()),
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
        assert!(errors[0]
            .message
            .contains("requires 'vpc_id' to be present"));
        assert!(errors[0].suggestion.is_some());
    }

    #[test]
    fn test_requires_absence_of() {
        let subnet_prop = PropertyBuilder::new(TypeDef::String {
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
        })
        .requires_absence_of("network_interface_id");

        let builder = SchemaBuilder::new("network_config")
            .field("subnet_id", subnet_prop)
            .optional_field(
                "network_interface_id",
                TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                },
            );

        let validator = Validator::from_builder(builder);

        // Both fields present (mutually exclusive)
        let module = Module {
            statements: vec![
                Statement::Assignment {
                    name: "subnet_id".to_string(),
                    value: Expression::Literal {
                        value: Value::String("subnet-123".to_string()),
                        span: None,
                    },
                    type_annotation: None,
                    mutable: false,
                    doc_comments: Some(vec![]),
                    span: None,
                },
                Statement::Assignment {
                    name: "network_interface_id".to_string(),
                    value: Expression::Literal {
                        value: Value::String("eni-456".to_string()),
                        span: None,
                    },
                    type_annotation: None,
                    mutable: false,
                    doc_comments: Some(vec![]),
                    span: None,
                },
            ],
        };
        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("mutually exclusive"));
    }

    #[test]
    fn test_mutually_exclusive_groups() {
        let builder = SchemaBuilder::new("config")
            .optional_field(
                "vpc_id",
                TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                },
            )
            .optional_field(
                "subnet_id",
                TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                },
            )
            .optional_field(
                "network_interface_id",
                TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                },
            )
            .mutually_exclusive(vec![
                "vpc_id".to_string(),
                "subnet_id".to_string(),
                "network_interface_id".to_string(),
            ]);

        let validator = Validator::from_builder(builder);

        // Two fields from exclusive group present
        let module = Module {
            statements: vec![
                Statement::Assignment {
                    name: "vpc_id".to_string(),
                    value: Expression::Literal {
                        value: Value::String("vpc-123".to_string()),
                        span: None,
                    },
                    type_annotation: None,
                    mutable: false,
                    doc_comments: Some(vec![]),
                    span: None,
                },
                Statement::Assignment {
                    name: "subnet_id".to_string(),
                    value: Expression::Literal {
                        value: Value::String("subnet-456".to_string()),
                        span: None,
                    },
                    type_annotation: None,
                    mutable: false,
                    doc_comments: Some(vec![]),
                    span: None,
                },
            ],
        };
        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .message
            .contains("Only one of these fields can be present"));
    }

    #[test]
    fn test_multiple_custom_validators() {
        // CIDR validator
        let cidr_validator: ValidatorFn = Box::new(|value| match value {
            Value::String(s) => {
                if s.contains('/') && s.split('/').count() == 2 {
                    Ok(())
                } else {
                    Err(ValidationError::custom(
                        "Invalid CIDR format (expected: 10.0.0.0/16)".to_string(),
                    ))
                }
            }
            _ => Err(ValidationError::custom(
                "Expected string for CIDR".to_string(),
            )),
        });

        // Port range validator
        let port_validator: ValidatorFn = Box::new(|value| match value {
            Value::Int(port) => {
                if (1..=65535).contains(port) {
                    Ok(())
                } else {
                    Err(ValidationError::custom(
                        "Port must be between 1 and 65535".to_string(),
                    ))
                }
            }
            _ => Err(ValidationError::custom(
                "Expected integer for port".to_string(),
            )),
        });

        let cidr_prop = PropertyBuilder::new(TypeDef::String {
            min_length: None,
            max_length: None,
            pattern: None,
            enum_values: None,
        })
        .with_validator("cidr");

        let port_prop = PropertyBuilder::new(TypeDef::Number {
            minimum: None,
            maximum: None,
            integer_only: true,
        })
        .with_validator("port");

        let builder = SchemaBuilder::new("network")
            .field("cidr_block", cidr_prop)
            .field("port", port_prop);

        let mut validator = Validator::from_builder(builder);
        validator
            .register_validator("cidr", cidr_validator)
            .register_validator("port", port_validator);

        // Valid config
        let module = Module {
            statements: vec![
                Statement::Assignment {
                    name: "cidr_block".to_string(),
                    value: Expression::Literal {
                        value: Value::String("10.0.0.0/16".to_string()),
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
        let errors = validator.validate_module(&module).unwrap();
        assert_eq!(errors.len(), 0);
    }

    // ========== Phase 3: Complex Types Tests ==========

    #[test]
    fn test_discriminated_union_valid() {
        // Create schema for storage configuration with discriminated union
        let mut s3_props = HashMap::new();
        s3_props.insert(
            "type".to_string(),
            Property {
                type_def: TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: Some(vec!["s3".to_string()]),
                },
                description: None,
                default: None,
            },
        );
        s3_props.insert(
            "bucket".to_string(),
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
        s3_props.insert(
            "region".to_string(),
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

        let mut local_props = HashMap::new();
        local_props.insert(
            "type".to_string(),
            Property {
                type_def: TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: Some(vec!["local".to_string()]),
                },
                description: None,
                default: None,
            },
        );
        local_props.insert(
            "path".to_string(),
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

        let mut variants = HashMap::new();
        variants.insert(
            "s3".to_string(),
            Box::new(TypeDef::Map {
                properties: s3_props,
                required: vec![
                    "type".to_string(),
                    "bucket".to_string(),
                    "region".to_string(),
                ],
                additional_properties: false,
            }),
        );
        variants.insert(
            "local".to_string(),
            Box::new(TypeDef::Map {
                properties: local_props,
                required: vec!["type".to_string(), "path".to_string()],
                additional_properties: false,
            }),
        );

        // Schema with a "storage" field that is a discriminated union
        let mut root_props = HashMap::new();
        root_props.insert(
            "storage".to_string(),
            Property {
                type_def: TypeDef::DiscriminatedUnion {
                    discriminator: "type".to_string(),
                    variants,
                },
                description: None,
                default: None,
            },
        );

        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: root_props,
                required: vec!["storage".to_string()],
                additional_properties: false,
            },
        };

        let validator = Validator::new(schema);

        // Valid S3 configuration
        let mut storage_map = HashMap::new();
        storage_map.insert("type".to_string(), Value::String("s3".to_string()));
        storage_map.insert("bucket".to_string(), Value::String("my-bucket".to_string()));
        storage_map.insert("region".to_string(), Value::String("us-west-2".to_string()));

        let module = Module {
            statements: vec![Statement::Assignment {
                name: "storage".to_string(),
                value: Expression::Literal {
                    value: Value::Map(storage_map),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };

        let errors = validator.validate_module(&module).unwrap();
        if !errors.is_empty() {
            eprintln!("Errors in test_discriminated_union_valid:");
            for (i, err) in errors.iter().enumerate() {
                eprintln!("  {}: {:?}", i, err);
            }
        }
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_discriminated_union_unknown_variant() {
        let mut variants = HashMap::new();
        variants.insert(
            "s3".to_string(),
            Box::new(TypeDef::Map {
                properties: HashMap::new(),
                required: vec![],
                additional_properties: true,
            }),
        );
        variants.insert(
            "local".to_string(),
            Box::new(TypeDef::Map {
                properties: HashMap::new(),
                required: vec![],
                additional_properties: true,
            }),
        );

        // Schema with a "storage" field that is a discriminated union
        let mut root_props = HashMap::new();
        root_props.insert(
            "storage".to_string(),
            Property {
                type_def: TypeDef::DiscriminatedUnion {
                    discriminator: "type".to_string(),
                    variants,
                },
                description: None,
                default: None,
            },
        );

        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: root_props,
                required: vec![],
                additional_properties: false,
            },
        };

        let validator = Validator::new(schema);

        // Invalid variant
        let mut storage_map = HashMap::new();
        storage_map.insert("type".to_string(), Value::String("azure".to_string()));

        let module = Module {
            statements: vec![Statement::Assignment {
                name: "storage".to_string(),
                value: Expression::Literal {
                    value: Value::Map(storage_map),
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
        assert!(errors[0].message.contains("Unknown variant 'azure'"));
        assert!(errors[0].message.contains("Valid variants"));
        assert!(errors[0].suggestion.is_some());
    }

    #[test]
    fn test_discriminated_union_missing_discriminator() {
        let mut variants = HashMap::new();
        variants.insert(
            "s3".to_string(),
            Box::new(TypeDef::Map {
                properties: HashMap::new(),
                required: vec![],
                additional_properties: true,
            }),
        );

        // Schema with a "storage" field that is a discriminated union
        let mut root_props = HashMap::new();
        root_props.insert(
            "storage".to_string(),
            Property {
                type_def: TypeDef::DiscriminatedUnion {
                    discriminator: "type".to_string(),
                    variants,
                },
                description: None,
                default: None,
            },
        );

        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: root_props,
                required: vec![],
                additional_properties: false,
            },
        };

        let validator = Validator::new(schema);

        // Missing discriminator field
        let storage_map = HashMap::new();

        let module = Module {
            statements: vec![Statement::Assignment {
                name: "storage".to_string(),
                value: Expression::Literal {
                    value: Value::Map(storage_map),
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
        assert_eq!(errors[0].error_type, ErrorType::Required);
        assert!(errors[0].message.contains("type"));
        assert!(errors[0].suggestion.is_some());
    }

    #[test]
    fn test_discriminated_union_non_map_value() {
        let mut variants = HashMap::new();
        variants.insert(
            "s3".to_string(),
            Box::new(TypeDef::Map {
                properties: HashMap::new(),
                required: vec![],
                additional_properties: true,
            }),
        );

        // Schema with a "storage" field that is a discriminated union
        let mut root_props = HashMap::new();
        root_props.insert(
            "storage".to_string(),
            Property {
                type_def: TypeDef::DiscriminatedUnion {
                    discriminator: "type".to_string(),
                    variants,
                },
                description: None,
                default: None,
            },
        );

        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: root_props,
                required: vec![],
                additional_properties: false,
            },
        };

        let validator = Validator::new(schema);

        // Non-map value
        let module = Module {
            statements: vec![Statement::Assignment {
                name: "storage".to_string(),
                value: Expression::Literal {
                    value: Value::String("invalid".to_string()),
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
        assert!(errors[0].message.contains("map"));
    }

    #[test]
    fn test_recursive_type_reference() {
        // Test that Ref type returns appropriate error message
        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Ref {
                name: "TreeNode".to_string(),
            },
        };

        let validator = Validator::new(schema);

        let module = Module {
            statements: vec![Statement::Assignment {
                name: "node".to_string(),
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
        assert!(errors[0].message.contains("Recursive type reference"));
        assert!(errors[0].message.contains("TreeNode"));
        assert!(errors[0].suggestion.is_some());
    }

    #[test]
    fn test_simple_union_type() {
        // Test existing union type functionality
        let mut root_props = HashMap::new();
        root_props.insert(
            "value".to_string(),
            Property {
                type_def: TypeDef::Union {
                    types: vec![
                        TypeDef::String {
                            min_length: None,
                            max_length: None,
                            pattern: None,
                            enum_values: None,
                        },
                        TypeDef::Number {
                            minimum: None,
                            maximum: None,
                            integer_only: false,
                        },
                    ],
                },
                description: None,
                default: None,
            },
        );

        let schema = Schema {
            version: "1.0".to_string(),
            title: None,
            description: None,
            type_def: TypeDef::Map {
                properties: root_props,
                required: vec![],
                additional_properties: false,
            },
        };

        let validator = Validator::new(schema);

        // String value (valid)
        let module1 = Module {
            statements: vec![Statement::Assignment {
                name: "value".to_string(),
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
        let errors = validator.validate_module(&module1).unwrap();
        assert_eq!(errors.len(), 0);

        // Number value (valid)
        let module2 = Module {
            statements: vec![Statement::Assignment {
                name: "value".to_string(),
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
        let errors = validator.validate_module(&module2).unwrap();
        assert_eq!(errors.len(), 0);

        // Boolean value (invalid)
        let module3 = Module {
            statements: vec![Statement::Assignment {
                name: "value".to_string(),
                value: Expression::Literal {
                    value: Value::Bool(true),
                    span: None,
                },
                type_annotation: None,
                mutable: false,
                doc_comments: Some(vec![]),
                span: None,
            }],
        };
        let errors = validator.validate_module(&module3).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .message
            .contains("Value does not match any type in union"));
    }

    // ============================================================================
    // Phase 4: Schema Composition Tests
    // ============================================================================

    #[test]
    fn test_schema_versioning() {
        let schema = SchemaBuilder::new("User")
            .version("1.2.3")
            .description("User schema with version")
            .field(
                "name",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                }),
            )
            .mark_required("name")
            .build();

        assert_eq!(schema.version, "1.2.3");
        assert_eq!(schema.title, Some("User".to_string()));
    }

    #[test]
    fn test_schema_inheritance_basic() {
        // Parent schema
        let parent = SchemaBuilder::new("BaseEntity")
            .description("Base entity with common fields")
            .field(
                "id",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                })
                .description("Unique identifier"),
            )
            .field(
                "created_at",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                })
                .description("Creation timestamp"),
            )
            .mark_required("id");

        // Child schema extends parent
        let child = SchemaBuilder::new("User")
            .description("User entity")
            .extends(parent)
            .field(
                "email",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                })
                .description("User email"),
            )
            .mark_required("email")
            .build();

        // Should have all three properties
        if let TypeDef::Map {
            properties,
            required,
            ..
        } = child.type_def
        {
            assert_eq!(properties.len(), 3);
            assert!(properties.contains_key("id"));
            assert!(properties.contains_key("created_at"));
            assert!(properties.contains_key("email"));

            // Should have both required fields
            assert_eq!(required.len(), 2);
            assert!(required.contains(&"id".to_string()));
            assert!(required.contains(&"email".to_string()));
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_schema_inheritance_override() {
        // Parent schema
        let parent = SchemaBuilder::new("BaseEntity").field(
            "status",
            PropertyBuilder::new(TypeDef::String {
                min_length: None,
                max_length: None,
                pattern: None,
                enum_values: Some(vec!["active".to_string(), "inactive".to_string()]),
            })
            .description("Parent status"),
        );

        // Child overrides parent's status field
        let child = SchemaBuilder::new("User")
            .extends(parent)
            .field(
                "status",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: Some(vec![
                        "active".to_string(),
                        "inactive".to_string(),
                        "suspended".to_string(),
                    ]),
                })
                .description("User status with more options"),
            )
            .build();

        // Check that child's property overwrote parent's
        if let TypeDef::Map { properties, .. } = child.type_def {
            let status_prop = properties.get("status").unwrap();
            if let TypeDef::String { enum_values, .. } = &status_prop.type_def {
                assert_eq!(enum_values.as_ref().unwrap().len(), 3);
                assert!(enum_values
                    .as_ref()
                    .unwrap()
                    .contains(&"suspended".to_string()));
            } else {
                panic!("Expected String type");
            }
            assert_eq!(
                status_prop.description,
                Some("User status with more options".to_string())
            );
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_schema_merging_multiple() {
        let schema1 = SchemaBuilder::new("Schema1").field(
            "field1",
            PropertyBuilder::new(TypeDef::String {
                min_length: None,
                max_length: None,
                pattern: None,
                enum_values: None,
            }),
        );

        let schema2 = SchemaBuilder::new("Schema2").field(
            "field2",
            PropertyBuilder::new(TypeDef::Number {
                minimum: None,
                maximum: None,
                integer_only: false,
            }),
        );

        let schema3 =
            SchemaBuilder::new("Schema3").field("field3", PropertyBuilder::new(TypeDef::Boolean));

        // Merge all three schemas
        let merged = SchemaBuilder::new("Merged")
            .merge(vec![schema1, schema2, schema3])
            .build();

        // Should have all three properties
        if let TypeDef::Map { properties, .. } = merged.type_def {
            assert_eq!(properties.len(), 3);
            assert!(properties.contains_key("field1"));
            assert!(properties.contains_key("field2"));
            assert!(properties.contains_key("field3"));
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_schema_merging_with_required() {
        let schema1 = SchemaBuilder::new("Schema1")
            .field(
                "field1",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                }),
            )
            .mark_required("field1");

        let schema2 = SchemaBuilder::new("Schema2")
            .field(
                "field2",
                PropertyBuilder::new(TypeDef::Number {
                    minimum: None,
                    maximum: None,
                    integer_only: false,
                }),
            )
            .mark_required("field2");

        // Merge schemas
        let merged = SchemaBuilder::new("Merged")
            .merge(vec![schema1, schema2])
            .build();

        // Should have both required fields
        if let TypeDef::Map { required, .. } = merged.type_def {
            assert_eq!(required.len(), 2);
            assert!(required.contains(&"field1".to_string()));
            assert!(required.contains(&"field2".to_string()));
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_schema_merging_override() {
        // First schema defines field with one constraint
        let schema1 = SchemaBuilder::new("Schema1").field(
            "value",
            PropertyBuilder::new(TypeDef::Number {
                minimum: Some(0.0),
                maximum: Some(100.0),
                integer_only: false,
            })
            .description("Original description"),
        );

        // Second schema redefines the same field (should override)
        let schema2 = SchemaBuilder::new("Schema2").field(
            "value",
            PropertyBuilder::new(TypeDef::Number {
                minimum: Some(0.0),
                maximum: Some(1000.0),
                integer_only: true,
            })
            .description("Updated description"),
        );

        // Merge - schema2 should win
        let merged = SchemaBuilder::new("Merged")
            .merge(vec![schema1, schema2])
            .build();

        if let TypeDef::Map { properties, .. } = merged.type_def {
            let value_prop = properties.get("value").unwrap();
            if let TypeDef::Number {
                maximum,
                integer_only,
                ..
            } = &value_prop.type_def
            {
                assert_eq!(*maximum, Some(1000.0));
                assert!(*integer_only);
            } else {
                panic!("Expected Number type");
            }
            assert_eq!(
                value_prop.description,
                Some("Updated description".to_string())
            );
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_documentation_generation_basic() {
        let schema = SchemaBuilder::new("User")
            .version("1.0.0")
            .description("User account schema")
            .field(
                "username",
                PropertyBuilder::new(TypeDef::String {
                    min_length: Some(3),
                    max_length: Some(20),
                    pattern: None,
                    enum_values: None,
                })
                .description("Unique username for the account"),
            )
            .field(
                "age",
                PropertyBuilder::new(TypeDef::Number {
                    minimum: Some(0.0),
                    maximum: Some(150.0),
                    integer_only: true,
                })
                .description("User's age in years"),
            )
            .mark_required("username")
            .build();

        let docs = SchemaBuilder::from_schema(schema).generate_docs();

        // Check that documentation contains expected content
        assert!(docs.contains("# User"));
        assert!(docs.contains("**Version**: 1.0.0"));
        assert!(docs.contains("User account schema"));
        assert!(docs.contains("## Fields"));
        assert!(docs.contains("### `username` (required)"));
        assert!(docs.contains("Unique username for the account"));
        assert!(docs.contains("### `age`"));
        assert!(docs.contains("User's age in years"));
    }

    #[test]
    fn test_documentation_generation_with_validators() {
        let schema_builder = SchemaBuilder::new("Contact")
            .description("Contact information")
            .field(
                "email",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                })
                .description("Email address")
                .with_validator("email_validator"),
            )
            .field(
                "phone",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                })
                .description("Phone number")
                .with_validator("phone_validator")
                .requires("email"),
            );

        let docs = schema_builder.generate_docs();

        // Check that validators are documented
        assert!(docs.contains("**Validators**: email_validator"));
        assert!(docs.contains("**Validators**: phone_validator"));
        assert!(docs.contains("**Requires**: email"));
    }

    #[test]
    fn test_documentation_generation_with_mutual_exclusion() {
        let schema_builder = SchemaBuilder::new("Payment")
            .description("Payment method")
            .field(
                "credit_card",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                }),
            )
            .field(
                "bank_account",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                }),
            )
            .mutually_exclusive(vec!["credit_card".to_string(), "bank_account".to_string()]);

        let docs = schema_builder.generate_docs();

        // Check that mutual exclusion is documented
        assert!(docs.contains("## Mutually Exclusive Groups"));
        assert!(docs.contains("Only one of: credit_card, bank_account"));
    }

    #[test]
    fn test_schema_builder_from_schema() {
        // Create a schema using the builder
        let original = SchemaBuilder::new("Test")
            .version("1.0.0")
            .description("Test schema")
            .field(
                "field",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                }),
            )
            .mark_required("field")
            .build();

        // Convert back to builder
        let builder = SchemaBuilder::from_schema(original.clone());

        // Build again and compare
        let rebuilt = builder.build();

        assert_eq!(rebuilt.version, original.version);
        assert_eq!(rebuilt.title, original.title);
        assert_eq!(rebuilt.description, original.description);

        // Note: We can't directly compare type_defs due to the complexity,
        // but we verified the builder pattern works
    }

    // ============================================================================
    // Phase 5: Schema Export Tests
    // ============================================================================

    #[test]
    fn test_json_schema_export_basic() {
        let schema = SchemaBuilder::new("User")
            .version("1.0.0")
            .description("User account schema")
            .field(
                "username",
                PropertyBuilder::new(TypeDef::String {
                    min_length: Some(3),
                    max_length: Some(20),
                    pattern: None,
                    enum_values: None,
                }),
            )
            .field(
                "age",
                PropertyBuilder::new(TypeDef::Number {
                    minimum: Some(0.0),
                    maximum: Some(150.0),
                    integer_only: true,
                }),
            )
            .mark_required("username")
            .build();

        let json_schema = schema.to_json_schema();

        // Check JSON Schema structure
        assert!(json_schema.contains("\"$schema\": \"http://json-schema.org/draft-07/schema#\""));
        assert!(json_schema.contains("\"title\": \"User\""));
        assert!(json_schema.contains("\"description\": \"User account schema\""));
        assert!(json_schema.contains("\"type\": \"object\""));
        assert!(json_schema.contains("\"username\""));
        assert!(json_schema.contains("\"age\""));
        assert!(json_schema.contains("\"required\""));
        assert!(json_schema.contains("\"minLength\": 3"));
        assert!(json_schema.contains("\"maxLength\": 20"));
        assert!(json_schema.contains("\"minimum\": 0"));
        assert!(json_schema.contains("\"maximum\": 150"));
        assert!(json_schema.contains("\"type\": \"integer\""));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_schema).unwrap();
        assert_eq!(parsed["$schema"], "http://json-schema.org/draft-07/schema#");
        assert_eq!(parsed["title"], "User");
    }

    #[test]
    fn test_json_schema_export_with_constraints() {
        let schema = SchemaBuilder::new("Email")
            .field(
                "address",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: Some("^[^@]+@[^@]+$".to_string()),
                    enum_values: None,
                }),
            )
            .build();

        let json_schema = schema.to_json_schema();

        assert!(json_schema.contains("\"pattern\": \"^[^@]+@[^@]+$\""));
    }

    #[test]
    fn test_json_schema_export_with_enums() {
        let schema = SchemaBuilder::new("Config")
            .field(
                "environment",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: Some(vec![
                        "development".to_string(),
                        "staging".to_string(),
                        "production".to_string(),
                    ]),
                }),
            )
            .build();

        let json_schema = schema.to_json_schema();

        assert!(json_schema.contains("\"enum\""));
        assert!(json_schema.contains("\"development\""));
        assert!(json_schema.contains("\"staging\""));
        assert!(json_schema.contains("\"production\""));
    }

    #[test]
    fn test_json_schema_export_with_list() {
        let schema = SchemaBuilder::new("Tags")
            .field(
                "tags",
                PropertyBuilder::new(TypeDef::List {
                    items: Box::new(TypeDef::String {
                        min_length: None,
                        max_length: None,
                        pattern: None,
                        enum_values: None,
                    }),
                    min_items: Some(1),
                    max_items: Some(10),
                }),
            )
            .build();

        let json_schema = schema.to_json_schema();

        assert!(json_schema.contains("\"type\": \"array\""));
        assert!(json_schema.contains("\"items\""));
        assert!(json_schema.contains("\"minItems\": 1"));
        assert!(json_schema.contains("\"maxItems\": 10"));
    }

    #[test]
    fn test_json_schema_export_with_union() {
        let schema = SchemaBuilder::new("Value")
            .field(
                "data",
                PropertyBuilder::new(TypeDef::Union {
                    types: vec![
                        TypeDef::String {
                            min_length: None,
                            max_length: None,
                            pattern: None,
                            enum_values: None,
                        },
                        TypeDef::Number {
                            minimum: None,
                            maximum: None,
                            integer_only: false,
                        },
                    ],
                }),
            )
            .build();

        let json_schema = schema.to_json_schema();

        assert!(json_schema.contains("\"anyOf\""));
        assert!(json_schema.contains("\"type\": \"string\""));
        assert!(json_schema.contains("\"type\": \"number\""));
    }

    #[test]
    fn test_openapi_export_basic() {
        let schema = SchemaBuilder::new("Product")
            .description("Product resource")
            .field(
                "name",
                PropertyBuilder::new(TypeDef::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                    enum_values: None,
                })
                .description("Product name"),
            )
            .field(
                "price",
                PropertyBuilder::new(TypeDef::Number {
                    minimum: Some(0.0),
                    maximum: None,
                    integer_only: false,
                })
                .description("Product price"),
            )
            .mark_required("name")
            .mark_required("price")
            .build();

        let openapi_schema = schema.to_openapi();

        // Check OpenAPI structure
        assert!(openapi_schema.contains("\"title\": \"Product\""));
        assert!(openapi_schema.contains("\"description\": \"Product resource\""));
        assert!(openapi_schema.contains("\"type\": \"object\""));
        assert!(openapi_schema.contains("\"name\""));
        assert!(openapi_schema.contains("\"price\""));
        assert!(openapi_schema.contains("\"required\""));
        assert!(openapi_schema.contains("\"minimum\": 0"));

        // Verify field descriptions are included
        assert!(openapi_schema.contains("\"Product name\""));
        assert!(openapi_schema.contains("\"Product price\""));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&openapi_schema).unwrap();
        assert_eq!(parsed["title"], "Product");
        assert_eq!(parsed["type"], "object");
    }

    #[test]
    fn test_openapi_export_with_nested_objects() {
        let schema = SchemaBuilder::new("Order")
            .field(
                "customer",
                PropertyBuilder::new(TypeDef::Map {
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "name".to_string(),
                            Property {
                                type_def: TypeDef::String {
                                    min_length: None,
                                    max_length: None,
                                    pattern: None,
                                    enum_values: None,
                                },
                                description: Some("Customer name".to_string()),
                                default: None,
                            },
                        );
                        props
                    },
                    required: vec!["name".to_string()],
                    additional_properties: false,
                }),
            )
            .build();

        let openapi_schema = schema.to_openapi();

        assert!(openapi_schema.contains("\"customer\""));
        assert!(openapi_schema.contains("\"type\": \"object\""));
        assert!(openapi_schema.contains("\"additionalProperties\": false"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&openapi_schema).unwrap();
        assert!(parsed["properties"]["customer"].is_object());
    }

    #[test]
    fn test_json_schema_export_discriminated_union() {
        let mut variants = HashMap::new();
        variants.insert(
            "local".to_string(),
            Box::new(TypeDef::Map {
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "path".to_string(),
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
                    props
                },
                required: vec![],
                additional_properties: false,
            }),
        );

        let schema = SchemaBuilder::new("Storage")
            .field(
                "config",
                PropertyBuilder::new(TypeDef::DiscriminatedUnion {
                    discriminator: "type".to_string(),
                    variants,
                }),
            )
            .build();

        let json_schema = schema.to_json_schema();

        // Should have oneOf with discriminator
        assert!(json_schema.contains("\"oneOf\""));
        assert!(json_schema.contains("\"discriminator\""));
        assert!(json_schema.contains("\"propertyName\": \"type\""));
        assert!(json_schema.contains("\"const\": \"local\""));
    }

    #[test]
    fn test_json_schema_ref_type() {
        let schema = SchemaBuilder::new("Container")
            .field(
                "reference",
                PropertyBuilder::new(TypeDef::Ref {
                    name: "ExternalType".to_string(),
                }),
            )
            .build();

        let json_schema = schema.to_json_schema();

        assert!(json_schema.contains("\"$ref\": \"#/definitions/ExternalType\""));
    }

    #[test]
    fn test_json_schema_any_type() {
        let schema = SchemaBuilder::new("Dynamic")
            .field("value", PropertyBuilder::new(TypeDef::Any))
            .build();

        let json_schema = schema.to_json_schema();

        // Any type should be an empty schema object (allows anything)
        assert!(json_schema.contains("\"value\""));
    }

    // ========== Schema Generation Tests ==========

    #[test]
    fn test_generate_from_simple_example() {
        use crate::parse_str;

        let example = r#"
        name = "test"
        port = 8080
        enabled = true
        "#;

        let module = parse_str(example).unwrap();
        let schema = generate_from_examples(&[module], GenerateOptions::default()).unwrap();

        // Check that all fields are present
        if let TypeDef::Map { properties, .. } = &schema.type_def {
            assert!(properties.contains_key("name"));
            assert!(properties.contains_key("port"));
            assert!(properties.contains_key("enabled"));

            // Check types
            assert!(matches!(
                properties.get("name").unwrap().type_def,
                TypeDef::String { .. }
            ));
            assert!(matches!(
                properties.get("port").unwrap().type_def,
                TypeDef::Number {
                    integer_only: true,
                    ..
                }
            ));
            assert!(matches!(
                properties.get("enabled").unwrap().type_def,
                TypeDef::Boolean
            ));
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_generate_from_nested_map() {
        use crate::parse_str;

        let example = r#"
        server = (
            host = "localhost",
            port = 8080
        )
        "#;

        let module = parse_str(example).unwrap();
        let schema = generate_from_examples(&[module], GenerateOptions::default()).unwrap();

        // Check nested structure
        if let TypeDef::Map { properties, .. } = &schema.type_def {
            assert!(properties.contains_key("server"));

            if let TypeDef::Map {
                properties: nested_props,
                ..
            } = &properties.get("server").unwrap().type_def
            {
                assert!(nested_props.contains_key("host"));
                assert!(nested_props.contains_key("port"));
            } else {
                panic!("Expected nested Map type");
            }
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_generate_from_list() {
        use crate::parse_str;

        let example = r#"
        tags = ["web", "production"]
        numbers = [1, 2, 3]
        "#;

        let module = parse_str(example).unwrap();
        let schema = generate_from_examples(&[module], GenerateOptions::default()).unwrap();

        if let TypeDef::Map { properties, .. } = &schema.type_def {
            // Check string list
            if let TypeDef::List { items, .. } = &properties.get("tags").unwrap().type_def {
                assert!(matches!(**items, TypeDef::String { .. }));
            } else {
                panic!("Expected List type for tags");
            }

            // Check number list
            if let TypeDef::List { items, .. } = &properties.get("numbers").unwrap().type_def {
                assert!(matches!(**items, TypeDef::Number { .. }));
            } else {
                panic!("Expected List type for numbers");
            }
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_generate_required_fields() {
        use crate::parse_str;

        let example1 = r#"
        name = "test1"
        port = 8080
        optional_field = "value1"
        "#;

        let example2 = r#"
        name = "test2"
        port = 9000
        "#;

        let module1 = parse_str(example1).unwrap();
        let module2 = parse_str(example2).unwrap();

        let schema =
            generate_from_examples(&[module1, module2], GenerateOptions::default()).unwrap();

        // Fields present in all examples should be required
        if let TypeDef::Map { required, .. } = &schema.type_def {
            assert!(required.contains(&"name".to_string()));
            assert!(required.contains(&"port".to_string()));
            // optional_field only in first example, so not required
            assert!(!required.contains(&"optional_field".to_string()));
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_generate_with_all_optional() {
        use crate::parse_str;

        let example = r#"
        name = "test"
        port = 8080
        "#;

        let module = parse_str(example).unwrap();
        let options = GenerateOptions {
            infer_types: true,
            infer_constraints: true,
            mark_all_optional: true,
        };
        let schema = generate_from_examples(&[module], options).unwrap();

        // No fields should be required
        if let TypeDef::Map { required, .. } = &schema.type_def {
            assert!(required.is_empty());
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_generate_without_type_inference() {
        use crate::parse_str;

        let example = r#"
        name = "test"
        port = 8080
        "#;

        let module = parse_str(example).unwrap();
        let options = GenerateOptions {
            infer_types: false,
            infer_constraints: false,
            mark_all_optional: false,
        };
        let schema = generate_from_examples(&[module], options).unwrap();

        // All fields should be Any type when type inference is disabled
        if let TypeDef::Map { properties, .. } = &schema.type_def {
            assert!(matches!(
                properties.get("name").unwrap().type_def,
                TypeDef::Any
            ));
            assert!(matches!(
                properties.get("port").unwrap().type_def,
                TypeDef::Any
            ));
        } else {
            panic!("Expected Map type");
        }
    }

    #[test]
    fn test_generate_with_pattern_detection() {
        use crate::parse_str;

        let example = r#"
        email = "user@example.com"
        url = "https://example.com"
        file_path = "/etc/config.json"
        "#;

        let module = parse_str(example).unwrap();
        let schema = generate_from_examples(&[module], GenerateOptions::default()).unwrap();

        if let TypeDef::Map { properties, .. } = &schema.type_def {
            // Email should have pattern
            if let TypeDef::String { pattern, .. } = &properties.get("email").unwrap().type_def {
                assert!(pattern.is_some());
                assert!(pattern.as_ref().unwrap().contains("@"));
            } else {
                panic!("Expected String type for email");
            }

            // URL should have pattern
            if let TypeDef::String { pattern, .. } = &properties.get("url").unwrap().type_def {
                assert!(pattern.is_some());
                assert!(pattern.as_ref().unwrap().contains("https?"));
            } else {
                panic!("Expected String type for url");
            }
        } else {
            panic!("Expected Map type");
        }
    }
}
