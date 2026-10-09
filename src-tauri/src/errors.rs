use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TodaiError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Logger error: {0}")]
    LoggerError(#[from] log::SetLoggerError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Date/time error: {0}")]
    DateTimeError(#[from] jiff::Error),
    #[error("Invalid day {0:?}: expected YYYY-MM-DD")]
    InvalidDay(String),
    #[error("Invalid color {0:?}: expected #RRGGBB")]
    InvalidColor(String),
    #[error("Invalid position {0:?}")]
    InvalidPosition(String),
    #[error("Invalid estimate: {0}")]
    InvalidEstimate(String),
    #[error("Invalid database: {0}")]
    InvalidDatabase(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("No workspace database at {0}")]
    WorkspaceUnavailable(String),
    #[error("No workspace is open")]
    NoActiveWorkspace,
    #[error("Command error: {0}")]
    CommandError(String),
}

/// Sent to the frontend as the error message of a rejected `invoke`.
impl Serialize for TodaiError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        // Commands' errors are only serialized when returned to the frontend: log them here, once.
        log::error!("{self}");
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TodaiError>;
