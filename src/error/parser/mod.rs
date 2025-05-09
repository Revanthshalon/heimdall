use crate::schema::token::Span;

#[derive(Debug)]
pub struct ParserError {
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<usize>,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(f, "Parse error at {}:{}: {}", line, column, self.message)
        } else {
            write!(f, "Parse error at {}", self.message)
        }
    }
}

impl nom::error::ContextError<Span<'_>> for ParserError {
    fn add_context(input: Span<'_>, ctx: &'static str, _other: Self) -> Self {
        Self {
            message: ctx.into(),
            line: Some(input.location_line()),
            column: Some(input.get_utf8_column()),
        }
    }
}

impl nom::error::ParseError<Span<'_>> for ParserError {
    fn from_error_kind(input: Span<'_>, _kind: nom::error::ErrorKind) -> Self {
        Self {
            message: (*input.fragment()).into(),
            line: Some(input.location_line()),
            column: Some(input.get_utf8_column()),
        }
    }

    fn append(input: Span<'_>, _kind: nom::error::ErrorKind, _other: Self) -> Self {
        Self {
            message: (*input.fragment()).into(),
            line: Some(input.location_line()),
            column: Some(input.get_utf8_column()),
        }
    }
}
