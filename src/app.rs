use crate::cache::DiffCache;
use crate::commit::Commit;
use crate::error::Result;
use crate::ui::file_picker::FilePickerState;
use std::collections::HashMap;
use std::path::PathBuf;

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
    Initial, // Started with no file argument
    SwitchFile { previous_file: PathBuf }, // Switching from an existing file
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
    pub diff_scroll: usize,
    pub diff_cache: DiffCache,
    
    // UI state
    pub split_ratio: f32,
    pub show_help: bool,
    pub loading: bool,
    pub error_message: Option<String>,
    pub terminal_height: u16,
    
    // Diff range selection
    pub diff_range_start: Option<usize>,
    pub current_diff_range: Option<(usize, usize)>, // (older_index, newer_index)
}

impl App {
    pub fn new_file_picker(
        repo_root: PathBuf,
        context_lines: u32,
        follow_renames: bool,
        first_parent: bool,
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
            diff_scroll: 0,
            diff_cache: DiffCache::new(50),
            split_ratio: 0.4,
            show_help: false,
            loading: false,
            error_message: None,
            terminal_height: 24,
            diff_range_start: None,
            current_diff_range: None,
        })
    }

    pub fn new_history(
        repo_root: PathBuf,
        file_path: PathBuf,
        context_lines: u32,
        follow_renames: bool,
        first_parent: bool,
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
            diff_scroll: 0,
            diff_cache: DiffCache::new(50),
            split_ratio: 0.4, // 40% commits, 60% diff
            show_help: false,
            loading: false,
            error_message: None,
            terminal_height: 24,
            diff_range_start: None,
            current_diff_range: None,
        }
    }

    pub fn switch_to_history(&mut self, file_path: PathBuf) -> Result<()> {
        self.mode = AppMode::History {
            file_path,
            focused_panel: FocusedPanel::Commits,
        };
        
        // Clear existing data
        self.commits.clear();
        self.selected_index = 0;
        self.rename_map.clear();
        self.current_diff.clear();
        self.diff_scroll = 0;
        self.diff_cache.clear();
        
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

        Ok(())
    }

    pub fn return_to_previous_file(&mut self) -> Result<()> {
        // Only works if we're in file picker with SwitchFile context
        let previous_file = match &self.mode {
            AppMode::FilePicker { context: FilePickerContext::SwitchFile { previous_file }, .. } => {
                previous_file.clone()
            }
            _ => return Ok(()), // No-op if not in the right context
        };

        // Switch back to the previous file
        self.switch_to_history(previous_file)
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
        let wd_status = crate::git::working::check_working_directory_status(
            &self.repo_root,
            &file_path,
        )?;

        if wd_status != crate::git::working::WorkingDirectoryStatus::Clean {
            let status_text = match wd_status {
                crate::git::working::WorkingDirectoryStatus::Modified => "Modified".to_string(),
                crate::git::working::WorkingDirectoryStatus::Staged => "Staged".to_string(),
                crate::git::working::WorkingDirectoryStatus::ModifiedAndStaged => "Modified + Staged".to_string(),
                crate::git::working::WorkingDirectoryStatus::Clean => unreachable!(),
            };
            
            let wd_commit = crate::commit::Commit::new_working_directory(status_text);
            commits.insert(0, wd_commit);
        }

        self.commits = commits;

        // Build rename map
        if self.follow_renames {
            self.rename_map = crate::git::history::build_rename_map(
                &self.repo_root,
                &file_path,
            )?;
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
            self.current_diff = cached_diff;
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
            let commit_file_path = self.rename_map
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
        self.current_diff = diff;

        Ok(())
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
            self.load_diff_for_selected_commit()?;
        }
        Ok(())
    }

    pub fn scroll_diff_up(&mut self) {
        if self.diff_scroll > 0 {
            self.diff_scroll -= 1;
        }
    }

    pub fn scroll_diff_down(&mut self) {
        self.diff_scroll += 1;
    }

    pub fn scroll_diff_page_up(&mut self) {
        let page_size = self.get_page_scroll_size();
        self.diff_scroll = self.diff_scroll.saturating_sub(page_size);
    }

    pub fn scroll_diff_page_down(&mut self) {
        let page_size = self.get_page_scroll_size();
        self.diff_scroll += page_size;
    }

    pub fn get_page_scroll_size(&self) -> usize {
        // Calculate scroll size based on visible diff area
        // Accounting for borders (2 lines) and status bar (1 line)
        let visible_height = self.terminal_height.saturating_sub(3) as usize;
        // Use 60% of the visible area for diff (based on split_ratio)
        let diff_height = ((visible_height as f32) * (1.0 - self.split_ratio)) as usize;
        // Scroll by half a page for better readability
        diff_height.saturating_sub(2) / 2
    }

    pub fn update_terminal_height(&mut self, height: u16) {
        self.terminal_height = height;
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

    pub fn increase_split_ratio(&mut self) {
        self.split_ratio = (self.split_ratio + 0.05).min(0.7);
    }

    pub fn decrease_split_ratio(&mut self) {
        self.split_ratio = (self.split_ratio - 0.05).max(0.2);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
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
        if self.commits.is_empty() || start_index >= self.commits.len() || end_index >= self.commits.len() {
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
        if let Some(cached_diff) = self.diff_cache.get(&cache_key) {
            self.current_diff = cached_diff.clone();
            self.diff_scroll = 0;
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
        self.current_diff = diff;
        self.diff_scroll = 0;
        
        // Store the current range for UI display
        self.current_diff_range = Some((older_index, newer_index));

        Ok(())
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Handle file picker mode separately
        if matches!(self.mode, AppMode::FilePicker { .. }) {
            return self.handle_file_picker_key(key);
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) => self.quit(),
            (KeyCode::Esc, _) => {
                if self.show_help {
                    self.show_help = false;
                } else if self.diff_range_start.is_some() {
                    self.clear_diff_range_selection();
                } else {
                    self.quit();
                }
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.switch_focus();
            }
            (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                if let Some(focused_panel) = self.get_focused_panel() {
                    match focused_panel {
                        FocusedPanel::Commits => self.move_selection_up()?,
                        FocusedPanel::Diff => self.scroll_diff_up(),
                    }
                }
            }
            (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                if let Some(focused_panel) = self.get_focused_panel() {
                    match focused_panel {
                        FocusedPanel::Commits => self.move_selection_down()?,
                        FocusedPanel::Diff => self.scroll_diff_down(),
                    }
                }
            }
            (KeyCode::PageUp, _) => {
                // Always scroll diff for PageUp/PageDown regardless of focus
                self.scroll_diff_page_up();
            }
            (KeyCode::PageDown, _) => {
                // Always scroll diff for PageUp/PageDown regardless of focus
                self.scroll_diff_page_down();
            }
            // Mac-friendly vim-style navigation
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                // Ctrl+U = Page Up (vim-style)
                self.scroll_diff_page_up();
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                // Ctrl+D = Page Down (vim-style)
                self.scroll_diff_page_down();
            }
            // Mac-friendly emacs-style navigation
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                // Ctrl+B = Page Up (emacs-style)
                self.scroll_diff_page_up();
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                // Ctrl+F = Page Down (emacs-style)
                self.scroll_diff_page_down();
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.decrease_split_ratio();
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.increase_split_ratio();
            }
            (KeyCode::Char('f'), KeyModifiers::NONE) => {
                // Open file picker to switch files
                if let Err(e) = self.switch_to_file_picker() {
                    self.error_message = Some(format!("Failed to open file picker: {}", e));
                }
            }
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                // TODO: Start search
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                self.toggle_diff_range_selection()?;
            }
            (KeyCode::Char('?'), KeyModifiers::NONE) => {
                self.toggle_help();
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_file_picker_key(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // Special commands first
            (KeyCode::Esc, _) => {
                // Context-aware escape behavior
                match &self.mode {
                    AppMode::FilePicker { context: FilePickerContext::Initial, .. } => {
                        // Initial file picker (no file argument) - quit app
                        self.quit();
                    }
                    AppMode::FilePicker { context: FilePickerContext::SwitchFile { .. }, .. } => {
                        // Switching files - return to previous file
                        if let Err(e) = self.return_to_previous_file() {
                            self.error_message = Some(format!("Failed to return to previous file: {}", e));
                        }
                    }
                    _ => {
                        // Shouldn't happen in file picker mode, but safe fallback
                        self.quit();
                    }
                }
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                // Select file and switch to history mode
                if let AppMode::FilePicker { ref state, .. } = self.mode {
                    if let Some(selected_file) = state.get_selected_file() {
                        let file_path = selected_file.path.clone();
                        self.switch_to_history(file_path)?;
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
                if let AppMode::FilePicker { ref mut state, .. } = self.mode {
                    state.append_char(c);
                }
            }
            
            _ => {}
        }
        Ok(())
    }
}