//! JCL - Jack-of-All Configuration Language
//!
//! A general-purpose configuration language with powerful built-in functions
//! that prioritizes safety, ease of use, and flexibility.

pub mod ast;
pub mod docgen;
pub mod error;
pub mod evaluator;
pub mod formatter;
pub mod functions;
pub mod linter;
pub mod parser;
pub mod types;

// CLI-only modules
#[cfg(feature = "cli")]
pub mod lsp;
#[cfg(feature = "cli")]
pub mod repl;

// WebAssembly support
#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export commonly used types
pub use ast::{Expression, Module, Statement, Value};
pub use parser::{parse_file, parse_str};

use anyhow::Result;

/// JCL version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core JCL context for parsing and evaluating JCL code
pub struct JclContext {
    // Context will be expanded as we implement evaluator
}

impl JclContext {
    /// Create a new JCL context
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Parse a JCL file
    pub fn parse_file(&self, path: &str) -> Result<Module> {
        parse_file(path)
    }

    /// Parse JCL from a string
    pub fn parse_str(&self, input: &str) -> Result<Module> {
        parse_str(input)
    }
}

impl Default for JclContext {
    fn default() -> Self {
        Self::new().expect("Failed to create JCL context")
    }
}
