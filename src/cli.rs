//! CLI utilities and helpers

use colored::*;

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an error message
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

/// Print a warning message
pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".cyan().bold(), msg);
}

/// Print a step header
pub fn step(msg: &str) {
    println!("\n{}", msg.bold());
}

/// Prompt for confirmation
pub fn confirm(prompt: &str) -> bool {
    println!("{} {}", "?".yellow().bold(), prompt);
    print!("  Type 'yes' to confirm: ");
    use std::io::{self, Write};
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim() == "yes"
}

/// Format a resource address
pub fn format_resource(resource_type: &str, name: &str) -> String {
    format!("{}.{}", resource_type.cyan(), name.bold())
}

/// Format a duration
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(std::time::Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(std::time::Duration::from_secs(90)), "1m 30s");
        assert_eq!(
            format_duration(std::time::Duration::from_secs(3661)),
            "1h 1m"
        );
    }

    #[test]
    fn test_format_resource() {
        let result = format_resource("aws_instance", "web_server");
        assert!(result.contains("aws_instance"));
        assert!(result.contains("web_server"));
    }
}
