//! JCL Schema Generation Tool
//!
//! Generate schemas from example JCL configuration files.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use jcl::parse_file;
use jcl::schema::{generate_from_examples, GenerateOptions};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jcl-schema-gen")]
#[command(about = "Generate JCL schemas from example configurations", long_about = None)]
struct Args {
    /// Example JCL files to analyze (can specify multiple)
    #[arg(value_name = "FILES", num_args = 1..)]
    files: Vec<PathBuf>,

    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "json-schema")]
    format: OutputFormat,

    /// Infer specific types from values (vs. using Any type)
    #[arg(long, default_value_t = true)]
    infer_types: bool,

    /// Infer constraints (min/max lengths, ranges, patterns)
    #[arg(long, default_value_t = true)]
    infer_constraints: bool,

    /// Mark all fields as optional (permissive schema)
    #[arg(long)]
    all_optional: bool,

    /// Schema title
    #[arg(short, long, value_name = "TITLE")]
    title: Option<String>,

    /// Schema description
    #[arg(short, long, value_name = "DESCRIPTION")]
    description: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// JCL schema format (JSON)
    JsonSchema,
    /// JCL schema format (YAML)
    Yaml,
    /// OpenAPI 3.0 schema
    OpenApi,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.files.is_empty() {
        eprintln!("{}", "Error: No input files specified".red());
        eprintln!();
        eprintln!("Usage: jcl-schema-gen <FILES>...");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  jcl-schema-gen config.jcf");
        eprintln!("  jcl-schema-gen config1.jcf config2.jcf --output schema.json");
        eprintln!("  jcl-schema-gen examples/*.jcf --format yaml");
        std::process::exit(1);
    }

    if args.verbose {
        println!("📄 Analyzing {} example file(s)...", args.files.len());
    }

    // Parse all example files
    let mut modules = Vec::new();
    for file_path in &args.files {
        if args.verbose {
            println!("  Reading: {}", file_path.display());
        }

        let module = parse_file(file_path)
            .with_context(|| format!("Failed to parse example file: {}", file_path.display()))?;

        modules.push(module);
    }

    // Generate schema
    let options = GenerateOptions {
        infer_types: args.infer_types,
        infer_constraints: args.infer_constraints,
        mark_all_optional: args.all_optional,
    };

    if args.verbose {
        println!();
        println!("🔍 Generating schema...");
        println!("  Infer types: {}", args.infer_types);
        println!("  Infer constraints: {}", args.infer_constraints);
        println!("  All optional: {}", args.all_optional);
        println!();
    }

    let mut schema = generate_from_examples(&modules, options)?;

    // Override title and description if provided
    if let Some(title) = args.title {
        schema.title = Some(title);
    }
    if let Some(description) = args.description {
        schema.description = Some(description);
    }

    // Generate output based on format
    let output = match args.format {
        OutputFormat::JsonSchema => schema.to_json_schema(),
        OutputFormat::Yaml => {
            // Convert schema to YAML
            serde_yaml::to_string(&schema).with_context(|| "Failed to serialize schema to YAML")?
        }
        OutputFormat::OpenApi => schema.to_openapi(),
    };

    // Write output
    if let Some(output_path) = args.output {
        fs::write(&output_path, output).with_context(|| {
            format!("Failed to write to output file: {}", output_path.display())
        })?;

        if args.verbose {
            println!(
                "{} Schema written to {}",
                "✅".green(),
                output_path.display()
            );
        } else {
            println!("{}", output_path.display());
        }
    } else {
        // Print to stdout
        println!("{}", output);
    }

    Ok(())
}
