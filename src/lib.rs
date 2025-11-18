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
pub mod lexer;
pub mod linter;
pub mod migration;
pub mod parser;
pub mod schema;
pub mod symbol_table;
pub mod token_parser;
pub mod types;

// CLI-only modules
#[cfg(feature = "cli")]
pub mod lsp;
#[cfg(feature = "cli")]
pub mod repl;

// Language bindings (organized under bindings module)
pub mod bindings;

use anyhow::{Context, Result};

// Re-export commonly used types
pub use ast::{Expression, Module, Statement, Value};
pub use lexer::{Lexer, Token, TokenKind};
pub use token_parser::TokenParser;

// Legacy Pest parser (deprecated - use token_parser instead)
pub use parser::{parse_file as parse_file_legacy, parse_str as parse_str_legacy};

/// Parse JCL from a string using the token-based parser
pub fn parse_str(input: &str) -> Result<Module> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = TokenParser::new(tokens);
    parser.parse_module()
}

/// Parse JCL from a file using the token-based parser
pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> Result<Module> {
    let content = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("Failed to read file: {}", path.as_ref().display()))?;
    parse_str(&content)
}

/// Alias for parse_str (for backwards compatibility)
pub fn parse_with_tokens(input: &str) -> Result<Module> {
    parse_str(input)
}

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
