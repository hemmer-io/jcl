//! Lexer module for JCL - tokenizes source code before parsing
//!
//! This module provides a two-phase compilation approach:
//! 1. Lexer: Source code → Token stream
//! 2. Parser: Token stream → AST
//!
//! This separation allows proper keyword/identifier distinction.

use anyhow::{anyhow, Result};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lexer.pest"]
struct LexerParser;

/// Position information for a token
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// A token with its value and position
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Span of source text
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub text: String,
}

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Import,
    From,
    Fn,
    If,
    Then,
    Else,
    When,
    For,
    In,
    As,
    Mut,
    Try,
    And,
    Or,
    Not,
    True,
    False,
    Null,
    Match,

    // Literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(StringValue),

    // Operators
    Plus,             // +
    Minus,            // -
    Star,             // *
    Slash,            // /
    Percent,          // %
    Equal,            // =
    EqualEqual,       // ==
    NotEqual,         // !=
    Less,             // <
    LessEqual,        // <=
    Greater,          // >
    GreaterEqual,     // >=
    Bang,             // !
    Pipe,             // |
    Question,         // ?
    QuestionDot,      // ?.
    QuestionQuestion, // ??
    Colon,            // :
    Arrow,            // =>
    DotDot,           // ..
    DotDotLess,       // ..<

    // Punctuation
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Dot,          // .

    // Special
    DocComment(String),
    Eof,
}

/// String value that may contain interpolations
#[derive(Debug, Clone, PartialEq)]
pub enum StringValue {
    Simple(String),
    Multiline(String),
    Interpolated(Vec<StringPart>),
    Heredoc {
        parts: Vec<StringPart>,
        strip_indent: bool,
    },
}

/// Part of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Interpolation(String), // Raw expression text to be parsed later
}

