mod parser;

pub use self::parser::ParserError;

pub type HeimdallResult<T> = Result<T, HeimdallError>;

#[derive(Debug)]
pub enum HeimdallError {
    Parser(ParserError),
}

impl std::fmt::Display for HeimdallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parser(e) => e.fmt(f),
        }
    }
}
