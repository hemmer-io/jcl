//! Error handling and formatting for JCL
//!
//! Provides detailed, user-friendly error messages for parse and evaluation errors

use anyhow::anyhow;
use colored::Colorize;
use pest::error::{Error as PestError, LineColLocation};

use crate::parser::Rule;

/// Format a Pest parsing error with context and helpful information
pub fn format_parse_error(error: &PestError<Rule>, input: &str) -> String {
    let mut output = String::new();

    // Get line and column information
    let (line, col) = match error.line_col {
        LineColLocation::Pos((line, col)) => (line, col),
        LineColLocation::Span((line, col), _) => (line, col),
    };

    // Error header
    output.push_str(&format!(
        "{} {}\n",
        "Parse error:".red().bold(),
        error.variant.message()
    ));

    // Location information
    output.push_str(&format!(
        "  {} {}:{}\n",
        "-->".blue().bold(),
        "input".dimmed(),
        format!("{}:{}", line, col).cyan()
    ));

    // Show the problematic line with context
    let lines: Vec<&str> = input.lines().collect();
    if line > 0 && line <= lines.len() {
        let line_idx = line - 1;

        output.push_str(&format!("   {}\n", "|".blue()));

        // Show previous line for context if available
        if line_idx > 0 {
            output.push_str(&format!(
                " {} | {}\n",
                format!("{:3}", line - 1).blue().dimmed(),
                lines[line_idx - 1].dimmed()
            ));
        }

        // Show the error line
        output.push_str(&format!(
            " {} | {}\n",
            format!("{:3}", line).blue().bold(),
            lines[line_idx]
        ));

        // Show error indicator
        let indicator = format!("{}^", " ".repeat(col - 1 + 7));
        output.push_str(&format!("   {} {}\n", "|".blue(), indicator.red().bold()));

        // Show next line for context if available
        if line_idx + 1 < lines.len() {
            output.push_str(&format!(
                " {} | {}\n",
                format!("{:3}", line + 1).blue().dimmed(),
                lines[line_idx + 1].dimmed()
            ));
        }

        output.push_str(&format!("   {}\n", "|".blue()));
    }

    // Show what was expected
    use pest::error::ErrorVariant;
    match &error.variant {
        ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            if !positives.is_empty() {
                output.push_str(&format!("  {} ", "Expected:".green().bold()));
                let expected: Vec<String> = positives
                    .iter()
                    .map(|r| format_rule_name(r))
                    .collect();
                output.push_str(&expected.join(", "));
                output.push('\n');
            }

            if !negatives.is_empty() {
                output.push_str(&format!("  {} ", "Unexpected:".red().bold()));
                let unexpected: Vec<String> = negatives
                    .iter()
                    .map(|r| format_rule_name(r))
                    .collect();
                output.push_str(&unexpected.join(", "));
                output.push('\n');
            }
        }
        ErrorVariant::CustomError { message } => {
            output.push_str(&format!("  {} {}\n", "Note:".yellow().bold(), message));
        }
    }

    // Add helpful hints based on common errors
    output.push_str(&get_error_hint(&error.variant, lines.get(line.saturating_sub(1))));

    output
}

/// Format a rule name to be more user-friendly
fn format_rule_name(rule: &Rule) -> String {
    match rule {
        Rule::EOI => "end of input".to_string(),
        Rule::identifier => "identifier".to_string(),
        Rule::number => "number".to_string(),
        Rule::quoted_string => "string".to_string(),
        Rule::boolean => "boolean (true/false)".to_string(),
        Rule::null => "null".to_string(),
        Rule::expression => "expression".to_string(),
        Rule::assignment => "assignment".to_string(),
        Rule::function_def => "function definition".to_string(),
        Rule::if_expr => "if expression".to_string(),
        Rule::when_expr => "when expression".to_string(),
        Rule::list => "list".to_string(),
        Rule::map => "map".to_string(),
        Rule::add_op => "'+'".to_string(),
        Rule::sub_op => "'-'".to_string(),
        Rule::mul_op => "'*'".to_string(),
        Rule::div_op => "'/'".to_string(),
        Rule::eq_op => "'=='".to_string(),
        Rule::ne_op => "'!='".to_string(),
        Rule::lt_op => "'<'".to_string(),
        Rule::gt_op => "'>'".to_string(),
        Rule::and_op => "'and'".to_string(),
        Rule::or_op => "'or'".to_string(),
        _ => format!("{:?}", rule).to_lowercase(),
    }
}

/// Get a helpful hint based on the error type and context
fn get_error_hint(variant: &pest::error::ErrorVariant<Rule>, line: Option<&&str>) -> String {
    use pest::error::ErrorVariant;

    if let Some(line_text) = line {
        let line = line_text.trim();

        // Check for common mistakes
        if line.contains("==") && !line.contains("==") {
            return format!(
                "\n  {} Use '==' for equality comparison, not '='\n",
                "Hint:".yellow().bold()
            );
        }

        if line.contains('[') && !line.contains(']') {
            return format!(
                "\n  {} Missing closing bracket ']' for list\n",
                "Hint:".yellow().bold()
            );
        }

        if line.contains('(') && !line.contains(')') {
            return format!(
                "\n  {} Missing closing parenthesis ')'\n",
                "Hint:".yellow().bold()
            );
        }

        if line.contains("fn ") && !line.contains('=') {
            return format!(
                "\n  {} Function definition needs '=' followed by the body\n",
                "Hint:".yellow().bold()
            );
        }

        if line.starts_with("if ") && !line.contains("then") {
            return format!(
                "\n  {} if expressions need 'then' keyword: if condition then value else other\n",
                "Hint:".yellow().bold()
            );
        }
    }

    match variant {
        ErrorVariant::ParsingError { positives, .. } => {
            if positives.contains(&Rule::EOI) {
                format!(
                    "\n  {} Unexpected input after statement. Did you forget a semicolon or newline?\n",
                    "Hint:".yellow().bold()
                )
            } else if positives.contains(&Rule::expression) {
                format!(
                    "\n  {} An expression (number, string, variable, etc.) was expected here\n",
                    "Hint:".yellow().bold()
                )
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

/// Create a parse error with formatted message
pub fn create_parse_error(pest_error: PestError<Rule>, input: &str) -> anyhow::Error {
    let formatted = format_parse_error(&pest_error, input);
    anyhow!("{}", formatted)
}
