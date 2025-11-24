//! REPL (Read-Eval-Print Loop) for JCL
//!
//! Provides an interactive shell for evaluating JCL expressions

use crate::{ast::Value, evaluator::Evaluator};
use anyhow::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{history::FileHistory, CompletionType, Config, Editor};
use std::env;
use std::path::PathBuf;

/// Run the interactive REPL
pub fn run_repl() -> Result<()> {
    println!("{}", "JCL REPL v0.1.0".cyan().bold());
    println!("{}", "Type :help for help, :quit to exit".dimmed());
    println!();

    // Configure rustyline with enhanced features
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .auto_add_history(true)
        .build();

    let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;

    // Set up history file
    let history_path = get_history_path();
    if let Some(path) = &history_path {
        let _ = rl.load_history(path); // Ignore errors if history doesn't exist yet
    }

    let mut evaluator = Evaluator::new();
    let mut line_number = 1;
    let mut multiline_buffer = String::new();
    let mut in_multiline = false;

    loop {
        let prompt = if in_multiline {
            "   ... ".to_string().yellow().bold().to_string()
        } else {
            format!("jcl:{} ", line_number).green().bold().to_string()
        };

        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // Check for multiline continuation (lines ending with \)
                if trimmed.ends_with('\\') && !in_multiline {
                    in_multiline = true;
                    multiline_buffer = trimmed.strip_suffix('\\').unwrap_or(trimmed).to_string();
                    continue;
                } else if in_multiline {
                    if trimmed.ends_with('\\') {
                        multiline_buffer.push(' ');
                        multiline_buffer.push_str(trimmed.strip_suffix('\\').unwrap_or(trimmed));
                        continue;
                    } else {
                        // End of multiline input
                        multiline_buffer.push(' ');
                        multiline_buffer.push_str(trimmed);
                        in_multiline = false;
                    }
                }

                let input_line = if multiline_buffer.is_empty() {
                    trimmed.to_string()
                } else {
                    let result = multiline_buffer.clone();
                    multiline_buffer.clear();
                    result
                };

                // Skip empty lines
                if input_line.is_empty() {
                    continue;
                }

                // Handle special commands
                if input_line.starts_with(':') {
                    match input_line.as_str() {
                        ":quit" | ":q" | ":exit" => {
                            println!("{}", "Goodbye!".cyan());
                            break;
                        }
                        ":help" | ":h" => {
                            print_help();
                            continue;
                        }
                        ":clear" | ":c" => {
                            // Clear evaluator state
                            evaluator = Evaluator::new();
                            println!("{}", "✓ State cleared".green());
                            continue;
                        }
                        ":vars" | ":v" => {
                            // Show all variables
                            print_variables(&evaluator);
                            continue;
                        }
                        _ => {
                            eprintln!("{} {}", "Unknown command:".red(), input_line);
                            println!("{}", "Type :help for available commands".dimmed());
                            continue;
                        }
                    }
                }

                // Try to parse as an expression first
                let expr_input = format!("_result = {}", input_line);
                match crate::parse_str(&expr_input) {
                    Ok(module) => {
                        // Evaluate the module
                        match evaluator.evaluate(module) {
                            Ok(result) => {
                                // Print the result
                                if let Some(value) = result.bindings.get("_result") {
                                    println!("{}", format_value(value));
                                }
                                line_number += 1;
                            }
                            Err(e) => {
                                eprintln!("{} {}", "✗".red().bold(), e);
                            }
                        }
                    }
                    Err(_) => {
                        // If expression parsing fails, try as a statement
                        match crate::parse_str(&input_line) {
                            Ok(module) => match evaluator.evaluate(module) {
                                Ok(_) => {
                                    println!("{}", "✓".green());
                                    line_number += 1;
                                }
                                Err(e) => {
                                    eprintln!("{} {}", "✗".red().bold(), e);
                                }
                            },
                            Err(e) => {
                                eprintln!("{} {}", "✗".red().bold(), e);
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".dimmed());
                println!("{}", "Use :quit to exit".dimmed());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".cyan());
                break;
            }
            Err(err) => {
                eprintln!("{} {:?}", "Error:".red().bold(), err);
                break;
            }
        }
    }

    // Save history before exiting
    if let Some(path) = history_path {
        let _ = rl.save_history(&path); // Ignore errors on save
    }

    Ok(())
}

/// Get the history file path
fn get_history_path() -> Option<PathBuf> {
    if let Some(mut path) = env::var_os("HOME").map(PathBuf::from) {
        path.push(".jcf_history");
        Some(path)
    } else {
        None
    }
}

fn print_help() {
    println!("{}", "JCL REPL Commands:".cyan().bold());
    println!("  {}  - Show this help message", ":help, :h".green());
    println!("  {}  - Exit the REPL", ":quit, :q, :exit".green());
    println!("  {}  - Clear all variables", ":clear, :c".green());
    println!("  {}  - Show all variables", ":vars, :v".green());
    println!();
    println!("{}", "Features:".cyan().bold());
    println!(
        "  {} - Persistent command history (~/.jcf_history)",
        "Up/Down arrows".dimmed()
    );
    println!("  {} - Complete and search history", "Tab/Ctrl-R".dimmed());
    println!(
        "  {} - Multi-line input (end line with \\)",
        "Backslash".dimmed()
    );
    println!();
    println!("{}", "Examples:".cyan().bold());
    println!("  {}  - Evaluate an expression", "2 + 2".dimmed());
    println!("  {}  - Define a variable", "x = 42".dimmed());
    println!("  {}  - Define a function", "fn double(x) = x * 2".dimmed());
    println!("  {}  - Use a function", "double(21)".dimmed());
    println!("  {}  - Multi-line input", "x = 1 + \\".dimmed());
    println!("  {}                    ", "    2 + \\".dimmed());
    println!("  {}                    ", "    3".dimmed());
}

fn print_variables(evaluator: &Evaluator) {
    if evaluator.variables.is_empty() {
        println!("{}", "No variables defined".dimmed());
        return;
    }

    println!("{}", "Variables:".cyan().bold());
    let mut vars: Vec<_> = evaluator.variables.iter().collect();
    vars.sort_by_key(|(name, _)| *name);

    for (name, value) in vars {
        if !name.starts_with('_') {
            // Skip internal variables
            println!("  {} = {}", name.green(), format_value(value));
        }
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s).yellow().to_string(),
        Value::Int(i) => i.to_string().cyan().to_string(),
        Value::Float(f) => f.to_string().cyan().to_string(),
        Value::Bool(b) => b.to_string().magenta().to_string(),
        Value::Null => "null".dimmed().to_string(),
        Value::List(items) => {
            let formatted: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", formatted.join(", "))
        }
        Value::Map(map) => {
            let mut entries: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            entries.sort();
            format!("({})", entries.join(", "))
        }
        Value::Function { params, .. } => {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            format!("fn({})", param_names.join(", ")).blue().to_string()
        }
        Value::Stream(id) => format!("<stream:{}>", id).green().to_string(),
    }
}
