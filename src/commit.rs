use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub date: String,
    pub author_name: String,
    pub author_email: String,
    pub committer_name: String,
    pub committer_email: String,
    pub author_date: String,
    pub committer_date: String,
    pub subject: String,
    pub body: String,
    pub refs: Vec<String>,
    pub pr_info: Option<PullRequestInfo>,
    pub stats: Option<CommitStats>,
    pub _rename_info: Option<RenameInfo>,
    pub is_working_directory: bool,
}

#[derive(Debug, Clone)]
pub struct PullRequestInfo {
    pub number: u32,
    pub title: String,
    pub url: String,
    #[allow(dead_code)]
    pub status: PRStatus,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PRStatus {
    Open,
    Closed,
    Merged,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CommitStats {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone)]
pub struct RenameInfo {
    pub _old_path: PathBuf,
    pub _new_path: PathBuf,
    pub _similarity: u8,
}

impl Commit {
    pub fn new(
        hash: String,
        short_hash: String,
        date: String,
        author: String,
        subject: String,
    ) -> Self {
        // Parse author string to extract name and email
        let (author_name, author_email) = Self::parse_author(&author);

        Self {
            hash,
            short_hash,
            date: date.clone(),
            author_name,
            author_email,
            committer_name: String::new(), // Will be filled later when more data is loaded
            committer_email: String::new(),
            author_date: date, // Use same date for now
            committer_date: String::new(),
            subject,
            body: String::new(),
            refs: Vec::new(),
            pr_info: None,
            stats: None,
            _rename_info: None,
            is_working_directory: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_enhanced(
        hash: String,
        short_hash: String,
        author_name: String,
        author_email: String,
        author_date: String,
        committer_name: String,
        committer_email: String,
        committer_date: String,
        subject: String,
        body: String,
    ) -> Self {
        Self {
            hash,
            short_hash,
            date: author_date.clone(),
            author_name,
            author_email,
            committer_name,
            committer_email,
            author_date,
            committer_date,
            subject,
            body,
            refs: Vec::new(),
            pr_info: None,
            stats: None,
            _rename_info: None,
            is_working_directory: false,
        }
    }

    pub fn new_working_directory(status_text: String) -> Self {
        Self {
            hash: "WORKING_DIR".to_string(),
            short_hash: "WD".to_string(),
            date: "Working".to_string(),
            author_name: "Working".to_string(),
            author_email: String::new(),
            committer_name: "Directory".to_string(),
            committer_email: String::new(),
            author_date: "Working".to_string(),
            committer_date: String::new(),
            subject: status_text,
            body: String::new(),
            refs: Vec::new(),
            pr_info: None,
            stats: None,
            _rename_info: None,
            is_working_directory: true,
        }
    }

    fn parse_author(author: &str) -> (String, String) {
        // Parse "Name <email>" format
        if let Some(email_start) = author.rfind('<') {
            if let Some(email_end) = author.rfind('>') {
                if email_start < email_end {
                    let name = author[..email_start].trim().to_string();
                    let email = author[email_start + 1..email_end].to_string();
                    return (name, email);
                }
            }
        }
        // Fallback if parsing fails
        (author.to_string(), String::new())
    }

    pub fn author(&self) -> String {
        if self.author_email.is_empty() {
            self.author_name.clone()
        } else {
            format!("{} <{}>", self.author_name, self.author_email)
        }
    }
}
