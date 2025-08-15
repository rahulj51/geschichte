use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeschichteError {
    #[error("Not a git repository: {path}")]
    NotGitRepository { path: PathBuf },

    #[error("File not found in repository: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Git command failed: {command}\n{output}")]
    GitCommandFailed { command: String, output: String },

    #[error("Failed to parse git output: {reason}")]
    ParseError { reason: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    TerminalError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Cache error: {0}")]
    CacheError(String),
}

pub type Result<T> = std::result::Result<T, GeschichteError>;