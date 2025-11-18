//! Python bindings for JCL using PyO3
//!
//! This module provides Python bindings for the Jack-of-All Configuration Language.
//! It allows Python code to parse, evaluate, format, and lint JCL code.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::ast::Value;
use crate::evaluator::Evaluator;
use crate::formatter;
use crate::linter;

/// Parse JCL source code and return the AST as a Python dict
#[pyfunction]
fn parse(source: &str) -> PyResult<String> {
    match crate::parse_str(source) {
        Ok(_module) => Ok(format!("Parsed {} statements", _module.statements.len())),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!(
            "Parse error: {}",
            e
        ))),
    }
}

/// Evaluate JCL source code and return the result
#[pyfunction]
fn eval(py: Python, source: &str) -> PyResult<PyObject> {
    // Parse the source
    let module = crate::parse_str(source).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!("Parse error: {}", e))
    })?;

    // Evaluate
    let mut evaluator = Evaluator::new();
    let result_module = evaluator.evaluate(module).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Evaluation error: {}", e))
    })?;

    // Convert the environment to Python dict
    let result = PyDict::new(py);
    for (name, value) in &result_module.bindings {
        result.set_item(name, value_to_python(py, value)?)?;
    }

    Ok(result.into())
}

/// Evaluate JCL from a file
#[pyfunction]
fn eval_file(py: Python, path: &str) -> PyResult<PyObject> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read file: {}", e))
    })?;
    eval(py, &content)
}

/// Format JCL source code
#[pyfunction]
fn format(source: &str) -> PyResult<String> {
    let module = crate::parse_str(source).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!("Parse error: {}", e))
    })?;

    formatter::format(&module).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Format error: {}", e))
    })
}

/// Lint JCL source code and return issues
#[pyfunction]
fn lint(py: Python, source: &str) -> PyResult<PyObject> {
    let module = crate::parse_str(source).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!("Parse error: {}", e))
    })?;

    let issues = linter::lint(&module).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Linter error: {}", e))
    })?;

    let py_issues = PyList::empty(py);
    for issue in issues {
        let issue_dict = PyDict::new(py);
        issue_dict.set_item("rule", issue.rule)?;
        issue_dict.set_item("message", issue.message)?;
        issue_dict.set_item(
            "severity",
            match issue.severity {
                linter::Severity::Error => "error",
                linter::Severity::Warning => "warning",
                linter::Severity::Info => "info",
            },
        )?;
        if let Some(suggestion) = issue.suggestion {
            issue_dict.set_item("suggestion", suggestion)?;
        }
        py_issues.append(issue_dict)?;
    }

    Ok(py_issues.into())
}

/// Convert JCL Value to Python object
fn value_to_python(py: Python, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::String(s) => Ok(s.to_object(py)),
        Value::Int(i) => Ok(i.to_object(py)),
        Value::Float(f) => Ok(f.to_object(py)),
        Value::Bool(b) => Ok(b.to_object(py)),
        Value::Null => Ok(py.None()),
        Value::List(items) => {
            let py_list = PyList::empty(py);
            for item in items {
                py_list.append(value_to_python(py, item)?)?;
            }
            Ok(py_list.into())
        }
        Value::Map(map) => {
            let py_dict = PyDict::new(py);
            for (k, v) in map {
                py_dict.set_item(k, value_to_python(py, v)?)?;
            }
            Ok(py_dict.into())
        }
        Value::Function { .. } => {
            // Functions can't be directly converted to Python
            Ok("<function>".to_object(py))
        }
    }
}

/// JCL Python module
#[pymodule]
fn jcl(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_function(wrap_pyfunction!(eval_file, m)?)?;
    m.add_function(wrap_pyfunction!(format, m)?)?;
    m.add_function(wrap_pyfunction!(lint, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
