//! Ruby bindings for JCL using Magnus
//!
//! This module provides Ruby bindings for the Jack-of-All Configuration Language.
//! It allows Ruby code to parse, evaluate, format, and lint JCL code.

use magnus::{define_module, function, prelude::*, Error, RHash, RString, Value};

use crate::ast::Value as JclValue;
use crate::evaluator::Evaluator;
use crate::formatter;
use crate::linter;

/// Parse JCL source code and return a status message
fn parse(source: String) -> Result<String, Error> {
    match crate::parse_str(&source) {
        Ok(module) => Ok(format!("Parsed {} statements", module.statements.len())),
        Err(e) => Err(Error::new(
            magnus::exception::runtime_error(),
            format!("Parse error: {}", e),
        )),
    }
}

/// Evaluate JCL source code and return variables as a Ruby Hash
fn eval(source: String) -> Result<RHash, Error> {
    // Parse the source
    let module = crate::parse_str(&source).map_err(|e| {
        Error::new(
            magnus::exception::runtime_error(),
            format!("Parse error: {}", e),
        )
    })?;

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result_module = evaluator.evaluate(module).map_err(|e| {
        Error::new(
            magnus::exception::runtime_error(),
            format!("Evaluation error: {}", e),
        )
    })?;

    // Convert bindings to Ruby Hash
    let hash = RHash::new();
    for (name, value) in &result_module.bindings {
        let ruby_value = value_to_ruby(value)?;
        hash.aset(name.as_str(), ruby_value)?;
    }

    Ok(hash)
}

/// Evaluate JCL from a file and return variables as a Ruby Hash
fn eval_file(path: String) -> Result<RHash, Error> {
    let content = std::fs::read_to_string(&path).map_err(|e| {
        Error::new(
            magnus::exception::runtime_error(),
            format!("Failed to read file: {}", e),
        )
    })?;

    eval(content)
}

/// Format JCL source code
fn format_jcl(source: String) -> Result<String, Error> {
    let module = crate::parse_str(&source).map_err(|e| {
        Error::new(
            magnus::exception::runtime_error(),
            format!("Parse error: {}", e),
        )
    })?;

    formatter::format(&module).map_err(|e| {
        Error::new(
            magnus::exception::runtime_error(),
            format!("Format error: {}", e),
        )
    })
}

/// Lint JCL source code and return issues as a Ruby Array
fn lint(source: String) -> Result<Value, Error> {
    let module = crate::parse_str(&source).map_err(|e| {
        Error::new(
            magnus::exception::runtime_error(),
            format!("Parse error: {}", e),
        )
    })?;

    let issues = linter::lint(&module).map_err(|e| {
        Error::new(
            magnus::exception::runtime_error(),
            format!("Linter error: {}", e),
        )
    })?;

    // Convert to Ruby Array of Hashes
    let ruby_issues = magnus::RArray::new();
    for issue in issues {
        let issue_hash = RHash::new();
        issue_hash.aset("rule", issue.rule)?;
        issue_hash.aset("message", issue.message)?;
        issue_hash.aset(
            "severity",
            match issue.severity {
                linter::Severity::Error => "error",
                linter::Severity::Warning => "warning",
                linter::Severity::Info => "info",
            },
        )?;
        if let Some(suggestion) = issue.suggestion {
            issue_hash.aset("suggestion", suggestion)?;
        }
        ruby_issues.push(issue_hash)?;
    }

    Ok(ruby_issues.as_value())
}

/// Get JCL version
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Convert JCL Value to Ruby Value
fn value_to_ruby(value: &JclValue) -> Result<Value, Error> {
    match value {
        JclValue::String(s) => Ok(RString::new(s).as_value()),
        JclValue::Int(i) => Ok(magnus::Integer::from_i64(*i).as_value()),
        JclValue::Float(f) => Ok(magnus::Float::from_f64(*f).as_value()),
        JclValue::Bool(b) => {
            if *b {
                Ok(magnus::value::qtrue().as_value())
            } else {
                Ok(magnus::value::qfalse().as_value())
            }
        }
        JclValue::Null => Ok(magnus::value::qnil().as_value()),
        JclValue::List(items) => {
            let ruby_array = magnus::RArray::new();
            for item in items {
                ruby_array.push(value_to_ruby(item)?)?;
            }
            Ok(ruby_array.as_value())
        }
        JclValue::Map(map) => {
            let ruby_hash = RHash::new();
            for (k, v) in map {
                ruby_hash.aset(k.as_str(), value_to_ruby(v)?)?;
            }
            Ok(ruby_hash.as_value())
        }
        JclValue::Function { .. } => Ok(RString::new("<function>").as_value()),
    }
}

/// Initialize the Ruby module
#[magnus::init]
fn init() -> Result<(), Error> {
    let module = define_module("JCL")?;

    module.define_module_function("parse", function!(parse, 1))?;
    module.define_module_function("eval", function!(eval, 1))?;
    module.define_module_function("eval_file", function!(eval_file, 1))?;
    module.define_module_function("format", function!(format_jcl, 1))?;
    module.define_module_function("lint", function!(lint, 1))?;
    module.define_module_function("version", function!(version, 0))?;

    Ok(())
}
