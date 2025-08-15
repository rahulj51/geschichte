use crate::error::{GeschichteError, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkingDirectoryStatus {
    Clean,
    Modified,
    Staged,
    ModifiedAndStaged,
}

/// Checks if the working directory has changes for the specified file
pub fn check_working_directory_status(
    repo_root: &Path,
    file_path: &Path,
) -> Result<WorkingDirectoryStatus> {
    // Check for staged changes
    let staged_output = Command::new("git")
        .current_dir(repo_root)
        .arg("diff")
        .arg("--cached")
        .arg("--name-only")
        .arg("--")
        .arg(file_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git diff --cached --name-only {}", file_path.display()),
            output: e.to_string(),
        })?;

    let has_staged = staged_output.status.success() && !staged_output.stdout.is_empty();

    // Check for unstaged changes
    let unstaged_output = Command::new("git")
        .current_dir(repo_root)
        .arg("diff")
        .arg("--name-only")
        .arg("--")
        .arg(file_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git diff --name-only {}", file_path.display()),
            output: e.to_string(),
        })?;

    let has_unstaged = unstaged_output.status.success() && !unstaged_output.stdout.is_empty();

    match (has_staged, has_unstaged) {
        (true, true) => Ok(WorkingDirectoryStatus::ModifiedAndStaged),
        (true, false) => Ok(WorkingDirectoryStatus::Staged),
        (false, true) => Ok(WorkingDirectoryStatus::Modified),
        (false, false) => Ok(WorkingDirectoryStatus::Clean),
    }
}

/// Fetches the working directory diff vs HEAD
pub fn fetch_working_directory_diff(
    repo_root: &Path,
    file_path: &Path,
    context_lines: u32,
) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .arg("diff")
        .arg(format!("--unified={}", context_lines))
        .arg("HEAD")
        .arg("--")
        .arg(file_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git diff HEAD {}", file_path.display()),
            output: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // It's okay if the file doesn't exist in HEAD (new file)
        if stderr.contains("does not exist") || stderr.contains("pathspec") {
            // For new files, show the entire file as additions
            return fetch_new_file_diff(repo_root, file_path);
        }
        return Err(GeschichteError::GitCommandFailed {
            command: format!("git diff HEAD {}", file_path.display()),
            output: stderr.to_string(),
        });
    }

    let diff_output = String::from_utf8_lossy(&output.stdout).to_string();
    
    // If no diff output, the working directory might be clean
    if diff_output.trim().is_empty() {
        Ok("Working directory is clean - no changes detected".to_string())
    } else {
        Ok(diff_output)
    }
}

/// Handles new files that don't exist in HEAD
fn fetch_new_file_diff(repo_root: &Path, file_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .arg("diff")
        .arg("--no-index")
        .arg("/dev/null")
        .arg(file_path)
        .output()
        .map_err(|e| GeschichteError::GitCommandFailed {
            command: format!("git diff --no-index /dev/null {}", file_path.display()),
            output: e.to_string(),
        })?;

    // git diff --no-index returns exit code 1 when files differ, which is expected
    let diff_output = String::from_utf8_lossy(&output.stdout).to_string();
    
    if diff_output.trim().is_empty() {
        // Fallback: try to show the file as entirely new
        Ok(format!(
            "diff --git a/{} b/{}\nnew file mode 100644\nindex 0000000..0000000\n--- /dev/null\n+++ b/{}\n@@ -0,0 +1,? @@\n(New file - content not shown)",
            file_path.display(),
            file_path.display(),
            file_path.display()
        ))
    } else {
        Ok(diff_output)
    }
}