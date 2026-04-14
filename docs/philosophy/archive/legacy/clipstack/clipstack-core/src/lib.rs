use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;
use rusqlite::Error as SqliteError;

/// Represents a single clipboard entry in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: Uuid,
    pub content: String,
    pub mime_type: String,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub hash: String,
}

impl fmt::Display for ClipboardEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} bytes | {} | {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.content.len(),
            self.mime_type,
            self.device_id
        )
    }
}

/// Errors that can occur in the core module
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("entry not found: {0}")]
    EntryNotFound(String),

    #[error("invalid content: {0}")]
    InvalidContent(String),

    #[error("hash mismatch: expected {0}, got {1}")]
    HashMismatch(String, String),

    #[error("database error: {0}")]
    DatabaseError(String),
}

/// Trait for storing and retrieving clipboard entries
#[async_trait::async_trait]
pub trait Store {
    /// Insert a new clipboard entry
    async fn insert(&self, entry: ClipboardEntry) -> Result<(), CoreError>;

    /// Get the latest clipboard entry
    async fn latest(&self, device_id: &str) -> Result<Option<ClipboardEntry>, CoreError>;

    /// Get clipboard history for a device
    async fn history(&self, device_id: &str, limit: usize) -> Result<Vec<ClipboardEntry>, CoreError>;

    /// Delete an entry by ID
    async fn delete(&self, id: &Uuid) -> Result<(), CoreError>;
}

impl From<SqliteError> for CoreError {
    fn from(error: SqliteError) -> Self {
        CoreError::DatabaseError(error.to_string())
    }
}
