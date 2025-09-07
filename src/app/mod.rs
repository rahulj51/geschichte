pub mod events;

use crate::cache::DiffCache;
use crate::cli::LayoutMode;
use crate::commit::Commit;
use crate::copy::{CommitCopier, CopyFormat, CopyMode};
use crate::diff::side_by_side::SideBySideDiff;
use crate::error::{self, Result};
use crate::ui::file_picker::FilePickerState;
use crate::ui::state::UIState;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, process::Command};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    Commits,
    Diff,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AppMode {
    FilePicker {
        state: FilePickerState,
        context: FilePickerContext,
    },
    History {
        file_path: PathBuf,
        focused_panel: FocusedPanel,
    },
}

#[derive(Debug, Clone)]
pub enum FilePickerContext {
    Initial,                               // Started with no file argument
    SwitchFile { previous_file: PathBuf }, // Switching from an existing file
}

#[derive(Debug, Clone)]
pub struct DiffSearchState {
    pub query: String,
    pub is_active: bool,               // Currently in search mode
    pub is_input_mode: bool,           // Currently typing search query
    pub results: Vec<SearchMatch>,     // All matches found
    pub current_result: Option<usize>, // Index of highlighted result
    pub regex: Option<Regex>,          // Compiled regex for performance
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchMatch {
    pub line_index: usize, // Index in diff lines
    pub char_start: usize, // Start position in line
    pub char_end: usize,   // End position in line
    pub content: String,   // Matched text for highlighting
}

pub struct App {
    pub repo_root: PathBuf,
    pub should_quit: bool,
    pub context_lines: u32,
    pub follow_renames: bool,
    pub first_parent: bool,

    // Application mode
    pub mode: AppMode,

    // History mode data (only valid when in History mode)
    pub commits: Vec<Commit>,
    pub selected_index: usize,
    pub rename_map: HashMap<String, PathBuf>,
    pub current_diff: String,
    pub current_side_by_side_diff: Option<SideBySideDiff>,
    pub diff_cache: DiffCache,

    // UI state (moved to separate struct)
    pub ui_state: UIState,

    // Core app state
    pub layout_mode: LayoutMode,
    pub loading: bool,
    pub error_message: Option<String>,

    // Diff range selection
    pub diff_range_start: Option<usize>,
    pub current_diff_range: Option<(usize, usize)>, // (older_index, newer_index)

    // Copy functionality
    pub copy_mode: Option<CopyMode>,
    pub copier: CommitCopier,
    pub copy_message: Option<String>,

    // Commit info popup
    pub show_commit_info: bool,
    pub commit_info_popup: Option<crate::ui::commit_info::CommitInfoPopup>,

    // Change navigation cache
    pub current_changes: Vec<usize>, // Line indices of all changes
    pub current_change_index: Option<usize>, // Index into current_changes array

    // Message timing
    pub message_timer: Option<std::time::Instant>,

    // Diff search state
    pub diff_search_state: Option<DiffSearchState>,

    // File picker navigation state
    pub came_from_file_picker: bool,

    // Signal for redrawing TUI.
    pub redraw_tui: bool,
}

impl App {
    /// Get the effective layout mode based on terminal width (for Auto mode)
    pub fn effective_layout(&self) -> LayoutMode {
        match self.layout_mode {
            LayoutMode::Auto => {
                // Use side-by-side if terminal is wide enough (120+ columns)
                if self.ui_state.terminal_width >= 120 {
                    LayoutMode::SideBySide
                } else {
                    LayoutMode::Unified
                }
            }
            other => other,
        }
    }

