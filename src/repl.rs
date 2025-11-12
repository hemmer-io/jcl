//! REPL (Read-Eval-Print Loop) for JCL
//!
//! Provides an interactive shell for evaluating JCL expressions

use anyhow::Result;
use colored::Colorize;
use crate::{ast::Value, evaluator::Evaluator, parser};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Run the interactive REPL
pub fn run_repl() -> Result<()> {
    println!("{}", "JCL REPL v0.1.0".cyan().bold());
    println!("{}", "Type :help for help, :quit to exit".dimmed());
    println!();

    let mut rl = DefaultEditor::new()?;
    let mut evaluator = Evaluator::new();
    let mut line_number = 1;

    loop {
        let prompt = format!("jcl:{} ", line_number).green().bold().to_string();
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let line = line.trim();

                // Skip empty lines
                if line.is_empty() {
                    continue;
                }

                // Add to history
                rl.add_history_entry(line)?;

                // Handle special commands
                if line.starts_with(':') {
                    match line {
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
                            eprintln!("{} {}", "Unknown command:".red(), line);
                            println!("{}", "Type :help for available commands".dimmed());
                            continue;
                        }
                    }
                }

                // Try to parse as an expression first
                let expr_input = format!("_result = {}", line);
                match parser::parse_str(&expr_input) {
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
                        match parser::parse_str(line) {
                            Ok(module) => {
                                match evaluator.evaluate(module) {
                                    Ok(_) => {
                                        println!("{}", "✓".green());
                                        line_number += 1;
                                    }
                                    Err(e) => {
                                        eprintln!("{} {}", "✗".red().bold(), e);
                                    }
                                }
                            }
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

    Ok(())
}

fn print_help() {
    println!("{}", "JCL REPL Commands:".cyan().bold());
    println!("  {}  - Show this help message", ":help, :h".green());
    println!("  {}  - Exit the REPL", ":quit, :q, :exit".green());
    println!("  {}  - Clear all variables", ":clear, :c".green());
    println!("  {}  - Show all variables", ":vars, :v".green());
    println!();
    println!("{}", "Examples:".cyan().bold());
    println!("  {}  - Evaluate an expression", "2 + 2".dimmed());
    println!("  {}  - Define a variable", "x = 42".dimmed());
    println!("  {}  - Define a function", "fn double(x) = x * 2".dimmed());
    println!("  {}  - Use a function", "double(21)".dimmed());
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
    }
}
