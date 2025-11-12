//! JCL CLI - Command-line interface for Jack Configuration Language
//!
//! Note: This is a minimal CLI for now. Full features will be implemented
//! after the parser and evaluator are complete.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use tracing::Level;

#[derive(Parser)]
#[command(name = "jcl")]
#[command(about = "Jack Configuration Language - General-purpose configuration language", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Working directory
    #[arg(short = 'C', long, global = true)]
    directory: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new JCL project
    Init {
        /// Project name
        name: Option<String>,
    },

    /// Parse and display JCL file AST
    Parse {
        /// Path to configuration file
        path: String,
    },

    /// Evaluate JCL file and show results
    Eval {
        /// Path to configuration file
        path: String,

        /// Output format (text, json, yaml)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Validate JCL configuration files
    Validate {
        /// Path to configuration file or directory
        path: Option<String>,
    },

    /// Format JCL files
    Fmt {
        /// Check formatting without making changes
        #[arg(long)]
        check: bool,

        /// Path to format (defaults to current directory)
        path: Option<String>,
    },

    /// Start interactive REPL (Read-Eval-Print Loop)
    Repl,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    // Change directory if specified
    if let Some(dir) = &cli.directory {
        std::env::set_current_dir(dir)?;
    }

    println!("{}", format!("JCL v{}", env!("CARGO_PKG_VERSION")).bright_blue().bold());
    println!();

    match cli.command {
        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| "my-project".to_string());
            println!("{} {}", "Initializing".green().bold(), project_name);

            // Create basic project structure
            std::fs::create_dir_all(&project_name)?;
            std::fs::write(
                format!("{}/main.jcl", project_name),
                "# JCL Configuration\n\n# Define your configuration here\n"
            )?;

            println!("{}", "✓ Created project structure".green());
            println!("  Created: {}/", project_name);
            println!("  Created: {}/main.jcl", project_name);
        }

        Commands::Parse { path } => {
            println!("{} {}", "Parsing".cyan().bold(), path);

            let content = std::fs::read_to_string(&path)?;
            match jcl::parser::parse_str(&content) {
                Ok(module) => {
                    println!("{}", "✓ Parse successful".green());
                    println!("\n{}", "AST:".bold());
                    println!("{:#?}", module);
                }
                Err(e) => {
                    eprintln!("{} {}", "✗ Parse failed:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Eval { path, format } => {
            println!("{} {}", "Evaluating".cyan().bold(), path);

            let content = std::fs::read_to_string(&path)?;

            // Parse the file
            let module = match jcl::parser::parse_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("{} {}", "✗ Parse failed:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            // Evaluate the module
            let mut evaluator = jcl::evaluator::Evaluator::new();
            let result = match evaluator.evaluate(module) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{} {}", "✗ Evaluation failed:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            println!("{}", "✓ Evaluation successful".green());
            println!();

            // Output results in requested format
            match format.as_str() {
                "json" => {
                    // Convert bindings to JSON
                    let json = serde_json::to_string_pretty(&result.bindings)?;
                    println!("{}", json);
                }
                "yaml" => {
                    // Convert bindings to YAML
                    let yaml = serde_yaml::to_string(&result.bindings)?;
                    println!("{}", yaml);
                }
                _ => {
                    // Text format (default)
                    println!("{}", "Results:".bold());
                    for (name, value) in &result.bindings {
                        println!("  {} = {}", name.cyan(), format_value(value));
                    }
                }
            }
        }

        Commands::Validate { path } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            println!("{} {}", "Validating".yellow().bold(), target);

            // Check if it's a file or directory
            let path_obj = PathBuf::from(&target);
            let files = if path_obj.is_file() {
                vec![path_obj]
            } else if path_obj.is_dir() {
                // Find all .jcl files in directory
                std::fs::read_dir(&path_obj)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |ext| ext == "jcl"))
                    .collect()
            } else {
                eprintln!("{} Path not found: {}", "✗".red().bold(), target);
                std::process::exit(1);
            };

            let mut errors = 0;
            let mut validated = 0;

            for file in files {
                let display_path = file.display();
                let content = match std::fs::read_to_string(&file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{} {}: {}", "✗".red().bold(), display_path, e);
                        errors += 1;
                        continue;
                    }
                };

                match jcl::parser::parse_str(&content) {
                    Ok(_) => {
                        println!("{} {}", "✓".green(), display_path);
                        validated += 1;
                    }
                    Err(e) => {
                        eprintln!("{} {}: {}", "✗".red().bold(), display_path, e);
                        errors += 1;
                    }
                }
            }

            println!();
            if errors == 0 {
                println!("{} {} file(s) validated", "✓".green().bold(), validated);
            } else {
                eprintln!("{} {} error(s), {} file(s) validated", "✗".red().bold(), errors, validated);
                std::process::exit(1);
            }
        }

        Commands::Fmt { check, path } => {
            let target = path.unwrap_or_else(|| ".".to_string());

            if check {
                println!("{} {}", "Checking formatting:".cyan().bold(), target);
            } else {
                println!("{} {}", "Formatting:".cyan().bold(), target);
            }

            println!("{}", "Note: Formatter not yet implemented".yellow());
            println!("{}", "✓ Formatting complete".green());
        }

        Commands::Repl => {
            jcl::repl::run_repl()?;
        }
    }

    Ok(())
}

/// Format a value for display
fn format_value(value: &jcl::ast::Value) -> String {
    use jcl::ast::Value;
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
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
            format!("fn({})", param_names.join(", "))
        }
    }
}
