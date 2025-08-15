use crate::commit::Commit;
use crate::error::{GeschichteError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Fetches the commit history for a file with rename tracking
pub fn fetch_commit_history(
    repo_root: &Path,
    file_path: &Path,
    follow_renames: bool,
    first_parent: bool,
) -> Result<Vec<Commit>> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root)
        .arg("log");
    
    if follow_renames {
        cmd.arg("--follow");
    }
    
    if first_parent {
        cmd.arg("--first-parent");
    }
    
    cmd.arg("--format=%H%x00%h%x00%ad%x00%an%x00%s")
        .arg("--date=format:%Y-%m-%d")
        .arg("--")
        .arg(file_path);
    
    let output = cmd.output().map_err(|e| GeschichteError::GitCommandFailed {
        command: format!("git log --follow {}", file_path.display()),
        output: e.to_string(),
    })?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GeschichteError::GitCommandFailed {
            command: format!("git log --follow {}", file_path.display()),
            output: stderr.to_string(),
        });
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    
    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split('\0').collect();
        if parts.len() != 5 {
            continue;
        }
        
        commits.push(Commit::new(
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
            parts[3].to_string(),
            parts[4].to_string(),
        ));
    }
    
    Ok(commits)
}

/// Builds a map of commit hashes to file paths for rename tracking
pub fn build_rename_map(
    repo_root: &Path,
    file_path: &Path,
) -> Result<HashMap<String, PathBuf>> {
    let mut rename_map = HashMap::new();
    
    let output = Command::new("git")
        .current_dir(repo_root)
        .arg("log")
        .arg("--follow")
        .arg("--name-status")
        .arg("--format=%H")
        .arg("--")
        .arg(file_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git log --follow --name-status {}", file_path.display()),
            output: e.to_string(),
        })?;
    
    if !output.status.success() {
        return Ok(rename_map); // Return empty map on failure
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_hash = String::new();
    let mut current_path = file_path.to_path_buf();
    
    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        
        // Check if this is a commit hash (40 chars)
        if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
            current_hash = line.to_string();
            rename_map.insert(current_hash.clone(), current_path.clone());
        } else if line.starts_with('R') {
            // Parse rename: R100	old_path	new_path
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                let old_path = PathBuf::from(parts[1]);
                let new_path = PathBuf::from(parts[2]);
                current_path = old_path; // Track the old name for previous commits
                
                // Update the current commit's path
                if !current_hash.is_empty() {
                    rename_map.insert(current_hash.clone(), new_path);
                }
            }
        } else if line.starts_with('A') || line.starts_with('M') || line.starts_with('D') {
            // Parse regular status: A	path or M	path or D	path
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 2 {
                current_path = PathBuf::from(parts[1]);
                if !current_hash.is_empty() {
                    rename_map.insert(current_hash.clone(), current_path.clone());
                }
            }
        }
    }
    
    Ok(rename_map)
}

/// Gets the parent commits for a given commit
pub fn get_commit_parents(repo_root: &Path, commit_hash: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .arg("rev-list")
        .arg("--parents")
        .arg("-n1")
        .arg(commit_hash)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git rev-list --parents -n1 {}", commit_hash),
            output: e.to_string(),
        })?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
    
    // First part is the commit itself, rest are parents
    if parts.len() > 1 {
        Ok(parts[1..].iter().map(|s| s.to_string()).collect())
    } else {
        Ok(vec![])
    }
}