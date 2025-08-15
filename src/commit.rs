use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub date: String,
    pub _author: String,
    pub subject: String,
    pub _rename_info: Option<RenameInfo>,
    pub is_working_directory: bool,
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
        Self {
            hash,
            short_hash,
            date,
            _author: author,
            subject,
            _rename_info: None,
            is_working_directory: false,
        }
    }

    pub fn new_working_directory(status_text: String) -> Self {
        Self {
            hash: "WORKING_DIR".to_string(),
            short_hash: "WD".to_string(),
            date: "Working".to_string(),
            _author: "Directory".to_string(),
            subject: status_text,
            _rename_info: None,
            is_working_directory: true,
        }
    }
}