#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    CycleDetected,
    MaxDepthExceeded,
    InvalidSubjectSet(String),
    Mapping(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Database(e) => write!(f, "Database error: {}", e),
            Error::CycleDetected => write!(f, "Cycle detected in traversal"),
            Error::MaxDepthExceeded => write!(f, "Max depth exceeded"),
            Error::InvalidSubjectSet(e) => write!(f, "Invalid subject set: {}", e),
            Error::Mapping(e) => write!(f, "Mapping error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Database(value)
    }
}