    pub fn new_file_picker(
        repo_root: PathBuf,
        context_lines: u32,
        follow_renames: bool,
        first_parent: bool,
        layout_mode: LayoutMode,
    ) -> Result<Self> {
        use crate::git::files::get_git_files;

        let files = get_git_files(&repo_root)?;
        let file_picker_state = FilePickerState::new(files);

        Ok(Self {
            repo_root,
            should_quit: false,
            context_lines,
            follow_renames,
            first_parent,
            mode: AppMode::FilePicker {
                state: file_picker_state,
                context: FilePickerContext::Initial,
            },
            commits: Vec::new(),
            selected_index: 0,
            rename_map: HashMap::new(),
            current_diff: String::new(),
            current_side_by_side_diff: None,
            diff_cache: DiffCache::new(50),
            ui_state: UIState::new(),
            layout_mode,
            loading: false,
            error_message: None,
            diff_range_start: None,
            current_diff_range: None,
            copy_mode: None,
            copier: CommitCopier::new(),
            copy_message: None,
            show_commit_info: false,
            commit_info_popup: None,
            current_changes: Vec::new(),
            current_change_index: None,
            message_timer: None,
            diff_search_state: None,
            came_from_file_picker: false,
            redraw_tui: false,
        })
    }

    pub fn new_history(
        repo_root: PathBuf,
        file_path: PathBuf,
        context_lines: u32,
        follow_renames: bool,
        first_parent: bool,
        layout_mode: LayoutMode,
    ) -> Self {
        Self {
            repo_root,
            should_quit: false,
            context_lines,
            follow_renames,
            first_parent,
            mode: AppMode::History {
                file_path,
                focused_panel: FocusedPanel::Commits,
            },
            commits: Vec::new(),
            selected_index: 0,
            rename_map: HashMap::new(),
            current_diff: String::new(),
            current_side_by_side_diff: None,
            diff_cache: DiffCache::new(50),
            ui_state: UIState::new(),
            layout_mode,
            loading: false,
            error_message: None,
            diff_range_start: None,
            current_diff_range: None,
            copy_mode: None,
            copier: CommitCopier::new(),
            copy_message: None,
            show_commit_info: false,
            commit_info_popup: None,
            current_changes: Vec::new(),
            current_change_index: None,
            message_timer: None,
            diff_search_state: None,
            came_from_file_picker: false,
            redraw_tui: false,
        }
    }

    pub fn switch_to_history(&mut self, file_path: PathBuf, from_picker: bool) -> Result<()> {
        self.mode = AppMode::History {
            file_path,
            focused_panel: FocusedPanel::Commits,
        };

        // Track whether we came from file picker
        self.came_from_file_picker = from_picker;

        // Clear existing data
        self.commits.clear();
        self.selected_index = 0;
        self.rename_map.clear();
        self.current_diff.clear();
        self.current_side_by_side_diff = None;
        self.ui_state.reset_diff_scroll();
        self.diff_cache.clear();
        self.clear_change_cache();
        self.clear_diff_search();

        // Load git data for the new file
        self.load_git_data()
    }

    pub fn switch_to_file_picker(&mut self) -> Result<()> {
        // Only switch to file picker if we're currently in history mode
        let previous_file = match &self.mode {
            AppMode::History { file_path, .. } => file_path.clone(),
            AppMode::FilePicker { .. } => return Ok(()), // Already in file picker
        };

        // Load git files
        use crate::git::files::get_git_files;
        let files = get_git_files(&self.repo_root)?;
        let file_picker_state = FilePickerState::new(files);

        // Switch to file picker with context
        self.mode = AppMode::FilePicker {
            state: file_picker_state,
            context: FilePickerContext::SwitchFile { previous_file },
        };

        // Clear the flag since we're now in file picker mode
        self.came_from_file_picker = false;

        Ok(())
    }

    pub fn load_git_data(&mut self) -> Result<()> {
        // Only load git data when in History mode
        let file_path = match &self.mode {
            AppMode::History { file_path, .. } => file_path.clone(),
            AppMode::FilePicker { .. } => return Ok(()), // No-op for file picker mode
        };

        self.loading = true;
        self.error_message = None;

        // Load commits
        let mut commits = crate::git::history::fetch_commit_history(
            &self.repo_root,
            &file_path,
            self.follow_renames,
            self.first_parent,
        )?;

        // Check for working directory changes and prepend if found
        let wd_status =
            crate::git::working::check_working_directory_status(&self.repo_root, &file_path)?;

        if wd_status != crate::git::working::WorkingDirectoryStatus::Clean {
            let status_text = match wd_status {
                crate::git::working::WorkingDirectoryStatus::Modified => "Modified".to_string(),
                crate::git::working::WorkingDirectoryStatus::Staged => "Staged".to_string(),
                crate::git::working::WorkingDirectoryStatus::ModifiedAndStaged => {
                    "Modified + Staged".to_string()
                }
                crate::git::working::WorkingDirectoryStatus::Clean => unreachable!(),
            };

            let wd_commit = crate::commit::Commit::new_working_directory(status_text);
            commits.insert(0, wd_commit);
        }

        self.commits = commits;

        // Build rename map
        if self.follow_renames {
            self.rename_map = crate::git::history::build_rename_map(&self.repo_root, &file_path)?;
        }

        // Load initial diff if we have commits
        if !self.commits.is_empty() {
            self.load_diff_for_selected_commit()?;
        }

        self.loading = false;
        Ok(())
    }