/// Lexer that converts source code to tokens
pub struct Lexer<'a> {
    source: &'a str,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
        }
    }

    /// Tokenize the source code
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        // Manually scan for heredocs since they require context-sensitive parsing
        let mut offset = 0;
        let chars_vec: Vec<char> = self.source.chars().collect();

        while offset < chars_vec.len() {
            // Check for heredoc start
            if offset < chars_vec.len() - 1
                && chars_vec[offset] == '<'
                && chars_vec[offset + 1] == '<'
            {
                // Try to parse heredoc
                if let Some((heredoc_token, end_offset)) = self.try_parse_heredoc(offset)? {
                    self.tokens.push(heredoc_token);
                    offset = end_offset;
                    continue;
                }
            }

            // Find next potential heredoc or end of current segment
            let segment_end = self.find_next_heredoc_or_end(offset);
            if segment_end > offset {
                // Parse this segment with Pest
                let segment = &self.source
                    [self.char_offset_to_byte(offset)..self.char_offset_to_byte(segment_end)];
                let pairs = LexerParser::parse(Rule::tokens, segment)
                    .map_err(|e| anyhow!("Lexer error at offset {}: {}", offset, e))?;

                for pair in pairs {
                    if pair.as_rule() == Rule::tokens {
                        for inner in pair.into_inner() {
                            if inner.as_rule() == Rule::token {
                                if let Some(token) = self.process_token_with_offset(
                                    inner,
                                    self.char_offset_to_byte(offset),
                                )? {
                                    self.tokens.push(token);
                                }
                            }
                        }
                    }
                }
                offset = segment_end;
            } else {
                break;
            }
        }

        // Add EOF token
        let eof_pos = self.position_from_offset(self.source.len());
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: eof_pos.clone(),
                end: eof_pos,
                text: String::new(),
            },
        });

        Ok(self.tokens.clone())
    }

    /// Convert character offset to byte offset
    fn char_offset_to_byte(&self, char_offset: usize) -> usize {
        self.source
            .chars()
            .take(char_offset)
            .map(|c| c.len_utf8())
            .sum()
    }

    /// Find the next heredoc start or end of source
    fn find_next_heredoc_or_end(&self, start_offset: usize) -> usize {
        let chars_vec: Vec<char> = self.source.chars().collect();
        for i in start_offset..chars_vec.len() - 1 {
            if chars_vec[i] == '<' && chars_vec[i + 1] == '<' {
                return i;
            }
        }
        chars_vec.len()
    }

    /// Try to parse a heredoc starting at the given character offset
    /// Returns the heredoc token and the character offset after the heredoc
    fn try_parse_heredoc(&self, char_offset: usize) -> Result<Option<(Token, usize)>> {
        let chars_vec: Vec<char> = self.source.chars().collect();
        let mut offset = char_offset;

        // Verify << prefix
        if offset + 1 >= chars_vec.len() || chars_vec[offset] != '<' || chars_vec[offset + 1] != '<'
        {
            return Ok(None);
        }
        offset += 2;

        // Check for strip_indent flag (<<-)
        let strip_indent = offset < chars_vec.len() && chars_vec[offset] == '-';
        if strip_indent {
            offset += 1;
        }

        // Read delimiter
        let delimiter_start = offset;
        while offset < chars_vec.len()
            && (chars_vec[offset].is_alphanumeric() || chars_vec[offset] == '_')
        {
            offset += 1;
        }

        if offset == delimiter_start {
            return Ok(None); // No valid delimiter
        }

        let delimiter: String = chars_vec[delimiter_start..offset].iter().collect();

        // Skip whitespace until newline
        while offset < chars_vec.len()
            && chars_vec[offset].is_whitespace()
            && chars_vec[offset] != '\n'
        {
            offset += 1;
        }

        // Expect newline after delimiter
        if offset >= chars_vec.len() || chars_vec[offset] != '\n' {
            return Err(anyhow!("Expected newline after heredoc delimiter"));
        }
        offset += 1; // skip newline

        // Collect heredoc lines
        let mut lines = Vec::new();

        while offset < chars_vec.len() {
            // Check if current line is the closing delimiter
            let line_start = offset;
            let mut test_line = String::new();
            let mut test_offset = line_start;

            while test_offset < chars_vec.len() && chars_vec[test_offset] != '\n' {
                test_line.push(chars_vec[test_offset]);
                test_offset += 1;
            }

            if test_line.trim() == delimiter {
                // Found closing delimiter
                offset = test_offset;
                if offset < chars_vec.len() && chars_vec[offset] == '\n' {
                    offset += 1; // skip newline after delimiter
                }
                break;
            }

            // Not the delimiter, add this line to content
            lines.push(test_line);
            offset = test_offset;
            if offset < chars_vec.len() && chars_vec[offset] == '\n' {
                offset += 1; // skip newline
            }
        }

        // Process lines (strip indentation, parse interpolations)
        let parts = self.process_heredoc_content(&lines, strip_indent)?;

        // Create token
        let start_pos = self.position_from_offset(self.char_offset_to_byte(char_offset));
        let end_pos = self.position_from_offset(self.char_offset_to_byte(offset));
        let token = Token {
            kind: TokenKind::String(StringValue::Heredoc {
                parts,
                strip_indent,
            }),
            span: Span {
                start: start_pos,
                end: end_pos,
                text: format!("<<{}{}", if strip_indent { "-" } else { "" }, delimiter),
            },
        };

        Ok(Some((token, offset)))
    }

    /// Process heredoc content lines
    fn process_heredoc_content(
        &self,
        lines: &[String],
        strip_indent: bool,
    ) -> Result<Vec<StringPart>> {
        // Calculate indentation to strip
        let indent_to_strip = if strip_indent {
            lines
                .iter()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    line.chars()
                        .take_while(|c| c.is_whitespace() && *c != '\n')
                        .count()
                })
                .min()
                .unwrap_or(0)
        } else {
            0
        };

        // Process lines, adding newlines between them
        let mut all_parts = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let processed = if indent_to_strip > 0 {
                line.chars().skip(indent_to_strip.min(line.len())).collect()
            } else {
                line.clone()
            };

            let parts = self.parse_string_interpolations(&processed)?;
            all_parts.extend(parts);

            // Add newline after each line except the last
            if i < lines.len() - 1 {
                // If last part is a literal, append newline to it
                if let Some(StringPart::Literal(s)) = all_parts.last_mut() {
                    s.push('\n');
                } else {
                    // Otherwise add a new literal part with newline
                    all_parts.push(StringPart::Literal("\n".to_string()));
                }
            }
        }

        // Handle empty heredoc case
        if all_parts.is_empty() {
            all_parts.push(StringPart::Literal(String::new()));
        }

        Ok(all_parts)
    }

    /// Parse string interpolations (${...}) in a string
    fn parse_string_interpolations(&self, s: &str) -> Result<Vec<StringPart>> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() == Some(&'{') {
                // Start of interpolation
                if !current.is_empty() {
                    parts.push(StringPart::Literal(current.clone()));
                    current.clear();
                }

                chars.next(); // consume {
                let mut expr = String::new();
                let mut depth = 1;

                // Collect expression content, handling nested braces
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        depth += 1;
                        expr.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr.push(ch);
                    } else {
                        expr.push(ch);
                    }
                }

                if depth != 0 {
                    return Err(anyhow!("Unclosed interpolation expression"));
                }

                parts.push(StringPart::Interpolation(expr));
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            parts.push(StringPart::Literal(current));
        }

        // If no interpolations found, return a single literal
        if parts.is_empty() {
            parts.push(StringPart::Literal(String::new()));
        }

        Ok(parts)
    }

    /// Process a single token pair with an offset adjustment
    fn process_token_with_offset(
        &self,
        pair: pest::iterators::Pair<Rule>,
        _base_offset: usize,
    ) -> Result<Option<Token>> {
        // For now, delegate to process_token
        // In a full implementation, we'd adjust spans by base_offset
        self.process_token(pair)
    }
    /// Process a single token pair
    fn process_token(&self, pair: pest::iterators::Pair<Rule>) -> Result<Option<Token>> {
        let span = self.span_from_pair(&pair);

        for inner in pair.into_inner() {
            let kind = match inner.as_rule() {
                Rule::doccomment_token => {
                    let text = inner.as_str();
                    // Strip leading "///" and trim
                    let content = text.strip_prefix("///").unwrap_or(text).trim().to_string();
                    TokenKind::DocComment(content)
                }

                Rule::keyword_token => match inner.as_str() {
                    "import" => TokenKind::Import,
                    "from" => TokenKind::From,
                    "fn" => TokenKind::Fn,
                    "if" => TokenKind::If,
                    "then" => TokenKind::Then,
                    "else" => TokenKind::Else,
                    "when" => TokenKind::When,
                    "for" => TokenKind::For,
                    "in" => TokenKind::In,
                    "as" => TokenKind::As,
                    "mut" => TokenKind::Mut,
                    "try" => TokenKind::Try,
                    "and" => TokenKind::And,
                    "or" => TokenKind::Or,
                    "not" => TokenKind::Not,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "null" => TokenKind::Null,
                    "match" => TokenKind::Match,
                    kw => return Err(anyhow!("Unknown keyword: {}", kw)),
                },

                Rule::identifier_token => TokenKind::Identifier(inner.as_str().to_string()),

                Rule::number_token => {
                    let text = inner.as_str();
                    if text.contains('.') {
                        let f: f64 = text
                            .parse()
                            .map_err(|_| anyhow!("Invalid float: {}", text))?;
                        TokenKind::Float(f)
                    } else {
                        let i: i64 = text
                            .parse()
                            .map_err(|_| anyhow!("Invalid integer: {}", text))?;
                        TokenKind::Integer(i)
                    }
                }

                Rule::string_token => self.process_string_token(inner)?,

                Rule::operator_token => match inner.as_str() {
                    "=>" => TokenKind::Arrow,
                    "?." => TokenKind::QuestionDot,
                    "??" => TokenKind::QuestionQuestion,
                    "==" => TokenKind::EqualEqual,
                    "!=" => TokenKind::NotEqual,
                    "<=" => TokenKind::LessEqual,
                    ">=" => TokenKind::GreaterEqual,
                    "..<" => TokenKind::DotDotLess,
                    ".." => TokenKind::DotDot,
                    "+" => TokenKind::Plus,
                    "-" => TokenKind::Minus,
                    "*" => TokenKind::Star,
                    "/" => TokenKind::Slash,
                    "%" => TokenKind::Percent,
                    "<" => TokenKind::Less,
                    ">" => TokenKind::Greater,
                    "=" => TokenKind::Equal,
                    "!" => TokenKind::Bang,
                    "|" => TokenKind::Pipe,
                    "?" => TokenKind::Question,
                    ":" => TokenKind::Colon,
                    op => return Err(anyhow!("Unknown operator: {}", op)),
                },

                Rule::punctuation_token => match inner.as_str() {
                    "(" => TokenKind::LeftParen,
                    ")" => TokenKind::RightParen,
                    "[" => TokenKind::LeftBracket,
                    "]" => TokenKind::RightBracket,
                    "{" => TokenKind::LeftBrace,
                    "}" => TokenKind::RightBrace,
                    "," => TokenKind::Comma,
                    "." => TokenKind::Dot,
                    p => return Err(anyhow!("Unknown punctuation: {}", p)),
                },

                _ => continue,
            };

            return Ok(Some(Token { kind, span }));
        }

        Ok(None)
    }

    /// Process a string token (simple, multiline, or interpolated)
    fn process_string_token(&self, pair: pest::iterators::Pair<Rule>) -> Result<TokenKind> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::simple_string_token => {
                    let text = inner.as_str();
                    // Remove quotes and unescape
                    let content = self.unescape_string(&text[1..text.len() - 1])?;
                    return Ok(TokenKind::String(StringValue::Simple(content)));
                }

                Rule::multiline_string_token => {
                    let text = inner.as_str();
                    // Remove triple quotes
                    let content = text[3..text.len() - 3].to_string();
                    return Ok(TokenKind::String(StringValue::Multiline(content)));
                }

                Rule::interpolated_string_token => {
                    let mut parts = Vec::new();
                    for part in inner.into_inner() {
                        if part.as_rule() == Rule::interpolation_part {
                            for sub in part.into_inner() {
                                match sub.as_rule() {
                                    Rule::string_literal_part => {
                                        let content = self.unescape_string(sub.as_str())?;
                                        parts.push(StringPart::Literal(content));
                                    }
                                    Rule::interpolation_expr => {
                                        for expr in sub.into_inner() {
                                            if expr.as_rule() == Rule::interpolated_content {
                                                parts.push(StringPart::Interpolation(
                                                    expr.as_str().to_string(),
                                                ));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    return Ok(TokenKind::String(StringValue::Interpolated(parts)));
                }

                _ => {}
            }
        }

        Err(anyhow!("Invalid string token"))
    }

    /// Unescape a string (handle \n, \t, etc.)
    fn unescape_string(&self, s: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('$') => result.push('$'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }

        Ok(result)
    }

    /// Create a Span from a pest Pair
    fn span_from_pair(&self, pair: &pest::iterators::Pair<Rule>) -> Span {
        let pest_span = pair.as_span();
        Span {
            start: self.position_from_offset(pest_span.start()),
            end: self.position_from_offset(pest_span.end()),
            text: pair.as_str().to_string(),
        }
    }

    /// Calculate line and column from byte offset
    fn position_from_offset(&self, offset: usize) -> Position {
        let mut line = 1;
        let mut column = 1;

        for (i, c) in self.source.chars().enumerate() {
            if i >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Position {
            line,
            column,
            offset,
        }
    }
}

/// Convenience function to tokenize a string
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_keywords() {
        let tokens = tokenize("if then else").unwrap();
        assert_eq!(tokens.len(), 4); // 3 keywords + EOF
        assert!(matches!(tokens[0].kind, TokenKind::If));
        assert!(matches!(tokens[1].kind, TokenKind::Then));
        assert!(matches!(tokens[2].kind, TokenKind::Else));
    }

    #[test]
    fn test_tokenize_identifiers() {
        let tokens = tokenize("format info order android").unwrap();
        assert_eq!(tokens.len(), 5); // 4 identifiers + EOF
        assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "format"));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "info"));
        assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "order"));
        assert!(matches!(&tokens[3].kind, TokenKind::Identifier(s) if s == "android"));
    }

    #[test]
    fn test_keyword_vs_identifier() {
        // "for" is a keyword, "format" is an identifier
        let tokens = tokenize("for format").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].kind, TokenKind::For));
        assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "format"));
    }

    #[test]
    fn test_tokenize_assignment() {
        let tokens = tokenize("format = 42").unwrap();
        assert_eq!(tokens.len(), 4); // identifier, =, number, EOF
        assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "format"));
        assert!(matches!(tokens[1].kind, TokenKind::Equal));
        assert!(matches!(tokens[2].kind, TokenKind::Integer(42)));
    }

    #[test]
    fn test_tokenize_string() {
        let tokens = tokenize(r#""hello world""#).unwrap();
        println!("Tokens: {:?}", tokens);
        assert_eq!(tokens.len(), 2);
        assert!(
            matches!(&tokens[0].kind, TokenKind::String(StringValue::Simple(s)) if s == "hello world")
        );
    }

    #[test]
    fn test_tokenize_operators() {
        let tokens = tokenize("+ - * / == != <= >=").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Plus));
        assert!(matches!(tokens[1].kind, TokenKind::Minus));
        assert!(matches!(tokens[2].kind, TokenKind::Star));
        assert!(matches!(tokens[3].kind, TokenKind::Slash));
        assert!(matches!(tokens[4].kind, TokenKind::EqualEqual));
        assert!(matches!(tokens[5].kind, TokenKind::NotEqual));
        assert!(matches!(tokens[6].kind, TokenKind::LessEqual));
        assert!(matches!(tokens[7].kind, TokenKind::GreaterEqual));
    }

    #[test]
    fn test_position_tracking() {
        let tokens = tokenize("x\ny").unwrap();
        assert_eq!(tokens[0].span.start.line, 1);
        assert_eq!(tokens[0].span.start.column, 1);
        assert_eq!(tokens[1].span.start.line, 2);
        assert_eq!(tokens[1].span.start.column, 1);
    }

    #[test]
    fn test_heredoc_basic() {
        let source = r#"script = <<EOF
hello world
this is a test
EOF
"#;
        let tokens = tokenize(source).unwrap();

        // Find the string token (should be heredoc)
        let string_token = tokens
            .iter()
            .find(|t| matches!(t.kind, TokenKind::String(_)));
        assert!(string_token.is_some(), "Should have a string token");

        if let TokenKind::String(StringValue::Heredoc {
            parts,
            strip_indent,
        }) = &string_token.unwrap().kind
        {
            assert!(!strip_indent, "Should not strip indent for <<");
            assert!(!parts.is_empty(), "Should have content");
        } else {
            panic!("Expected heredoc, got {:?}", string_token.unwrap().kind);
        }
    }

    #[test]
    fn test_heredoc_with_indent_stripping() {
        let source = r#"script = <<-EOF
    hello world
    this is indented
EOF
"#;
        let tokens = tokenize(source).unwrap();

        let string_token = tokens
            .iter()
            .find(|t| matches!(t.kind, TokenKind::String(_)));
        assert!(string_token.is_some(), "Should have a string token");

        if let TokenKind::String(StringValue::Heredoc {
            parts,
            strip_indent,
        }) = &string_token.unwrap().kind
        {
            assert!(*strip_indent, "Should strip indent for <<-");
            // Verify that content has indentation stripped
            if let Some(StringPart::Literal(s)) = parts.first() {
                assert!(
                    s.starts_with("hello"),
                    "Indentation should be stripped: {}",
                    s
                );
            }
        } else {
            panic!("Expected heredoc, got {:?}", string_token.unwrap().kind);
        }
    }

    #[test]
    fn test_heredoc_with_interpolation() {
        let source = r#"name = "World"
greeting = <<EOF
Hello, ${name}!
Welcome to JCL.
EOF
"#;
        let tokens = tokenize(source).unwrap();

        // Find the heredoc token (should be the second string)
        let heredoc_token = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::String(_)))
            .nth(1);
        assert!(heredoc_token.is_some(), "Should have a heredoc token");

        if let TokenKind::String(StringValue::Heredoc { parts, .. }) = &heredoc_token.unwrap().kind
        {
            // Should have: "Hello, " + interpolation + "!\nWelcome to JCL.\n"
            assert!(
                parts.len() >= 2,
                "Should have literal and interpolation parts, got {} parts",
                parts.len()
            );

            // Check for interpolation
            let has_interpolation = parts
                .iter()
                .any(|p| matches!(p, StringPart::Interpolation(_)));
            assert!(has_interpolation, "Should have interpolation part");
        } else {
            panic!("Expected heredoc, got {:?}", heredoc_token.unwrap().kind);
        }
    }

    #[test]
    fn test_multiple_heredocs() {
        let source = r#"script1 = <<EOF1
first heredoc
EOF1
script2 = <<EOF2
second heredoc
EOF2
"#;
        let tokens = tokenize(source).unwrap();

        let heredoc_count = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::String(StringValue::Heredoc { .. })))
            .count();

        assert_eq!(heredoc_count, 2, "Should have two heredoc tokens");
    }

    #[test]
    fn test_heredoc_empty() {
        let source = r#"script = <<EOF
EOF
"#;
        let tokens = tokenize(source).unwrap();

        let string_token = tokens
            .iter()
            .find(|t| matches!(t.kind, TokenKind::String(_)));
        assert!(string_token.is_some(), "Should have a string token");

        if let TokenKind::String(StringValue::Heredoc { parts, .. }) = &string_token.unwrap().kind {
            // Empty heredoc may have one empty literal part or be truly empty
            println!("Parts: {:?}", parts);
            // For an empty heredoc, we should either have no parts or one empty part
            if parts.len() == 1 {
                if let Some(StringPart::Literal(s)) = parts.first() {
                    println!("Content: '{}'", s);
                    assert!(
                        s.is_empty() || s == "\n",
                        "Content should be empty or just newline, got: '{}'",
                        s
                    );
                }
            } else {
                assert!(parts.is_empty(), "Should have no parts or one empty part");
            }
        } else {
            panic!("Expected heredoc, got {:?}", string_token.unwrap().kind);
        }
    }

    #[test]
    fn test_heredoc_preserves_blank_lines() {
        let source = r#"script = <<EOF
line1

line3
EOF
"#;
        let tokens = tokenize(source).unwrap();

        let string_token = tokens
            .iter()
            .find(|t| matches!(t.kind, TokenKind::String(_)));
        assert!(string_token.is_some(), "Should have a string token");

        if let TokenKind::String(StringValue::Heredoc { parts, .. }) = &string_token.unwrap().kind {
            // Reconstruct the full content from all parts
            let mut full_content = String::new();
            for part in parts {
                if let StringPart::Literal(s) = part {
                    full_content.push_str(s);
                }
            }

            // Should contain the blank line - check for presence of double newline
            assert!(
                full_content.contains("\n\n"),
                "Should preserve blank lines, got: '{}'",
                full_content
            );
        } else {
            panic!("Expected heredoc, got {:?}", string_token.unwrap().kind);
        }
    }

    #[test]
    fn test_heredoc_with_special_chars() {
        let source = r#"script = <<EOF
#!/bin/bash
echo "quotes and $vars"
special: @#$%^&*
EOF
"#;
        let tokens = tokenize(source).unwrap();

        let string_token = tokens
            .iter()
            .find(|t| matches!(t.kind, TokenKind::String(_)));
        assert!(
            string_token.is_some(),
            "Should tokenize heredoc with special chars"
        );

        if let TokenKind::String(StringValue::Heredoc { parts, .. }) = &string_token.unwrap().kind {
            assert!(!parts.is_empty(), "Should have content");
        } else {
            panic!("Expected heredoc, got {:?}", string_token.unwrap().kind);
        }
    }
}
