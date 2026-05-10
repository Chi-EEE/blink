use std::collections::HashMap;
use std::fs;

use crate::error::{BlinkError, ErrorCode, Span};
use crate::lexer::{Lexer, LexerMode, Token, TokenType, TokenValue};
use crate::settings::{self, NumberRange, Primitive};

// ---- AST Types ----

#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationType {
    Map,
    Set,
    Enum,
    TagEnum,
    Tuple,
    Struct,
    Primitive,
    Generic,
    Array,
    Event,
    Function,
    Scope,
    Optional,
}

impl std::fmt::Display for DeclarationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeclarationType::Map => write!(f, "Map"),
            DeclarationType::Set => write!(f, "Set"),
            DeclarationType::Enum => write!(f, "Enum"),
            DeclarationType::TagEnum => write!(f, "TagEnum"),
            DeclarationType::Tuple => write!(f, "Tuple"),
            DeclarationType::Struct => write!(f, "Struct"),
            DeclarationType::Primitive => write!(f, "Primitive"),
            DeclarationType::Generic => write!(f, "Generic"),
            DeclarationType::Array => write!(f, "Array"),
            DeclarationType::Event => write!(f, "Event"),
            DeclarationType::Function => write!(f, "Function"),
            DeclarationType::Scope => write!(f, "Scope"),
            DeclarationType::Optional => write!(f, "Optional"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Generics {
    pub span: Span,
    pub list: Vec<String>,
    pub keys: HashMap<String, Vec<usize>>, // indices into the AST node list where generics are used
}

#[derive(Debug, Clone)]
pub struct Options {
    pub casing: Option<String>,
    pub use_colon: Option<bool>,
    pub use_polling: Option<bool>,
    pub typescript: Option<bool>,
    pub types_output: Option<String>,
    pub client_output: Option<String>,
    pub server_output: Option<String>,
    pub future_library: Option<String>,
    pub promise_library: Option<String>,
    pub sync_validation: Option<bool>,
    pub write_validations: Option<bool>,
    pub manual_replication: Option<bool>,
    pub remote_scope: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

impl Options {
    pub fn new() -> Self {
        Options {
            casing: None,
            use_colon: None,
            use_polling: None,
            typescript: None,
            types_output: None,
            client_output: None,
            server_output: None,
            future_library: None,
            promise_library: None,
            sync_validation: None,
            write_validations: None,
            manual_replication: None,
            remote_scope: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeNode {
    pub node_type: DeclarationType,
    pub name: String,
    pub value: TypeNodeValue,
}

#[derive(Debug, Clone)]
pub enum TypeNodeValue {
    Primitive {
        class: Option<String>,
        primitive: Primitive,
        primitive_token: Token,
        range: Option<NumberRange>,
        range_token: Option<Token>,
        scope: Option<usize>, // scope id
        export: Option<bool>,
        generics: Option<Generics>,
        parameters: Option<Vec<TypeNode>>,
        components: Option<Vec<String>>,
    },
    Array {
        of: Box<TypeNode>,
        range: Option<NumberRange>,
        range_token: Option<Token>,
        scope: Option<usize>,
        export: Option<bool>,
        generics: Option<Generics>,
    },
    Optional {
        of: Box<TypeNode>,
        scope: Option<usize>,
        export: Option<bool>,
    },
    Set {
        values: Vec<String>,
        scope: Option<usize>,
        export: Option<bool>,
        generics: Option<Generics>,
    },
    Map {
        key: Box<TypeNode>,
        value: Box<TypeNode>,
        scope: Option<usize>,
        export: Option<bool>,
        generics: Option<Generics>,
    },
    Enum {
        values: Vec<String>,
        scope: Option<usize>,
        export: Option<bool>,
        generics: Option<Generics>,
    },
    TagEnum {
        tag: String,
        values: Vec<TypeNode>,
        scope: Option<usize>,
        export: Option<bool>,
        generics: Option<Generics>,
    },
    Struct {
        values: Vec<TypeNode>,
        scope: Option<usize>,
        export: Option<bool>,
        generics: Option<Generics>,
    },
    Tuple {
        values: Vec<TypeNode>,
        scope: Option<usize>,
        export: Option<bool>,
    },
    Generic {
        generic: String,
        scope: Option<usize>,
        export: Option<bool>,
    },
    Event {
        from: String,
        event_type: String,
        call: String,
        data: Option<Box<TypeNode>>,
    },
    Function {
        yield_type: String,
        data: Option<Box<TypeNode>>,
        return_type: Option<Box<TypeNode>>,
    },
    Scope {
        scope_id: usize,
        values: Vec<Declaration>,
    },
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub decl_type: DeclarationType,
    pub name: String,
    pub value: TypeNodeValue,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub name: String,
    pub options: Options,
    pub symbols: HashMap<String, bool>,
    pub declarations: Vec<Declaration>,
}

// ---- Scope ----

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: Token,
    pub parent: Option<usize>,
    pub types: HashMap<String, Reference>,
    pub scopes: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub name: Token,
    pub to: TypeNode,
}

// ---- Event Option ----

#[derive(Debug, Clone)]
struct EventOption {
    key: String,
    token_type: TokenType,
    values: Vec<String>,
    optional: bool,
}

// ---- Keyword ----

struct KeywordInfo {
    decl_type: DeclarationType,
    class: String,
    generics: bool,
    exportable: bool,
}

// ---- Parser ----

pub struct Parser {
    pub source: String,
    pub file: Option<String>,
    pub directory: String,

    lexer: Lexer,
    look_ahead: Option<Token>,

    scopes: Vec<Scope>,
    current_scope: usize,
    symbols: HashMap<String, bool>,
    generics: Option<Generics>,
    primitives: HashMap<String, Primitive>,
}

fn get_options_map() -> HashMap<String, TokenType> {
    let mut m = HashMap::new();
    m.insert("Casing".to_string(), TokenType::Identifier);
    m.insert("UseColon".to_string(), TokenType::Boolean);
    m.insert("UsePolling".to_string(), TokenType::Boolean);
    m.insert("Typescript".to_string(), TokenType::Boolean);
    m.insert("TypesOutput".to_string(), TokenType::String);
    m.insert("ClientOutput".to_string(), TokenType::String);
    m.insert("ServerOutput".to_string(), TokenType::String);
    m.insert("FutureLibrary".to_string(), TokenType::String);
    m.insert("PromiseLibrary".to_string(), TokenType::String);
    m.insert("SyncValidation".to_string(), TokenType::Boolean);
    m.insert("WriteValidations".to_string(), TokenType::Boolean);
    m.insert("ManualReplication".to_string(), TokenType::Boolean);
    m.insert("RemoteScope".to_string(), TokenType::String);
    m
}

fn get_event_structure() -> Vec<EventOption> {
    vec![
        EventOption {
            key: "From".to_string(),
            token_type: TokenType::Identifier,
            values: vec!["Client".to_string(), "Server".to_string()],
            optional: false,
        },
        EventOption {
            key: "Type".to_string(),
            token_type: TokenType::Identifier,
            values: vec!["Reliable".to_string(), "Unreliable".to_string()],
            optional: false,
        },
        EventOption {
            key: "Call".to_string(),
            token_type: TokenType::Identifier,
            values: vec![
                "SingleSync".to_string(),
                "SingleAsync".to_string(),
                "ManySync".to_string(),
                "ManyAsync".to_string(),
                "Polling".to_string(),
            ],
            optional: false,
        },
        EventOption {
            key: "Poll".to_string(),
            token_type: TokenType::Boolean,
            values: vec!["true".to_string(), "false".to_string()],
            optional: true,
        },
        EventOption {
            key: "Data".to_string(),
            token_type: TokenType::Identifier,
            values: vec![],
            optional: true,
        },
    ]
}

fn get_function_structure() -> Vec<EventOption> {
    vec![
        EventOption {
            key: "Yield".to_string(),
            token_type: TokenType::Identifier,
            values: vec![
                "Future".to_string(),
                "Promise".to_string(),
                "Coroutine".to_string(),
            ],
            optional: false,
        },
        EventOption {
            key: "Data".to_string(),
            token_type: TokenType::Identifier,
            values: vec![],
            optional: true,
        },
        EventOption {
            key: "Return".to_string(),
            token_type: TokenType::Identifier,
            values: vec![],
            optional: true,
        },
    ]
}

fn get_keywords_map() -> HashMap<String, KeywordInfo> {
    let mut m = HashMap::new();
    m.insert(
        "map".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Map,
            class: "Type".to_string(),
            generics: true,
            exportable: true,
        },
    );
    m.insert(
        "set".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Set,
            class: "Type".to_string(),
            generics: true,
            exportable: true,
        },
    );
    m.insert(
        "enum".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Enum,
            class: "Type".to_string(),
            generics: true,
            exportable: true,
        },
    );
    m.insert(
        "type".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Primitive,
            class: "Type".to_string(),
            generics: false,
            exportable: true,
        },
    );
    m.insert(
        "struct".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Struct,
            class: "Type".to_string(),
            generics: true,
            exportable: true,
        },
    );
    m.insert(
        "event".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Event,
            class: "Event".to_string(),
            generics: false,
            exportable: false,
        },
    );
    m.insert(
        "function".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Function,
            class: "Function".to_string(),
            generics: false,
            exportable: false,
        },
    );
    m.insert(
        "scope".to_string(),
        KeywordInfo {
            decl_type: DeclarationType::Scope,
            class: "Scope".to_string(),
            generics: false,
            exportable: false,
        },
    );
    m
}

