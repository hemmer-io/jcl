//! Language Server Protocol implementation for JCL

use crate::linter;
use crate::symbol_table::SymbolTable;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// JCL Language Server
pub struct JclLanguageServer {
    client: Client,
    document_map: Arc<RwLock<HashMap<String, String>>>,
}

impl JclLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get diagnostics for a document
    async fn get_diagnostics(&self, _uri: &Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Parse the document
        match crate::parse_str(text) {
            Ok(module) => {
                // Run linter
                if let Ok(issues) = linter::lint(&module) {
                    for issue in issues {
                        let severity = match issue.severity {
                            linter::Severity::Error => DiagnosticSeverity::ERROR,
                            linter::Severity::Warning => DiagnosticSeverity::WARNING,
                            linter::Severity::Info => DiagnosticSeverity::INFORMATION,
                        };

                        let message = if let Some(suggestion) = &issue.suggestion {
                            format!("{}\nHelp: {}", issue.message, suggestion)
                        } else {
                            issue.message.clone()
                        };

                        // Use precise span if available, otherwise use entire document range
                        let range = if let Some(span) = &issue.span {
                            Range {
                                start: Position {
                                    line: span.line.saturating_sub(1) as u32, // LSP is 0-indexed
                                    character: span.column.saturating_sub(1) as u32,
                                },
                                end: Position {
                                    line: span.line.saturating_sub(1) as u32,
                                    character: (span.column.saturating_sub(1) + span.length) as u32,
                                },
                            }
                        } else {
                            Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: text.lines().count() as u32,
                                    character: 0,
                                },
                            }
                        };

                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(severity),
                            code: Some(NumberOrString::String(issue.rule.clone())),
                            source: Some("jcl".to_string()),
                            message,
                            ..Default::default()
                        });
                    }
                }
            }
            Err(e) => {
                // Parse error
                let range = Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: text.lines().count() as u32,
                        character: 0,
                    },
                };

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("parse-error".to_string())),
                    source: Some("jcl".to_string()),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// Get completion items for built-in functions
    fn get_completions(&self) -> Vec<CompletionItem> {
        let builtins = vec![
            // String functions
            (
                "upper",
                "Converts string to uppercase",
                CompletionItemKind::FUNCTION,
            ),
            (
                "lower",
                "Converts string to lowercase",
                CompletionItemKind::FUNCTION,
            ),
            (
                "trim",
                "Trims whitespace from string",
                CompletionItemKind::FUNCTION,
            ),
            (
                "split",
                "Splits string by delimiter",
                CompletionItemKind::FUNCTION,
            ),
            (
                "join",
                "Joins list of strings",
                CompletionItemKind::FUNCTION,
            ),
            (
                "replace",
                "Replaces substring in string",
                CompletionItemKind::FUNCTION,
            ),
            ("substr", "Extracts substring", CompletionItemKind::FUNCTION),
            (
                "strlen",
                "Returns string length",
                CompletionItemKind::FUNCTION,
            ),
            (
                "startswith",
                "Checks if string starts with prefix",
                CompletionItemKind::FUNCTION,
            ),
            (
                "endswith",
                "Checks if string ends with suffix",
                CompletionItemKind::FUNCTION,
            ),
            (
                "contains",
                "Checks if string contains substring",
                CompletionItemKind::FUNCTION,
            ),
            (
                "format",
                "Formats string with arguments",
                CompletionItemKind::FUNCTION,
            ),
            // List functions
            (
                "map",
                "Maps function over list",
                CompletionItemKind::FUNCTION,
            ),
            (
                "filter",
                "Filters list by predicate",
                CompletionItemKind::FUNCTION,
            ),
            (
                "reduce",
                "Reduces list to single value",
                CompletionItemKind::FUNCTION,
            ),
            (
                "length",
                "Returns list length",
                CompletionItemKind::FUNCTION,
            ),
            (
                "range",
                "Creates range of numbers",
                CompletionItemKind::FUNCTION,
            ),
            ("concat", "Concatenates lists", CompletionItemKind::FUNCTION),
            (
                "flatten",
                "Flattens nested lists",
                CompletionItemKind::FUNCTION,
            ),
            (
                "distinct",
                "Returns distinct values",
                CompletionItemKind::FUNCTION,
            ),
            ("sort", "Sorts list", CompletionItemKind::FUNCTION),
            ("reverse", "Reverses list", CompletionItemKind::FUNCTION),
            ("zip", "Zips multiple lists", CompletionItemKind::FUNCTION),
            (
                "any",
                "Checks if any element matches",
                CompletionItemKind::FUNCTION,
            ),
            (
                "all",
                "Checks if all elements match",
                CompletionItemKind::FUNCTION,
            ),
            // Map functions
            ("keys", "Returns map keys", CompletionItemKind::FUNCTION),
            ("values", "Returns map values", CompletionItemKind::FUNCTION),
            ("merge", "Merges maps", CompletionItemKind::FUNCTION),
            // Type functions
            ("type", "Returns value type", CompletionItemKind::FUNCTION),
            ("int", "Converts to integer", CompletionItemKind::FUNCTION),
            ("float", "Converts to float", CompletionItemKind::FUNCTION),
            ("string", "Converts to string", CompletionItemKind::FUNCTION),
            ("bool", "Converts to boolean", CompletionItemKind::FUNCTION),
            // Math functions
            ("abs", "Absolute value", CompletionItemKind::FUNCTION),
            ("min", "Minimum value", CompletionItemKind::FUNCTION),
            ("max", "Maximum value", CompletionItemKind::FUNCTION),
            ("ceil", "Ceiling function", CompletionItemKind::FUNCTION),
            ("floor", "Floor function", CompletionItemKind::FUNCTION),
            (
                "round",
                "Round to nearest integer",
                CompletionItemKind::FUNCTION,
            ),
            ("pow", "Power function", CompletionItemKind::FUNCTION),
            ("sqrt", "Square root", CompletionItemKind::FUNCTION),
            // Encoding functions
            (
                "jsonencode",
                "Encodes to JSON",
                CompletionItemKind::FUNCTION,
            ),
            (
                "jsondecode",
                "Decodes from JSON",
                CompletionItemKind::FUNCTION,
            ),
            (
                "yamlencode",
                "Encodes to YAML",
                CompletionItemKind::FUNCTION,
            ),
            (
                "yamldecode",
                "Decodes from YAML",
                CompletionItemKind::FUNCTION,
            ),
            (
                "base64encode",
                "Encodes to base64",
                CompletionItemKind::FUNCTION,
            ),
            (
                "base64decode",
                "Decodes from base64",
                CompletionItemKind::FUNCTION,
            ),
            (
                "urlencode",
                "URL encodes string",
                CompletionItemKind::FUNCTION,
            ),
            // Hash functions
            ("md5", "MD5 hash", CompletionItemKind::FUNCTION),
            ("sha1", "SHA1 hash", CompletionItemKind::FUNCTION),
            ("sha256", "SHA256 hash", CompletionItemKind::FUNCTION),
            ("sha512", "SHA512 hash", CompletionItemKind::FUNCTION),
            // File functions
            ("file", "Reads file content", CompletionItemKind::FUNCTION),
            (
                "fileexists",
                "Checks if file exists",
                CompletionItemKind::FUNCTION,
            ),
            // Template functions
            (
                "template",
                "Renders template string",
                CompletionItemKind::FUNCTION,
            ),
            (
                "templatefile",
                "Renders template file",
                CompletionItemKind::FUNCTION,
            ),
            // UUID function
            ("uuid", "Generates UUID", CompletionItemKind::FUNCTION),
            // Date/Time functions
            ("now", "Current timestamp", CompletionItemKind::FUNCTION),
            (
                "timestamp",
                "Creates timestamp",
                CompletionItemKind::FUNCTION,
            ),
            ("formatdate", "Formats date", CompletionItemKind::FUNCTION),
            // Keywords
            ("fn", "Function definition", CompletionItemKind::KEYWORD),
            ("if", "Conditional expression", CompletionItemKind::KEYWORD),
            ("then", "Then clause", CompletionItemKind::KEYWORD),
            ("else", "Else clause", CompletionItemKind::KEYWORD),
            ("for", "For loop", CompletionItemKind::KEYWORD),
            ("in", "In operator", CompletionItemKind::KEYWORD),
            ("when", "When clause", CompletionItemKind::KEYWORD),
            ("import", "Import statement", CompletionItemKind::KEYWORD),
            ("from", "From clause", CompletionItemKind::KEYWORD),
            ("mut", "Mutable variable", CompletionItemKind::KEYWORD),
            // Types
            ("string", "String type", CompletionItemKind::CLASS),
            ("int", "Integer type", CompletionItemKind::CLASS),
            ("float", "Float type", CompletionItemKind::CLASS),
            ("bool", "Boolean type", CompletionItemKind::CLASS),
            ("list", "List type", CompletionItemKind::CLASS),
            ("map", "Map type", CompletionItemKind::CLASS),
            ("any", "Any type", CompletionItemKind::CLASS),
            // Constants
            ("true", "Boolean true", CompletionItemKind::CONSTANT),
            ("false", "Boolean false", CompletionItemKind::CONSTANT),
            ("null", "Null value", CompletionItemKind::CONSTANT),
        ];

        builtins
            .into_iter()
            .map(|(label, detail, kind)| CompletionItem {
                label: label.to_string(),
                kind: Some(kind),
                detail: Some(detail.to_string()),
                insert_text: Some(label.to_string()),
                ..Default::default()
            })
            .collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for JclLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "JCL Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "(".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "JCL Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;

        // Store document
        self.document_map
            .write()
            .await
            .insert(uri.clone(), text.clone());

        // Send diagnostics
        let diagnostics = self.get_diagnostics(&params.text_document.uri, &text).await;
        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            // Update document
            self.document_map.write().await.insert(uri, text.clone());

            // Send diagnostics
            let diagnostics = self.get_diagnostics(&params.text_document.uri, &text).await;
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.document_map.write().await.remove(&uri);
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = self.get_completions();
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();

        if let Some(_text) = self.document_map.read().await.get(&uri) {
            // For now, provide basic hover info
            // In the future, we could analyze the position and provide context-specific info
            let contents = HoverContents::Scalar(MarkedString::String(
                "JCL Language Server\nHover information will be enhanced in future versions"
                    .to_string(),
            ));

            Ok(Some(Hover {
                contents,
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            // Parse the document
            if let Ok(module) = crate::parse_str(text) {
                // Build symbol table
                let symbol_table = SymbolTable::from_module(&module);

                // Convert LSP position (0-indexed) to symbol table position (1-indexed)
                let line = position.line as usize + 1;
                let column = position.character as usize;

                // Find symbol at cursor position
                if let Some(symbol) = symbol_table.find_symbol_at_position(line, column) {
                    // Convert symbol table location to LSP location
                    let def_location = Location {
                        uri: params
                            .text_document_position_params
                            .text_document
                            .uri
                            .clone(),
                        range: Range {
                            start: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: symbol.definition.column as u32,
                            },
                            end: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: (symbol.definition.column + symbol.definition.length)
                                    as u32,
                            },
                        },
                    };

                    return Ok(Some(GotoDefinitionResponse::Scalar(def_location)));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            // Parse the document
            if let Ok(module) = crate::parse_str(text) {
                // Build symbol table
                let symbol_table = SymbolTable::from_module(&module);

                // Convert LSP position (0-indexed) to symbol table position (1-indexed)
                let line = position.line as usize + 1;
                let column = position.character as usize;

                // Find symbol at cursor position
                if let Some(symbol) = symbol_table.find_symbol_at_position(line, column) {
                    let mut locations = Vec::new();

                    // Include definition if requested
                    if params.context.include_declaration {
                        locations.push(Location {
                            uri: params.text_document_position.text_document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: (symbol.definition.line - 1) as u32,
                                    character: symbol.definition.column as u32,
                                },
                                end: Position {
                                    line: (symbol.definition.line - 1) as u32,
                                    character: (symbol.definition.column + symbol.definition.length)
                                        as u32,
                                },
                            },
                        });
                    }

                    // Add all references
                    for reference in &symbol.references {
                        locations.push(Location {
                            uri: params.text_document_position.text_document.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: (reference.line - 1) as u32,
                                    character: reference.column as u32,
                                },
                                end: Position {
                                    line: (reference.line - 1) as u32,
                                    character: (reference.column + reference.length) as u32,
                                },
                            },
                        });
                    }

                    return Ok(Some(locations));
                }
            }
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            // Parse the document
            if let Ok(module) = crate::parse_str(text) {
                // Build symbol table
                let symbol_table = SymbolTable::from_module(&module);

                // Convert LSP position (0-indexed) to symbol table position (1-indexed)
                let line = position.line as usize + 1;
                let column = position.character as usize;

                // Find symbol at cursor position
                if let Some(symbol) = symbol_table.find_symbol_at_position(line, column) {
                    let mut edits = Vec::new();

                    // Add edit for definition
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: symbol.definition.column as u32,
                            },
                            end: Position {
                                line: (symbol.definition.line - 1) as u32,
                                character: (symbol.definition.column + symbol.definition.length)
                                    as u32,
                            },
                        },
                        new_text: new_name.clone(),
                    });

                    // Add edits for all references
                    for reference in &symbol.references {
                        edits.push(TextEdit {
                            range: Range {
                                start: Position {
                                    line: (reference.line - 1) as u32,
                                    character: reference.column as u32,
                                },
                                end: Position {
                                    line: (reference.line - 1) as u32,
                                    character: (reference.column + reference.length) as u32,
                                },
                            },
                            new_text: new_name.clone(),
                        });
                    }

                    // Create workspace edit
                    let mut changes = HashMap::new();
                    changes.insert(
                        params.text_document_position.text_document.uri.clone(),
                        edits,
                    );

                    return Ok(Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }));
                }
            }
        }

        Ok(None)
    }
}

/// Run the LSP server
pub async fn run_server() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(JclLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
