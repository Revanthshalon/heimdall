#![allow(unused)]

mod traversal;

use uuid::Uuid;

#[derive(Debug)]
pub enum HeimdallError {
    Database(sqlx::Error),
    UuidParse(uuid::Error),
    NamespaceNotFound(String),
    NoUuidForString(String),
    NoStringForUuid(Uuid),
    InvalidRelationTuple(String),
    Traversal(traversal::Error),
    MalformedInput,
}

impl std::fmt::Display for HeimdallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeimdallError::Database(e) => write!(f, "Database error: {}", e),
            HeimdallError::UuidParse(e) => write!(f, "UUID parse error: {}", e),
            HeimdallError::NamespaceNotFound(e) => write!(f, "Namespace `{}` not found", e),
            HeimdallError::NoUuidForString(e) => {
                write!(f, "No UUID mapping found for string: {}", e)
            }
            HeimdallError::NoStringForUuid(e) => {
                write!(f, "No string mapping found for UUID: {}", e)
            }
            HeimdallError::InvalidRelationTuple(e) => write!(f, "Invalid relation tuple: {}", e),
            HeimdallError::Traversal(e) => write!(f, "Traversal Error: {}", e),
            HeimdallError::MalformedInput => write!(f, "Malformed Input"),
        }
    }
}

impl From<sqlx::Error> for HeimdallError {
    fn from(value: sqlx::Error) -> Self {
        HeimdallError::Database(value)
    }
}

impl From<uuid::Error> for HeimdallError {
    fn from(value: uuid::Error) -> Self {
        HeimdallError::UuidParse(value)
    }
}

impl From<traversal::Error> for HeimdallError {
    fn from(value: traversal::Error) -> Self {
        HeimdallError::Traversal(value)
    }
}

pub type HeimdallResult<T> = Result<T, HeimdallError>;