fn get_bucket(class: &str) -> &'static str {
    match class {
        "Type" => "Types",
        "Event" | "Function" | "Scope" => "Scopes",
        _ => "Types",
    }
}

static RESERVED: &[&str] = &["StepReplication"];

fn is_integer(n: f64) -> bool {
    (n as i64 as f64) == n
}

fn is_declaration_type_similar(a: &DeclarationType, b: &DeclarationType) -> bool {
    if a == b {
        return true;
    }
    if (*a == DeclarationType::Enum && *b == DeclarationType::TagEnum)
        || (*a == DeclarationType::TagEnum && *b == DeclarationType::Enum)
    {
        return true;
    }
    false
}

fn get_file_source(path: &str, directory: &str) -> Option<(String, String)> {
    let full_path = if path.starts_with('/') || path.contains(':') {
        path.to_string()
    } else {
        format!("{}{}", directory, path)
    };

    // Add .blink extension if missing
    let full_path = if !full_path.ends_with(".blink") && !full_path.contains('.') {
        format!("{}.blink", full_path)
    } else {
        full_path
    };

    match fs::read_to_string(&full_path) {
        Ok(content) => {
            let dir = std::path::Path::new(&full_path)
                .parent()
                .map(|p| {
                    let mut s = p.to_string_lossy().to_string();
                    if !s.ends_with('/') {
                        s.push('/');
                    }
                    s
                })
                .unwrap_or_else(|| "./".to_string());
            Some((content, dir))
        }
        Err(_) => None,
    }
}

impl Parser {
    pub fn new(directory: Option<&str>, file: Option<&str>) -> Self {
        let primitives = settings::get_primitives();
        let initial_scope = Scope {
            name: Token {
                token_type: TokenType::Identifier,
                value: TokenValue::Str(String::new()),
                start: 0,
                end: 0,
            },
            parent: None,
            types: HashMap::new(),
            scopes: HashMap::new(),
        };

        Parser {
            source: String::new(),
            file: file.map(|s| s.to_string()),
            directory: directory.unwrap_or("./").to_string(),
            lexer: Lexer::new(Some(LexerMode::Parsing)),
            look_ahead: None,
            scopes: vec![initial_scope],
            current_scope: 0,
            symbols: HashMap::new(),
            generics: None,
            primitives,
        }
    }

    // ---- Lexer Functions ----

    fn peek(&self) -> &Token {
        match &self.look_ahead {
            Some(token) => token,
            None => {
                BlinkError::new(
                    ErrorCode::ParserUnexpectedEndOfFile,
                    &self.source,
                    "Unexpected end of file",
                    self.file.as_deref(),
                )
                .primary(
                    Span {
                        start: self.source.len(),
                        end: self.source.len(),
                    },
                    "File ends here",
                )
                .emit();
            }
        }
    }

    fn consume(&mut self, expected_type: TokenType) -> Token {
        let token = match self.look_ahead.take() {
            Some(t) => t,
            None => {
                BlinkError::new(
                    ErrorCode::ParserUnexpectedEndOfFile,
                    &self.source,
                    "Unexpected end of file",
                    self.file.as_deref(),
                )
                .primary(
                    Span {
                        start: self.source.len(),
                        end: self.source.len(),
                    },
                    "File ends here",
                )
                .emit();
            }
        };

        if token.token_type != expected_type {
            BlinkError::new(
                ErrorCode::ParserUnexpectedToken,
                &self.source,
                "Unexpected token",
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: token.start,
                    end: token.end,
                },
                &format!(
                    "Expected \"{}\", found \"{}\"",
                    expected_type, token.token_type
                ),
            )
            .emit();
        }

