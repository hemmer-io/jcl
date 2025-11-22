//! JCL Schema Validation Tool
//!
//! Validates JCL configuration files against schemas.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use jcl::{parse_file, parse_files_parallel, schema::Validator};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "jcl-validate")]
#[command(about = "Validate JCL configuration files against schemas", long_about = None)]
struct Args {
    /// JCL configuration files to validate (can specify multiple)
    #[arg(value_name = "CONFIG", num_args = 0..)]
    config: Vec<PathBuf>,

    /// Schema file (JSON or YAML)
    #[arg(short, long, value_name = "SCHEMA")]
    schema: PathBuf,

    /// Validate all .jcf files in a directory
    #[arg(short, long, value_name = "DIR")]
    dir: Option<PathBuf>,

    /// Validate files matching glob pattern (e.g., "**/*.jcf")
    #[arg(short, long, value_name = "PATTERN")]
    pattern: Option<String>,

    /// Schema format (auto-detect if not specified)
    #[arg(short = 'f', long, value_name = "FORMAT")]
    schema_format: Option<SchemaFormat>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Exit with status 0 even if validation fails
    #[arg(long)]
    no_fail: bool,

    /// Continue validating all files even if some fail (don't stop on first error)
    #[arg(long)]
    no_fail_fast: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SchemaFormat {
    Json,
    Yaml,
}

struct ValidationResult {
    passed: bool,
    error_count: usize,
    errors: Vec<jcl::schema::ValidationError>,
}

fn collect_files(args: &Args) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Add files from direct arguments
    files.extend(args.config.clone());

    // Add files from directory
    if let Some(dir) = &args.dir {
        if !dir.is_dir() {
            anyhow::bail!("Specified directory does not exist: {}", dir.display());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jcf") {
                files.push(path);
            }
        }
    }

    // Add files from glob pattern
    if let Some(pattern) = &args.pattern {
        let glob_paths =
            glob::glob(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?;
        for path in glob_paths {
            let path = path?;
            if path.extension().and_then(|s| s.to_str()) == Some("jcf") {
                files.push(path);
            }
        }
    }

    if files.is_empty() {
        anyhow::bail!("No files specified. Provide files directly, use --dir, or --pattern");
    }

    // Deduplicate files
    files.sort();
    files.dedup();

    Ok(files)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Collect all files to validate
    let files = collect_files(&args)?;

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

    println!("🔍 Validating {} file(s)...", files.len());
    if args.verbose {
        println!();
    }

    let start_time = Instant::now();

    // Parse all files (use parallel parsing for better performance with multiple files)
    let parsed_files = if files.len() > 1 {
        parse_files_parallel(&files)?
    } else {
        // Single file - use regular parsing
        vec![(
            files[0].clone(),
            parse_file(&files[0])
                .with_context(|| format!("Failed to parse JCL file: {}", files[0].display()))?,
        )]
    };

    // Validate each parsed file
    let mut results = Vec::new();
    let mut failed_count = 0;

    for (file_path, module) in parsed_files {
        // Validate the module
        let errors = validator.validate_module(&module)?;
        let passed = errors.is_empty();
        let error_count = errors.len();

        if !passed {
            failed_count += 1;
        }

        let result = ValidationResult {
            passed,
            error_count,
            errors,
        };

        // Print per-file result
        if result.passed {
            println!(
                "  {} {} - {}",
                "✅".green(),
                file_path.display(),
                "passed".green()
            );
        } else {
            println!(
                "  {} {} - {} error(s)",
                "❌".red(),
                file_path.display(),
                result.error_count
            );
            if args.verbose || !args.no_fail_fast {
                for error in &result.errors {
                    println!("      {} {}", "•".red(), error.path.yellow());
                    println!("        {}", error.message);
                }
            }
        }

        results.push(result);

        // Stop on first error if fail-fast is enabled
        if !passed && !args.no_fail_fast {
            break;
        }
    }

    let elapsed = start_time.elapsed();

    // Print summary
    println!();
    println!("Summary");
    println!("──────────────────────────────────────────────────");
    println!("Total files:     {}", results.len());
    println!(
        "Passed:          {} ✅",
        results.iter().filter(|r| r.passed).count()
    );
    println!(
        "Failed:          {} {}",
        failed_count,
        if failed_count > 0 { "❌" } else { "" }
    );
    println!("Validation time: {:?}", elapsed);

    if failed_count > 0 && !args.no_fail_fast {
        println!();
        println!(
            "{} Stopped on first error. Use {} to validate all files.",
            "ℹ️".blue(),
            "--no-fail-fast".yellow()
        );
    }

    println!();

    // Exit based on validation results
    if failed_count == 0 {
        println!(
            "{} {}",
            "✅".green(),
            "All files passed validation!".green().bold()
        );
        Ok(())
    } else {
        println!(
            "{} {}",
            "❌".red(),
            format!("Validation failed ({} file(s) with errors)", failed_count)
                .red()
                .bold()
        );
        if args.no_fail {
            Ok(())
        } else {
            std::process::exit(1);
        }
    }
}
