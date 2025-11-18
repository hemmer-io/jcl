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
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Equal,       // =
    EqualEqual,  // ==
    NotEqual,    // !=
    Less,        // <
    LessEqual,   // <=
    Greater,     // >
    GreaterEqual,// >=
    Bang,        // !
    Pipe,        // |
    Question,    // ?
    QuestionDot, // ?.
    QuestionQuestion, // ??
    Colon,       // :
    Arrow,       // =>

    // Punctuation
    LeftParen,   // (
    RightParen,  // )
    LeftBracket, // [
    RightBracket,// ]
    LeftBrace,   // {
    RightBrace,  // }
    Comma,       // ,
    Dot,         // .

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
        let pairs = LexerParser::parse(Rule::tokens, self.source)
            .map_err(|e| anyhow!("Lexer error: {}", e))?;

        for pair in pairs {
            if pair.as_rule() == Rule::tokens {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::token {
                        if let Some(token) = self.process_token(inner)? {
                            self.tokens.push(token);
                        }
                    }
                }
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

                Rule::keyword_token => {
                    match inner.as_str() {
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
                    }
                }

                Rule::identifier_token => {
                    TokenKind::Identifier(inner.as_str().to_string())
                }

                Rule::number_token => {
                    let text = inner.as_str();
                    if text.contains('.') {
                        let f: f64 = text.parse()
                            .map_err(|_| anyhow!("Invalid float: {}", text))?;
                        TokenKind::Float(f)
                    } else {
                        let i: i64 = text.parse()
                            .map_err(|_| anyhow!("Invalid integer: {}", text))?;
                        TokenKind::Integer(i)
                    }
                }

                Rule::string_token => {
                    self.process_string_token(inner)?
                }

                Rule::operator_token => {
                    match inner.as_str() {
                        "=>" => TokenKind::Arrow,
                        "?." => TokenKind::QuestionDot,
                        "??" => TokenKind::QuestionQuestion,
                        "==" => TokenKind::EqualEqual,
                        "!=" => TokenKind::NotEqual,
                        "<=" => TokenKind::LessEqual,
                        ">=" => TokenKind::GreaterEqual,
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
                    }
                }

                Rule::punctuation_token => {
                    match inner.as_str() {
                        "(" => TokenKind::LeftParen,
                        ")" => TokenKind::RightParen,
                        "[" => TokenKind::LeftBracket,
                        "]" => TokenKind::RightBracket,
                        "{" => TokenKind::LeftBrace,
                        "}" => TokenKind::RightBrace,
                        "," => TokenKind::Comma,
                        "." => TokenKind::Dot,
                        p => return Err(anyhow!("Unknown punctuation: {}", p)),
                    }
                }

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
                    let content = self.unescape_string(&text[1..text.len()-1])?;
                    return Ok(TokenKind::String(StringValue::Simple(content)));
                }

                Rule::multiline_string_token => {
                    let text = inner.as_str();
                    // Remove triple quotes
                    let content = text[3..text.len()-3].to_string();
                    return Ok(TokenKind::String(StringValue::Multiline(content)));
                }

                Rule::interpolated_string_token => {
                    let mut parts = Vec::new();
                    for part in inner.into_inner() {
                        match part.as_rule() {
                            Rule::interpolation_part => {
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
                                                        expr.as_str().to_string()
                                                    ));
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
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

        Position { line, column, offset }
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
        assert!(matches!(&tokens[0].kind, TokenKind::String(StringValue::Simple(s)) if s == "hello world"));
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
}
