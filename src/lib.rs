//! JCL - Jack Configuration Language
//!
//! A unified infrastructure as code and configuration management language
//! that prioritizes safety, ease of use, and flexibility.

pub mod ast;
pub mod cli;
pub mod evaluator;
pub mod parser;
pub mod planner;
pub mod providers;
pub mod state;
pub mod types;

pub use ast::{Environment, Resource, Stack};
pub use evaluator::Evaluator;
pub use parser::Parser;
pub use planner::Planner;

use anyhow::Result;

/// JCL version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core JCL context that orchestrates the entire workflow
pub struct JclContext {
    parser: Parser,
    evaluator: Evaluator,
    planner: Planner,
}

impl JclContext {
    /// Create a new JCL context
    pub fn new() -> Result<Self> {
        Ok(Self {
            parser: Parser::new(),
            evaluator: Evaluator::new(),
            planner: Planner::new(),
        })
    }

    /// Parse a JCL file
    pub fn parse_file(&mut self, path: &str) -> Result<ast::Module> {
        self.parser.parse_file(path)
    }

    /// Evaluate a parsed module
    pub fn evaluate(&mut self, module: ast::Module) -> Result<evaluator::EvaluatedModule> {
        self.evaluator.evaluate(module)
    }

    /// Plan changes for a stack
    pub fn plan(&mut self, stack: &str) -> Result<planner::Plan> {
        self.planner.plan(stack)
    }

    /// Apply a plan
    pub fn apply(&mut self, plan: planner::Plan) -> Result<()> {
        self.planner.apply(plan)
    }
}

impl Default for JclContext {
    fn default() -> Self {
        Self::new().expect("Failed to create JCL context")
    }
}
