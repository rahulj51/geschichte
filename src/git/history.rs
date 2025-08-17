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
    
    cmd.arg("--format=%H%x00%h%x00%ad%x00%an%x00%ae%x00%cn%x00%ce%x00%cd%x00%s%x00%B")
        .arg("--date=format:%Y-%m-%d %H:%M:%S")
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
        if parts.len() >= 10 {
            // New enhanced format: hash, short_hash, author_date, author_name, author_email, 
            // committer_name, committer_email, committer_date, subject, body
            commits.push(Commit::new_enhanced(
                parts[0].to_string(), // hash
                parts[1].to_string(), // short_hash
                parts[3].to_string(), // author_name
                parts[4].to_string(), // author_email
                parts[2].to_string(), // author_date
                parts[5].to_string(), // committer_name
                parts[6].to_string(), // committer_email
                parts[7].to_string(), // committer_date
                parts[8].to_string(), // subject
                parts[9].to_string(), // body
            ));
        } else if parts.len() >= 5 {
            // Fallback to old format for compatibility
            commits.push(Commit::new(
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
                parts[4].to_string(),
            ));
        }
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

/// Fetches additional metadata for a commit (refs, stats, etc.)
pub fn fetch_commit_refs(repo_root: &Path, commit_hash: &str) -> Result<Vec<String>> {
    let mut refs = Vec::new();
    
    // Get branches containing this commit
    if let Ok(output) = Command::new("git")
        .args(["branch", "--contains", commit_hash])
        .current_dir(repo_root)
        .output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let branch = line.trim().trim_start_matches("* ").trim();
                if !branch.is_empty() && !branch.starts_with("(HEAD detached") {
                    refs.push(format!("branch:{}", branch));
                }
            }
        }
    }
    
    // Get tags at this commit
    if let Ok(output) = Command::new("git")
        .args(["tag", "--points-at", commit_hash])
        .current_dir(repo_root)
        .output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let tag = line.trim();
                if !tag.is_empty() {
                    refs.push(format!("tag:{}", tag));
                }
            }
        }
    }
    
    Ok(refs)
}

/// Fetches commit statistics (files changed, insertions, deletions)
pub fn fetch_commit_stats(repo_root: &Path, commit_hash: &str) -> Result<Option<crate::commit::CommitStats>> {
    let output = Command::new("git")
        .args(["show", "--stat", "--format=", commit_hash])
        .current_dir(repo_root)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git show --stat {}", commit_hash),
            output: e.to_string(),
        })?;
    
    if !output.status.success() {
        return Ok(None);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    // Look for the summary line like " 3 files changed, 45 insertions(+), 12 deletions(-)"
    for line in lines.iter().rev() {
        if line.contains("file") && (line.contains("insertion") || line.contains("deletion")) {
            return Ok(parse_stat_line(line));
        }
    }
    
    Ok(None)
}

fn parse_stat_line(line: &str) -> Option<crate::commit::CommitStats> {
    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;
    
    // Parse patterns like "3 files changed, 45 insertions(+), 12 deletions(-)"
    for part in line.split(',') {
        let part = part.trim();
        if part.contains("file") {
            if let Some(num_str) = part.split_whitespace().next() {
                files_changed = num_str.parse().unwrap_or(0);
            }
        } else if part.contains("insertion") {
            if let Some(num_str) = part.split_whitespace().next() {
                insertions = num_str.parse().unwrap_or(0);
            }
        } else if part.contains("deletion") {
            if let Some(num_str) = part.split_whitespace().next() {
                deletions = num_str.parse().unwrap_or(0);
            }
        }
    }
    
    Some(crate::commit::CommitStats {
        files_changed,
        insertions,
        deletions,
    })
}

/// Detects PR information from commit message
pub fn detect_pr_info(commit: &crate::commit::Commit) -> Option<crate::commit::PullRequestInfo> {
    // Method 1: Check for merge commit patterns first (more specific)
    if commit.subject.starts_with("Merge pull request #") {
        if let Some(pr_num) = extract_pr_number(&commit.subject) {
            return Some(crate::commit::PullRequestInfo {
                number: pr_num,
                title: commit.subject.clone(),
                url: build_pr_url(pr_num),
                status: crate::commit::PRStatus::Merged,
            });
        }
    }
    
    // Method 2: Parse commit message for other PR patterns
    if let Some(pr_num) = extract_pr_number(&commit.subject) {
        return Some(crate::commit::PullRequestInfo {
            number: pr_num,
            title: extract_pr_title(&commit.subject),
            url: build_pr_url(pr_num),
            status: crate::commit::PRStatus::Unknown,
        });
    }
    
    None
}

fn extract_pr_number(message: &str) -> Option<u32> {
    // Regex patterns for common PR formats: "(#123)", "#123", "PR-123", etc.
    if let Some(start) = message.find('#') {
        let number_part = &message[start + 1..];
        if let Some(end) = number_part.find(|c: char| !c.is_ascii_digit()) {
            number_part[..end].parse().ok()
        } else {
            number_part.parse().ok()
        }
    } else {
        None
    }
}

fn extract_pr_title(message: &str) -> String {
    // Try to extract meaningful title from PR message
    if message.starts_with("Merge pull request #") {
        if let Some(from_pos) = message.find(" from ") {
            let after_from = &message[from_pos + 6..];
            if let Some(newline) = after_from.find('\n') {
                after_from[..newline].trim().to_string()
            } else {
                after_from.trim().to_string()
            }
        } else {
            message.to_string()
        }
    } else {
        message.to_string()
    }
}

fn build_pr_url(pr_number: u32) -> String {
    // This would ideally detect the remote origin and build appropriate URL
    // For now, return a placeholder
    format!("https://github.com/repo/pull/{}", pr_number)
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
    let parts: Vec<&str> = stdout.split_whitespace().collect();
    
    // First part is the commit itself, rest are parents
    if parts.len() > 1 {
        Ok(parts[1..].iter().map(|s| s.to_string()).collect())
    } else {
        Ok(vec![])
    }
}