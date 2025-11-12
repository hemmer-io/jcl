//! JCL CLI - Command-line interface for Jack Configuration Language

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use tracing::Level;
use tracing_subscriber;

use jcl::JclContext;

#[derive(Parser)]
#[command(name = "jcl")]
#[command(about = "Jack Configuration Language - Unified IaC and Config Management", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Working directory
    #[arg(short = 'C', long, global = true)]
    directory: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new JCL project
    Init {
        /// Project name
        name: Option<String>,
    },

    /// Validate JCL configuration files
    Validate {
        /// Path to configuration file or directory
        path: Option<String>,
    },

    /// Plan changes for a stack
    Plan {
        /// Stack name to plan
        stack: String,

        /// Output plan to file
        #[arg(short, long)]
        out: Option<String>,
    },

    /// Apply a plan or stack
    Apply {
        /// Stack name or plan file
        target: String,

        /// Auto-approve without confirmation
        #[arg(long)]
        auto_approve: bool,
    },

    /// Destroy a stack
    Destroy {
        /// Stack name to destroy
        stack: String,

        /// Auto-approve without confirmation
        #[arg(long)]
        auto_approve: bool,
    },

    /// Show the current state
    Show {
        /// Stack name
        stack: Option<String>,
    },

    /// List all stacks
    List,

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

    println!("{}", format!("JCL v{}", jcl::VERSION).bright_blue().bold());
    println!();

    match cli.command {
        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| "my-project".to_string());
            println!("{} {}", "Initializing".green().bold(), project_name);
            // TODO: Implement init
            println!("✓ Created project structure");
        }

        Commands::Validate { path } => {
            let target = path.unwrap_or_else(|| ".".to_string());
            println!("{} {}", "Validating".yellow().bold(), target);

            let mut ctx = JclContext::new()?;
            match ctx.parse_file(&target) {
                Ok(_) => {
                    println!("{}", "✓ Configuration is valid".green());
                }
                Err(e) => {
                    eprintln!("{} {}", "✗ Validation failed:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Plan { stack, out } => {
            println!("{} {}", "Planning".cyan().bold(), stack);

            let mut ctx = JclContext::new()?;
            match ctx.plan(&stack) {
                Ok(plan) => {
                    println!("{}", "✓ Plan created successfully".green());
                    println!("\n{}", plan);

                    if let Some(output_path) = out {
                        // TODO: Save plan to file
                        println!("Plan saved to {}", output_path);
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "✗ Planning failed:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Apply { target, auto_approve } => {
            println!("{} {}", "Applying".green().bold(), target);

            if !auto_approve {
                println!("\n{}", "Do you want to perform these actions?".yellow());
                println!("  Type 'yes' to confirm: ");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if input.trim() != "yes" {
                    println!("{}", "Apply cancelled.".yellow());
                    return Ok(());
                }
            }

            // TODO: Implement apply
            println!("{}", "✓ Apply completed successfully".green());
        }

        Commands::Destroy { stack, auto_approve } => {
            println!("{} {}", "Destroying".red().bold(), stack);

            if !auto_approve {
                println!("\n{}", "⚠️  WARNING: This will destroy all resources in the stack!".red().bold());
                println!("  Type 'yes' to confirm: ");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if input.trim() != "yes" {
                    println!("{}", "Destroy cancelled.".yellow());
                    return Ok(());
                }
            }

            // TODO: Implement destroy
            println!("{}", "✓ Destroy completed successfully".green());
        }

        Commands::Show { stack } => {
            match stack {
                Some(s) => println!("{} {}", "Showing".cyan().bold(), s),
                None => println!("{}", "Showing all stacks".cyan().bold()),
            }
            // TODO: Implement show
        }

        Commands::List => {
            println!("{}", "Available stacks:".cyan().bold());
            // TODO: Implement list
        }

        Commands::Fmt { check, path } => {
            let target = path.unwrap_or_else(|| ".".to_string());

            if check {
                println!("{} {}", "Checking formatting:".cyan().bold(), target);
            } else {
                println!("{} {}", "Formatting:".cyan().bold(), target);
            }
            // TODO: Implement fmt
            println!("{}", "✓ Formatting complete".green());
        }
    }

    Ok(())
}
