use std::fmt;

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    LexerUnexpectedToken = 1001,

    ParserUnexpectedEndOfFile = 2001,
    ParserUnexpectedToken = 2002,
    ParserUnknownOption = 2003,
    ParserUnexpectedSymbol = 2004,
    ParserExpectedExtraToken = 2005,

    AnalyzeReferenceInvalidType = 3001,
    AnalyzeInvalidOptionalType = 3002,
    AnalyzeNestedMap = 3003,
    AnalyzeDuplicateField = 3004,
    AnalyzeReservedIdentifier = 3005,
    AnalyzeNestedScope = 3006,
    AnalyzeUnknownReference = 3007,
    AnalyzeInvalidRangeType = 3008,
    AnalyzeInvalidRange = 3009,
    AnalyzeDuplicateDeclaration = 3010,
    AnalyzeDuplicateTypeGeneric = 3011,
    AnalyzeInvalidGenerics = 3012,
    AnalyzeInvalidExport = 3013,
    AnalyzeUnknownImport = 3014,
    AnalyzeErrorWhileImporting = 3015,
    AnalyzeOptionAfterStart = 3016,
}

impl ErrorCode {
    pub fn code(&self) -> u32 {
        *self as u32
    }
}

#[derive(Debug, Clone)]
pub struct BlinkError {
    pub labels: Vec<Label>,
    pub source: Vec<String>,
    pub message: String,
    pub code: ErrorCode,
    pub file: String,
    pub raw_message: String,
    pub primary_span: Option<Span>,
}

impl BlinkError {
    pub fn new(code: ErrorCode, source: &str, message: &str, file: Option<&str>) -> Self {
        let lines: Vec<String> = source.split('\n').map(|s| s.to_string()).collect();
        let file = file.unwrap_or("input.blink").to_string();
        let content = format!(
            "[E{:04}] Error: {}\n    ╭─[{}:1:{}]\n    │",
            code.code(),
            message,
            file,
            lines.len()
        );

        BlinkError {
            labels: Vec::new(),
            source: lines,
            message: content,
            code,
            file,
            raw_message: message.to_string(),
            primary_span: None,
        }
    }

    pub fn slice(&self, span: &Span) -> Vec<Slice> {
        let mut slices = Vec::new();
        let mut cursor: usize = 0;

        for (line_num, text) in self.source.iter().enumerate() {
            let start = cursor;
            let length = text.len();
            cursor += length + 1; // +1 for newline

            if span.start > cursor {
                continue;
            }

            let spaces = span.start.saturating_sub(start);
            let underlines = (span.end - span.start).max(1).min(length);

            slices.push(Slice {
                line: line_num + 1,
                text: text.clone(),
                spaces,
                underlines,
            });

            if span.end <= cursor {
                break;
            }
        }

        slices
    }

    pub fn label(mut self, span: Span, text: &str, _color: &str) -> Self {
        self.labels.push(Label {
            span: span.clone(),
            text: text.to_string(),
        });

        let slices = self.slice(&span);
        for (i, slice) in slices.iter().enumerate() {
            self.message
                .push_str(&format!("\n{:03} │ {}", slice.line, slice.text));
            if i == slices.len() - 1 {
                let length = slice.underlines / 2;
                let indent = format!("    ┆ {}", " ".repeat(slice.spaces));
                let underlines = "─".repeat(length);
                let extra_indent = " ".repeat(length);

                self.message
                    .push_str(&format!("\n{}{}┬{}", indent, underlines, underlines));
                self.message
                    .push_str(&format!("\n{}{}│", indent, extra_indent));
                self.message
                    .push_str(&format!("\n{}{}╰── {}", indent, extra_indent, text));
            }
        }

        self
    }

    pub fn primary(mut self, span: Span, text: &str) -> Self {
        self.primary_span = Some(span.clone());
        self.label(span, text, "red")
    }

    pub fn secondary(self, span: Span, text: &str) -> Self {
        self.label(span, text, "blue")
    }

    pub fn emit(mut self) -> ! {
        self.message.push_str("\n    │");
        self.message.push_str("\n────╯\n");
        panic!("{}", self.message)
    }
}

impl fmt::Display for BlinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Clone)]
pub struct Slice {
    pub line: usize,
    pub text: String,
    pub spaces: usize,
    pub underlines: usize,
}
