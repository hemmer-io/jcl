//! JCL Watch and Auto-format Tool
//!
//! Watches JCL files for changes and automatically formats them.

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use jcl::{formatter, parse_file};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "jcl-watch")]
#[command(about = "Watch JCL files and auto-format on changes", long_about = None)]
struct Args {
    /// Files or directories to watch
    #[arg(value_name = "PATH", required = true)]
    paths: Vec<PathBuf>,

    /// Recursive watch for directories
    #[arg(short, long)]
    recursive: bool,

    /// Check only (don't modify files)
    #[arg(short, long)]
    check: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{}", "🔍 JCL Watch Mode".bold().cyan());
    println!("Watching for changes... (Press Ctrl+C to stop)");
    println!();

    // Create channel for receiving file events
    let (tx, rx) = channel();

    // Create watcher
    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).context("Failed to create file watcher")?;

    // Watch all specified paths
    for path in &args.paths {
        let mode = if args.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watcher
            .watch(path, mode)
            .with_context(|| format!("Failed to watch path: {}", path.display()))?;

        if args.verbose {
            println!("👁️  Watching: {}", path.display());
        }
    }

    println!();

    // Process events
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let Err(e) = handle_event(&event, &args) {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                }
            }
            Ok(Err(e)) => {
                eprintln!("{} {}", "Watch error:".red().bold(), e);
            }
            Err(e) => {
                eprintln!("{} {}", "Channel error:".red().bold(), e);
                break;
            }
        }

        // Small delay to debounce multiple events
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn handle_event(event: &Event, args: &Args) -> Result<()> {
    // Only process modify and create events
    if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
        return Ok(());
    }

    for path in &event.paths {
        // Only process .jcf files
        if path.extension().and_then(|s| s.to_str()) != Some("jcf") {
            continue;
        }

        if !path.exists() {
            continue;
        }

        format_file(path, args)?;
    }

    Ok(())
}

fn format_file(path: &Path, args: &Args) -> Result<()> {
    if args.verbose {
        println!("{} {}", "📝 Processing:".blue(), path.display());
    }

    // Parse the file
    let module = match parse_file(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "{} {} - {}",
                "✗".red(),
                path.display(),
                format!("Parse error: {}", e).red()
            );
            return Ok(()); // Don't fail the watcher, just skip this file
        }
    };

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
        return Ok(());
    }

    if args.check {
        println!("{} {} - Needs formatting", "!".yellow(), path.display());
    } else {
        // Write formatted content
        fs::write(path, &formatted)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        println!("{} {} - Formatted", "✓".green(), path.display());
    }

    Ok(())
}
