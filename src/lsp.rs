//! Language Server Protocol implementation for JCL

use crate::linter;
use crate::schema::Validator;
use crate::symbol_table::SymbolTable;
use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// JCL Language Server
pub struct JclLanguageServer {
    client: Client,
    document_map: Arc<RwLock<HashMap<String, String>>>,
    /// Schema validator (if a schema file is found)
    schema_validator: Arc<RwLock<Option<Validator>>>,
    /// Path to the schema file being used
    schema_path: Arc<RwLock<Option<PathBuf>>>,
    /// Workspace root URI
    workspace_root: Arc<RwLock<Option<Url>>>,
}

impl JclLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: Arc::new(RwLock::new(HashMap::new())),
            schema_validator: Arc::new(RwLock::new(None)),
            schema_path: Arc::new(RwLock::new(None)),
            workspace_root: Arc::new(RwLock::new(None)),
        }
    }

    /// Discover and load schema file from workspace
    async fn discover_schema(&self, workspace_root: Option<&Url>) {
        if let Some(root) = workspace_root {
            if let Ok(root_path) = root.to_file_path() {
                // Look for schema files in order of preference
                let schema_candidates = vec![
                    root_path.join(".jcf-schema.json"),
                    root_path.join(".jcf-schema.yaml"),
                    root_path.join(".jcf-schema.yml"),
                    root_path.join("jcl-schema.json"),
                    root_path.join("jcl-schema.yaml"),
                ];

                for schema_file in schema_candidates {
                    if schema_file.exists() {
                        match self.load_schema(&schema_file).await {
                            Ok(()) => {
                                self.client
                                    .log_message(
                                        MessageType::INFO,
                                        format!("Loaded schema from {}", schema_file.display()),
                                    )
                                    .await;
                                return;
                            }
                            Err(e) => {
                                self.client
                                    .log_message(
                                        MessageType::ERROR,
                                        format!(
                                            "Failed to load schema from {}: {}",
                                            schema_file.display(),
                                            e
                                        ),
                                    )
                                    .await;
                            }
                        }
                    }
                }

                self.client
                    .log_message(
                        MessageType::INFO,
                        "No schema file found in workspace. Schema validation disabled."
                            .to_string(),
                    )
                    .await;
            }
        }
    }

    /// Load schema from file
    async fn load_schema(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;

        let validator = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            Validator::from_json(&content)?
        } else {
            Validator::from_yaml(&content)?
        };

        *self.schema_validator.write().await = Some(validator);
        *self.schema_path.write().await = Some(path.clone());

        // Start watching the schema file for changes
        self.watch_schema_file(path.clone()).await;

        Ok(())
    }

    /// Watch schema file for changes and reload automatically
    async fn watch_schema_file(&self, path: PathBuf) {
        let client = self.client.clone();
        let schema_validator = self.schema_validator.clone();
        let _document_map = self.document_map.clone();

        // Spawn a task to watch the file
        tokio::spawn(async move {
            // Create a channel for receiving watch events
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            // Create watcher with event handler
            let mut watcher =
                match notify::recommended_watcher(move |res: notify::Result<Event>| {
                    if let Ok(event) = res {
                        // Send event through channel
                        let _ = tx.blocking_send(event);
                    }
                }) {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("Failed to create schema file watcher: {}", e);
                        return;
                    }
                };

            // Start watching the schema file
            if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
                eprintln!("Failed to watch schema file: {}", e);
                return;
            }

            // Process file change events
            while let Some(event) = rx.recv().await {
                // Only reload on modify events
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    // Reload the schema
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            let validator_result =
                                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                                    Validator::from_json(&content)
                                } else {
                                    Validator::from_yaml(&content)
                                };

                            match validator_result {
                                Ok(validator) => {
                                    // Update the validator
                                    *schema_validator.write().await = Some(validator);

                                    // Notify the client
                                    client
                                        .show_message(
                                            MessageType::INFO,
                                            format!("Schema file reloaded: {}", path.display()),
                                        )
                                        .await;

                                    // Re-validate all open documents
                                    // Note: This would require access to get_diagnostics which needs self
                                    // For now, we'll just notify - documents will be re-validated on next change
                                }
                                Err(e) => {
                                    // Notify about schema reload error
                                    client
                                        .show_message(
                                            MessageType::ERROR,
                                            format!("Failed to reload schema: {}", e),
                                        )
                                        .await;
                                }
                            }
                        }
                        Err(e) => {
                            client
                                .show_message(
                                    MessageType::ERROR,
                                    format!("Failed to read schema file: {}", e),
                                )
                                .await;
                        }
                    }
                }
            }

            // Keep the watcher alive
            drop(watcher);
        });
    }

    /// Find the field path at a given cursor position in the document
    /// Returns something like "config.database.port" if the cursor is on a map field
    fn find_field_path_at_position(
        &self,
        module: &crate::ast::Module,
        _text: &str,
        position: &Position,
    ) -> Option<String> {
        use crate::ast::{Expression, Statement};

        // Convert LSP position (0-indexed) to line/column (1-indexed)
        let target_line = (position.line + 1) as usize;
        let target_col = (position.character + 1) as usize;

        // Find the statement and field at this position
        for stmt in &module.statements {
            if let Statement::Assignment {
                name, value, span, ..
            } = stmt
            {
                // Check if we're in this assignment
                if let Some(s) = span {
                    if s.line == target_line
                        && target_col >= s.column
                        && target_col < s.column + name.len()
                    {
                        // Cursor is on the assignment name itself
                        return Some(name.clone());
                    }
                }

                // Check if we're inside the value (a map)
                if let Expression::Map { entries, .. } = value {
                    if let Some((key, _val)) = entries.first() {
                        // Build full path: root.key
                        // This is a simplified approach - we'd need to handle nested maps
                        let field_path = format!("{}.{}", name, key);

                        // For now, return the first field path we find
                        // A more sophisticated implementation would check exact positions
                        return Some(field_path);
                    }
                }
            }
        }

        None
    }

    /// Get hover information for a field from the schema
    fn get_schema_hover_info(&self, validator: &Validator, field_path: &str) -> Option<String> {
        use crate::schema::TypeDef;

        // Navigate the schema to find the field
        let parts: Vec<&str> = field_path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // Start from the root type
        let schema = validator.schema();
        let mut current_type = &schema.type_def;

        // Navigate through the path
        for part in &parts {
            match current_type {
                TypeDef::Map {
                    properties,
                    required,
                    ..
                } => {
                    if let Some(property) = properties.get(*part) {
                        current_type = &property.type_def;

                        // If this is the last part, generate hover text
                        if part == parts.last().unwrap() {
                            let is_required = required.contains(&part.to_string());
                            return Some(self.format_type_hover(
                                &property.type_def,
                                is_required,
                                property.description.as_deref(),
                            ));
                        }
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        None
    }

    /// Get schema-based completions for the current context
    fn get_schema_completions(
        &self,
        module: &crate::ast::Module,
        validator: &Validator,
        _position: &Position,
    ) -> Option<Vec<CompletionItem>> {
        use crate::ast::{Expression, Statement};
        use crate::schema::TypeDef;

        // Simplified implementation: suggest properties if we're inside a map
        // A more sophisticated implementation would determine the exact context

        let mut completions = Vec::new();
        let schema = validator.schema();

        // Check each top-level assignment to see if we're inside its map
        for stmt in &module.statements {
            if let Statement::Assignment { name, value, .. } = stmt {
                if let Expression::Map { entries, .. } = value {
                    // Get the schema type for this assignment
                    if let TypeDef::Map { properties, .. } = &schema.type_def {
                        if let Some(prop) = properties.get(name) {
                            if let TypeDef::Map {
                                properties: nested_props,
                                required: nested_required,
                                ..
                            } = &prop.type_def
                            {
                                // Get existing keys
                                let existing_keys: std::collections::HashSet<&str> =
                                    entries.iter().map(|(k, _)| k.as_str()).collect();

                                // Suggest properties that aren't already defined
                                for (key, property) in nested_props {
                                    if !existing_keys.contains(key.as_str()) {
                                        let is_required = nested_required.contains(key);
                                        let label = if is_required {
                                            format!("{} (required)", key)
                                        } else {
                                            key.clone()
                                        };

                                        let detail = match &property.type_def {
                                            TypeDef::String { .. } => Some("String".to_string()),
                                            TypeDef::Number {
                                                integer_only: true, ..
                                            } => Some("Int".to_string()),
                                            TypeDef::Number {
                                                integer_only: false,
                                                ..
                                            } => Some("Number".to_string()),
                                            TypeDef::Boolean => Some("Boolean".to_string()),
                                            TypeDef::List { .. } => Some("List".to_string()),
                                            TypeDef::Map { .. } => Some("Map".to_string()),
                                            _ => None,
                                        };

                                        completions.push(CompletionItem {
                                            label,
                                            kind: Some(CompletionItemKind::PROPERTY),
                                            detail,
                                            documentation: property.description.as_ref().map(|d| {
                                                Documentation::MarkupContent(MarkupContent {
                                                    kind: MarkupKind::Markdown,
                                                    value: d.clone(),
                                                })
                                            }),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if completions.is_empty() {
            None
        } else {
            Some(completions)
        }
    }

    /// Format type information for hover display
    fn format_type_hover(
        &self,
        type_def: &crate::schema::TypeDef,
        is_required: bool,
        description: Option<&str>,
    ) -> String {
        use crate::schema::TypeDef;

        let mut hover = String::new();

        // Type name
        let type_name = match type_def {
            TypeDef::String { .. } => "String",
            TypeDef::Number {
                integer_only: true, ..
            } => "Int",
            TypeDef::Number {
                integer_only: false,
                ..
            } => "Number",
            TypeDef::Boolean => "Boolean",
            TypeDef::Null => "Null",
            TypeDef::List { .. } => "List",
            TypeDef::Map { .. } => "Map",
            TypeDef::Any => "Any",
            TypeDef::Union { .. } => "Union",
            TypeDef::DiscriminatedUnion { .. } => "DiscriminatedUnion",
            TypeDef::Ref { .. } => "Reference",
        };

        hover.push_str(&format!("**Type:** `{}`\n\n", type_name));

        // Required/Optional
        hover.push_str(&format!(
            "**Required:** {}\n\n",
            if is_required { "Yes" } else { "No" }
        ));

        // Constraints
        match type_def {
            TypeDef::String {
                min_length,
                max_length,
                pattern,
                enum_values,
            } => {
                if min_length.is_some()
                    || max_length.is_some()
                    || pattern.is_some()
                    || enum_values.is_some()
                {
                    hover.push_str("**Constraints:**\n");
                    if let Some(min) = min_length {
                        hover.push_str(&format!("- Min length: {}\n", min));
                    }
                    if let Some(max) = max_length {
                        hover.push_str(&format!("- Max length: {}\n", max));
                    }
                    if let Some(pat) = pattern {
                        hover.push_str(&format!("- Pattern: `{}`\n", pat));
                    }
                    if let Some(enums) = enum_values {
                        hover.push_str(&format!("- Allowed values: {}\n", enums.join(", ")));
                    }
                    hover.push('\n');
                }
            }
            TypeDef::Number {
                minimum,
                maximum,
                integer_only,
            } => {
                hover.push_str("**Constraints:**\n");
                if let Some(min) = minimum {
                    hover.push_str(&format!("- Minimum: {}\n", min));
                }
                if let Some(max) = maximum {
                    hover.push_str(&format!("- Maximum: {}\n", max));
                }
                if *integer_only {
                    hover.push_str("- Integer only\n");
                }
                hover.push('\n');
            }
            _ => {}
        }

        // Description
        if let Some(desc) = description {
            hover.push_str(&format!("**Description:** {}\n", desc));
        }

        hover
    }

    /// Find a node in the AST by field path (e.g., "config.database.port")
    /// Returns the source span of the matching expression
    fn find_span_by_path(
        module: &crate::ast::Module,
        path: &str,
    ) -> Option<crate::ast::SourceSpan> {
        use crate::ast::{Expression, Statement};

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // Find the root assignment
        let root_name = parts[0];
        let root_stmt = module.statements.iter().find(|stmt| {
            if let Statement::Assignment { name, .. } = stmt {
                name == root_name
            } else {
                false
            }
        })?;

        // Get the root expression
        let root_expr = if let Statement::Assignment { value, .. } = root_stmt {
            value
        } else {
            return None;
        };

        // If path has only one component, return the span of the entire assignment value
        if parts.len() == 1 {
            return root_expr.span().cloned();
        }

        // Traverse nested structure for remaining path components
        let mut current_expr = root_expr;
        for &key in &parts[1..] {
            match current_expr {
                Expression::Map { entries, .. } => {
                    // Find the entry with this key
                    if let Some((_k, expr)) = entries.iter().find(|(k, _)| k == key) {
                        current_expr = expr;
                    } else {
                        return None;
                    }
                }
                Expression::Variable { name, span } => {
                    // This is a reference to another variable - try to resolve it
                    // For now, just return its span as a fallback
                    if name == key {
                        return span.clone();
                    }
                    return None;
                }
                _ => return None,
            }
        }

        current_expr.span().cloned()
    }

    /// Get diagnostics for a document
    async fn get_diagnostics(&self, _uri: &Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Parse the document
        match crate::parse_str(text) {
            Ok(module) => {
                // Run linter
                if let Ok(issues) = linter::lint(&module) {
                    for issue in issues {
                        let severity = match issue.severity {
                            linter::Severity::Error => DiagnosticSeverity::ERROR,
                            linter::Severity::Warning => DiagnosticSeverity::WARNING,
                            linter::Severity::Info => DiagnosticSeverity::INFORMATION,
                        };

                        let message = if let Some(suggestion) = &issue.suggestion {
                            format!("{}\nHelp: {}", issue.message, suggestion)
                        } else {
                            issue.message.clone()
                        };

                        // Use precise span if available, otherwise use entire document range
                        let range = if let Some(span) = &issue.span {
                            Range {
                                start: Position {
                                    line: span.line.saturating_sub(1) as u32, // LSP is 0-indexed
                                    character: span.column.saturating_sub(1) as u32,
                                },
                                end: Position {
                                    line: span.line.saturating_sub(1) as u32,
                                    character: (span.column.saturating_sub(1) + span.length) as u32,
                                },
                            }
                        } else {
                            Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: text.lines().count() as u32,
                                    character: 0,
                                },
                            }
                        };

                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(severity),
                            code: Some(NumberOrString::String(issue.rule.clone())),
                            source: Some("jcl".to_string()),
                            message,
                            ..Default::default()
                        });
                    }
                }

                // Run schema validation if a schema is loaded
                if let Some(validator) = self.schema_validator.read().await.as_ref() {
                    if let Ok(schema_errors) = validator.validate_module(&module) {
                        for error in schema_errors {
                            let message = if let Some(suggestion) = &error.suggestion {
                                format!("{}\nHelp: {}", error.message, suggestion)
                            } else {
                                error.message.clone()
                            };

                            // Try to find the precise location using the field path
                            let range =
                                if let Some(span) = Self::find_span_by_path(&module, &error.path) {
                                    // Found the field - use its precise location
                                    Range {
                                        start: Position {
                                            line: span.line.saturating_sub(1) as u32, // LSP is 0-indexed
                                            character: span.column.saturating_sub(1) as u32,
                                        },
                                        end: Position {
                                            line: span.line.saturating_sub(1) as u32,
                                            character: (span.column.saturating_sub(1) + span.length)
                                                as u32,
                                        },
                                    }
                                } else {
                                    // Couldn't find the field - fall back to (0,0)
                                    Range {
                                        start: Position {
                                            line: 0,
                                            character: 0,
                                        },
                                        end: Position {
                                            line: 0,
                                            character: 1,
                                        },
                                    }
                                };

                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: Some(NumberOrString::String(format!(
                                    "schema-{:?}",
                                    error.error_type
                                ))),
                                source: Some("jcl-schema".to_string()),
                                message: format!("{} (at {})", message, error.path),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
            Err(e) => {
                // Parse error
                let range = Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: text.lines().count() as u32,
                        character: 0,
                    },
                };

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("parse-error".to_string())),
                    source: Some("jcl".to_string()),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// Get completion items for built-in functions
    fn get_completions(&self) -> Vec<CompletionItem> {
        let builtins = vec![
            // String functions
            (
                "upper",
                "Converts string to uppercase",
                CompletionItemKind::FUNCTION,
            ),
            (
                "lower",
                "Converts string to lowercase",
                CompletionItemKind::FUNCTION,
            ),
            (
                "trim",
                "Trims whitespace from string",
                CompletionItemKind::FUNCTION,
            ),
            (
                "split",
                "Splits string by delimiter",
                CompletionItemKind::FUNCTION,
            ),
            (
                "join",
                "Joins list of strings",
                CompletionItemKind::FUNCTION,
            ),
            (
                "replace",
                "Replaces substring in string",
                CompletionItemKind::FUNCTION,
            ),
            ("substr", "Extracts substring", CompletionItemKind::FUNCTION),
            (
                "strlen",
                "Returns string length",
                CompletionItemKind::FUNCTION,
            ),
            (
                "startswith",
                "Checks if string starts with prefix",
                CompletionItemKind::FUNCTION,
            ),
            (
                "endswith",
                "Checks if string ends with suffix",
                CompletionItemKind::FUNCTION,
            ),
            (
                "contains",
                "Checks if string contains substring",
                CompletionItemKind::FUNCTION,
            ),
            (
                "format",
                "Formats string with arguments",
                CompletionItemKind::FUNCTION,
            ),
            // List functions
            (
                "map",
                "Maps function over list",
                CompletionItemKind::FUNCTION,
            ),
            (
                "filter",
                "Filters list by predicate",
                CompletionItemKind::FUNCTION,
            ),
            (
                "reduce",
                "Reduces list to single value",
                CompletionItemKind::FUNCTION,
            ),
            (
                "length",
                "Returns list length",
                CompletionItemKind::FUNCTION,
            ),
            (
                "range",
                "Creates range of numbers",
                CompletionItemKind::FUNCTION,
            ),
            ("concat", "Concatenates lists", CompletionItemKind::FUNCTION),
            (
                "flatten",
                "Flattens nested lists",
                CompletionItemKind::FUNCTION,
            ),
            (
                "distinct",
                "Returns distinct values",
                CompletionItemKind::FUNCTION,
            ),
            ("sort", "Sorts list", CompletionItemKind::FUNCTION),
            ("reverse", "Reverses list", CompletionItemKind::FUNCTION),
            ("zip", "Zips multiple lists", CompletionItemKind::FUNCTION),
            (
                "any",
                "Checks if any element matches",
                CompletionItemKind::FUNCTION,
            ),
            (
                "all",
                "Checks if all elements match",
                CompletionItemKind::FUNCTION,
            ),
            // Map functions
            ("keys", "Returns map keys", CompletionItemKind::FUNCTION),
            ("values", "Returns map values", CompletionItemKind::FUNCTION),
            ("merge", "Merges maps", CompletionItemKind::FUNCTION),
            // Type functions
            ("type", "Returns value type", CompletionItemKind::FUNCTION),
            ("int", "Converts to integer", CompletionItemKind::FUNCTION),
            ("float", "Converts to float", CompletionItemKind::FUNCTION),
            ("string", "Converts to string", CompletionItemKind::FUNCTION),
            ("bool", "Converts to boolean", CompletionItemKind::FUNCTION),
            // Math functions
            ("abs", "Absolute value", CompletionItemKind::FUNCTION),
            ("min", "Minimum value", CompletionItemKind::FUNCTION),
            ("max", "Maximum value", CompletionItemKind::FUNCTION),
            ("ceil", "Ceiling function", CompletionItemKind::FUNCTION),
            ("floor", "Floor function", CompletionItemKind::FUNCTION),
            (
                "round",
                "Round to nearest integer",
                CompletionItemKind::FUNCTION,
            ),
            ("pow", "Power function", CompletionItemKind::FUNCTION),
            ("sqrt", "Square root", CompletionItemKind::FUNCTION),
            // Encoding functions
            (
                "jsonencode",
                "Encodes to JSON",
                CompletionItemKind::FUNCTION,
            ),
            (
                "jsondecode",
                "Decodes from JSON",
                CompletionItemKind::FUNCTION,
            ),
            (
                "yamlencode",
                "Encodes to YAML",
                CompletionItemKind::FUNCTION,
            ),
            (
                "yamldecode",
                "Decodes from YAML",
                CompletionItemKind::FUNCTION,
            ),
            (
                "base64encode",
                "Encodes to base64",
                CompletionItemKind::FUNCTION,
            ),
            (
                "base64decode",
                "Decodes from base64",
                CompletionItemKind::FUNCTION,
            ),
            (
                "urlencode",
                "URL encodes string",
                CompletionItemKind::FUNCTION,
            ),
            // Hash functions
            ("md5", "MD5 hash", CompletionItemKind::FUNCTION),
            ("sha1", "SHA1 hash", CompletionItemKind::FUNCTION),
            ("sha256", "SHA256 hash", CompletionItemKind::FUNCTION),
            ("sha512", "SHA512 hash", CompletionItemKind::FUNCTION),
            // File functions
            ("file", "Reads file content", CompletionItemKind::FUNCTION),
            (
                "fileexists",
                "Checks if file exists",
                CompletionItemKind::FUNCTION,
            ),
            // Template functions
            (
                "template",
                "Renders template string",
                CompletionItemKind::FUNCTION,
            ),
            (
                "templatefile",
                "Renders template file",
                CompletionItemKind::FUNCTION,
            ),
            // UUID function
            ("uuid", "Generates UUID", CompletionItemKind::FUNCTION),
            // Date/Time functions
            ("now", "Current timestamp", CompletionItemKind::FUNCTION),
            (
                "timestamp",
                "Creates timestamp",
                CompletionItemKind::FUNCTION,
            ),
            ("formatdate", "Formats date", CompletionItemKind::FUNCTION),
            // Keywords
            ("fn", "Function definition", CompletionItemKind::KEYWORD),
            ("if", "Conditional expression", CompletionItemKind::KEYWORD),
            ("then", "Then clause", CompletionItemKind::KEYWORD),
            ("else", "Else clause", CompletionItemKind::KEYWORD),
            ("for", "For loop", CompletionItemKind::KEYWORD),
            ("in", "In operator", CompletionItemKind::KEYWORD),
            ("when", "When clause", CompletionItemKind::KEYWORD),
            ("import", "Import statement", CompletionItemKind::KEYWORD),
            ("from", "From clause", CompletionItemKind::KEYWORD),
            ("mut", "Mutable variable", CompletionItemKind::KEYWORD),
            // Types
            ("string", "String type", CompletionItemKind::CLASS),
            ("int", "Integer type", CompletionItemKind::CLASS),
            ("float", "Float type", CompletionItemKind::CLASS),
            ("bool", "Boolean type", CompletionItemKind::CLASS),
            ("list", "List type", CompletionItemKind::CLASS),
            ("map", "Map type", CompletionItemKind::CLASS),
            ("any", "Any type", CompletionItemKind::CLASS),
            // Constants
            ("true", "Boolean true", CompletionItemKind::CONSTANT),
            ("false", "Boolean false", CompletionItemKind::CONSTANT),
            ("null", "Null value", CompletionItemKind::CONSTANT),
        ];

        builtins
            .into_iter()
            .map(|(label, detail, kind)| CompletionItem {
                label: label.to_string(),
                kind: Some(kind),
                detail: Some(detail.to_string()),
                insert_text: Some(label.to_string()),
                ..Default::default()
            })
            .collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for JclLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Store workspace root
        if let Some(root_uri) = params.root_uri {
            *self.workspace_root.write().await = Some(root_uri);
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "JCL Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "(".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "JCL Language Server initialized")
            .await;

        // Discover and load schema from workspace
        let workspace_root = self.workspace_root.read().await.clone();
        self.discover_schema(workspace_root.as_ref()).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;

        // Store document
        self.document_map
            .write()
            .await
            .insert(uri.clone(), text.clone());

        // Send diagnostics
        let diagnostics = self.get_diagnostics(&params.text_document.uri, &text).await;
        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            // Update document
            self.document_map.write().await.insert(uri, text.clone());

            // Send diagnostics
            let diagnostics = self.get_diagnostics(&params.text_document.uri, &text).await;
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.document_map.write().await.remove(&uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut items = self.get_completions();

        // Add schema-based completions if a schema is loaded
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            if let Some(validator) = self.schema_validator.read().await.as_ref() {
                if let Ok(module) = crate::parse_str(text) {
                    // Get schema-based completions for the current context
                    if let Some(schema_items) =
                        self.get_schema_completions(&module, validator, &position)
                    {
                        items.extend(schema_items);
                    }
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            // Check if we have a schema loaded
            if let Some(validator) = self.schema_validator.read().await.as_ref() {
                // Parse the document
                if let Ok(module) = crate::parse_str(text) {
                    // Try to find the field path at the cursor position
                    if let Some(field_path) =
                        self.find_field_path_at_position(&module, text, &position)
                    {
                        // Look up the field in the schema
                        if let Some(hover_text) = self.get_schema_hover_info(validator, &field_path)
                        {
                            let contents = HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hover_text,
                            });

                            return Ok(Some(Hover {
                                contents,
                                range: None,
                            }));
                        }
                    }
                }
            }

            // Fall back to basic hover info
            let contents = HoverContents::Scalar(MarkedString::String(
                "JCL Configuration Language".to_string(),
            ));

            Ok(Some(Hover {
                contents,
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            // Parse the document
            if let Ok(module) = crate::parse_str(text) {
                // Build symbol table
                let symbol_table = SymbolTable::from_module(&module);

                // Convert LSP position (0-indexed) to symbol table position (1-indexed)
                let line = position.line as usize + 1;
                let column = position.character as usize;

                // Find symbol at cursor position
                if let Some(symbol) = symbol_table.find_symbol_at_position(line, column) {
                    // Convert symbol table location to LSP location
                    let def_location = Location {
                        uri: params
                            .text_document_position_params
                            .text_document
                            .uri
                            .clone(),
                        range: Range {
                            start: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: symbol.definition.column as u32,
                            },
                            end: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: (symbol.definition.column + symbol.definition.length)
                                    as u32,
                            },
                        },
                    };

                    return Ok(Some(GotoDefinitionResponse::Scalar(def_location)));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            // Parse the document
            if let Ok(module) = crate::parse_str(text) {
                // Build symbol table
                let symbol_table = SymbolTable::from_module(&module);

                // Convert LSP position (0-indexed) to symbol table position (1-indexed)
                let line = position.line as usize + 1;
                let column = position.character as usize;

                // Find symbol at cursor position
                if let Some(symbol) = symbol_table.find_symbol_at_position(line, column) {
                    let mut locations = Vec::new();

                    // Include definition if requested
                    if params.context.include_declaration {
                        locations.push(Location {
                            uri: params.text_document_position.text_document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: (symbol.definition.line - 1) as u32,
                                    character: symbol.definition.column as u32,
                                },
                                end: Position {
                                    line: (symbol.definition.line - 1) as u32,
                                    character: (symbol.definition.column + symbol.definition.length)
                                        as u32,
                                },
                            },
                        });
                    }

                    // Add all references
                    for reference in &symbol.references {
                        locations.push(Location {
                            uri: params.text_document_position.text_document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: (reference.line - 1) as u32,
                                    character: reference.column as u32,
                                },
                                end: Position {
                                    line: (reference.line - 1) as u32,
                                    character: (reference.column + reference.length) as u32,
                                },
                            },
                        });
                    }

                    return Ok(Some(locations));
                }
            }
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            // Parse the document
            if let Ok(module) = crate::parse_str(text) {
                // Build symbol table
                let symbol_table = SymbolTable::from_module(&module);

                // Convert LSP position (0-indexed) to symbol table position (1-indexed)
                let line = position.line as usize + 1;
                let column = position.character as usize;

                // Find symbol at cursor position
                if let Some(symbol) = symbol_table.find_symbol_at_position(line, column) {
                    let mut edits = Vec::new();

                    // Add edit for definition
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: symbol.definition.column as u32,
                            },
                            end: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: (symbol.definition.column + symbol.definition.length)
                                    as u32,
                            },
                        },
                        new_text: new_name.clone(),
                    });

                    // Add edits for all references
                    for reference in &symbol.references {
                        edits.push(TextEdit {
                            range: Range {
                                start: Position {
                                    line: (reference.line - 1) as u32,
                                    character: reference.column as u32,
                                },
                                end: Position {
                                    line: (reference.line - 1) as u32,
                                    character: (reference.column + reference.length) as u32,
                                },
                            },
                            new_text: new_name.clone(),
                        });
                    }

                    // Create workspace edit
                    let mut changes = HashMap::new();
                    changes.insert(
                        params.text_document_position.text_document.uri.clone(),
                        edits,
                    );

                    return Ok(Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }));
                }
            }
        }

        Ok(None)
    }
}

/// Run the LSP server
pub async fn run_server() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(JclLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_span_by_path_simple() {
        let input = r#"
config = (
    host = "localhost",
    port = 8080
)
"#;
        let module = crate::parse_str(input).unwrap();

        // Test finding the root "config" assignment
        let span = JclLanguageServer::find_span_by_path(&module, "config");
        assert!(span.is_some());

        // Test finding nested "host" field
        let span = JclLanguageServer::find_span_by_path(&module, "config.host");
        assert!(span.is_some());
        let span = span.unwrap();
        // The span should point to "localhost" (the value)
        assert!(span.line > 0);

        // Test finding nested "port" field
        let span = JclLanguageServer::find_span_by_path(&module, "config.port");
        assert!(span.is_some());
        let span = span.unwrap();
        // The span should point to 8080 (the value)
        assert!(span.line > 0);
    }

    #[test]
    fn test_find_span_by_path_deeply_nested() {
        let input = r#"
app = (
    database = (
        connection = (
            host = "db.local",
            port = 5432
        )
    )
)
"#;
        let module = crate::parse_str(input).unwrap();

        // Test finding deeply nested field
        let span = JclLanguageServer::find_span_by_path(&module, "app.database.connection.port");
        assert!(span.is_some());
        let span = span.unwrap();
        assert!(span.line > 0);
    }

    #[test]
    fn test_find_span_by_path_not_found() {
        let input = r#"
config = (
    host = "localhost"
)
"#;
        let module = crate::parse_str(input).unwrap();

        // Test field that doesn't exist
        let span = JclLanguageServer::find_span_by_path(&module, "config.port");
        assert!(span.is_none());

        // Test root that doesn't exist
        let span = JclLanguageServer::find_span_by_path(&module, "missing");
        assert!(span.is_none());
    }
}