    pub fn load_diff_for_selected_commit(&mut self) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        // Only load diff when in History mode
        let file_path = match &self.mode {
            AppMode::History { file_path, .. } => file_path.clone(),
            AppMode::FilePicker { .. } => return Ok(()), // No-op for file picker mode
        };

        let commit = &self.commits[self.selected_index];

        // Check cache first
        if let Some(cached_diff) = self.diff_cache.get(&commit.hash).cloned() {
            self.current_diff = cached_diff.clone();
            self.update_side_by_side_diff(&cached_diff);
            self.update_change_cache();
            self.reset_diff_scroll();
            return Ok(());
        }

        let diff = if commit.is_working_directory {
            // Handle working directory diff
            crate::git::working::fetch_working_directory_diff(
                &self.repo_root,
                &file_path,
                self.context_lines,
            )?
        } else {
            // Handle regular commit diff
            let parents = crate::git::history::get_commit_parents(&self.repo_root, &commit.hash)?;
            let parent_hash = parents.first().map(|s| s.as_str());

            // Resolve file path at this commit
            let commit_file_path = self
                .rename_map
                .get(&commit.hash)
                .cloned()
                .unwrap_or_else(|| file_path.clone());

            crate::git::diff::fetch_diff(
                &self.repo_root,
                &commit.hash,
                parent_hash,
                &commit_file_path,
                self.context_lines,
            )?
        };

        // Cache and store
        self.diff_cache.put(commit.hash.clone(), diff.clone());
        self.current_diff = diff.clone();
        self.update_side_by_side_diff(&diff);
        self.update_change_cache();

        self.reset_diff_scroll();

        Ok(())
    }

    /// Update the side-by-side diff representation
    fn update_side_by_side_diff(&mut self, diff: &str) {
        if matches!(self.effective_layout(), LayoutMode::SideBySide) {
            use crate::diff::HighlightedDiff;
            let highlighted_diff =
                HighlightedDiff::new(diff, self.get_file_path().map(|p| p.as_path()));
            self.current_side_by_side_diff =
                Some(SideBySideDiff::from_unified(&highlighted_diff.lines));
        } else {
            // Clear side-by-side diff when not needed
            self.current_side_by_side_diff = None;
        }
    }

    fn reset_diff_scroll(&mut self) {
        self.ui_state.reset_diff_scroll();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn move_selection_up(&mut self) -> Result<()> {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            // Clear range diff when navigating
            if self.diff_range_start.is_none() {
                self.current_diff_range = None;
            }
            // Clear search when navigating to different commit
            self.clear_diff_search();
            self.load_diff_for_selected_commit()?;
        }
        Ok(())
    }

    pub fn move_selection_down(&mut self) -> Result<()> {
        if self.selected_index + 1 < self.commits.len() {
            self.selected_index += 1;
            // Clear range diff when navigating
            if self.diff_range_start.is_none() {
                self.current_diff_range = None;
            }
            // Clear search when navigating to different commit
            self.clear_diff_search();
            self.load_diff_for_selected_commit()?;
        }
        Ok(())
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        let old_effective_layout = self.effective_layout();

        self.ui_state.handle_resize(width, height);

        // Check if effective layout changed (for Auto mode)
        let new_effective_layout = self.effective_layout();
        if old_effective_layout != new_effective_layout && !self.current_diff.is_empty() {
            self.update_side_by_side_diff(&self.current_diff.clone());
        }
    }

