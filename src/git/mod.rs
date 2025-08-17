pub mod commands;
pub mod files;
pub mod history;
pub mod diff;
pub mod working;

use crate::error::{GeschichteError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Discovers the git repository root from a given path
pub fn discover_repository(start_path: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(start_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: "git rev-parse --show-toplevel".to_string(),
            output: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GeschichteError::NotGitRepository {
                path: start_path.to_path_buf(),
            });
        }
        return Err(GeschichteError::GitCommandFailed {
            command: "git rev-parse --show-toplevel".to_string(),
            output: stderr.to_string(),
        });
    }

    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(path_str))
}

