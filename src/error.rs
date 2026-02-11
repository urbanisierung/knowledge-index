use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Repository not found: {0}")]
    RepoNotFound(PathBuf),

    #[error("Repository already indexed: {0}")]
    RepoAlreadyIndexed(PathBuf),

    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("Not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File watcher error: {0}")]
    Watcher(#[from] notify::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("No repositories indexed yet")]
    NoRepositories,

    #[error("Search returned no results")]
    NoResults,

    #[error("Terminal too small: {width}x{height} (minimum: {min_width}x{min_height})")]
    TerminalTooSmall {
        width: u16,
        height: u16,
        min_width: u16,
        min_height: u16,
    },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
