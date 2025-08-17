use crate::error::{GeschichteError, Result as GeschichteResult};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct GitFile {
    pub path: PathBuf,
    pub display_path: String,
    pub modified: Option<SystemTime>,
    pub size: Option<u64>,
    pub status: FileStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Clean,     // Tracked, no changes
    Modified,  // Modified in working directory
    Staged,    // Changes staged for commit
    Untracked, // Not tracked by git
    Mixed,     // Both staged and working directory changes
}

impl FileStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            FileStatus::Clean => " ",
            FileStatus::Modified => "M",
            FileStatus::Staged => "A",
            FileStatus::Untracked => "?",
            FileStatus::Mixed => "±",
        }
    }

    pub fn style_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            FileStatus::Clean => Color::Gray,
            FileStatus::Modified => Color::Yellow,
            FileStatus::Staged => Color::Green,
            FileStatus::Untracked => Color::Red,
            FileStatus::Mixed => Color::Magenta,
        }
    }
}

/// Get all files in the repository (tracked + untracked, excluding ignored)
pub fn get_git_files(repo_root: &Path) -> Result<Vec<GitFile>> {
    let mut files = Vec::new();
    
    // Get all files: tracked + untracked (excluding ignored)
    let output = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard", "-z"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to list git files: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }

    // Parse null-terminated file list
    let file_list = String::from_utf8_lossy(&output.stdout);
    let file_paths: Vec<&str> = file_list
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();

    // Get file status for all files
    let status_map = get_file_status_map(repo_root)?;

    for file_path in file_paths {
        let path = repo_root.join(file_path);
        let display_path = file_path.to_string();
        
        // Get file metadata
        let (modified, size) = get_file_metadata(&path);
        
        // Determine file status
        let status = status_map.get(file_path)
            .cloned()
            .unwrap_or(FileStatus::Clean);

        files.push(GitFile {
            path,
            display_path,
            modified,
            size,
            status,
        });
    }

    // Sort by path for consistent ordering
    files.sort_by(|a, b| a.display_path.cmp(&b.display_path));

    Ok(files)
}

/// Get file status map for all files in repository
fn get_file_status_map(repo_root: &Path) -> Result<std::collections::HashMap<String, FileStatus>> {
    let mut status_map = std::collections::HashMap::new();

    // Get git status --porcelain output
    let output = Command::new("git")
        .args(["status", "--porcelain", "-z"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        return Ok(status_map); // Return empty map if status fails
    }

    let status_output = String::from_utf8_lossy(&output.stdout);
    let status_lines: Vec<&str> = status_output
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();

    for line in status_lines {
        if line.len() < 3 {
            continue;
        }

        let index_status = line.chars().next().unwrap_or(' ');
        let worktree_status = line.chars().nth(1).unwrap_or(' ');
        let file_path = &line[3..];

        let status = match (index_status, worktree_status) {
            (' ', 'M') => FileStatus::Modified,
            ('M', ' ') => FileStatus::Staged,
            ('A', ' ') => FileStatus::Staged,
            ('D', ' ') => FileStatus::Staged,
            ('R', ' ') => FileStatus::Staged,
            ('C', ' ') => FileStatus::Staged,
            ('M', 'M') => FileStatus::Mixed,
            ('A', 'M') => FileStatus::Mixed,
            ('?', '?') => FileStatus::Untracked,
            _ => FileStatus::Modified, // Default for other combinations
        };

        status_map.insert(file_path.to_string(), status);
    }

    Ok(status_map)
}

/// Get file metadata (modification time and size)
fn get_file_metadata(path: &Path) -> (Option<SystemTime>, Option<u64>) {
    if let Ok(metadata) = std::fs::metadata(path) {
        let modified = metadata.modified().ok();
        let size = Some(metadata.len());
        (modified, size)
    } else {
        (None, None)
    }
}

/// Format file size in human-readable format
pub fn format_file_size(size: Option<u64>) -> String {
    match size {
        Some(size) => {
            if size < 1024 {
                format!("{}B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1}K", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.1}M", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1}G", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        }
        None => "-".to_string(),
    }
}

/// Format modification time in human-readable format
pub fn format_modified_time(modified: Option<SystemTime>) -> String {
    match modified {
        Some(time) => {
            let datetime: DateTime<Utc> = time.into();
            let now = Utc::now();
            let duration = now.signed_duration_since(datetime);

            if let Ok(duration) = duration.to_std() {
                let seconds = duration.as_secs();
                if seconds < 60 {
                    format!("{}s ago", seconds)
                } else if seconds < 3600 {
                    format!("{}m ago", seconds / 60)
                } else if seconds < 86400 {
                    format!("{}h ago", seconds / 3600)
                } else if seconds < 86400 * 7 {
                    format!("{}d ago", seconds / 86400)
                } else if seconds < 86400 * 30 {
                    format!("{}w ago", seconds / (86400 * 7))
                } else {
                    format!("{}mo ago", seconds / (86400 * 30))
                }
            } else {
                "unknown".to_string()
            }
        }
        None => "-".to_string(),
    }
}

/// Verifies that a file exists in the git repository
pub fn verify_file_in_repo(repo_root: &Path, file_path: &Path) -> GeschichteResult<PathBuf> {
    let relative_path = if file_path.is_absolute() {
        file_path
            .strip_prefix(repo_root)
            .map_err(|_| GeschichteError::FileNotFound {
                path: file_path.to_path_buf(),
            })?
    } else {
        file_path
    };

    let output = Command::new("git")
        .arg("ls-files")
        .arg("--error-unmatch")
        .arg(relative_path)
        .current_dir(repo_root)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git ls-files --error-unmatch {}", relative_path.display()),
            output: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(GeschichteError::FileNotFound {
            path: file_path.to_path_buf(),
        });
    }

    Ok(relative_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(Some(512)), "512B");
        assert_eq!(format_file_size(Some(1536)), "1.5K");
        assert_eq!(format_file_size(Some(1048576)), "1.0M");
        assert_eq!(format_file_size(None), "-");
    }

    #[test]
    fn test_file_status_symbol() {
        assert_eq!(FileStatus::Clean.symbol(), " ");
        assert_eq!(FileStatus::Modified.symbol(), "M");
        assert_eq!(FileStatus::Staged.symbol(), "A");
        assert_eq!(FileStatus::Untracked.symbol(), "?");
        assert_eq!(FileStatus::Mixed.symbol(), "±");
    }
}