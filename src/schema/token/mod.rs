use std::sync::Arc;

use kind::TokenKind;
use nom_locate::LocatedSpan;

use crate::error::ParserError;

pub mod kind;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub value: Arc<str>,
    pub line: Option<u32>,
    pub column: Option<usize>,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self {
            kind,
            value: Arc::from(*span.fragment()),
            line: Some(span.location_line()),
            column: Some(span.get_utf8_column()),
        }
    }

    pub fn incomplete(value: impl Into<String>) -> Self {
        Self {
            kind: TokenKind::Eof,
            value: Arc::from(value.into()),
            line: None,
            column: None,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(
                f,
                "{:?}({:?}) at {}:{}",
                self.kind, self.value, line, column
            )
        } else {
            write!(f, "{:?}({:?})", self.kind, self.value)
        }
    }
}

impl From<ParserError> for Token {
    fn from(value: ParserError) -> Self {
        Self {
            kind: TokenKind::Error,
            value: Arc::from(value.message),
            line: value.line,
            column: value.column,
        }
    }
}
