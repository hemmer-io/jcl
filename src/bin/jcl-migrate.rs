//! JCL Migration Tool
//!
//! Converts configuration files from JSON, YAML, or TOML to JCL format.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use jcl::migration::{json_to_jcl, toml_to_jcl, yaml_to_jcl};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jcl-migrate")]
#[command(about = "Convert JSON/YAML/TOML configuration files to JCL", long_about = None)]
struct Args {
    /// Input file to convert
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file (prints to stdout if not specified)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Input format (auto-detect if not specified)
    #[arg(short = 'f', long, value_name = "FORMAT")]
    from: Option<InputFormat>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InputFormat {
    Json,
    Yaml,
    Toml,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read input file
    let input_content = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file: {}", args.input.display()))?;

    // Determine input format
    let format = if let Some(fmt) = args.from {
        fmt
    } else {
        // Auto-detect from extension
        match args.input.extension().and_then(|s| s.to_str()) {
            Some("json") => InputFormat::Json,
            Some("yaml") | Some("yml") => InputFormat::Yaml,
            Some("toml") => InputFormat::Toml,
            _ => {
                eprintln!(
                    "Could not determine input format from extension. Please specify with --from"
                );
                std::process::exit(1);
            }
        }
    };

    if args.verbose {
        println!("📄 Converting from {:?} to JCL...", format);
    }

    // Convert to JCL
    let jcl_output = match format {
        InputFormat::Json => json_to_jcl(&input_content)?,
        InputFormat::Yaml => yaml_to_jcl(&input_content)?,
        InputFormat::Toml => toml_to_jcl(&input_content)?,
    };

    // Output
    if let Some(output_path) = args.output {
        fs::write(&output_path, &jcl_output)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

        if args.verbose {
            println!(
                "✅ Conversion complete! Output written to {}",
                output_path.display()
            );
        }
    } else {
        println!("{}", jcl_output);
    }

    Ok(())
}
