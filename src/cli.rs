use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum LayoutMode {
    /// Traditional unified diff view (two panels)
    Unified,
    /// Side-by-side diff view (three panels)
    SideBySide,
    /// Automatically choose based on terminal size
    Auto,
}

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

    /// Show full file content in diffs instead of just context around changes
    #[arg(long = "full-file")]
    pub full_file: bool,

    /// Show only first-parent commits (linearize merge commits)
    #[arg(long = "first-parent")]
    pub first_parent: bool,

    /// Disable rename tracking
    #[arg(long = "no-follow")]
    pub no_follow: bool,

    /// Enable debug logging
    #[arg(long = "debug")]
    pub debug: bool,

    /// Enable side-by-side diff view (three-panel layout)
    #[arg(short = 's', long = "side-by-side")]
    pub side_by_side: bool,

    /// Layout mode for the UI
    #[arg(long = "layout", value_enum, default_value = "unified")]
    pub layout: LayoutMode,
}

impl Args {
    pub fn validate(&self) -> Result<(), String> {
        if !self.full_file && self.context_lines > 100 {
            return Err("Context lines must be between 0 and 100".to_string());
        }

        Ok(())
    }

    /// Get the effective context lines, considering the full-file flag
    pub fn effective_context_lines(&self) -> u32 {
        if self.full_file {
            // Use a very large number to show the full file
            9999
        } else {
            self.context_lines
        }
    }

    /// Get the effective layout mode, considering both --side-by-side flag and --layout option
    pub fn effective_layout(&self) -> LayoutMode {
        // --side-by-side flag takes precedence for backwards compatibility
        if self.side_by_side {
            LayoutMode::SideBySide
        } else {
            self.layout
        }
    }
}