        self.look_ahead = Some(self.lexer.get_next_token(false, None));
        token
    }

    fn try_consume(&mut self, expected_type: TokenType) -> Option<Token> {
        if let Some(token) = &self.look_ahead {
            if token.token_type == expected_type {
                let token = self.look_ahead.take().unwrap();
                self.look_ahead = Some(self.lexer.get_next_token(false, None));
                return Some(token);
            }
        }
        None
    }

    fn consume_any(&mut self, types: &[TokenType]) -> Token {
        for tt in types {
            if let Some(token) = self.try_consume(tt.clone()) {
                return token;
            }
        }

        let token = self.peek().clone();
        let type_names: Vec<String> = types.iter().map(|t| format!("\"{}\"", t)).collect();
        BlinkError::new(
            ErrorCode::ParserUnexpectedToken,
            &self.source,
            "Unexpected token",
            self.file.as_deref(),
        )
        .primary(
            Span {
                start: token.start,
                end: token.end,
            },
            &format!(
                "Expected one of {}, found \"{}\"",
                type_names.join(", "),
                token.token_type
            ),
        )
        .emit();
    }

    fn consume_text(&mut self, consume_primitives: bool, consume_strings: bool) -> Token {
        let peek_type = self.peek().token_type.clone();
        let token = if consume_strings && peek_type == TokenType::String {
            self.consume(TokenType::String)
        } else if consume_primitives {
            self.consume_any(&[
                TokenType::As,
                TokenType::Import,
                TokenType::Boolean,
                TokenType::Keyword,
                TokenType::Identifier,
                TokenType::Primitive,
            ])
        } else {
            self.consume_any(&[
                TokenType::As,
                TokenType::Import,
                TokenType::Boolean,
                TokenType::Keyword,
                TokenType::Identifier,
            ])
        };

        Token {
            end: token.end,
            start: token.start,
            token_type: TokenType::Identifier,
            value: token.value,
        }
    }

    // ---- Reference Functions ----

    fn reference_type(&self, identifier: &str) -> Option<Reference> {
        let path: Vec<&str> = identifier.split('.').collect();
        let mut scope_id = self.current_scope;
        let mut offset = 0;
        let length = path.len();

        loop {
            let index = path[offset];

            if offset == length - 1 {
                // Looking for type
                if let Some(reference) = self.scopes[scope_id].types.get(index) {
                    return Some(reference.clone());
                }
            } else {
                // Looking for scope
                if let Some(&child_scope_id) = self.scopes[scope_id].scopes.get(index) {
                    offset += 1;
                    scope_id = child_scope_id;
                    continue;
                }
            }

            // Search upward
            if let Some(parent_id) = self.scopes[scope_id].parent {
                scope_id = parent_id;
                offset = 0;
                continue;
            }

            return None;
        }
    }

    fn reference_scope(&self, identifier: &str) -> Option<usize> {
        let path: Vec<&str> = identifier.split('.').collect();
        let mut scope_id = self.current_scope;
        let mut offset = 0;
        let length = path.len();

        loop {
            let index = path[offset];

            if let Some(&child_scope_id) = self.scopes[scope_id].scopes.get(index) {
                offset += 1;
                if offset == length {
                    return Some(child_scope_id);
                }
                scope_id = child_scope_id;
                continue;
            }

            if let Some(parent_id) = self.scopes[scope_id].parent {
                scope_id = parent_id;
                offset = 0;
                continue;
            }

            return None;
        }
    }

    fn set_type_reference(&mut self, identifier: &Token, declaration: TypeNode) {
        let name = token_value_str(&identifier.value);

        if RESERVED.contains(&name.as_str()) {
            BlinkError::new(
                ErrorCode::AnalyzeReservedIdentifier,
                &self.source,
                "Reserved identifier",
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: identifier.start,
                    end: identifier.end,
                },
                &format!(
                    "\"{}\" is reserved and cannot be used as an identifier.",
                    name
                ),
            )
            .emit();
        }

        self.symbols.insert(name.clone(), true);
        self.scopes[self.current_scope].types.insert(
            name.clone(),
            Reference {
                name: identifier.clone(),
                to: declaration,
            },
        );
    }

    fn set_scope_reference(&mut self, name: &str, scope_id: usize) {
        self.symbols.insert(name.to_string(), true);
        self.scopes[self.current_scope]
            .scopes
            .insert(name.to_string(), scope_id);
    }

    // ---- Generics ----

    fn get_generics(&mut self) -> Option<Generics> {
        if self.generics.is_some() {
            return self.generics.clone();
        }

        let open = self.try_consume(TokenType::OpenChevrons)?;

        let mut generics = Generics {
            keys: HashMap::new(),
            list: Vec::new(),
            span: Span {
                start: open.start,
                end: open.end + 1,
            },
        };

        loop {
            if self.try_consume(TokenType::CloseChevrons).is_some() {
                break;
            }

            let token = self.consume_text(false, false);
            let name = token_value_str(&token.value);

            generics.span.end = token.end;
            generics.keys.insert(name.clone(), Vec::new());
            generics.list.push(name);

            if self.peek().token_type != TokenType::CloseChevrons {
                self.consume(TokenType::Comma);
            }
        }

        self.generics = Some(generics.clone());
        Some(generics)
    }

    // ---- Number Range Parsing ----

    fn parse_number_range(
        &self,
        token: &Token,
        bounds: &NumberRange,
        integer: bool,
    ) -> NumberRange {
        let value = token_value_str(&token.value);
        let inner = &value[1..value.len() - 1]; // strip parens or brackets

        let range = self.parse_range_inner(inner, bounds, token);

        // Check integer
        if integer && (!is_integer(range.min) || !is_integer(range.max)) {
            BlinkError::new(
                ErrorCode::AnalyzeInvalidRange,
                &self.source,
                "Expected an integer",
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: token.start,
                    end: token.end,
                },
                "Expected an integer",
            )
            .emit();
        }

        // Check bounds
        if range.min < bounds.min || range.max > bounds.max {
            BlinkError::new(
                ErrorCode::AnalyzeInvalidRange,
                &self.source,
                "Range outside bounds",
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: token.start,
                    end: token.end,
                },
                &format!("Expected a range within ({}..{})", bounds.min, bounds.max),
            )
            .emit();
        }

        range
    }

    fn parse_range_inner(&self, inner: &str, bounds: &NumberRange, token: &Token) -> NumberRange {
        // Single number
        if let Ok(single) = inner.parse::<f64>() {
            return NumberRange::new(single, None);
        }

        // Lower..
        if let Some(num_str) = inner.strip_suffix("..") {
            if let Ok(lower) = num_str.parse::<f64>() {
                return NumberRange::new(lower, Some(bounds.max));
            }
        }

        // ..Upper
        if let Some(num_str) = inner.strip_prefix("..") {
            if let Ok(upper) = num_str.parse::<f64>() {
                return NumberRange::new(bounds.min, Some(upper));
            }
        }

        // Lower..Upper
        if let Some(dot_pos) = inner.find("..") {
            let lower_str = &inner[..dot_pos];
            let upper_str = &inner[dot_pos + 2..];
            if let (Ok(lower), Ok(upper)) = (lower_str.parse::<f64>(), upper_str.parse::<f64>()) {
                return NumberRange::new(lower, Some(upper));
            }
        }

        BlinkError::new(
            ErrorCode::AnalyzeInvalidRange,
            &self.source,
            "Malformed range",
            self.file.as_deref(),
        )
        .primary(
            Span {
                start: token.start,
                end: token.end,
            },
            "Unable to parse range",
        )
        .emit();
    }

    fn parse_range(&mut self, primitive: &Primitive) -> Option<(NumberRange, Token)> {
        let token = self.try_consume(TokenType::Range)?;
        let default_bounds = NumberRange::new(-9007199254740992.0, Some(9007199254740992.0));
        let bounds = primitive.bounds.as_ref().unwrap_or(&default_bounds);
        let integer = primitive.integer.unwrap_or(false);
        let range = self.parse_number_range(
            &token,
            &NumberRange {
                min: bounds.min,
                max: bounds.max,
            },
            integer,
        );
        Some((range, token))
    }

    fn parse_array_bounds(&mut self) -> Option<(NumberRange, Token)> {
        let token = self.try_consume(TokenType::Array)?;
        let value = token_value_str(&token.value);

        // Empty bounds []
        if value == "[]" {
            return Some((NumberRange::new(0.0, Some(65535.0)), token));
        }

        let array_bounds = NumberRange::new(0.0, Some(65535.0));
        let range = self.parse_number_range(&token, &array_bounds, true);
        Some((range, token))
    }

    fn parse_components(&mut self, primitive: &Primitive) -> Option<Vec<String>> {
        let open = self.try_consume(TokenType::OpenChevrons)?;

        if primitive.allowed_components == 0 {
            BlinkError::new(
                ErrorCode::AnalyzeInvalidRange,
                &self.source,
                "Type doesn't accept components",
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: open.start,
                    end: open.end,
                },
                "Remove component here",
            )
            .emit();
        }

        let mut components = Vec::new();
        let mut prim_tokens = Vec::new();

        for _ in 0..primitive.allowed_components {
            prim_tokens.push(self.consume(TokenType::Primitive));
            if self.try_consume(TokenType::CloseChevrons).is_some() {
                break;
            }
            self.consume(TokenType::Comma);
        }

        for token in &prim_tokens {
            let component = token_value_str(&token.value);
            let component_primitive = self.primitives.get(&component);

            match component_primitive {
                None => {
                    BlinkError::new(
                        ErrorCode::AnalyzeInvalidRange,
                        &self.source,
                        "Unknown primitive used as component",
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: token.start,
                            end: token.end,
                        },
                        &format!("\"{}\" is invalid", component),
                    )
                    .emit();
                }
                Some(p) => {
                    if !p.component {
                        BlinkError::new(
                            ErrorCode::AnalyzeInvalidRange,
                            &self.source,
                            "Invalid primitive used as component",
                            self.file.as_deref(),
                        )
                        .primary(
                            Span {
                                start: token.start,
                                end: token.end,
                            },
                            &format!("\"{}\" cannot be used as a component", component),
                        )
                        .emit();
                    }
                }
            }

            components.push(component);
        }

        Some(components)
    }

    fn parse_optional(&mut self) -> Option<Token> {
        self.try_consume(TokenType::Optional)
    }

    // ---- Top-Level Parsing ----

    pub fn parse(&mut self, source: &str, scope_id: Option<usize>) -> Body {
        let scope_id = scope_id.unwrap_or_else(|| {
            let id = self.scopes.len();
            self.scopes.push(Scope {
                name: Token {
                    token_type: TokenType::Identifier,
                    value: TokenValue::Str(String::new()),
                    start: 0,
                    end: 0,
                },
                parent: None,
                types: HashMap::new(),
                scopes: HashMap::new(),
            });
            id
        });

        self.current_scope = scope_id;
        self.source = source.to_string();
        self.symbols.clear();

        self.lexer.initialize(source);
        self.look_ahead = Some(self.lexer.get_next_token(false, None));

        let options = self.parse_options();
        let declarations = self.parse_declarations();

        let mut final_options = options;
        if final_options.sync_validation.is_none() {
            final_options.sync_validation = Some(true);
        }

        Body {
            name: self.scopes[scope_id].name.value.to_string(),
            options: final_options,
            symbols: self.symbols.clone(),
            declarations,
        }
    }

    fn parse_import(&mut self) -> Declaration {
        self.consume(TokenType::Import);

        let path_token = self.consume(TokenType::String);
        let import_path = token_value_str(&path_token.value);

        let (import_source, import_directory) = match get_file_source(&import_path, &self.directory)
        {
            Some(r) => r,
            None => {
                BlinkError::new(
                    ErrorCode::AnalyzeUnknownImport,
                    &self.source,
                    "Unknown require",
                    self.file.as_deref(),
                )
                .primary(
                    Span {
                        start: path_token.start,
                        end: path_token.end,
                    },
                    &format!("Unknown require: \"{}\"", import_path),
                )
                .emit();
            }
        };

        // Determine name
        let (name, name_token) = if self.try_consume(TokenType::As).is_some() {
            let t = self.consume_text(false, false);
            let n = token_value_str(&t.value);
            (n, t)
        } else {
            let filename = std::path::Path::new(&import_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| import_path.clone());
            let t = Token {
                token_type: TokenType::Identifier,
                value: TokenValue::Str(filename.clone()),
                start: path_token.start,
                end: path_token.end,
            };
            (filename, t)
        };

        // Check duplicate
        if self.reference_scope(&name).is_some() {
            BlinkError::new(
                ErrorCode::AnalyzeDuplicateDeclaration,
                &self.source,
                &format!("Duplicate declaration: \"{}\"", name),
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: name_token.start,
                    end: name_token.end,
                },
                "Duplicate declared here",
            )
            .emit();
        }

        // Create import scope
        let scope_id = self.scopes.len();
        self.scopes.push(Scope {
            name: name_token.clone(),
            parent: Some(self.current_scope),
            types: HashMap::new(),
            scopes: HashMap::new(),
        });

        // Parse imported file
        let parent_source = self.source.clone();
        let parent_file = self.file.clone();
        let parent_directory = self.directory.clone();
        let parent_scope = self.current_scope;
        let parent_look_ahead = self.look_ahead.take();
        let parent_symbols = self.symbols.clone();
        let parent_generics = self.generics.take();
        let parent_lexer_source = self.lexer.source.clone();
        let parent_lexer_cursor = self.lexer.cursor;
        let parent_lexer_size = self.lexer.size;

        self.file = Some(import_path.clone());
        self.directory = import_directory;

        let body = self.parse(&import_source, Some(scope_id));

        // Restore state
        self.source = parent_source;
        self.file = parent_file;
        self.directory = parent_directory;
        self.current_scope = parent_scope;
        self.symbols = parent_symbols;
        self.generics = parent_generics;
        self.lexer.source = parent_lexer_source;
        self.lexer.cursor = parent_lexer_cursor;
        self.lexer.size = parent_lexer_size;
        self.look_ahead = parent_look_ahead;

        self.scopes[scope_id].parent = Some(parent_scope);
        self.set_scope_reference(&name, scope_id);

        Declaration {
            decl_type: DeclarationType::Scope,
            name: name.clone(),
            value: TypeNodeValue::Scope {
                scope_id,
                values: body.declarations,
            },
        }
    }

    fn parse_options(&mut self) -> Options {
        let options_map = get_options_map();
        let total_options = options_map.len();
        let mut options = Options::new();

        for _ in 0..total_options {
            let keyword = self.peek();
            let keyword_val = token_value_str(&keyword.value);
            if keyword_val != "option" {
                break;
            }

            self.consume(TokenType::Keyword);
            let key = self.consume(TokenType::Identifier);
            self.consume(TokenType::Assign);
            let name = token_value_str(&key.value);

            let expected_type = match options_map.get(&name) {
                Some(t) => t.clone(),
                None => {
                    let examples: Vec<String> =
                        options_map.keys().map(|k| format!("\"{}\"", k)).collect();
                    BlinkError::new(
                        ErrorCode::ParserUnknownOption,
                        &self.source,
                        &format!("Unknown option \"{}\"", name),
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: key.start,
                            end: key.end,
                        },
                        &format!("Expected one of {}", examples.join(" or ")),
                    )
                    .emit();
                }
            };

            let value_token = self.consume(expected_type);
            let value = &value_token.value;

            match name.as_str() {
                "Casing" => options.casing = Some(token_value_str(value)),
                "UseColon" => options.use_colon = Some(token_value_bool(value)),
                "UsePolling" => options.use_polling = Some(token_value_bool(value)),
                "Typescript" => options.typescript = Some(token_value_bool(value)),
                "TypesOutput" => options.types_output = Some(token_value_str(value)),
                "ClientOutput" => options.client_output = Some(token_value_str(value)),
                "ServerOutput" => options.server_output = Some(token_value_str(value)),
                "FutureLibrary" => options.future_library = Some(token_value_str(value)),
                "PromiseLibrary" => options.promise_library = Some(token_value_str(value)),
                "SyncValidation" => options.sync_validation = Some(token_value_bool(value)),
                "WriteValidations" => options.write_validations = Some(token_value_bool(value)),
                "ManualReplication" => options.manual_replication = Some(token_value_bool(value)),
                "RemoteScope" => options.remote_scope = Some(token_value_str(value)),
                _ => {}
            }
        }

        options
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let keywords_map = get_keywords_map();
        let mut declarations = Vec::new();

        loop {
            if self.try_consume(TokenType::EndOfFile).is_some() {
                break;
            }

            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }

            // Is import?
            let look_ahead = self.peek();
            if look_ahead.token_type == TokenType::Import {
                declarations.push(self.parse_import());
                continue;
            }

            // Is export?
            let mut export_token: Option<Token> = None;
            if look_ahead.token_type == TokenType::Keyword
                && token_value_str(&look_ahead.value) == "export"
            {
                export_token = Some(self.consume(TokenType::Keyword));
            }

            let keyword = self.consume(TokenType::Keyword);
            let keyword_val = token_value_str(&keyword.value);
            let name = self.consume_text(false, false);

            if keyword_val == "option" {
                BlinkError::new(
                    ErrorCode::AnalyzeOptionAfterStart,
                    &self.source,
                    "Option set after start of file",
                    self.file.as_deref(),
                )
                .primary(
                    Span {
                        start: keyword.start,
                        end: keyword.end,
                    },
                    "Move to the start of the file",
                )
                .emit();
            }

            let keyword_settings = match keywords_map.get(&keyword_val) {
                Some(k) => k,
                None => {
                    BlinkError::new(
                        ErrorCode::ParserUnexpectedToken,
                        &self.source,
                        "Unexpected keyword",
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: keyword.start,
                            end: keyword.end,
                        },
                        &format!("Unknown keyword \"{}\"", keyword_val),
                    )
                    .emit();
                }
            };

            let class = &keyword_settings.class;
            let bucket = get_bucket(class);

            // Check duplicate
            let name_str = token_value_str(&name.value);
            if bucket == "Types" {
                if let Some(existing) = self.reference_type(&name_str) {
                    BlinkError::new(
                        ErrorCode::AnalyzeDuplicateDeclaration,
                        &self.source,
                        "Duplicate declaration",
                        self.file.as_deref(),
                    )
                    .secondary(
                        Span {
                            start: existing.name.start,
                            end: existing.name.end,
                        },
                        "Previously declared here",
                    )
                    .primary(
                        Span {
                            start: name.start,
                            end: name.end,
                        },
                        "Duplicate declared here",
                    )
                    .emit();
                }
            } else if bucket == "Scopes" && self.reference_scope(&name_str).is_some() {
                BlinkError::new(
                    ErrorCode::AnalyzeDuplicateDeclaration,
                    &self.source,
                    "Duplicate declaration",
                    self.file.as_deref(),
                )
                .primary(
                    Span {
                        start: name.start,
                        end: name.end,
                    },
                    "Duplicate declared here",
                )
                .emit();
            }

            // Generics
            let mut generics: Option<Generics> = None;
            if keyword_settings.generics {
                generics = self.get_generics();
            }

            // Parse declaration
            let mut declaration = if class == "Type" {
                let node = self.parse_type(&name, Some(&keyword), false);
                Declaration {
                    decl_type: node.node_type.clone(),
                    name: name_str.clone(),
                    value: node.value,
                }
            } else if class == "Event" {
                self.parse_event(&name)
            } else if class == "Function" {
                self.parse_function(&name)
            } else if class == "Scope" {
                self.parse_namespace(&name)
            } else {
                unreachable!("Unknown class: {}", class)
            };

            // Handle export
            if let Some(ref export_tok) = export_token {
                if !keyword_settings.exportable {
                    BlinkError::new(
                        ErrorCode::AnalyzeInvalidExport,
                        &self.source,
                        &format!("Declaration of type \"{}\" is not exportable", keyword_val),
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: export_tok.start,
                            end: export_tok.end,
                        },
                        "Remove export here",
                    )
                    .emit();
                } else if generics.is_some() {
                    BlinkError::new(
                        ErrorCode::AnalyzeInvalidExport,
                        &self.source,
                        "Generic types can't be exported",
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: export_tok.start,
                            end: export_tok.end,
                        },
                        "Remove export here",
                    )
                    .emit();
                }

                set_export(&mut declaration.value, true);
            }

            if let Some(ref gen) = generics {
                if declaration.decl_type == DeclarationType::Enum {
                    BlinkError::new(
                        ErrorCode::AnalyzeInvalidGenerics,
                        &self.source,
                        "Unit enums don't support generics",
                        self.file.as_deref(),
                    )
                    .primary(gen.span.clone(), "Remove generics")
                    .emit();
                }

                self.generics = None;
                set_generics(&mut declaration.value, generics.clone());
            }

            // Set reference
            if class != "Scope" && bucket == "Types" {
                let type_node = TypeNode {
                    node_type: declaration.decl_type.clone(),
                    name: name_str.clone(),
                    value: declaration.value.clone(),
                };
                self.set_type_reference(&name, type_node);
            }

            declarations.push(declaration);
        }

        declarations
    }

    // ---- Structure Parsing ----

    fn get_option_from_token(&self, token: &Token, structure: &[EventOption]) -> EventOption {
        let value = token_value_str(&token.value);
        for option in structure {
            if option.key == value || option.key.to_lowercase() == value.to_lowercase() {
                return option.clone();
            }
        }

        let options_str: Vec<String> = structure.iter().map(|o| format!("\"{}\"", o.key)).collect();
        BlinkError::new(
            ErrorCode::ParserUnknownOption,
            &self.source,
            &format!("Unknown option \"{}\"", value),
            self.file.as_deref(),
        )
        .primary(
            Span {
                start: token.start,
                end: token.end,
            },
            &format!("Expected one of {}", options_str.join(" or ")),
        )
        .emit();
    }

    fn parse_structure(
        &mut self,
        name: &Token,
        structure: &[EventOption],
    ) -> HashMap<String, StructureValue> {
        let mut fields: HashMap<String, StructureValue> = HashMap::new();
        let open = self.consume(TokenType::OpenBraces);
        let close: Option<Token>;

        loop {
            if let Some(c) = self.try_consume(TokenType::CloseBraces) {
                close = Some(c);
                break;
            }

            let field = self.consume_text(false, false);
            let option = self.get_option_from_token(&field, structure);
            self.consume(TokenType::FieldAssign);

            if fields.contains_key(&option.key) {
                BlinkError::new(
                    ErrorCode::AnalyzeDuplicateField,
                    &self.source,
                    &format!("Field \"{}\" was already specified", option.key),
                    self.file.as_deref(),
                )
                .primary(
                    Span {
                        start: field.start,
                        end: field.end,
                    },
                    "Already specified earlier in the structure",
                )
                .emit();
            }

            let value = if option.key != "Data" && option.key != "Return" {
                let tok = self.consume(option.token_type.clone());
                let val_str = token_value_str(&tok.value);

                if !option.values.is_empty() && !option.values.contains(&val_str) {
                    BlinkError::new(
                        ErrorCode::ParserUnknownOption,
                        &self.source,
                        &format!("Unknown option \"{}\"", val_str),
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: tok.start,
                            end: tok.end,
                        },
                        &format!("Expected one of \"{}\"", option.values.join("\" or \"")),
                    )
                    .emit();
                }

                StructureValue::Str(val_str)
            } else {
                let type_node = self.parse_type(name, None, true);
                StructureValue::Type(Box::new(type_node))
            };

            fields.insert(option.key.clone(), value);

            if let Some(c) = self.try_consume(TokenType::CloseBraces) {
                close = Some(c);
                break;
            }

            self.consume(TokenType::Comma);
        }

        // Check required fields
        for option in structure {
            if option.optional {
                continue;
            }
            if fields.contains_key(&option.key) {
                continue;
            }

            let end = close
                .as_ref()
                .map(|c| c.end.saturating_sub(2))
                .unwrap_or(open.start);
            BlinkError::new(
                ErrorCode::ParserExpectedExtraToken,
                &self.source,
                &format!("Field \"{}\" is missing", option.key),
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: open.start,
                    end: end.max(open.start),
                },
                &format!("Add missing field \"{}\"", option.key),
            )
            .emit();
        }

        fields
    }

    fn parse_event(&mut self, name: &Token) -> Declaration {
        let structure = self.parse_structure(name, &get_event_structure());

        let mut call = structure
            .get("Call")
            .map(|v| v.as_str())
            .unwrap_or("SingleSync".to_string());

        if let Some(StructureValue::Str(poll)) = structure.get("Poll") {
            if poll == "true" {
                call = "Polling".to_string();
            }
        }

        let data = structure.get("Data").map(|v| match v {
            StructureValue::Type(t) => t.clone(),
            _ => unreachable!(),
        });

        Declaration {
            decl_type: DeclarationType::Event,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Event {
                from: structure
                    .get("From")
                    .map(|v| v.as_str())
                    .unwrap_or_default(),
                event_type: structure
                    .get("Type")
                    .map(|v| v.as_str())
                    .unwrap_or_default(),
                call,
                data,
            },
        }
    }

    fn parse_function(&mut self, name: &Token) -> Declaration {
        let structure = self.parse_structure(name, &get_function_structure());

        let data = structure.get("Data").map(|v| match v {
            StructureValue::Type(t) => t.clone(),
            _ => unreachable!(),
        });

        let return_type = structure.get("Return").map(|v| match v {
            StructureValue::Type(t) => t.clone(),
            _ => unreachable!(),
        });

        Declaration {
            decl_type: DeclarationType::Function,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Function {
                yield_type: structure
                    .get("Yield")
                    .map(|v| v.as_str())
                    .unwrap_or_default(),
                data,
                return_type,
            },
        }
    }

    fn parse_namespace(&mut self, name: &Token) -> Declaration {
        self.consume(TokenType::OpenBraces);

        let parent_scope = self.current_scope;
        let scope_id = self.scopes.len();
        self.scopes.push(Scope {
            name: name.clone(),
            parent: Some(parent_scope),
            types: HashMap::new(),
            scopes: HashMap::new(),
        });

        let name_str = token_value_str(&name.value);
        self.current_scope = scope_id;
        self.symbols.insert(name_str.clone(), true);
        self.scopes[parent_scope]
            .scopes
            .insert(name_str.clone(), scope_id);

        let sub_declarations = self.parse_declarations();

        self.current_scope = parent_scope;

        Declaration {
            decl_type: DeclarationType::Scope,
            name: name_str,
            value: TypeNodeValue::Scope {
                scope_id,
                values: sub_declarations,
            },
        }
    }

    // ---- Type Parsing ----

    fn parse_type(
        &mut self,
        name: &Token,
        keyword: Option<&Token>,
        is_data_field: bool,
    ) -> TypeNode {
        let mut declaration: Option<TypeNode> = None;

        // Fast path: keyword given
        if let Some(kw) = keyword {
            let kw_val = token_value_str(&kw.value);

            if kw_val != "struct" {
                self.consume(TokenType::Assign);

                // Handle reference declarations
                let keywords_map = get_keywords_map();
                let kw_info = keywords_map.get(&kw_val);
                let peek = self.peek().clone();

                if peek.token_type == TokenType::Identifier {
                    if let Some(info) = kw_info {
                        let decl = self.parse_declaration(name);
                        if !is_declaration_type_similar(&decl.node_type, &info.decl_type) {
                            BlinkError::new(
                                ErrorCode::AnalyzeReferenceInvalidType,
                                &self.source,
                                &format!(
                                    "Cannot cast \"{}\" to \"{}\"",
                                    decl.node_type, info.decl_type
                                ),
                                self.file.as_deref(),
                            )
                            .primary(
                                Span {
                                    start: peek.start,
                                    end: peek.end,
                                },
                                &format!(
                                    "Expected a reference to \"{}\", got \"{}\" instead",
                                    info.decl_type, decl.node_type
                                ),
                            )
                            .emit();
                        }
                        declaration = Some(decl);
                    }
                }
            }

            if declaration.is_none() {
                declaration = Some(match kw_val.as_str() {
                    "map" => self.parse_map(name),
                    "set" => self.parse_set(name),
                    "enum" => self.parse_enum(name),
                    "type" => self.parse_primitive(name),
                    "struct" => self.parse_struct(name),
                    _ => unreachable!("Unknown keyword: {}", kw_val),
                });
            }
        }

        // Slow path: no keyword
        if keyword.is_none() {
            let peek = self.peek().clone();

            match peek.token_type {
                TokenType::Keyword => {
                    let kw = self.consume(TokenType::Keyword);
                    let kw_val = token_value_str(&kw.value);
                    declaration = Some(match kw_val.as_str() {
                        "map" => self.parse_map(name),
                        "set" => self.parse_set(name),
                        "enum" => self.parse_enum(name),
                        "struct" => self.parse_struct(name),
                        _ => {
                            BlinkError::new(
                                ErrorCode::ParserUnexpectedToken,
                                &self.source,
                                "Unexpected token",
                                self.file.as_deref(),
                            )
                            .primary(
                                Span {
                                    start: peek.start,
                                    end: peek.end,
                                },
                                &format!(
                                    "Expected one of \"Identifier\" or \"Primitive\" or \"Keyword\", got \"{}\" instead",
                                    peek.token_type
                                ),
                            )
                            .emit();
                        }
                    });
                }
                TokenType::Primitive => {
                    declaration = Some(self.parse_primitive(name));
                }
                TokenType::Identifier => {
                    declaration = Some(self.parse_declaration(name));
                }
                TokenType::OpenParentheses if is_data_field => {
                    declaration = Some(self.parse_tuple(name));
                }
                _ => {
                    BlinkError::new(
                        ErrorCode::ParserUnexpectedToken,
                        &self.source,
                        "Unexpected token",
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: peek.start,
                            end: peek.end,
                        },
                        &format!(
                            "Expected one of \"Identifier\" or \"Primitive\" or \"Keyword\", got \"{}\" instead",
                            peek.token_type
                        ),
                    )
                    .emit();
                }
            }
        }

        let mut decl = declaration.unwrap();

        // Handle primitive-specific attributes (components, range)
        if decl.node_type == DeclarationType::Primitive {
            if let TypeNodeValue::Primitive {
                ref primitive,
                ref mut components,
                ref mut range,
                ref mut range_token,
                ..
            } = decl.value
            {
                let prim = primitive.clone();
                if let Some(comps) = self.parse_components(&prim) {
                    *components = Some(comps);
                }
                if let Some((r, tok)) = self.parse_range(&prim) {
                    *range = Some(r);
                    *range_token = Some(tok);
                }
            }
        }

        // Optional wrapper
        let decl = self.wrap_optional(decl, name);

        // Array wrappers
        self.wrap_arrays(decl, name)
    }

    fn wrap_optional(&mut self, mut decl: TypeNode, name: &Token) -> TypeNode {
        if self.parse_optional().is_some() {
            decl = TypeNode {
                node_type: DeclarationType::Optional,
                name: token_value_str(&name.value),
                value: TypeNodeValue::Optional {
                    of: Box::new(decl),
                    scope: None,
                    export: None,
                },
            };
        }
        decl
    }

    fn wrap_arrays(&mut self, mut decl: TypeNode, name: &Token) -> TypeNode {
        while let Some((range, range_tok)) = self.parse_array_bounds() {
            decl = TypeNode {
                node_type: DeclarationType::Array,
                name: token_value_str(&name.value),
                value: TypeNodeValue::Array {
                    of: Box::new(decl),
                    range: Some(range),
                    range_token: Some(range_tok),
                    scope: None,
                    export: None,
                    generics: None,
                },
            };

            decl = self.wrap_optional(decl, name);
        }
        decl
    }

    fn parse_set(&mut self, name: &Token) -> TypeNode {
        self.consume(TokenType::OpenBraces);

        let mut values = Vec::new();
        loop {
            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }

            let token = self.consume_text(true, true);
            values.push(token_value_str(&token.value));

            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }
            self.consume(TokenType::Comma);
        }

        TypeNode {
            node_type: DeclarationType::Set,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Set {
                values,
                scope: None,
                export: None,
                generics: None,
            },
        }
    }

    fn parse_map(&mut self, name: &Token) -> TypeNode {
        self.consume(TokenType::OpenBraces);
        self.consume(TokenType::OpenBrackets);

        // Parse key
        let key_peek = self.peek().clone();
        let key = self.parse_type(name, None, false);
        if key.node_type == DeclarationType::Optional {
            BlinkError::new(
                ErrorCode::AnalyzeInvalidOptionalType,
                &self.source,
                "Invalid optional type",
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: key_peek.start,
                    end: key_peek.end,
                },
                "Maps cannot have optionals as keys or values",
            )
            .emit();
        }

        self.consume(TokenType::CloseBrackets);
        self.consume(TokenType::FieldAssign);

        // Parse value
        let val_peek = self.peek().clone();
        let value = self.parse_type(name, None, false);
        if value.node_type == DeclarationType::Optional {
            BlinkError::new(
                ErrorCode::AnalyzeInvalidOptionalType,
                &self.source,
                "Invalid optional type",
                self.file.as_deref(),
            )
            .primary(
                Span {
                    start: val_peek.start,
                    end: val_peek.end,
                },
                "Maps cannot have optionals as keys or values",
            )
            .emit();
        }

        self.consume(TokenType::CloseBraces);

        TypeNode {
            node_type: DeclarationType::Map,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Map {
                key: Box::new(key),
                value: Box::new(value),
                scope: None,
                export: None,
                generics: None,
            },
        }
    }

    fn parse_enum(&mut self, name: &Token) -> TypeNode {
        // Is it a tagged enum?
        if let Some(tag) = self.try_consume(TokenType::String) {
            return self.parse_tag_enum(&tag, name);
        }

        self.consume(TokenType::OpenBraces);

        let mut values = Vec::new();
        loop {
            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }

            let token = self.consume_text(true, true);
            values.push(token_value_str(&token.value));

            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }
            self.consume(TokenType::Comma);
        }

        TypeNode {
            node_type: DeclarationType::Enum,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Enum {
                values,
                scope: None,
                export: None,
                generics: None,
            },
        }
    }

    fn parse_tag_enum(&mut self, tag: &Token, name: &Token) -> TypeNode {
        let tag_value = token_value_str(&tag.value);
        let mut values = Vec::new();
        self.consume(TokenType::OpenBraces);

        loop {
            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }

            let variant_token = if self.peek().token_type == TokenType::OpenBrackets {
                self.consume(TokenType::OpenBrackets);
                let t = self.consume(TokenType::String);
                self.consume(TokenType::CloseBrackets);
                t
            } else {
                self.consume_text(true, false)
            };

            let struct_node = self.parse_struct(&variant_token);

            // Check tag field conflict
            if let TypeNodeValue::Struct { ref values, .. } = struct_node.value {
                for field in values {
                    if field.name == tag_value {
                        BlinkError::new(
                            ErrorCode::AnalyzeReservedIdentifier,
                            &self.source,
                            "Enum tag used as field in variant",
                            self.file.as_deref(),
                        )
                        .secondary(
                            Span {
                                start: variant_token.start,
                                end: variant_token.end,
                            },
                            "Used in variant",
                        )
                        .primary(
                            Span {
                                start: tag.start,
                                end: tag.end,
                            },
                            "Enum tag",
                        )
                        .emit();
                    }
                }
            }

            values.push(struct_node);

            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }
            self.consume(TokenType::Comma);
        }

        TypeNode {
            node_type: DeclarationType::TagEnum,
            name: token_value_str(&name.value),
            value: TypeNodeValue::TagEnum {
                tag: tag_value,
                values,
                scope: None,
                export: None,
                generics: None,
            },
        }
    }

    fn parse_tuple(&mut self, name: &Token) -> TypeNode {
        self.consume(TokenType::OpenParentheses);

        let mut values = Vec::new();
        loop {
            if self.try_consume(TokenType::CloseParentheses).is_some() {
                break;
            }

            let value = self.parse_type(name, None, false);
            values.push(value);

            if self.try_consume(TokenType::CloseParentheses).is_some() {
                break;
            }
            self.consume(TokenType::Comma);
        }

        TypeNode {
            node_type: DeclarationType::Tuple,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Tuple {
                values,
                scope: None,
                export: None,
            },
        }
    }

    fn parse_struct(&mut self, name: &Token) -> TypeNode {
        let mut fields: HashMap<String, bool> = HashMap::new();
        let mut values = Vec::new();
        self.consume(TokenType::OpenBraces);

        loop {
            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }

            // Struct merging
            if self.try_consume(TokenType::Merge).is_some() {
                let text = self.peek().clone();
                let merge_decl = self.parse_declaration(name);

                if merge_decl.node_type != DeclarationType::Struct {
                    BlinkError::new(
                        ErrorCode::AnalyzeReferenceInvalidType,
                        &self.source,
                        &format!(
                            "Expected a struct to merge, got \"{}\" instead",
                            merge_decl.node_type
                        ),
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: text.start,
                            end: text.end,
                        },
                        &format!(
                            "Expected a \"Struct\", got \"{}\" instead",
                            merge_decl.node_type
                        ),
                    )
                    .emit();
                }

                if let TypeNodeValue::Struct {
                    values: struct_values,
                    ..
                } = &merge_decl.value
                {
                    for field in struct_values {
                        if fields.contains_key(&field.name) {
                            BlinkError::new(
                                ErrorCode::AnalyzeDuplicateField,
                                &self.source,
                                "Merged struct contains a duplicate field",
                                self.file.as_deref(),
                            )
                            .primary(
                                Span {
                                    start: text.start,
                                    end: text.end,
                                },
                                &format!("Field \"{}\" already exists in struct", field.name),
                            )
                            .emit();
                        }
                        values.push(field.clone());
                        fields.insert(field.name.clone(), true);
                    }
                }
            } else {
                // Regular field
                let text = if self.peek().token_type == TokenType::OpenBrackets {
                    self.consume(TokenType::OpenBrackets);
                    let t = self.consume(TokenType::String);
                    self.consume(TokenType::CloseBrackets);
                    t
                } else {
                    self.consume_text(true, false)
                };

                let field_name = token_value_str(&text.value);
                if fields.contains_key(&field_name) {
                    BlinkError::new(
                        ErrorCode::AnalyzeDuplicateField,
                        &self.source,
                        &format!("Duplicate field \"{}\"", field_name),
                        self.file.as_deref(),
                    )
                    .primary(
                        Span {
                            start: text.start,
                            end: text.end,
                        },
                        "Duplicate field",
                    )
                    .emit();
                }

                self.consume(TokenType::FieldAssign);
                fields.insert(field_name.clone(), true);
                values.push(self.parse_type(&text, None, false));
            }

            if self.try_consume(TokenType::CloseBraces).is_some() {
                break;
            }
            self.consume(TokenType::Comma);
        }

        TypeNode {
            node_type: DeclarationType::Struct,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Struct {
                values,
                scope: None,
                export: None,
                generics: None,
            },
        }
    }

    fn parse_primitive(&mut self, name: &Token) -> TypeNode {
        let token = self.consume(TokenType::Primitive);
        let value = token_value_str(&token.value);
        let primitive = self.primitives.get(&value).unwrap().clone();

        let class = if value == "Instance" {
            if let Some(class_token) = self.try_consume(TokenType::Class) {
                let class_val = token_value_str(&class_token.value);
                Some(class_val[1..class_val.len() - 1].to_string())
            } else {
                Some("Instance".to_string())
            }
        } else {
            None
        };

        TypeNode {
            node_type: DeclarationType::Primitive,
            name: token_value_str(&name.value),
            value: TypeNodeValue::Primitive {
                class,
                primitive,
                primitive_token: token,
                range: None,
                range_token: None,
                scope: None,
                export: None,
                generics: None,
                parameters: None,
                components: None,
            },
        }
    }

    fn parse_declaration(&mut self, name: &Token) -> TypeNode {
        let token = self.consume_text(false, false);
        let token_name = token_value_str(&token.value);

        // Generics reference resolution
        if let Some(ref generics) = self.generics {
            if generics.keys.contains_key(&token_name) {
                return TypeNode {
                    node_type: DeclarationType::Generic,
                    name: token_value_str(&name.value),
                    value: TypeNodeValue::Generic {
                        generic: token_name,
                        scope: None,
                        export: None,
                    },
                };
            }
        }

        // Normal reference resolution
        let reference = match self.reference_type(&token_name) {
            Some(r) => r,
            None => {
                BlinkError::new(
                    ErrorCode::AnalyzeUnknownReference,
                    &self.source,
                    "Unknown reference",
                    self.file.as_deref(),
                )
                .primary(
                    Span {
                        start: token.start,
                        end: token.end,
                    },
                    "Unknown reference",
                )
                .emit();
            }
        };

        let mut to = reference.to.clone();
        to.name = token_value_str(&name.value);

        // Handle generics replacement
        let has_generics = match &to.value {
            TypeNodeValue::Map { generics, .. }
            | TypeNodeValue::Struct { generics, .. }
            | TypeNodeValue::TagEnum { generics, .. } => generics.is_some(),
            _ => false,
        };

        if has_generics {
            to = self.replace_generics(to);
        }

        to
    }

    fn replace_generics(&mut self, node: TypeNode) -> TypeNode {
        let mut clone = node.clone();
        let generics = match &clone.value {
            TypeNodeValue::Map { generics, .. }
            | TypeNodeValue::Struct { generics, .. }
            | TypeNodeValue::TagEnum { generics, .. } => generics.clone(),
            _ => None,
        };

        let generics = match generics {
            Some(g) => g,
            None => return clone,
        };

        // Remove generics from clone
        set_generics(&mut clone.value, None);

        // Parse generics input
        self.consume(TokenType::OpenChevrons);

        let mut replacements: HashMap<String, TypeNode> = HashMap::new();

        for (i, key) in generics.list.iter().enumerate() {
            let _peek = self.peek().clone();
            let decl = self.parse_type(
                &Token {
                    token_type: TokenType::Identifier,
                    value: TokenValue::Str(key.clone()),
                    start: 0,
                    end: 0,
                },
                None,
                false,
            );
            replacements.insert(key.clone(), decl);

            if i < generics.list.len() - 1 {
                self.consume(TokenType::Comma);
            }
        }

        self.consume(TokenType::CloseChevrons);

        // Replace generics in clone
        fn traverse(node: &mut TypeNode, replacements: &HashMap<String, TypeNode>) {
            match &mut node.value {
                TypeNodeValue::Array { of, .. } | TypeNodeValue::Optional { of, .. } => {
                    traverse(of, replacements);
                }
                TypeNodeValue::Struct { values, .. } | TypeNodeValue::TagEnum { values, .. } => {
                    for v in values.iter_mut() {
                        traverse(v, replacements);
                    }
                }
                TypeNodeValue::Map { key, value, .. } => {
                    traverse(key, replacements);
                    traverse(value, replacements);
                }
                TypeNodeValue::Generic { generic, .. } => {
                    if let Some(replacement) = replacements.get(generic) {
                        let rep = replacement.clone();
                        node.node_type = rep.node_type;
                        node.value = rep.value;
                    }
                }
                _ => {}
            }
        }

        traverse(&mut clone, &replacements);

        clone
    }
}