    pub fn switch_focus(&mut self) {
        if let AppMode::History { focused_panel, .. } = &mut self.mode {
            *focused_panel = match *focused_panel {
                FocusedPanel::Commits => FocusedPanel::Diff,
                FocusedPanel::Diff => FocusedPanel::Commits,
            };
        }
    }

    pub fn get_focused_panel(&self) -> Option<FocusedPanel> {
        match &self.mode {
            AppMode::History { focused_panel, .. } => Some(*focused_panel),
            AppMode::FilePicker { .. } => None,
        }
    }

    pub fn get_file_path(&self) -> Option<&PathBuf> {
        match &self.mode {
            AppMode::History { file_path, .. } => Some(file_path),
            AppMode::FilePicker { .. } => None,
        }
    }

    pub fn toggle_diff_range_selection(&mut self) -> Result<()> {
        match &self.mode {
            AppMode::History { .. } => {
                if let Some(start_index) = self.diff_range_start {
                    // We have a start commit, create range diff
                    let end_index = self.selected_index;
                    if start_index != end_index {
                        self.show_diff_range(start_index, end_index)?;
                    } else {
                        // Same commit selected, just clear the selection
                        self.current_diff_range = None;
                    }
                    self.diff_range_start = None;
                } else {
                    // Mark the current commit as start
                    self.diff_range_start = Some(self.selected_index);
                }
                Ok(())
            }
            AppMode::FilePicker { .. } => Ok(()),
        }
    }

    pub fn clear_diff_range_selection(&mut self) {
        self.diff_range_start = None;
        self.current_diff_range = None;
    }

    pub fn is_commit_marked_for_diff(&self, index: usize) -> bool {
        self.diff_range_start == Some(index)
    }

    fn show_diff_range(&mut self, start_index: usize, end_index: usize) -> Result<()> {
        if self.commits.is_empty()
            || start_index >= self.commits.len()
            || end_index >= self.commits.len()
        {
            return Ok(());
        }

        // Determine the correct chronological order (older commit first)
        // In the commits list, newer commits are at the top (lower index)
        // So lower index = newer, higher index = older
        let (older_index, newer_index) = if start_index > end_index {
            (start_index, end_index) // start is older (higher index)
        } else {
            (end_index, start_index) // end is older (higher index)
        };

        let older_commit = &self.commits[older_index];
        let newer_commit = &self.commits[newer_index];

        // Create cache key for the range diff (always older..newer)
        let cache_key = format!("{}..{}", older_commit.hash, newer_commit.hash);

        // Check cache first
        if let Some(cached_diff) = self.diff_cache.get(&cache_key).cloned() {
            self.current_diff = cached_diff.clone();
            self.update_side_by_side_diff(&cached_diff);
            self.update_change_cache();
            self.reset_diff_scroll();
            self.current_diff_range = Some((older_index, newer_index));
            return Ok(());
        }

        // Get the file path
        let file_path = match &self.mode {
            AppMode::History { file_path, .. } => file_path.clone(),
            AppMode::FilePicker { .. } => return Ok(()), // Should not happen
        };

        // Generate diff between the two commits (older..newer)
        let diff = crate::git::diff::get_diff_between_commits(
            &self.repo_root,
            &older_commit.hash,
            &newer_commit.hash,
            &file_path,
            self.context_lines,
        )?;

        // Cache and set the diff
        self.diff_cache.put(cache_key, diff.clone());
        self.current_diff = diff.clone();
        self.update_side_by_side_diff(&diff);
        self.update_change_cache();
        self.reset_diff_scroll();

        // Store the current range for UI display
        self.current_diff_range = Some((older_index, newer_index));

        Ok(())
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        // Handle search input first if active
        if self.handle_search_input_keys(key)? {
            return Ok(());
        }

        // Handle file picker mode separately
        if matches!(self.mode, AppMode::FilePicker { .. }) {
            return self.handle_file_picker_key(key);
        }

        // Try handling with the specialized event handlers
        if self.handle_navigation_keys(key)? {
            return Ok(());
        }
        if self.handle_change_navigation_keys(key)? {
            return Ok(());
        }
        if self.handle_scrolling_keys(key)? {
            return Ok(());
        }
        if self.handle_copy_keys(key)? {
            return Ok(());
        }
        if self.handle_ui_keys(key)? {
            return Ok(());
        }

        Ok(())
    }

