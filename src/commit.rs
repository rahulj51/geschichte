use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub date: String,
    pub author: String,
    pub subject: String,
    pub rename_info: Option<RenameInfo>,
    pub is_working_directory: bool,
}

#[derive(Debug, Clone)]
pub struct RenameInfo {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub similarity: u8,
}

impl Commit {
    pub fn new(
        hash: String,
        short_hash: String,
        date: String,
        author: String,
        subject: String,
    ) -> Self {
        Self {
            hash,
            short_hash,
            date,
            author,
            subject,
            rename_info: None,
            is_working_directory: false,
        }
    }

    pub fn new_working_directory(status_text: String) -> Self {
        Self {
            hash: "WORKING_DIR".to_string(),
            short_hash: "WD".to_string(),
            date: "Working".to_string(),
            author: "Directory".to_string(),
            subject: status_text,
            rename_info: None,
            is_working_directory: true,
        }
    }
}