use regex::Regex;

use crate::error::{BlinkError, ErrorCode, Span};
use crate::settings;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Comma,
    OpenParentheses,
    CloseParentheses,
    OpenBraces,
    CloseBraces,
    OpenBrackets,
    CloseBrackets,
    Merge,
    String,
    Boolean,
    Number,
    Array,
    Range,
    Optional,
    Class,
    Component,
    OpenChevrons,
    CloseChevrons,
    Assign,
    FieldAssign,
    Keyword,
    Primitive,
    Identifier,
    Import,
    As,
    Whitespace,
    Comment,
    Unknown,
    EndOfFile,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Comma => write!(f, "Comma"),
            TokenType::OpenParentheses => write!(f, "OpenParentheses"),
            TokenType::CloseParentheses => write!(f, "CloseParentheses"),
            TokenType::OpenBraces => write!(f, "OpenBraces"),
            TokenType::CloseBraces => write!(f, "CloseBraces"),
            TokenType::OpenBrackets => write!(f, "OpenBrackets"),
            TokenType::CloseBrackets => write!(f, "CloseBrackets"),
            TokenType::Merge => write!(f, "Merge"),
            TokenType::String => write!(f, "String"),
            TokenType::Boolean => write!(f, "Boolean"),
            TokenType::Number => write!(f, "Number"),
            TokenType::Array => write!(f, "Array"),
            TokenType::Range => write!(f, "Range"),
            TokenType::Optional => write!(f, "Optional"),
            TokenType::Class => write!(f, "Class"),
            TokenType::Component => write!(f, "Component"),
            TokenType::OpenChevrons => write!(f, "OpenChevrons"),
            TokenType::CloseChevrons => write!(f, "CloseChevrons"),
            TokenType::Assign => write!(f, "Assign"),
            TokenType::FieldAssign => write!(f, "FieldAssign"),
            TokenType::Keyword => write!(f, "Keyword"),
            TokenType::Primitive => write!(f, "Primitive"),
            TokenType::Identifier => write!(f, "Identifier"),
            TokenType::Import => write!(f, "Import"),
            TokenType::As => write!(f, "As"),
            TokenType::Whitespace => write!(f, "Whitespace"),
            TokenType::Comment => write!(f, "Comment"),
            TokenType::Unknown => write!(f, "Unknown"),
            TokenType::EndOfFile => write!(f, "EndOfFile"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Str(String),
    Bool(bool),
    None,
}

impl std::fmt::Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Str(s) => write!(f, "{}", s),
            TokenValue::Bool(b) => write!(f, "{}", b),
            TokenValue::None => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: TokenValue,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexerMode {
    Parsing,
    Highlighting,
}

pub struct Lexer {
    pub mode: LexerMode,
    pub size: usize,
    pub source: String,
    pub cursor: usize,
    token_patterns: Vec<TokenPattern>,
}

enum TokenPatternType {
    Simple(TokenType),
    #[allow(clippy::type_complexity)]
    Complex(Box<dyn Fn(&str) -> (TokenType, TokenValue)>),
}

struct TokenPattern {
    regex: Regex,
    handler: TokenPatternType,
}

