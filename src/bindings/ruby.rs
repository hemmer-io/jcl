//! Ruby bindings for JCL using Magnus
//!
//! This module provides Ruby bindings for the Jack-of-All Configuration Language.
//! It allows Ruby code to parse, evaluate, format, and lint JCL code.

use magnus::{function, prelude::*, Error, RHash, Ruby, Value};

use crate::ast::Value as JclValue;
use crate::evaluator::Evaluator;
use crate::formatter;
use crate::linter;

/// Parse JCL source code and return a status message
fn parse(ruby: &Ruby, source: String) -> Result<String, Error> {
    match crate::parse_str(&source) {
        Ok(module) => Ok(format!("Parsed {} statements", module.statements.len())),
        Err(e) => Err(Error::new(
            ruby.exception_runtime_error(),
            format!("Parse error: {}", e),
        )),
    }
}

/// Evaluate JCL source code and return variables as a Ruby Hash
fn eval(ruby: &Ruby, source: String) -> Result<RHash, Error> {
    // Parse the source
    let module = crate::parse_str(&source).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("Parse error: {}", e),
        )
    })?;

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result_module = evaluator.evaluate(module).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("Evaluation error: {}", e),
        )
    })?;

    // Convert bindings to Ruby Hash
    let hash = ruby.hash_new();
    for (name, value) in &result_module.bindings {
        let ruby_value = value_to_ruby(ruby, value)?;
        hash.aset(name.as_str(), ruby_value)?;
    }

    Ok(hash)
}

/// Evaluate JCL from a file and return variables as a Ruby Hash
fn eval_file(ruby: &Ruby, path: String) -> Result<RHash, Error> {
    let content = std::fs::read_to_string(&path).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("Failed to read file: {}", e),
        )
    })?;

    eval(ruby, content)
}

/// Format JCL source code
fn format_jcl(ruby: &Ruby, source: String) -> Result<String, Error> {
    let module = crate::parse_str(&source).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("Parse error: {}", e),
        )
    })?;

    formatter::format(&module).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("Format error: {}", e),
        )
    })
}

/// Lint JCL source code and return issues as a Ruby Array
fn lint(ruby: &Ruby, source: String) -> Result<Value, Error> {
    let module = crate::parse_str(&source).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("Parse error: {}", e),
        )
    })?;

    let issues = linter::lint(&module).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("Linter error: {}", e),
        )
    })?;

    // Convert to Ruby Array of Hashes
    let ruby_issues = ruby.ary_new();
    for issue in issues {
        let issue_hash = ruby.hash_new();
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
fn value_to_ruby(ruby: &Ruby, value: &JclValue) -> Result<Value, Error> {
    match value {
        JclValue::String(s) => Ok(ruby.str_new(s).as_value()),
        JclValue::Int(i) => Ok(ruby.integer_from_i64(*i).as_value()),
        JclValue::Float(f) => Ok(ruby.float_from_f64(*f).as_value()),
        JclValue::Bool(b) => {
            if *b {
                Ok(ruby.qtrue().as_value())
            } else {
                Ok(ruby.qfalse().as_value())
            }
        }
        JclValue::Null => Ok(ruby.qnil().as_value()),
        JclValue::List(items) => {
            let ruby_array = ruby.ary_new();
            for item in items {
                ruby_array.push(value_to_ruby(ruby, item)?)?;
            }
            Ok(ruby_array.as_value())
        }
        JclValue::Map(map) => {
            let ruby_hash = ruby.hash_new();
            for (k, v) in map {
                ruby_hash.aset(k.as_str(), value_to_ruby(ruby, v)?)?;
            }
            Ok(ruby_hash.as_value())
        }
        JclValue::Function { .. } => Ok(ruby.str_new("<function>").as_value()),
    }
}

/// Initialize the Ruby module
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("JCL")?;

    module.define_module_function("parse", function!(parse, 1))?;
    module.define_module_function("eval", function!(eval, 1))?;
    module.define_module_function("eval_file", function!(eval_file, 1))?;
    module.define_module_function("format", function!(format_jcl, 1))?;
    module.define_module_function("lint", function!(lint, 1))?;
    module.define_module_function("version", function!(version, 0))?;

    Ok(())
}
