//! JCL - Jack-of-All Configuration Language
//!
//! A general-purpose configuration language with powerful built-in functions
//! that prioritizes safety, ease of use, and flexibility.

pub mod ast;
pub mod cache;
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
use lazy_static::lazy_static;
use std::sync::Mutex;

// Re-export commonly used types
pub use ast::{Expression, Module, Statement, Value};
pub use cache::AstCache;
pub use lexer::{Lexer, Token, TokenKind};
pub use token_parser::TokenParser;

// Legacy Pest parser (deprecated - use token_parser instead)
pub use parser::{parse_file as parse_file_legacy, parse_str as parse_str_legacy};

// Global AST cache (lazy initialized)
lazy_static! {
    static ref GLOBAL_CACHE: Mutex<Option<AstCache>> = Mutex::new(Some(AstCache::default()));
}

/// Enable AST caching globally (enabled by default)
pub fn enable_cache() {
    let mut cache = GLOBAL_CACHE.lock().unwrap();
    if cache.is_none() {
        *cache = Some(AstCache::default());
    }
}

/// Disable AST caching globally
///
/// This is useful for benchmarking or when you want to ensure
/// files are always re-parsed.
pub fn disable_cache() {
    let mut cache = GLOBAL_CACHE.lock().unwrap();
    *cache = None;
}

/// Clear the global AST cache
pub fn clear_cache() {
    let cache = GLOBAL_CACHE.lock().unwrap();
    if let Some(ref cache) = *cache {
        cache.clear();
    }
}

/// Check if caching is enabled
pub fn is_cache_enabled() -> bool {
    let cache = GLOBAL_CACHE.lock().unwrap();
    cache.is_some()
}

/// Parse JCL from a string using the token-based parser
///
/// Note: String parsing is never cached (only file parsing is cached)
pub fn parse_str(input: &str) -> Result<Module> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = TokenParser::new(tokens);
    parser.parse_module()
}

/// Parse JCL from a file using the token-based parser
///
/// This function automatically uses the global AST cache if enabled.
/// Files are cached by path + modification time, so cache is automatically
/// invalidated when files change.
///
/// To disable caching, call `disable_cache()` before parsing.
pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> Result<Module> {
    let path = path.as_ref();

    // Check if caching is enabled
    let cache_enabled = {
        let cache = GLOBAL_CACHE.lock().unwrap();
        cache.is_some()
    };

    if cache_enabled {
        // Use cache
        let cache = GLOBAL_CACHE.lock().unwrap();
        let cache_ref = cache.as_ref().unwrap();

        let module_arc = cache_ref.get_or_parse(path, |p| {
            let content = std::fs::read_to_string(p)
                .with_context(|| format!("Failed to read file: {}", p.display()))?;
            parse_str(&content)
        })?;

        // Clone the Module from Arc (this is a deep clone)
        Ok((*module_arc).clone())
    } else {
        // Caching disabled - parse directly
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        parse_str(&content)
    }
}

/// Alias for parse_str (for backwards compatibility)
pub fn parse_with_tokens(input: &str) -> Result<Module> {
    parse_str(input)
}

/// JCL version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core JCL context for parsing and evaluating JCL code
pub struct JclContext {
    /// Whether this context uses caching (affects global cache state)
    cache_enabled: bool,
}

impl JclContext {
    /// Create a new JCL context with caching enabled
    pub fn new() -> Result<Self> {
        Ok(Self {
            cache_enabled: true,
        })
    }

    /// Create a new JCL context with caching disabled
    pub fn new_without_cache() -> Result<Self> {
        Ok(Self {
            cache_enabled: false,
        })
    }

    /// Parse a JCL file
    pub fn parse_file(&self, path: &str) -> Result<Module> {
        if !self.cache_enabled {
            // Temporarily disable cache for this parse
            let was_enabled = is_cache_enabled();
            if was_enabled {
                disable_cache();
            }
            let result = parse_file(path);
            if was_enabled {
                enable_cache();
            }
            result
        } else {
            parse_file(path)
        }
    }

    /// Parse JCL from a string
    pub fn parse_str(&self, input: &str) -> Result<Module> {
        parse_str(input)
    }

    /// Clear the cache for this context (clears global cache)
    pub fn clear_cache(&self) {
        clear_cache();
    }
}

impl Default for JclContext {
    fn default() -> Self {
        Self::new().expect("Failed to create JCL context")
    }
}
