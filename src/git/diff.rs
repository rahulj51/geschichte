use crate::error::{GeschichteError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Fetches the diff for a specific commit
pub fn fetch_diff(
    repo_root: &Path,
    commit_hash: &str,
    parent_hash: Option<&str>,
    file_path: &Path,
    context_lines: u32,
) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    
    if let Some(parent) = parent_hash {
        // Normal commit with parent
        cmd.arg("diff")
            .arg(format!("--unified={}", context_lines))
            .arg("--find-renames")
            .arg(parent)
            .arg(commit_hash)
            .arg("--")
            .arg(file_path);
    } else {
        // Root commit (no parent)
        cmd.arg("show")
            .arg("--patch")
            .arg(format!("--unified={}", context_lines))
            .arg(commit_hash)
            .arg("--")
            .arg(file_path);
    }
    
    let output = cmd.output().map_err(|e| GeschichteError::GitCommandFailed {
        command: format!("git diff/show for {}", commit_hash),
        output: e.to_string(),
    })?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // It's okay if the file doesn't exist in one of the commits
        if stderr.contains("does not exist") || stderr.contains("pathspec") {
            return Ok(String::from("File not present in this commit\n"));
        }
        return Err(GeschichteError::GitCommandFailed {
            command: format!("git diff/show for {}", commit_hash),
            output: stderr.to_string(),
        });
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Resolves the path of a file at a specific commit
pub fn resolve_path_at_commit(
    repo_root: &Path,
    commit_hash: &str,
    file_path: &Path,
) -> Result<PathBuf> {
    
    // Try to find the file at this commit
    let output = Command::new("git")
        .current_dir(repo_root)
        .arg("ls-tree")
        .arg("--name-only")
        .arg(commit_hash)
        .arg("--")
        .arg(file_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git ls-tree {} {}", commit_hash, file_path.display()),
            output: e.to_string(),
        })?;
    
    if output.status.success() && !output.stdout.is_empty() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            return Ok(PathBuf::from(path_str));
        }
    }
    
    // File might have been renamed, return original path as fallback
    Ok(file_path.to_path_buf())
}