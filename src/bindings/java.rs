//! Java bindings for JCL using JNI
//!
//! This module provides Java bindings for the Jack-of-All Configuration Language.
//! It allows Java code to parse, evaluate, format, and lint JCL code.

use jni::objects::{JClass, JObject, JString};
use jni::sys::jstring;
use jni::JNIEnv;

use crate::ast::Value;
use crate::evaluator::Evaluator;
use crate::formatter;
use crate::linter;

/// Parse JCL source code and return a status message
#[no_mangle]
pub extern "system" fn Java_JCL_parse(mut env: JNIEnv, _class: JClass, source: JString) -> jstring {
    let source_str: String = match env.get_string(&source) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to get string: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    match crate::parse_str(&source_str) {
        Ok(module) => {
            let msg = format!("Parsed {} statements", module.statements.len());
            match env.new_string(msg) {
                Ok(s) => s.into_raw(),
                Err(e) => {
                    let _ = env.throw_new(
                        "java/lang/RuntimeException",
                        format!("Failed to create string: {}", e),
                    );
                    JObject::null().into_raw()
                }
            }
        }
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("Parse error: {}", e));
            JObject::null().into_raw()
        }
    }
}

/// Evaluate JCL source code and return the result as JSON string
#[no_mangle]
pub extern "system" fn Java_JCL_eval(mut env: JNIEnv, _class: JClass, source: JString) -> jstring {
    let source_str: String = match env.get_string(&source) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to get string: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    // Parse the source
    let module = match crate::parse_str(&source_str) {
        Ok(m) => m,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("Parse error: {}", e));
            return JObject::null().into_raw();
        }
    };

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result_module = match evaluator.evaluate(module) {
        Ok(r) => r,
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Evaluation error: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    // Convert bindings to JSON
    let json_map: serde_json::Map<String, serde_json::Value> = result_module
        .bindings
        .iter()
        .map(|(k, v)| (k.clone(), value_to_json(v)))
        .collect();

    let json_str = match serde_json::to_string(&json_map) {
        Ok(s) => s,
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("JSON serialization error: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    match env.new_string(json_str) {
        Ok(s) => s.into_raw(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to create string: {}", e),
            );
            JObject::null().into_raw()
        }
    }
}

/// Evaluate JCL from a file and return the result as JSON string
#[no_mangle]
pub extern "system" fn Java_JCL_evalFile(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> jstring {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to get string: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    let content = match std::fs::read_to_string(&path_str) {
        Ok(c) => c,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("Failed to read file: {}", e));
            return JObject::null().into_raw();
        }
    };

    // Parse the source
    let module = match crate::parse_str(&content) {
        Ok(m) => m,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("Parse error: {}", e));
            return JObject::null().into_raw();
        }
    };

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result_module = match evaluator.evaluate(module) {
        Ok(r) => r,
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Evaluation error: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    // Convert bindings to JSON
    let json_map: serde_json::Map<String, serde_json::Value> = result_module
        .bindings
        .iter()
        .map(|(k, v)| (k.clone(), value_to_json(v)))
        .collect();

    let json_str = match serde_json::to_string(&json_map) {
        Ok(s) => s,
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("JSON serialization error: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    match env.new_string(json_str) {
        Ok(s) => s.into_raw(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to create string: {}", e),
            );
            JObject::null().into_raw()
        }
    }
}

/// Format JCL source code
#[no_mangle]
pub extern "system" fn Java_JCL_format(
    mut env: JNIEnv,
    _class: JClass,
    source: JString,
) -> jstring {
    let source_str: String = match env.get_string(&source) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to get string: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    let module = match crate::parse_str(&source_str) {
        Ok(m) => m,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("Parse error: {}", e));
            return JObject::null().into_raw();
        }
    };

    let formatted = match formatter::format(&module) {
        Ok(f) => f,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("Format error: {}", e));
            return JObject::null().into_raw();
        }
    };

    match env.new_string(formatted) {
        Ok(s) => s.into_raw(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to create string: {}", e),
            );
            JObject::null().into_raw()
        }
    }
}

/// Lint JCL source code and return issues as JSON array string
#[no_mangle]
pub extern "system" fn Java_JCL_lint(mut env: JNIEnv, _class: JClass, source: JString) -> jstring {
    let source_str: String = match env.get_string(&source) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to get string: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    let module = match crate::parse_str(&source_str) {
        Ok(m) => m,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("Parse error: {}", e));
            return JObject::null().into_raw();
        }
    };

    let issues = match linter::lint(&module) {
        Ok(i) => i,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("Linter error: {}", e));
            return JObject::null().into_raw();
        }
    };

    let json_issues: Vec<serde_json::Value> = issues
        .iter()
        .map(|issue| {
            let mut map = serde_json::Map::new();
            map.insert(
                "rule".to_string(),
                serde_json::Value::String(issue.rule.clone()),
            );
            map.insert(
                "message".to_string(),
                serde_json::Value::String(issue.message.clone()),
            );
            map.insert(
                "severity".to_string(),
                serde_json::Value::String(
                    match issue.severity {
                        linter::Severity::Error => "error",
                        linter::Severity::Warning => "warning",
                        linter::Severity::Info => "info",
                    }
                    .to_string(),
                ),
            );
            if let Some(ref suggestion) = issue.suggestion {
                map.insert(
                    "suggestion".to_string(),
                    serde_json::Value::String(suggestion.clone()),
                );
            }
            serde_json::Value::Object(map)
        })
        .collect();

    let json_str = match serde_json::to_string(&json_issues) {
        Ok(s) => s,
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("JSON serialization error: {}", e),
            );
            return JObject::null().into_raw();
        }
    };

    match env.new_string(json_str) {
        Ok(s) => s.into_raw(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to create string: {}", e),
            );
            JObject::null().into_raw()
        }
    }
}

/// Get JCL version
#[no_mangle]
pub extern "system" fn Java_JCL_version(mut env: JNIEnv, _class: JClass) -> jstring {
    match env.new_string(env!("CARGO_PKG_VERSION")) {
        Ok(s) => s.into_raw(),
        Err(e) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                format!("Failed to create string: {}", e),
            );
            JObject::null().into_raw()
        }
    }
}

/// Convert JCL Value to serde_json::Value
fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Null => serde_json::Value::Null,
        Value::List(items) => serde_json::Value::Array(items.iter().map(value_to_json).collect()),
        Value::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Function { .. } => serde_json::Value::String("<function>".to_string()),
        Value::Stream(id) => serde_json::Value::String(format!("<stream:{}>", id)),
    }
}
