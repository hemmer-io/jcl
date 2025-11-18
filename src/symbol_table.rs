//! Symbol table for tracking variable and function definitions and references
//! This enables LSP features like Go to Definition, Find References, and Rename

use crate::ast::{Expression, Module, Statement};
use crate::lexer::{Position, Span};
use std::collections::HashMap;

/// A symbol (variable or function) in the code
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub definition: Location,
    pub references: Vec<Location>,
}

/// Kind of symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Import,
}

/// Location in source code
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

impl From<&Span> for Location {
    fn from(span: &Span) -> Self {
        Location {
            line: span.start.line,
            column: span.start.column,
            offset: span.start.offset,
            length: span.text.len(),
        }
    }
}

impl From<&Position> for Location {
    fn from(pos: &Position) -> Self {
        Location {
            line: pos.line,
            column: pos.column,
            offset: pos.offset,
            length: 0,
        }
    }
}

impl From<&crate::ast::SourceSpan> for Location {
    fn from(span: &crate::ast::SourceSpan) -> Self {
        Location {
            line: span.line,
            column: span.column,
            offset: span.offset,
            length: span.length,
        }
    }
}

/// Symbol table containing all symbols in a document
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Map from symbol name to symbol info
    pub symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// Build a symbol table from a parsed module
    pub fn from_module(module: &Module) -> Self {
        let mut table = Self::new();

        for statement in &module.statements {
            table.process_statement(statement);
        }

        table
    }

    /// Add a symbol definition
    fn add_definition(&mut self, name: String, kind: SymbolKind, location: Location) {
        self.symbols.entry(name.clone()).or_insert_with(|| Symbol {
            name,
            kind,
            definition: location,
            references: Vec::new(),
        });
    }

    /// Add a symbol reference
    fn add_reference(&mut self, name: &str, location: Location) {
        if let Some(symbol) = self.symbols.get_mut(name) {
            symbol.references.push(location);
        }
    }

    /// Process a statement to extract symbols
    fn process_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assignment {
                name, value, span, ..
            } => {
                // Use span from AST if available, otherwise use placeholder
                let location =
                    span.as_ref()
                        .map(Location::from)
                        .unwrap_or_else(|| Location {
                            line: 0,
                            column: 0,
                            offset: 0,
                            length: name.len(),
                        });
                self.add_definition(name.clone(), SymbolKind::Variable, location);
                self.process_expression(value);
            }
            Statement::FunctionDef {
                name,
                params,
                body,
                span,
                ..
            } => {
                let location =
                    span.as_ref()
                        .map(Location::from)
                        .unwrap_or_else(|| Location {
                            line: 0,
                            column: 0,
                            offset: 0,
                            length: name.len(),
                        });
                self.add_definition(name.clone(), SymbolKind::Function, location);

                // Add parameters as symbols
                for param in params {
                    self.add_definition(
                        param.name.clone(),
                        SymbolKind::Parameter,
                        Location {
                            line: 0,
                            column: 0,
                            offset: 0,
                            length: param.name.len(),
                        },
                    );
                }

                self.process_expression(body);
            }
            Statement::Import { items, span, .. } => {
                for item in items {
                    if item != "*" {
                        let location =
                            span.as_ref()
                                .map(Location::from)
                                .unwrap_or_else(|| Location {
                                    line: 0,
                                    column: 0,
                                    offset: 0,
                                    length: item.len(),
                                });
                        self.add_definition(item.clone(), SymbolKind::Import, location);
                    }
                }
            }
            Statement::ForLoop {
                variables,
                iterables,
                body,
                span,
                ..
            } => {
                // Add loop variables
                for var in variables {
                    let location =
                        span.as_ref()
                            .map(Location::from)
                            .unwrap_or_else(|| Location {
                                line: 0,
                                column: 0,
                                offset: 0,
                                length: var.len(),
                            });
                    self.add_definition(var.clone(), SymbolKind::Variable, location);
                }

                for iterable in iterables {
                    self.process_expression(iterable);
                }

                for stmt in body {
                    self.process_statement(stmt);
                }
            }
            Statement::Expression { expr, .. } => {
                self.process_expression(expr);
            }
        }
    }

    /// Process an expression to find symbol references
    fn process_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Variable { name, span } => {
                let location =
                    span.as_ref()
                        .map(Location::from)
                        .unwrap_or_else(|| Location {
                            line: 0,
                            column: 0,
                            offset: 0,
                            length: name.len(),
                        });
                self.add_reference(name, location);
            }
            Expression::FunctionCall { name, args, span } => {
                let location =
                    span.as_ref()
                        .map(Location::from)
                        .unwrap_or_else(|| Location {
                            line: 0,
                            column: 0,
                            offset: 0,
                            length: name.len(),
                        });
                self.add_reference(name, location);
                for arg in args {
                    self.process_expression(arg);
                }
            }
            Expression::MethodCall { object, args, .. } => {
                self.process_expression(object);
                for arg in args {
                    self.process_expression(arg);
                }
            }
            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.process_expression(condition);
                self.process_expression(then_expr);
                self.process_expression(else_expr);
            }
            Expression::Spread { expr, .. } => {
                self.process_expression(expr);
            }
            Expression::BinaryOp { left, right, .. } => {
                self.process_expression(left);
                self.process_expression(right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.process_expression(operand);
            }
            Expression::If {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.process_expression(condition);
                self.process_expression(then_expr);
                if let Some(else_e) = else_expr {
                    self.process_expression(else_e);
                }
            }
            Expression::List { elements, .. } => {
                for item in elements {
                    self.process_expression(item);
                }
            }
            Expression::Map { entries, .. } => {
                for (_key, value) in entries {
                    self.process_expression(value);
                }
            }
            Expression::Index { object, index, .. } => {
                self.process_expression(object);
                self.process_expression(index);
            }
            Expression::MemberAccess { object, .. } => {
                self.process_expression(object);
            }
            Expression::OptionalChain { object, .. } => {
                self.process_expression(object);
            }
            Expression::Lambda { body, .. } => {
                // Note: lambda parameters create a new scope
                // For now, we'll process them in the same scope
                self.process_expression(body);
            }
            Expression::Pipeline { stages, .. } => {
                for stage in stages {
                    self.process_expression(stage);
                }
            }
            Expression::ListComprehension {
                expr,
                iterable,
                condition,
                ..
            } => {
                self.process_expression(expr);
                self.process_expression(iterable);
                if let Some(cond) = condition {
                    self.process_expression(cond);
                }
            }
            Expression::When { value, arms, .. } => {
                self.process_expression(value);
                for arm in arms {
                    self.process_expression(&arm.expr);
                    if let Some(guard) = &arm.guard {
                        self.process_expression(guard);
                    }
                }
            }
            Expression::Try { expr, default, .. } => {
                self.process_expression(expr);
                if let Some(def) = default {
                    self.process_expression(def);
                }
            }
            Expression::InterpolatedString { parts, .. } => {
                for part in parts {
                    if let crate::ast::StringPart::Interpolation(expr) = part {
                        self.process_expression(expr);
                    }
                }
            }
            Expression::Literal { .. } => {
                // Literals don't contain symbol references
            }
        }
    }

    /// Find the symbol at a given position
    pub fn find_symbol_at_position(&self, line: usize, column: usize) -> Option<&Symbol> {
        for symbol in self.symbols.values() {
            // Check definition
            if self.contains_position(&symbol.definition, line, column) {
                return Some(symbol);
            }

            // Check references
            for reference in &symbol.references {
                if self.contains_position(reference, line, column) {
                    return Some(symbol);
                }
            }
        }
        None
    }

    /// Check if a location contains a position
    fn contains_position(&self, location: &Location, line: usize, column: usize) -> bool {
        location.line == line
            && column >= location.column
            && column < location.column + location.length
    }

    /// Get all symbols of a specific kind
    pub fn get_symbols_by_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols.values().filter(|s| s.kind == kind).collect()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_basic() {
        let table = SymbolTable::new();
        assert_eq!(table.symbols.len(), 0);
    }

    #[test]
    fn test_find_symbol_at_position() {
        let mut table = SymbolTable::new();
        table.add_definition(
            "test_var".to_string(),
            SymbolKind::Variable,
            Location {
                line: 1,
                column: 0,
                offset: 0,
                length: 8,
            },
        );

        let symbol = table.find_symbol_at_position(1, 4);
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().name, "test_var");

        let symbol = table.find_symbol_at_position(1, 10);
        assert!(symbol.is_none());
    }
}
