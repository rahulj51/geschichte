use crate::cache::DiffCache;
use crate::commit::Commit;
use crate::error::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    Commits,
    Diff,
}

pub struct App {
    pub repo_root: PathBuf,
    pub file_path: PathBuf,
    pub should_quit: bool,
    pub context_lines: u32,
    pub follow_renames: bool,
    pub first_parent: bool,
    
    // Commit data
    pub commits: Vec<Commit>,
    pub selected_index: usize,
    pub rename_map: HashMap<String, PathBuf>,
    
    // Diff display
    pub current_diff: String,
    pub diff_scroll: usize,
    pub diff_cache: DiffCache,
    
    // UI state
    pub focused_panel: FocusedPanel,
    pub split_ratio: f32,
    pub show_help: bool,
    pub loading: bool,
    pub error_message: Option<String>,
}

impl App {
    pub fn new(
        repo_root: PathBuf,
        file_path: PathBuf,
        context_lines: u32,
        follow_renames: bool,
        first_parent: bool,
    ) -> Self {
        Self {
            repo_root,
            file_path,
            should_quit: false,
            context_lines,
            follow_renames,
            first_parent,
            commits: Vec::new(),
            selected_index: 0,
            rename_map: HashMap::new(),
            current_diff: String::new(),
            diff_scroll: 0,
            diff_cache: DiffCache::new(50),
            focused_panel: FocusedPanel::Commits,
            split_ratio: 0.4, // 40% commits, 60% diff
            show_help: false,
            loading: false,
            error_message: None,
        }
    }

    pub fn load_git_data(&mut self) -> Result<()> {
        self.loading = true;
        self.error_message = None;

        // Load commits
        let mut commits = crate::git::history::fetch_commit_history(
            &self.repo_root,
            &self.file_path,
            self.follow_renames,
            self.first_parent,
        )?;

        // Check for working directory changes and prepend if found
        let wd_status = crate::git::working::check_working_directory_status(
            &self.repo_root,
            &self.file_path,
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
                &self.file_path,
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
                &self.file_path,
                self.context_lines,
            )?
        } else {
            // Handle regular commit diff
            let parents = crate::git::history::get_commit_parents(&self.repo_root, &commit.hash)?;
            let parent_hash = parents.first().map(|s| s.as_str());

            // Resolve file path at this commit
            let file_path = self.rename_map
                .get(&commit.hash)
                .cloned()
                .unwrap_or_else(|| self.file_path.clone());

            crate::git::diff::fetch_diff(
                &self.repo_root,
                &commit.hash,
                parent_hash,
                &file_path,
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
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Commits => FocusedPanel::Diff,
            FocusedPanel::Diff => FocusedPanel::Commits,
        };
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
                match self.focused_panel {
                    FocusedPanel::Commits => self.move_selection_up()?,
                    FocusedPanel::Diff => self.scroll_diff_up(),
                }
            }
            (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                match self.focused_panel {
                    FocusedPanel::Commits => self.move_selection_down()?,
                    FocusedPanel::Diff => self.scroll_diff_down(),
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
}