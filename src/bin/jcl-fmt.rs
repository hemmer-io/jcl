//! JCL Format Tool
//!
//! Formats JCL files according to standard style.

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use jcl::{formatter, parse_file};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jcl-fmt")]
#[command(about = "Format JCL files", long_about = None)]
struct Args {
    /// Files to format
    #[arg(value_name = "FILE", required = true)]
    files: Vec<PathBuf>,

    /// Check only (don't modify files)
    #[arg(short, long)]
    check: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut needs_formatting = Vec::new();
    let mut errors = Vec::new();

    for file in &args.files {
        match format_file(file, &args) {
            Ok(true) => needs_formatting.push(file.clone()),
            Ok(false) => {}
            Err(e) => {
                errors.push((file.clone(), e));
            }
        }
    }

    // Summary
    if args.check {
        if needs_formatting.is_empty() && errors.is_empty() {
            println!("{}", "✅ All files are properly formatted!".green().bold());
            Ok(())
        } else {
            if !needs_formatting.is_empty() {
                println!();
                println!(
                    "{} {} file(s) need formatting:",
                    "⚠️".yellow(),
                    needs_formatting.len()
                );
                for file in &needs_formatting {
                    println!("  - {}", file.display());
                }
            }

            if !errors.is_empty() {
                println!();
                println!("{} {} error(s):", "❌".red(), errors.len());
                for (file, err) in &errors {
                    println!("  - {}: {}", file.display(), err);
                }
            }

            std::process::exit(1);
        }
    } else {
        let formatted_count = args.files.len() - errors.len();
        println!();
        println!("{} Formatted {} file(s)", "✅".green(), formatted_count);

        if !errors.is_empty() {
            println!("{} {} error(s)", "❌".red(), errors.len());
            std::process::exit(1);
        }

        Ok(())
    }
}

fn format_file(path: &PathBuf, args: &Args) -> Result<bool> {
    if args.verbose {
        println!("{} {}", "📝 Processing:".blue(), path.display());
    }

    // Parse the file
    let module =
        parse_file(path).with_context(|| format!("Failed to parse file: {}", path.display()))?;

    // Format the module
    let formatted = formatter::format(&module)
        .with_context(|| format!("Failed to format file: {}", path.display()))?;

    // Read current content
    let current_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Check if formatting is needed
    if formatted == current_content {
        if args.verbose {
            println!("{} {} - Already formatted", "✓".green(), path.display());
        }
        return Ok(false);
    }

    if args.check {
        println!("{} {} - Needs formatting", "!".yellow(), path.display());
        Ok(true)
    } else {
        // Write formatted content
        fs::write(path, &formatted)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        println!("{} {} - Formatted", "✓".green(), path.display());
        Ok(false)
    }
}
