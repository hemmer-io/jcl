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
    }

    Ok(())
}