impl Lexer {
    pub fn new(mode: Option<LexerMode>) -> Self {
        let keywords = settings::get_keywords();
        let primitives = settings::get_primitives();

        let number = r"-?\d*\.?\d+";
        let dots = r"\.\.";

        // Pre-compute formatted patterns
        let class_pat = r"^\([a-zA-Z]+\)".to_string();
        let range_single = format!(r"^\({n}\)", n = number);
        let range_lower = format!(r"^\({n}{d}\)", n = number, d = dots);
        let range_upper = format!(r"^\({d}{n}\)", n = number, d = dots);
        let range_both = format!(r"^\({n}{d}{n}\)", n = number, d = dots);
        let arr_single = format!(r"^\[{n}\]", n = number);
        let arr_lower = format!(r"^\[{n}{d}\]", n = number, d = dots);
        let arr_upper = format!(r"^\[{d}{n}\]", n = number, d = dots);
        let arr_both = format!(r"^\[{n}{d}{n}\]", n = number, d = dots);

        let patterns: Vec<(&str, TokenPatternType)> = vec![
            // Simple patterns
            (r"^\s+", TokenPatternType::Simple(TokenType::Whitespace)),
            (r"^=", TokenPatternType::Simple(TokenType::Assign)),
            (r"^:", TokenPatternType::Simple(TokenType::FieldAssign)),
            (r"^\{", TokenPatternType::Simple(TokenType::OpenBraces)),
            (r"^\}", TokenPatternType::Simple(TokenType::CloseBraces)),
            (r"^<", TokenPatternType::Simple(TokenType::OpenChevrons)),
            (r"^>", TokenPatternType::Simple(TokenType::CloseChevrons)),
            (r"^,", TokenPatternType::Simple(TokenType::Comma)),
            (r"^\.\.", TokenPatternType::Simple(TokenType::Merge)),
            // Comments
            (
                r"^--\[=*\[[\s\S]*?\]=*\]",
                TokenPatternType::Simple(TokenType::Comment),
            ),
            (
                r"^--\[\[[\s\S]*",
                TokenPatternType::Simple(TokenType::Comment),
            ),
            (r"^--[^\n]*\n", TokenPatternType::Simple(TokenType::Comment)),
            (r"^--[^\n]*", TokenPatternType::Simple(TokenType::Comment)),
            // Attribute patterns
            (r"^\?", TokenPatternType::Simple(TokenType::Optional)),
            // Class pattern: (ClassName)
            (&class_pat, TokenPatternType::Simple(TokenType::Class)),
            // Array patterns: []
            (r"^\[\]", TokenPatternType::Simple(TokenType::Array)),
            // Range patterns: (number..number)
            (&range_single, TokenPatternType::Simple(TokenType::Range)),
            (&range_lower, TokenPatternType::Simple(TokenType::Range)),
            (&range_upper, TokenPatternType::Simple(TokenType::Range)),
            (&range_both, TokenPatternType::Simple(TokenType::Range)),
            // Array with size: [number] or [number..number]
            (&arr_single, TokenPatternType::Simple(TokenType::Array)),
            (&arr_lower, TokenPatternType::Simple(TokenType::Array)),
            (&arr_upper, TokenPatternType::Simple(TokenType::Array)),
            (&arr_both, TokenPatternType::Simple(TokenType::Array)),
            // Parentheses and brackets (after attribute patterns)
            (r"^\(", TokenPatternType::Simple(TokenType::OpenParentheses)),
            (
                r"^\)",
                TokenPatternType::Simple(TokenType::CloseParentheses),
            ),
            (r"^\[", TokenPatternType::Simple(TokenType::OpenBrackets)),
            (r"^\]", TokenPatternType::Simple(TokenType::CloseBrackets)),
            // Empty string
            (
                r#"^""""#,
                TokenPatternType::Complex(Box::new(|_text: &str| {
                    (TokenType::String, TokenValue::Str(String::new()))
                })),
            ),
            // Quoted strings
            (
                r#"^"([^"\\]|\\.)*""#,
                TokenPatternType::Complex(Box::new(|text: &str| {
                    let inner = &text[1..text.len() - 1];
                    (TokenType::String, TokenValue::Str(inner.to_string()))
                })),
            ),
            (
                r#"^'([^'\\]|\\.)*'"#,
                TokenPatternType::Complex(Box::new(|text: &str| {
                    let inner = &text[1..text.len() - 1];
                    (TokenType::String, TokenValue::Str(inner.to_string()))
                })),
            ),
            // Unterminated strings
            (
                r#"^"[^"]*$"#,
                TokenPatternType::Complex(Box::new(|text: &str| {
                    let inner = &text[1..];
                    (TokenType::String, TokenValue::Str(inner.to_string()))
                })),
            ),
            (
                r"^'[^']*$",
                TokenPatternType::Complex(Box::new(|text: &str| {
                    let inner = &text[1..];
                    (TokenType::String, TokenValue::Str(inner.to_string()))
                })),
            ),
            // Dotted identifiers: word.word.word
            (
                r"^[a-zA-Z_]\w*(?:\.[a-zA-Z_]\w*)+",
                TokenPatternType::Simple(TokenType::Identifier),
            ),
        ];

        let mut token_patterns: Vec<TokenPattern> = patterns
            .into_iter()
            .map(|(pattern, handler)| TokenPattern {
                regex: Regex::new(pattern).unwrap(),
                handler,
            })
            .collect();

        // Word/identifier pattern with complex handler - needs cloned maps
        let kw = keywords.clone();
        let pr = primitives.clone();
        token_patterns.push(TokenPattern {
            regex: Regex::new(r"^[a-zA-Z_]\w*").unwrap(),
            handler: TokenPatternType::Complex(Box::new(move |text: &str| {
                if text == "import" {
                    return (TokenType::Import, TokenValue::Str(text.to_string()));
                } else if text == "as" {
                    return (TokenType::As, TokenValue::Str(text.to_string()));
                } else if kw.contains_key(text) {
                    return (TokenType::Keyword, TokenValue::Str(text.to_string()));
                } else if pr.contains_key(text) {
                    return (TokenType::Primitive, TokenValue::Str(text.to_string()));
                } else if text == "true" {
                    return (TokenType::Boolean, TokenValue::Bool(true));
                } else if text == "false" {
                    return (TokenType::Boolean, TokenValue::Bool(false));
                }
                (TokenType::Identifier, TokenValue::Str(text.to_string()))
            })),
        });

        Lexer {
            mode: mode.unwrap_or(LexerMode::Parsing),
            size: 0,
            source: String::new(),
            cursor: 0,
            token_patterns,
        }
    }

