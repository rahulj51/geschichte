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
    /// Path to the file to view history for
    #[arg(value_name = "FILE")]
    pub file_path: PathBuf,

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

    /// Configuration file path
    #[arg(long = "config", value_name = "FILE")]
    pub config_path: Option<PathBuf>,

    /// Color scheme (auto, always, never)
    #[arg(long = "color", default_value = "auto")]
    pub color: String,
}

impl Args {
    pub fn validate(&self) -> Result<(), String> {
        if self.context_lines > 100 {
            return Err("Context lines must be between 0 and 100".to_string());
        }

        if !matches!(self.color.as_str(), "auto" | "always" | "never") {
            return Err("Color must be one of: auto, always, never".to_string());
        }

        Ok(())
    }
}