    fn handle_file_picker_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // Special commands first
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                // Ctrl+Q always quits the app regardless of context
                self.quit();
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                // Select file and switch to history mode
                if let AppMode::FilePicker { ref state, .. } = self.mode {
                    if let Some(selected_file) = state.get_selected_file() {
                        let file_path = selected_file.path.clone();
                        self.switch_to_history(file_path, true)?;
                    }
                }
            }

            // Navigation keys (arrow keys and Ctrl+N/P)
            (KeyCode::Up, KeyModifiers::NONE) => {
                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.move_up();
                }
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.move_down();
                }
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                // Ctrl+P = up (emacs-style)
                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.move_up();
                }
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                // Ctrl+N = down (emacs-style)
                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.move_down();
                }
            }

            // Text editing keys
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.delete_char();
                }
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.clear_query();
                }
            }

            // All regular characters for typing (including j, k, q, etc.)
            (KeyCode::Char(c), KeyModifiers::NONE) => {
                // Filter out escape sequence fragments and control characters
                // This prevents '[' and other control chars from appearing during rapid arrow key presses
                if c.is_control() || c == '[' || c == '\x1b' {
                    // Skip these characters - likely escape sequence fragments or control chars
                    return Ok(());
                }

                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.append_char(c);
                }
            }

            _ => {}
        }
        Ok(())
    }

    // Helper functions for calculating content width
    pub fn calculate_max_diff_line_width(&self) -> usize {
        self.current_diff
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
    }

    pub fn calculate_max_commit_line_width(&self) -> usize {
        self.commits
            .iter()
            .map(|commit| {
                format!("{} {}", commit.hash, commit.subject)
                    .chars()
                    .count()
            })
            .max()
            .unwrap_or(0)
    }

    pub fn get_diff_line_count(&self) -> usize {
        match self.effective_layout() {
            crate::cli::LayoutMode::SideBySide => {
                if let Some(ref side_by_side) = self.current_side_by_side_diff {
                    // Use the maximum of old_lines and new_lines length for side-by-side
                    side_by_side
                        .old_lines
                        .len()
                        .max(side_by_side.new_lines.len())
                } else {
                    0
                }
            }
            _ => {
                // For unified view, count lines in current_diff
                self.current_diff.lines().count()
            }
        }
    }

    // Delegate to UIState for scroll calculation
    #[allow(dead_code)] // Used in tests
    pub fn get_page_scroll_size(&self) -> usize {
        self.ui_state.get_page_scroll_size()
    }

    // Copy functionality methods
    pub fn copy_commit_sha(&mut self, short: bool) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        let commit = &self.commits[self.selected_index];
        let format = if short {
            CopyFormat::ShortSha
        } else {
            CopyFormat::FullSha
        };

        match self.copier.copy_commit_info(commit, format) {
            Ok(content) => {
                self.copy_message = Some(format!("Copied: {}", content));
                self.copy_mode = None;
                self.start_message_timer();
            }
            Err(err) => {
                self.error_message = Some(err);
                self.start_message_timer();
            }
        }

        Ok(())
    }

    pub fn copy_commit_message(&mut self) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        let commit = &self.commits[self.selected_index];

        match self.copier.copy_commit_info(commit, CopyFormat::Message) {
            Ok(_) => {
                self.copy_message = Some("Copied commit message".to_string());
                self.copy_mode = None;
                self.start_message_timer();
            }
            Err(err) => {
                self.error_message = Some(err);
                self.start_message_timer();
            }
        }

        Ok(())
    }

    pub fn copy_commit_author(&mut self) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        let commit = &self.commits[self.selected_index];

        match self.copier.copy_commit_info(commit, CopyFormat::Author) {
            Ok(content) => {
                self.copy_message = Some(format!("Copied author: {}", content));
                self.copy_mode = None;
                self.start_message_timer();
            }
            Err(err) => {
                self.error_message = Some(err);
                self.start_message_timer();
            }
        }

        Ok(())
    }

    pub fn copy_commit_date(&mut self) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        let commit = &self.commits[self.selected_index];

        match self.copier.copy_commit_info(commit, CopyFormat::Date) {
            Ok(content) => {
                self.copy_message = Some(format!("Copied date: {}", content));
                self.copy_mode = None;
                self.start_message_timer();
            }
            Err(err) => {
                self.error_message = Some(err);
                self.start_message_timer();
            }
        }

        Ok(())
    }

    pub fn copy_github_url(&mut self) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        let commit = &self.commits[self.selected_index];

        match self.copier.copy_commit_info(commit, CopyFormat::GitHubUrl) {
            Ok(content) => {
                self.copy_message = Some(format!("Copied URL: {}", content));
                self.copy_mode = None;
                self.start_message_timer();
            }
            Err(err) => {
                self.error_message = Some(err);
                self.start_message_timer();
            }
        }

        Ok(())
    }

    pub fn copy_file_relative_path(&mut self) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        if let Some(file_path) = self.get_file_path() {
            // In CI environments, skip actual clipboard operations
            if error::is_ci_environment() {
                self.copy_message = Some(format!("Copied Path: {}", file_path.display()));
                self.copy_mode = None;
                self.start_message_timer();
                return Ok(());
            }

            use arboard::Clipboard;
            match Clipboard::new() {
                Ok(mut clipboard) => match clipboard.set_text(file_path.to_string_lossy()) {
                    Ok(_) => {
                        self.copy_message = Some(format!("Copied Path: {}", file_path.display()));
                        self.copy_mode = None;
                        self.start_message_timer();
                    }
                    Err(err) => {
                        self.error_message = Some(format!("Failed to copy to clipboard: {}", err));
                        self.start_message_timer();
                    }
                },
                Err(err) => {
                    self.error_message = Some(format!("Failed to initialize clipboard: {}", err));
                    self.start_message_timer();
                }
            }
        }

        Ok(())
    }

    pub fn start_copy_mode(&mut self) {
        self.copy_mode = Some(CopyMode::WaitingForTarget);
        self.copy_message = Some(
            "Copy mode: s=SHA, h=short, m=msg, a=author, d=date, u=URL, y=SHA, p=path".to_string(),
        );
    }

    pub fn cancel_copy_mode(&mut self) {
        self.copy_mode = None;
        self.copy_message = None;
    }

    #[allow(dead_code)]
    pub fn clear_copy_message(&mut self) {
        self.copy_message = None;
        self.message_timer = None;
    }

    pub fn start_message_timer(&mut self) {
        self.message_timer = Some(std::time::Instant::now());
    }

    pub fn check_message_timeout(&mut self) {
        if let Some(timer) = self.message_timer {
            if timer.elapsed().as_secs() >= 3 {
                self.copy_message = None;
                self.error_message = None;
                self.message_timer = None;
            }
        }
    }

    // Commit info popup methods
    pub fn show_commit_info_popup(&mut self) -> Result<()> {
        if self.commits.is_empty() || self.selected_index >= self.commits.len() {
            return Ok(());
        }

        let selected_index = self.selected_index;

        // Load additional commit metadata if not already loaded
        self.load_enhanced_commit_data_by_index(selected_index)?;

        let enhanced_commit = self.commits[selected_index].clone();
        self.commit_info_popup = Some(crate::ui::commit_info::CommitInfoPopup::new(
            enhanced_commit,
        ));
        self.show_commit_info = true;

        Ok(())
    }

    pub fn hide_commit_info_popup(&mut self) {
        self.show_commit_info = false;
        self.commit_info_popup = None;
    }

    pub fn scroll_commit_info_up(&mut self) {
        if let Some(ref mut popup) = self.commit_info_popup {
            popup.scroll_up();
        }
    }

    pub fn scroll_commit_info_down(&mut self) {
        if let Some(ref mut popup) = self.commit_info_popup {
            let total_lines = popup.get_total_lines();
            let viewport_height = 10; // Approximate viewport height
            popup.scroll_down(total_lines, viewport_height);
        }
    }

    /// Update the change cache when diff changes
    /// Call this in load_diff_for_selected_commit() and show_diff_range()
    fn update_change_cache(&mut self) {
        let highlighted_diff = crate::diff::HighlightedDiff::new(
            &self.current_diff,
            self.get_file_path().map(|p| p.as_path()),
        );
        self.current_changes = highlighted_diff.find_changes();
        self.current_change_index = None; // Reset position
    }

    /// Clear change cache when switching files or modes
    fn clear_change_cache(&mut self) {
        self.current_changes.clear();
        self.current_change_index = None;
    }

    /// Get current change status for UI display
    #[allow(dead_code)] // Reserved for future UI enhancement
    pub fn get_change_status(&self) -> Option<(usize, usize)> {
        self.current_change_index
            .map(|index| (index + 1, self.current_changes.len())) // 1-based for display
    }

    /// Navigate to the next change using binary search - O(log n)
    pub fn navigate_to_next_change(&mut self) -> Result<()> {
        if !matches!(self.get_focused_panel(), Some(FocusedPanel::Diff)) {
            return Ok(());
        }

        if self.current_changes.is_empty() {
            return Ok(());
        }

        let current_line = self.ui_state.diff_cursor_line;

        // Binary search for next change position
        let next_index = match self.current_changes.binary_search(&current_line) {
            Ok(idx) => idx + 1, // Currently on a change, go to next
            Err(idx) => idx,    // Insert position is the next change
        };

        if next_index < self.current_changes.len() {
            let next_change_line = self.current_changes[next_index];
            self.ui_state.diff_cursor_line = next_change_line;
            self.ui_state
                .ensure_cursor_visible(&self.effective_layout());
            self.current_change_index = Some(next_index);
        }

        Ok(())
    }

    /// Navigate to the previous change using binary search - O(log n)
    pub fn navigate_to_previous_change(&mut self) -> Result<()> {
        if !matches!(self.get_focused_panel(), Some(FocusedPanel::Diff)) {
            return Ok(());
        }

        if self.current_changes.is_empty() {
            return Ok(());
        }

        let current_line = self.ui_state.diff_cursor_line;

        // Binary search for previous change position
        let prev_index = match self.current_changes.binary_search(&current_line) {
            Ok(idx) => {
                if idx > 0 {
                    Some(idx - 1)
                } else {
                    None
                }
            }
            Err(idx) => {
                if idx > 0 {
                    Some(idx - 1)
                } else {
                    None
                }
            }
        };

        if let Some(index) = prev_index {
            let prev_change_line = self.current_changes[index];
            self.ui_state.diff_cursor_line = prev_change_line;
            self.ui_state
                .ensure_cursor_visible(&self.effective_layout());
            self.current_change_index = Some(index);
        }

        Ok(())
    }

    fn load_enhanced_commit_data_by_index(&mut self, index: usize) -> Result<()> {
        if index >= self.commits.len() {
            return Ok(());
        }

        let commit = &mut self.commits[index];
        if commit.is_working_directory {
            return Ok(());
        }

        // Load refs if not already loaded
        if commit.refs.is_empty() {
            if let Ok(refs) = crate::git::history::fetch_commit_refs(&self.repo_root, &commit.hash)
            {
                commit.refs = refs;
            }
        }

        // Load PR info if not already loaded
        if commit.pr_info.is_none() {
            commit.pr_info = crate::git::history::detect_pr_info(commit);
        }

        // Load stats if not already loaded
        if commit.stats.is_none() {
            if let Ok(stats) =
                crate::git::history::fetch_commit_stats(&self.repo_root, &commit.hash)
            {
                commit.stats = stats;
            }
        }

        Ok(())
    }

    // Diff search functionality
    pub fn start_diff_search(&mut self) {
        self.diff_search_state = Some(DiffSearchState {
            query: String::new(),
            is_active: true,
            is_input_mode: true,
            results: Vec::new(),
            current_result: None,
            regex: None,
        });
    }

    pub fn update_search_results(&mut self) -> Result<()> {
        if let Some(ref mut search_state) = self.diff_search_state {
            if search_state.query.is_empty() {
                search_state.results.clear();
                search_state.current_result = None;
                search_state.regex = None;
                return Ok(());
            }

            // Compile regex (case-insensitive by default, true regex search)
            let regex = match Regex::new(&format!("(?i){}", &search_state.query)) {
                Ok(r) => r,
                Err(_e) => {
                    // Clear search state on invalid regex and show error in status
                    search_state.results.clear();
                    search_state.current_result = None;
                    search_state.regex = None;

                    // Don't propagate error - just show no results for invalid regex
                    // This provides better UX as user types
                    return Ok(());
                }
            };

            // Search through current diff content, but only in actual code lines
            // Parse the diff to get structured information about line types
            let parsed_lines = crate::diff::parse_diff(&self.current_diff);
            let mut results = Vec::new();

            for (line_idx, parsed_line) in parsed_lines.iter().enumerate() {
                // Only search in actual code content lines, skip headers and hunk headers
                match parsed_line.line_type {
                    crate::diff::DiffLineType::Addition
                    | crate::diff::DiffLineType::Deletion
                    | crate::diff::DiffLineType::Context => {
                        // Search in this line's content
                        for mat in regex.find_iter(&parsed_line.content) {
                            results.push(SearchMatch {
                                line_index: line_idx,
                                char_start: mat.start(),
                                char_end: mat.end(),
                                content: mat.as_str().to_string(),
                            });
                        }
                    }
                    crate::diff::DiffLineType::Header | crate::diff::DiffLineType::HunkHeader => {
                        // Skip headers and hunk headers - don't search these
                        continue;
                    }
                }
            }

            search_state.results = results;
            search_state.regex = Some(regex);
        }
        Ok(())
    }

    pub fn navigate_to_next_search_result(&mut self) -> Result<()> {
        if let Some(ref mut search_state) = self.diff_search_state {
            if search_state.results.is_empty() {
                return Ok(());
            }

            let next_index = match search_state.current_result {
                Some(idx) => (idx + 1) % search_state.results.len(),
                None => 0,
            };

            search_state.current_result = Some(next_index);
            self.scroll_to_search_result(next_index)?;
        }
        Ok(())
    }

    pub fn navigate_to_previous_search_result(&mut self) -> Result<()> {
        if let Some(ref mut search_state) = self.diff_search_state {
            if search_state.results.is_empty() {
                return Ok(());
            }

            let prev_index = match search_state.current_result {
                Some(idx) => {
                    if idx == 0 {
                        search_state.results.len() - 1
                    } else {
                        idx - 1
                    }
                }
                None => search_state.results.len() - 1,
            };

            search_state.current_result = Some(prev_index);
            self.scroll_to_search_result(prev_index)?;
        }
        Ok(())
    }

    pub fn scroll_to_search_result(&mut self, result_index: usize) -> Result<()> {
        if let Some(ref search_state) = self.diff_search_state {
            if let Some(search_match) = search_state.results.get(result_index) {
                // Scroll diff view to ensure the match is visible
                let target_line = search_match.line_index;
                let layout_mode = self.effective_layout();
                self.ui_state
                    .ensure_diff_line_visible(target_line, &layout_mode);
            }
        }
        Ok(())
    }

    pub fn clear_diff_search(&mut self) {
        self.diff_search_state = None;
    }
    pub fn open_editor(&mut self) -> Result<()> {
        let current_file = self.get_file_path().expect("a legit path in string.");
        let current_diff_cursor = self.ui_state.diff_cursor_line;

        let highlighted_diff = crate::diff::HighlightedDiff::new(
            &self.current_diff,
            self.get_file_path().map(|p| p.as_path()),
        );

        let diff_detail = highlighted_diff.lines[current_diff_cursor].clone();
        let current_line = diff_detail.new_line_num.unwrap_or(0);

        let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

        // Launch the editor asynchronously
        let mut cmd = Command::new(editor);
        cmd.arg(current_file) // pass current file path
            .arg(format!("+{}", current_line)); // pass +line number)
        cmd.status()?;
        Ok(())
    }
}
