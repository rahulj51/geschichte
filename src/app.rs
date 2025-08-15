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
            self.load_diff_for_selected_commit()?;
        }
        Ok(())
    }

    pub fn move_selection_down(&mut self) -> Result<()> {
        if self.selected_index + 1 < self.commits.len() {
            self.selected_index += 1;
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
                for _ in 0..5 {
                    self.scroll_diff_up();
                }
            }
            (KeyCode::PageDown, _) => {
                // Always scroll diff for PageUp/PageDown regardless of focus
                for _ in 0..5 {
                    self.scroll_diff_down();
                }
            }
            // Mac-friendly vim-style navigation
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                // Ctrl+U = Page Up (vim-style)
                for _ in 0..5 {
                    self.scroll_diff_up();
                }
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                // Ctrl+D = Page Down (vim-style)
                for _ in 0..5 {
                    self.scroll_diff_down();
                }
            }
            // Mac-friendly emacs-style navigation
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                // Ctrl+B = Page Up (emacs-style)
                for _ in 0..5 {
                    self.scroll_diff_up();
                }
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                // Ctrl+F = Page Down (emacs-style)
                for _ in 0..5 {
                    self.scroll_diff_down();
                }
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