//! JCL Schema Validation Tool
//!
//! Validates JCL configuration files against schemas.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use jcl::{parse_file, schema::Validator};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jcl-validate")]
#[command(about = "Validate JCL configuration files against schemas", long_about = None)]
struct Args {
    /// JCL configuration file to validate
    #[arg(value_name = "CONFIG")]
    config: PathBuf,

    /// Schema file (JSON or YAML)
    #[arg(short, long, value_name = "SCHEMA")]
    schema: PathBuf,

    /// Schema format (auto-detect if not specified)
    #[arg(short = 'f', long, value_name = "FORMAT")]
    schema_format: Option<SchemaFormat>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Exit with status 0 even if validation fails
    #[arg(long)]
    no_fail: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SchemaFormat {
    Json,
    Yaml,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read schema file
    let schema_content = fs::read_to_string(&args.schema)
        .with_context(|| format!("Failed to read schema file: {}", args.schema.display()))?;

    // Determine schema format
    let format = if let Some(fmt) = args.schema_format {
        fmt
    } else {
        // Auto-detect from extension
        match args.schema.extension().and_then(|s| s.to_str()) {
            Some("json") => SchemaFormat::Json,
            Some("yaml") | Some("yml") => SchemaFormat::Yaml,
            _ => {
                eprintln!("{}", "Could not determine schema format from extension. Please specify with --schema-format".red());
                std::process::exit(1);
            }
        }
    };

    // Load validator
    let validator = match format {
        SchemaFormat::Json => Validator::from_json(&schema_content)?,
        SchemaFormat::Yaml => Validator::from_yaml(&schema_content)?,
    };

    if args.verbose {
        println!("📋 Schema loaded from {}", args.schema.display());
    }

    // Parse JCL file
    let module = parse_file(&args.config)
        .with_context(|| format!("Failed to parse JCL file: {}", args.config.display()))?;

    if args.verbose {
        println!("📄 Configuration loaded from {}", args.config.display());
        println!("🔍 Validating...");
        println!();
    }

    // Validate
    let errors = validator.validate_module(&module)?;

    if errors.is_empty() {
        println!("{} {}", "✅".green(), "Validation passed!".green().bold());
        Ok(())
    } else {
        println!("{} {} validation error(s) found:", "❌".red(), errors.len());
        println!();

        for error in &errors {
            println!("  {} {}", "•".red(), error.path.yellow());
            println!("    {}", error.message);
            println!();
        }

        if args.no_fail {
            Ok(())
        } else {
            std::process::exit(1);
        }
    }
}
