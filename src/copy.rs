use crate::commit::Commit;
use arboard::Clipboard;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CopyMode {
    WaitingForTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CopyFormat {
    FullSha,
    ShortSha,
    #[allow(dead_code)]
    Subject,
    Message,
    Author,
    Date,
    GitHubUrl,
}

impl fmt::Display for CopyFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CopyFormat::FullSha => write!(f, "Full SHA"),
            CopyFormat::ShortSha => write!(f, "Short SHA"),
            CopyFormat::Subject => write!(f, "Subject"),
            CopyFormat::Message => write!(f, "Message"),
            CopyFormat::Author => write!(f, "Author"),
            CopyFormat::Date => write!(f, "Date"),
            CopyFormat::GitHubUrl => write!(f, "GitHub URL"),
        }
    }
}

pub struct CommitCopier {
    clipboard: Option<Clipboard>,
}

impl CommitCopier {
    pub fn new() -> Self {
        let clipboard = Clipboard::new().ok();
        Self { clipboard }
    }

    pub fn copy_commit_info(&mut self, commit: &Commit, format: CopyFormat) -> Result<String, String> {
        let content = match format {
            CopyFormat::FullSha => commit.hash.clone(),
            CopyFormat::ShortSha => commit.short_hash.clone(),
            CopyFormat::Subject => commit.subject.clone(),
            CopyFormat::Message => {
                if commit.body.is_empty() {
                    commit.subject.clone()
                } else {
                    format!("{}\n\n{}", commit.subject, commit.body)
                }
            }
            CopyFormat::Author => commit.author(),
            CopyFormat::Date => commit.author_date.clone(),
            CopyFormat::GitHubUrl => {
                // This would need actual remote detection in real implementation
                if let Some(ref pr_info) = commit.pr_info {
                    pr_info.url.clone()
                } else {
                    format!("https://github.com/repo/commit/{}", commit.hash)
                }
            }
        };

        if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(&content).map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
            Ok(content)
        } else {
            Err("Clipboard not available".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }
}

impl Default for CommitCopier {
    fn default() -> Self {
        Self::new()
    }
}