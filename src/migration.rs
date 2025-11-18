//! Migration tool for converting other configuration formats to JCL
//!
//! This module provides functionality to convert from JSON, YAML, and TOML to JCL format.

use anyhow::{Context, Result};
use serde_json::Value as JsonValue;

/// Convert JSON string to JCL
pub fn json_to_jcl(json: &str) -> Result<String> {
    let value: JsonValue = serde_json::from_str(json)
        .context("Failed to parse JSON")?;

    Ok(json_value_to_jcl(&value, 0))
}

/// Convert YAML string to JCL
pub fn yaml_to_jcl(yaml: &str) -> Result<String> {
    let value: JsonValue = serde_yaml::from_str(yaml)
        .context("Failed to parse YAML")?;

    Ok(json_value_to_jcl(&value, 0))
}

/// Convert TOML string to JCL
pub fn toml_to_jcl(toml_str: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(toml_str)
        .context("Failed to parse TOML")?;

    Ok(toml_value_to_jcl(&value, 0))
}

/// Convert a JSON value to JCL syntax
fn json_value_to_jcl(value: &JsonValue, indent: usize) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{}\"", escape_string(s)),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr.iter()
                    .map(|v| json_value_to_jcl(v, indent))
                    .collect();
                format!("[{}]", items.join(", "))
            }
        }
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                "()".to_string()
            } else {
                object_to_jcl(obj, indent)
            }
        }
    }
}

/// Convert TOML value to JCL syntax
fn toml_value_to_jcl(value: &toml::Value, indent: usize) -> String {
    match value {
        toml::Value::String(s) => format!("\"{}\"", escape_string(s)),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Datetime(dt) => format!("\"{}\"", dt),
        toml::Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr.iter()
                    .map(|v| toml_value_to_jcl(v, indent))
                    .collect();
                format!("[{}]", items.join(", "))
            }
        }
        toml::Value::Table(table) => {
            if table.is_empty() {
                "()".to_string()
            } else {
                toml_table_to_jcl(table, indent)
            }
        }
    }
}

/// Convert JSON object to JCL map syntax with proper formatting
fn object_to_jcl(obj: &serde_json::Map<String, JsonValue>, indent: usize) -> String {
    let indent_str = "    ".repeat(indent);
    let inner_indent = "    ".repeat(indent + 1);

    // Check if this should be top-level assignments or a map
    if indent == 0 {
        // Top-level: use assignments
        let mut lines = Vec::new();
        for (key, value) in obj {
            let jcl_value = json_value_to_jcl(value, indent + 1);
            lines.push(format!("{} = {}", key, jcl_value));
        }
        lines.join("\n")
    } else {
        // Nested: use map syntax
        let mut lines = Vec::new();
        for (key, value) in obj {
            let jcl_value = json_value_to_jcl(value, indent + 1);
            lines.push(format!("{}{} = {}", inner_indent, key, jcl_value));
        }
        format!("(\n{}\n{})", lines.join(",\n"), indent_str)
    }
}

/// Convert TOML table to JCL
fn toml_table_to_jcl(table: &toml::map::Map<String, toml::Value>, indent: usize) -> String {
    let indent_str = "    ".repeat(indent);
    let inner_indent = "    ".repeat(indent + 1);

    // Check if this should be top-level assignments or a map
    if indent == 0 {
        // Top-level: use assignments
        let mut lines = Vec::new();
        for (key, value) in table {
            let jcl_value = toml_value_to_jcl(value, indent + 1);
            lines.push(format!("{} = {}", key, jcl_value));
        }
        lines.join("\n")
    } else {
        // Nested: use map syntax
        let mut lines = Vec::new();
        for (key, value) in table {
            let jcl_value = toml_value_to_jcl(value, indent + 1);
            lines.push(format!("{}{} = {}", inner_indent, key, jcl_value));
        }
        format!("(\n{}\n{})", lines.join(",\n"), indent_str)
    }
}

/// Escape string for JCL
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_jcl() {
        let json = r#"{
            "name": "test",
            "version": "1.0.0",
            "port": 8080,
            "enabled": true
        }"#;

        let jcl = json_to_jcl(json).unwrap();
        assert!(jcl.contains("name = \"test\""));
        assert!(jcl.contains("version = \"1.0.0\""));
        assert!(jcl.contains("port = 8080"));
        assert!(jcl.contains("enabled = true"));
    }

    #[test]
    fn test_yaml_to_jcl() {
        let yaml = r#"
name: test
version: 1.0.0
port: 8080
enabled: true
"#;

        let jcl = yaml_to_jcl(yaml).unwrap();
        assert!(jcl.contains("name = \"test\""));
        assert!(jcl.contains("port = 8080"));
    }

    #[test]
    fn test_toml_to_jcl() {
        let toml_str = r#"
name = "test"
version = "1.0.0"
port = 8080
enabled = true
"#;

        let jcl = toml_to_jcl(toml_str).unwrap();
        assert!(jcl.contains("name = \"test\""));
        assert!(jcl.contains("port = 8080"));
    }

    #[test]
    fn test_nested_objects() {
        let json = r#"{
            "database": {
                "host": "localhost",
                "port": 5432
            }
        }"#;

        let jcl = json_to_jcl(json).unwrap();
        assert!(jcl.contains("database ="));
        assert!(jcl.contains("host = \"localhost\""));
        assert!(jcl.contains("port = 5432"));
    }

    #[test]
    fn test_arrays() {
        let json = r#"{
            "ports": [8080, 8081, 8082]
        }"#;

        let jcl = json_to_jcl(json).unwrap();
        assert!(jcl.contains("ports = [8080, 8081, 8082]"));
    }
}
