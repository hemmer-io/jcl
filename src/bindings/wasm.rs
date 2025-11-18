//! WebAssembly bindings for JCL
//!
//! This module provides a JavaScript-friendly API for using JCL in the browser.

use crate::ast::Module;
use crate::{docgen, formatter, linter, parser};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize panic hook for better error messages in the browser console
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Result type returned to JavaScript
#[wasm_bindgen]
#[derive(Clone)]
pub struct JclResult {
    success: bool,
    value: String,
    error: Option<String>,
}

#[wasm_bindgen]
impl JclResult {
    /// Check if the operation was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the result value (empty string if error)
    pub fn value(&self) -> String {
        self.value.clone()
    }

    /// Get the error message (empty string if success)
    pub fn error(&self) -> String {
        self.error.clone().unwrap_or_default()
    }
}

/// Main JCL interface for WebAssembly
#[wasm_bindgen]
pub struct Jcl {
    module: Option<Module>,
}

#[wasm_bindgen]
impl Jcl {
    /// Create a new JCL instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Jcl {
        #[cfg(feature = "console_error_panic_hook")]
        set_panic_hook();

        Jcl { module: None }
    }

    /// Parse JCL source code
    pub fn parse(&mut self, source: &str) -> JclResult {
        match crate::parse_str(source) {
            Ok(module) => {
                self.module = Some(module);
                JclResult {
                    success: true,
                    value: "Parse successful".to_string(),
                    error: None,
                }
            }
            Err(e) => JclResult {
                success: false,
                value: String::new(),
                error: Some(format!("Parse error: {}", e)),
            },
        }
    }

    /// Format JCL source code
    pub fn format(&self, source: &str) -> JclResult {
        match crate::parse_str(source) {
            Ok(module) => match formatter::format(&module) {
                Ok(formatted) => JclResult {
                    success: true,
                    value: formatted,
                    error: None,
                },
                Err(e) => JclResult {
                    success: false,
                    value: String::new(),
                    error: Some(format!("Format error: {}", e)),
                },
            },
            Err(e) => JclResult {
                success: false,
                value: String::new(),
                error: Some(format!("Parse error: {}", e)),
            },
        }
    }

    /// Run linter on JCL source code
    pub fn lint(&self, source: &str) -> JclResult {
        match crate::parse_str(source) {
            Ok(module) => match linter::lint(&module) {
                Ok(issues) => {
                    if issues.is_empty() {
                        JclResult {
                            success: true,
                            value: "No issues found".to_string(),
                            error: None,
                        }
                    } else {
                        let issues_json = serde_json::to_string_pretty(&issues)
                            .unwrap_or_else(|_| format!("{:?}", issues));
                        JclResult {
                            success: true,
                            value: issues_json,
                            error: None,
                        }
                    }
                }
                Err(e) => JclResult {
                    success: false,
                    value: String::new(),
                    error: Some(format!("Linter error: {}", e)),
                },
            },
            Err(e) => JclResult {
                success: false,
                value: String::new(),
                error: Some(format!("Parse error: {}", e)),
            },
        }
    }

    /// Generate documentation from JCL source code
    pub fn generate_docs(&self, source: &str, module_name: &str) -> JclResult {
        match crate::parse_str(source) {
            Ok(module) => match docgen::generate(&module) {
                Ok(doc) => {
                    let markdown = docgen::format_markdown(&doc, module_name);
                    JclResult {
                        success: true,
                        value: markdown,
                        error: None,
                    }
                }
                Err(e) => JclResult {
                    success: false,
                    value: String::new(),
                    error: Some(format!("Documentation generation error: {}", e)),
                },
            },
            Err(e) => JclResult {
                success: false,
                value: String::new(),
                error: Some(format!("Parse error: {}", e)),
            },
        }
    }

    /// Get the JCL version
    pub fn version() -> String {
        crate::VERSION.to_string()
    }
}

/// Convenience function to parse JCL without creating an instance
#[wasm_bindgen]
pub fn parse_jcl(source: &str) -> JclResult {
    let mut jcl = Jcl::new();
    jcl.parse(source)
}

/// Convenience function to format JCL without creating an instance
#[wasm_bindgen]
pub fn format_jcl(source: &str) -> JclResult {
    let jcl = Jcl::new();
    jcl.format(source)
}

/// Convenience function to lint JCL without creating an instance
#[wasm_bindgen]
pub fn lint_jcl(source: &str) -> JclResult {
    let jcl = Jcl::new();
    jcl.lint(source)
}

/// Convenience function to generate docs without creating an instance
#[wasm_bindgen]
pub fn generate_jcl_docs(source: &str, module_name: &str) -> JclResult {
    let jcl = Jcl::new();
    jcl.generate_docs(source, module_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_code() {
        let mut jcl = Jcl::new();
        let result = jcl.parse("x = 42");
        assert!(result.is_success());
    }

    #[test]
    fn test_parse_invalid_code() {
        let mut jcl = Jcl::new();
        let result = jcl.parse("x = ");
        assert!(!result.is_success());
        assert!(!result.error().is_empty());
    }

    #[test]
    fn test_format() {
        let jcl = Jcl::new();
        let result = jcl.format("x=42");
        assert!(result.is_success());
        assert!(result.value().contains("x = 42"));
    }

    #[test]
    fn test_lint() {
        let jcl = Jcl::new();
        let result = jcl.lint("CONSTANT = 42");
        assert!(result.is_success());
    }

    #[test]
    fn test_generate_docs() {
        let jcl = Jcl::new();
        let result = jcl.generate_docs("fn add(x: int, y: int): int = x + y", "test");
        assert!(result.is_success());
        assert!(result.value().contains("add"));
    }
}