// ---- Helper Functions ----

#[derive(Debug, Clone)]
pub enum StructureValue {
    Str(String),
    Type(Box<TypeNode>),
}

impl StructureValue {
    pub fn as_str(&self) -> String {
        match self {
            StructureValue::Str(s) => s.clone(),
            StructureValue::Type(_) => String::new(),
        }
    }
}

pub fn token_value_str(value: &TokenValue) -> String {
    match value {
        TokenValue::Str(s) => s.clone(),
        TokenValue::Bool(b) => b.to_string(),
        TokenValue::None => String::new(),
    }
}

pub fn token_value_bool(value: &TokenValue) -> bool {
    match value {
        TokenValue::Bool(b) => *b,
        TokenValue::Str(s) => s == "true",
        TokenValue::None => false,
    }
}

fn set_export(value: &mut TypeNodeValue, export: bool) {
    match value {
        TypeNodeValue::Primitive { export: e, .. }
        | TypeNodeValue::Array { export: e, .. }
        | TypeNodeValue::Optional { export: e, .. }
        | TypeNodeValue::Set { export: e, .. }
        | TypeNodeValue::Map { export: e, .. }
        | TypeNodeValue::Enum { export: e, .. }
        | TypeNodeValue::TagEnum { export: e, .. }
        | TypeNodeValue::Struct { export: e, .. }
        | TypeNodeValue::Tuple { export: e, .. }
        | TypeNodeValue::Generic { export: e, .. } => {
            *e = Some(export);
        }
        _ => {}
    }
}

fn set_generics(value: &mut TypeNodeValue, generics: Option<Generics>) {
    match value {
        TypeNodeValue::Set { generics: g, .. }
        | TypeNodeValue::Map { generics: g, .. }
        | TypeNodeValue::Enum { generics: g, .. }
        | TypeNodeValue::TagEnum { generics: g, .. }
        | TypeNodeValue::Struct { generics: g, .. }
        | TypeNodeValue::Array { generics: g, .. } => {
            *g = generics;
        }
        TypeNodeValue::Primitive { generics: g, .. } => {
            *g = generics;
        }
        _ => {}
    }
}
