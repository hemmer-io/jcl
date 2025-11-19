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

    /// Disable AST caching (always re-parse files)
    #[arg(long, global = true)]
    no_cache: bool,
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

    /// Lint JCL files for style issues and best practices
    Lint {
        /// Path to lint (file or directory)
        path: Option<String>,

        /// Show all issues (including info level)
        #[arg(long)]
        all: bool,
    },

    /// Generate documentation from JCL files
    Doc {
        /// Path to JCL file
        path: String,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
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

    // Handle caching flag
    if cli.no_cache {
        jcl::disable_cache();
    }

    println!(
        "{}",
        format!("JCL v{}", env!("CARGO_PKG_VERSION"))
            .bright_blue()
            .bold()
    );
    println!();

    match cli.command {
        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| "my-project".to_string());
            println!("{} {}", "Initializing".green().bold(), project_name);

            // Create basic project structure
            std::fs::create_dir_all(&project_name)?;
            std::fs::write(
                format!("{}/main.jcl", project_name),
                "# JCL Configuration\n\n# Define your configuration here\n",
            )?;

            println!("{}", "✓ Created project structure".green());
            println!("  Created: {}/", project_name);
            println!("  Created: {}/main.jcl", project_name);
        }

        Commands::Parse { path } => {
            println!("{} {}", "Parsing".cyan().bold(), path);

            let content = std::fs::read_to_string(&path)?;
            match jcl::parse_str(&content) {
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
            let module = match jcl::parse_str(&content) {
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
                    .filter(|p| p.extension().is_some_and(|ext| ext == "jcl"))
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

                match jcl::parse_str(&content) {
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
                eprintln!(
                    "{} {} error(s), {} file(s) validated",
                    "✗".red().bold(),
                    errors,
                    validated
                );
                std::process::exit(1);
            }
        }

        Commands::Fmt { check, path } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            let path_buf = std::path::PathBuf::from(&target);

            let files = if path_buf.is_file() {
                vec![path_buf]
            } else if path_buf.is_dir() {
                // Find all .jcl files in directory
                glob::glob(&format!("{}/**/*.jcl", target))
                    .expect("Failed to read glob pattern")
                    .filter_map(Result::ok)
                    .collect()
            } else {
                eprintln!("{} Path not found: {}", "Error:".red().bold(), target);
                std::process::exit(1);
            };

            if check {
                println!("{} {}", "Checking formatting:".cyan().bold(), target);
            } else {
                println!("{} {}", "Formatting:".cyan().bold(), target);
            }

            let mut formatted_count = 0;
            let mut errors = 0;

            for file_path in files {
                let path_str = file_path.display().to_string();

                match jcl::parse_file(&path_str) {
                    Ok(module) => {
                        match jcl::formatter::format(&module) {
                            Ok(formatted_code) => {
                                if check {
                                    // Read original file and compare
                                    let original =
                                        std::fs::read_to_string(&file_path).unwrap_or_default();
                                    if original.trim() != formatted_code.trim() {
                                        println!("{} {}", "✗".red().bold(), path_str);
                                        errors += 1;
                                    } else {
                                        formatted_count += 1;
                                    }
                                } else {
                                    // Write formatted code back to file
                                    if let Err(e) = std::fs::write(&file_path, formatted_code) {
                                        eprintln!("{} {}: {}", "✗".red().bold(), path_str, e);
                                        errors += 1;
                                    } else {
                                        println!("{} {}", "✓".green(), path_str);
                                        formatted_count += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("{} {}: {}", "✗".red().bold(), path_str, e);
                                errors += 1;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {}: {}", "✗".red().bold(), path_str, e);
                        errors += 1;
                    }
                }
            }

            println!();
            if errors == 0 {
                if check {
                    println!(
                        "{} {} file(s) correctly formatted",
                        "✓".green().bold(),
                        formatted_count
                    );
                } else {
                    println!(
                        "{} {} file(s) formatted",
                        "✓".green().bold(),
                        formatted_count
                    );
                }
            } else {
                eprintln!(
                    "{} {} error(s), {} file(s) processed",
                    "✗".red().bold(),
                    errors,
                    formatted_count
                );
                std::process::exit(1);
            }
        }

        Commands::Lint { path, all } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            let path_buf = std::path::PathBuf::from(&target);

            let files = if path_buf.is_file() {
                vec![path_buf]
            } else if path_buf.is_dir() {
                glob::glob(&format!("{}/**/*.jcl", target))
                    .expect("Failed to read glob pattern")
                    .filter_map(Result::ok)
                    .collect()
            } else {
                eprintln!("{} Path not found: {}", "Error:".red().bold(), target);
                std::process::exit(1);
            };

            println!("{} {}", "Linting".cyan().bold(), target);
            println!();

            let mut total_issues = 0;
            let mut total_errors = 0;
            let mut total_warnings = 0;

            for file_path in files {
                let path_str = file_path.display().to_string();

                match jcl::parse_file(&path_str) {
                    Ok(module) => match jcl::linter::lint(&module) {
                        Ok(issues) => {
                            let filtered_issues: Vec<_> = if all {
                                issues
                            } else {
                                issues
                                    .into_iter()
                                    .filter(|i| i.severity != jcl::linter::Severity::Info)
                                    .collect()
                            };

                            if !filtered_issues.is_empty() {
                                println!("{}", path_str.bold());
                                for issue in &filtered_issues {
                                    let severity_str = match issue.severity {
                                        jcl::linter::Severity::Error => "error".red().bold(),
                                        jcl::linter::Severity::Warning => "warning".yellow().bold(),
                                        jcl::linter::Severity::Info => "info".blue(),
                                    };
                                    println!(
                                        "  {} [{}] {}",
                                        severity_str,
                                        issue.rule.dimmed(),
                                        issue.message
                                    );
                                    if let Some(suggestion) = &issue.suggestion {
                                        println!("    {}: {}", "help".cyan(), suggestion.dimmed());
                                    }

                                    match issue.severity {
                                        jcl::linter::Severity::Error => total_errors += 1,
                                        jcl::linter::Severity::Warning => total_warnings += 1,
                                        jcl::linter::Severity::Info => {}
                                    }
                                    total_issues += 1;
                                }
                                println!();
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}: {}", "✗".red().bold(), path_str, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("{} {}: {}", "✗".red().bold(), path_str, e);
                        total_errors += 1;
                    }
                }
            }

            if total_issues == 0 {
                println!("{} No issues found", "✓".green().bold());
            } else {
                println!("{}", "Summary:".bold());
                if total_errors > 0 {
                    println!("  {} {}", "Errors:".red().bold(), total_errors);
                }
                if total_warnings > 0 {
                    println!("  {} {}", "Warnings:".yellow().bold(), total_warnings);
                }
                println!("  Total issues: {}", total_issues);

                if total_errors > 0 {
                    std::process::exit(1);
                }
            }
        }

        Commands::Doc { path, output } => {
            println!("{} {}", "Generating documentation:".cyan().bold(), path);

            match jcl::parse_file(&path) {
                Ok(module) => {
                    match jcl::docgen::generate(&module) {
                        Ok(doc) => {
                            // Extract module name from path
                            let module_name = std::path::Path::new(&path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("module");

                            let markdown = jcl::docgen::format_markdown(&doc, module_name);

                            if let Some(output_path) = output {
                                // Write to file
                                if let Err(e) = std::fs::write(&output_path, &markdown) {
                                    eprintln!("{} Failed to write output: {}", "✗".red().bold(), e);
                                    std::process::exit(1);
                                }
                                println!(
                                    "{} Documentation written to {}",
                                    "✓".green().bold(),
                                    output_path
                                );
                            } else {
                                // Print to stdout
                                println!();
                                println!("{}", markdown);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "{} Documentation generation failed: {}",
                                "✗".red().bold(),
                                e
                            );
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Parse failed: {}", "✗".red().bold(), e);
                    std::process::exit(1);
                }
            }
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
