use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "geschichte",
    version,
    about = "A blazingly fast TUI for viewing git file history and diffs",
    long_about = None
)]
pub struct Args {
    /// Path to the file to view history for (optional - opens file picker if not provided)
    #[arg(value_name = "FILE")]
    pub file_path: Option<PathBuf>,

    /// Repository root directory (auto-discovered if not specified)
    #[arg(short = 'C', long = "repo", value_name = "DIR")]
    pub repo_path: Option<PathBuf>,

    /// Number of context lines in diffs
    #[arg(short = 'L', long = "lines", default_value = "3")]
    pub context_lines: u32,

    /// Show only first-parent commits (linearize merge commits)
    #[arg(long = "first-parent")]
    pub first_parent: bool,

    /// Disable rename tracking
    #[arg(long = "no-follow")]
    pub no_follow: bool,

    /// Enable debug logging
    #[arg(long = "debug")]
    pub debug: bool,

}

impl Args {
    pub fn validate(&self) -> Result<(), String> {
        if self.context_lines > 100 {
            return Err("Context lines must be between 0 and 100".to_string());
        }


        Ok(())
    }
}