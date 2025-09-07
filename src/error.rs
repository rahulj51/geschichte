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
    #[allow(dead_code)]
    ParseError { reason: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    TerminalError(String),

    #[error("UI rendering error: {0}")]
    #[allow(dead_code)] // Available for future use
    UIError(String),

    #[error("State management error: {0}")]
    #[allow(dead_code)] // Available for future use
    StateError(String),

    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    ConfigError(String),

    #[error("Cache error: {0}")]
    #[allow(dead_code)]
    CacheError(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, GeschichteError>;

/// Check if we're running in a CI environment where clipboard operations may not work
pub fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("CONTINUOUS_INTEGRATION").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("JENKINS_URL").is_ok()
        || std::env::var("BUILDKITE").is_ok()
        || std::env::var("CIRCLECI").is_ok()
        || std::env::var("TRAVIS").is_ok()
}