    pub fn initialize(&mut self, source: &str) {
        self.size = source.len();
        self.source = source.to_string();
        self.cursor = 0;
    }

    pub fn get_next_token(&mut self, dont_advance_cursor: bool, start_at: Option<usize>) -> Token {
        if self.cursor >= self.size {
            return Token {
                token_type: TokenType::EndOfFile,
                value: TokenValue::Str(String::new()),
                start: self.source.len(),
                end: self.source.len(),
            };
        }

        let position = start_at.unwrap_or(self.cursor);
        let is_highlighting = self.mode == LexerMode::Highlighting;

        let remaining = &self.source[position..];

        for pattern in &self.token_patterns {
            if let Some(mat) = pattern.regex.find(remaining) {
                let text = mat.as_str();
                let start = position;
                let end = (position + text.len()).min(self.size);

                let is_skipped =
                    |tt: &TokenType| matches!(tt, TokenType::Comment | TokenType::Whitespace);

                let (token_type, value) = match &pattern.handler {
                    TokenPatternType::Simple(tt) => {
                        let tt_clone = tt.clone();
                        (tt_clone, TokenValue::Str(text.to_string()))
                    }
                    TokenPatternType::Complex(f) => f(text),
                };

                if !dont_advance_cursor || (is_skipped(&token_type) && !is_highlighting) {
                    self.cursor = position + text.len();
                }

                if is_skipped(&token_type) && !is_highlighting {
                    return self.get_next_token(dont_advance_cursor, None);
                }

                let final_value = if is_highlighting {
                    TokenValue::Str(text.to_string())
                } else {
                    value
                };

                return Token {
                    token_type,
                    value: final_value,
                    start,
                    end,
                };
            }
        }

        if !is_highlighting {
            BlinkError::new(
                ErrorCode::LexerUnexpectedToken,
                &self.source,
                "Unexpected token",
                None,
            )
            .primary(
                Span {
                    start: self.cursor,
                    end: self.cursor,
                },
                "Unexpected token",
            )
            .emit();
        }

        // Attempt to recover
        let symbol = if position < self.source.len() {
            self.source[position..position + 1].to_string()
        } else {
            String::new()
        };

        if !dont_advance_cursor {
            self.cursor += 1;
        }

        Token {
            token_type: TokenType::Unknown,
            value: TokenValue::Str(symbol),
            start: position,
            end: position,
        }
    }
}
