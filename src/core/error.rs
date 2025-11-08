use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Index corrupted: {0}")]
    IndexCorrupted(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Not initialized: {0}")]
    NotInitialized(String),
}

impl From<r2d2::Error> for SearchError {
    fn from(err: r2d2::Error) -> Self {
        SearchError::Pool(err.to_string())
    }
}

impl From<notify::Error> for SearchError {
    fn from(err: notify::Error) -> Self {
        SearchError::Watch(err.to_string())
    }
}

impl From<globset::Error> for SearchError {
    fn from(err: globset::Error) -> Self {
        SearchError::Parse(err.to_string())
    }
}

impl From<regex::Error> for SearchError {
    fn from(err: regex::Error) -> Self {
        SearchError::Parse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SearchError>;
