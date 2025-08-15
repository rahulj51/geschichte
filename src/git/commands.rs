use crate::error::{GeschichteError, Result};
use std::path::Path;
use std::process::Command;

/// Executes a git command and returns the output
#[allow(dead_code)]
pub fn run_git_command(
    args: &[&str],
    repo_path: &Path,
) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git {}", args.join(" ")),
            output: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GeschichteError::GitCommandFailed {
            command: format!("git {}", args.join(" ")),
            output: stderr.to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}