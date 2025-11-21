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
}
