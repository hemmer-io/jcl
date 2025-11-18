//! JCL Benchmarking Tool
//!
//! Benchmarks JCL parsing and evaluation performance.

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use jcl::{evaluator::Evaluator, parse_str};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "jcl-bench")]
#[command(about = "Benchmark JCL parsing and evaluation", long_about = None)]
struct Args {
    /// JCL file to benchmark
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Number of iterations for each benchmark
    #[arg(short = 'n', long, default_value = "1000")]
    iterations: usize,

    /// Show detailed timing for each iteration
    #[arg(short, long)]
    verbose: bool,

    /// Run built-in benchmarks
    #[arg(long)]
    builtin: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{}", "JCL Benchmarking Tool".cyan().bold());
    println!();

    if args.builtin {
        run_builtin_benchmarks(&args)?;
    } else if let Some(file) = &args.file {
        benchmark_file(file, &args)?;
    } else {
        eprintln!(
            "{}",
            "Error: Please provide a file to benchmark or use --builtin".red()
        );
        std::process::exit(1);
    }

    Ok(())
}

fn benchmark_file(file: &PathBuf, args: &Args) -> Result<()> {
    println!("{} {}", "Benchmarking:".blue().bold(), file.display());
    println!("{} {}", "Iterations:".dimmed(), args.iterations);
    println!();

    // Read file
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    // Benchmark parsing
    let parse_duration = benchmark_parse(&content, args.iterations, args.verbose)?;

    // Benchmark evaluation
    let eval_duration = benchmark_eval(&content, args.iterations, args.verbose)?;

    // Print summary
    print_summary(parse_duration, eval_duration, args.iterations);

    Ok(())
}

fn benchmark_parse(content: &str, iterations: usize, verbose: bool) -> Result<Duration> {
    println!("{}", "📊 Parsing Benchmark".yellow().bold());

    let mut durations = Vec::new();
    let mut failures = 0;

    for i in 0..iterations {
        let start = Instant::now();
        match parse_str(content) {
            Ok(_) => {
                let duration = start.elapsed();
                durations.push(duration);

                if verbose {
                    println!("  Iteration {}: {:?}", i + 1, duration);
                }
            }
            Err(e) => {
                failures += 1;
                if verbose {
                    eprintln!("  Iteration {} failed: {}", i + 1, e);
                }
            }
        }
    }

    if failures > 0 {
        eprintln!("{} {} parsing failures", "⚠️".yellow(), failures);
    }

    if durations.is_empty() {
        eprintln!("{}", "All parsing attempts failed!".red());
        return Ok(Duration::default());
    }

    let total: Duration = durations.iter().sum();
    let avg = total / durations.len() as u32;
    let min = durations.iter().min().copied().unwrap_or_default();
    let max = durations.iter().max().copied().unwrap_or_default();

    println!("  {} {:?}", "Average:".green(), avg);
    println!("  {} {:?}", "Min:    ".dimmed(), min);
    println!("  {} {:?}", "Max:    ".dimmed(), max);
    println!();

    Ok(total)
}

fn benchmark_eval(content: &str, iterations: usize, verbose: bool) -> Result<Duration> {
    println!("{}", "📊 Evaluation Benchmark".yellow().bold());

    let mut durations = Vec::new();
    let mut failures = 0;

    for i in 0..iterations {
        // Parse once
        let module = match parse_str(content) {
            Ok(m) => m,
            Err(_) => {
                failures += 1;
                continue;
            }
        };

        // Benchmark evaluation
        let mut evaluator = Evaluator::new();
        let start = Instant::now();
        match evaluator.evaluate(module) {
            Ok(_) => {
                let duration = start.elapsed();
                durations.push(duration);

                if verbose {
                    println!("  Iteration {}: {:?}", i + 1, duration);
                }
            }
            Err(e) => {
                failures += 1;
                if verbose {
                    eprintln!("  Iteration {} failed: {}", i + 1, e);
                }
            }
        }
    }

    if failures > 0 {
        eprintln!("{} {} evaluation failures", "⚠️".yellow(), failures);
    }

    if durations.is_empty() {
        eprintln!("{}", "All evaluation attempts failed!".red());
        return Ok(Duration::default());
    }

    let total: Duration = durations.iter().sum();
    let avg = total / durations.len() as u32;
    let min = durations.iter().min().copied().unwrap_or_default();
    let max = durations.iter().max().copied().unwrap_or_default();

    println!("  {} {:?}", "Average:".green(), avg);
    println!("  {} {:?}", "Min:    ".dimmed(), min);
    println!("  {} {:?}", "Max:    ".dimmed(), max);
    println!();

    Ok(total)
}

fn print_summary(parse_duration: Duration, eval_duration: Duration, iterations: usize) {
    println!("{}", "Summary".cyan().bold());
    println!("{}", "─".repeat(50));

    let total = parse_duration + eval_duration;
    let parse_pct = (parse_duration.as_secs_f64() / total.as_secs_f64()) * 100.0;
    let eval_pct = (eval_duration.as_secs_f64() / total.as_secs_f64()) * 100.0;

    println!(
        "Total parsing time:    {:?} ({:.1}%)",
        parse_duration, parse_pct
    );
    println!(
        "Total evaluation time: {:?} ({:.1}%)",
        eval_duration, eval_pct
    );
    println!("Total time:            {:?}", total);
    println!();
    println!("Operations per second:");
    println!(
        "  Parsing:    {} ops/sec",
        (iterations as f64 / parse_duration.as_secs_f64()) as u64
    );
    println!(
        "  Evaluation: {} ops/sec",
        (iterations as f64 / eval_duration.as_secs_f64()) as u64
    );
    println!(
        "  Combined:   {} ops/sec",
        (iterations as f64 / total.as_secs_f64()) as u64
    );
}

fn run_builtin_benchmarks(args: &Args) -> Result<()> {
    println!("{}", "Running Built-in Benchmarks".cyan().bold());
    println!();

    let benchmarks = vec![
        ("Simple arithmetic", "x = 1 + 2 + 3"),
        ("String operations", r#"name = "hello" + " " + "world""#),
        ("List operations", "numbers = [1, 2, 3, 4, 5]"),
        (
            "Map operations",
            r#"config = (name = "app", version = "1.0.0")"#,
        ),
        ("Function call", "fn double(x) = x * 2\nresult = double(21)"),
        (
            "Complex expression",
            r#"
            x = 10
            y = 20
            z = x * y + 5
            result = if z > 100 then "large" else "small"
        "#,
        ),
    ];

    for (name, code) in benchmarks {
        println!("{} {}", "Testing:".blue().bold(), name);

        let parse_duration = benchmark_parse(code, args.iterations, false)?;
        let eval_duration = benchmark_eval(code, args.iterations, false)?;

        let total = parse_duration + eval_duration;
        println!(
            "  Total: {:?} ({} ops/sec)",
            total,
            (args.iterations as f64 / total.as_secs_f64()) as u64
        );
        println!();
    }

    Ok(())
}
