//! Node.js bindings for JCL using Neon
//!
//! This module provides Node.js/JavaScript bindings for the Jack-of-All Configuration Language.
//! It allows JavaScript/TypeScript code to parse, evaluate, format, and lint JCL code.

use neon::prelude::*;

use crate::ast::Value;
use crate::evaluator::Evaluator;
use crate::formatter;
use crate::linter;

/// Parse JCL source code
fn parse(mut cx: FunctionContext) -> JsResult<JsString> {
    let source = cx.argument::<JsString>(0)?.value(&mut cx);

    match crate::parse_str(&source) {
        Ok(module) => {
            let msg = format!("Parsed {} statements", module.statements.len());
            Ok(cx.string(msg))
        }
        Err(e) => cx.throw_error(format!("Parse error: {}", e)),
    }
}

/// Evaluate JCL source code and return variables as object
fn eval(mut cx: FunctionContext) -> JsResult<JsObject> {
    let source = cx.argument::<JsString>(0)?.value(&mut cx);

    // Parse
    let module = crate::parse_str(&source)
        .or_else(|e| cx.throw_error(format!("Parse error: {}", e)))?;

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result_module = evaluator.evaluate(module)
        .or_else(|e| cx.throw_error(format!("Evaluation error: {}", e)))?;

    // Convert to JavaScript object
    let result = cx.empty_object();
    for (name, value) in &result_module.bindings {
        let js_value = value_to_js(&mut cx, value)?;
        result.set(&mut cx, name.as_str(), js_value)?;
    }

    Ok(result)
}

/// Evaluate JCL from a file
fn eval_file(mut cx: FunctionContext) -> JsResult<JsObject> {
    let path = cx.argument::<JsString>(0)?.value(&mut cx);

    let content = std::fs::read_to_string(&path)
        .or_else(|e| cx.throw_error(format!("Failed to read file: {}", e)))?;

    // Parse
    let module = crate::parse_str(&content)
        .or_else(|e| cx.throw_error(format!("Parse error: {}", e)))?;

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result_module = evaluator.evaluate(module)
        .or_else(|e| cx.throw_error(format!("Evaluation error: {}", e)))?;

    // Convert to JavaScript object
    let result = cx.empty_object();
    for (name, value) in &result_module.bindings {
        let js_value = value_to_js(&mut cx, value)?;
        result.set(&mut cx, name.as_str(), js_value)?;
    }

    Ok(result)
}

/// Format JCL source code
fn format(mut cx: FunctionContext) -> JsResult<JsString> {
    let source = cx.argument::<JsString>(0)?.value(&mut cx);

    let module = crate::parse_str(&source)
        .or_else(|e| cx.throw_error(format!("Parse error: {}", e)))?;

    let formatted = formatter::format(&module)
        .or_else(|e| cx.throw_error(format!("Format error: {}", e)))?;

    Ok(cx.string(formatted))
}

/// Lint JCL source code and return issues
fn lint(mut cx: FunctionContext) -> JsResult<JsArray> {
    let source = cx.argument::<JsString>(0)?.value(&mut cx);

    let module = crate::parse_str(&source)
        .or_else(|e| cx.throw_error(format!("Parse error: {}", e)))?;

    let issues = linter::lint(&module)
        .or_else(|e| cx.throw_error(format!("Linter error: {}", e)))?;

    let js_issues = cx.empty_array();

    for (i, issue) in issues.iter().enumerate() {
        let issue_obj = cx.empty_object();

        let rule = cx.string(&issue.rule);
        issue_obj.set(&mut cx, "rule", rule)?;

        let message = cx.string(&issue.message);
        issue_obj.set(&mut cx, "message", message)?;

        let severity = cx.string(match issue.severity {
            linter::Severity::Error => "error",
            linter::Severity::Warning => "warning",
            linter::Severity::Info => "info",
        });
        issue_obj.set(&mut cx, "severity", severity)?;

        if let Some(ref suggestion) = issue.suggestion {
            let sug = cx.string(suggestion);
            issue_obj.set(&mut cx, "suggestion", sug)?;
        }

        js_issues.set(&mut cx, i as u32, issue_obj)?;
    }

    Ok(js_issues)
}

/// Convert JCL Value to JavaScript value
fn value_to_js<'a>(cx: &mut FunctionContext<'a>, value: &Value) -> JsResult<'a, JsValue> {
    match value {
        Value::String(s) => Ok(cx.string(s).upcast()),
        Value::Int(i) => Ok(cx.number(*i as f64).upcast()),
        Value::Float(f) => Ok(cx.number(*f).upcast()),
        Value::Bool(b) => Ok(cx.boolean(*b).upcast()),
        Value::Null => Ok(cx.null().upcast()),
        Value::List(items) => {
            let js_array = cx.empty_array();
            for (i, item) in items.iter().enumerate() {
                let js_value = value_to_js(cx, item)?;
                js_array.set(cx, i as u32, js_value)?;
            }
            Ok(js_array.upcast())
        }
        Value::Map(map) => {
            let js_obj = cx.empty_object();
            for (k, v) in map {
                let js_value = value_to_js(cx, v)?;
                js_obj.set(cx, k.as_str(), js_value)?;
            }
            Ok(js_obj.upcast())
        }
        Value::Function { .. } => Ok(cx.string("<function>").upcast()),
    }
}

/// Register Node.js module exports
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("parse", parse)?;
    cx.export_function("eval", eval)?;
    cx.export_function("evalFile", eval_file)?;
    cx.export_function("format", format)?;
    cx.export_function("lint", lint)?;
    Ok(())
}
