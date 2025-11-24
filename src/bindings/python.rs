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
fn eval(py: Python, source: &str) -> PyResult<Py<PyAny>> {
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
fn eval_file(py: Python, path: &str) -> PyResult<Py<PyAny>> {
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
fn lint(py: Python, source: &str) -> PyResult<Py<PyAny>> {
    let module = crate::parse_str(source).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!("Parse error: {}", e))
    })?;

    let issues = linter::lint(&module).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Linter error: {}", e))
    })?;

    let py_issue_dicts: Result<Vec<Py<PyAny>>, PyErr> = issues
        .into_iter()
        .map(|issue| {
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
            Ok(issue_dict.into())
        })
        .collect();

    Ok(PyList::new(py, py_issue_dicts?)?.into())
}

/// Convert JCL Value to Python object
fn value_to_python(py: Python, value: &Value) -> PyResult<Py<PyAny>> {
    use pyo3::types::{PyBool, PyFloat, PyInt, PyString};
    match value {
        Value::String(s) => Ok(PyString::new(py, s).into_any().unbind()),
        Value::Int(i) => Ok(PyInt::new(py, *i).into_any().unbind()),
        Value::Float(f) => Ok(PyFloat::new(py, *f).into_any().unbind()),
        Value::Bool(b) => Ok(PyBool::new(py, *b).to_owned().into_any().unbind()),
        Value::Null => Ok(py.None()),
        Value::List(items) => {
            let values: Result<Vec<Py<PyAny>>, PyErr> =
                items.iter().map(|item| value_to_python(py, item)).collect();
            Ok(PyList::new(py, values?)?.into())
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
            Ok(py.None())
        }
        Value::Stream(id) => Ok(PyString::new(py, &format!("<stream:{}>", id)).into()),
    }
}

/// JCL Python module
#[pymodule]
fn jcl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_function(wrap_pyfunction!(eval_file, m)?)?;
    m.add_function(wrap_pyfunction!(format, m)?)?;
    m.add_function(wrap_pyfunction!(lint, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
