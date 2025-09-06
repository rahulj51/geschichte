use crate::app::{App, AppMode, FilePickerContext, FocusedPanel};
use crate::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub fn handle_navigation_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                if self.show_commit_info {
                    self.scroll_commit_info_up();
                } else if let Some(focused_panel) = self.get_focused_panel() {
                    match focused_panel {
                        FocusedPanel::Commits => self.move_selection_up()?,
                        FocusedPanel::Diff => {
                            let layout_mode = self.effective_layout();
                            self.ui_state.move_cursor_up(&layout_mode);
                        }
                    }
                }
                Ok(true)
            }
            (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                if self.show_commit_info {
                    self.scroll_commit_info_down();
                } else if let Some(focused_panel) = self.get_focused_panel() {
                    match focused_panel {
                        FocusedPanel::Commits => self.move_selection_down()?,
                        FocusedPanel::Diff => {
                            let max_lines = self.get_diff_line_count();
                            let layout_mode = self.effective_layout();
                            self.ui_state.move_cursor_down(max_lines, &layout_mode);
                        }
                    }
                }
                Ok(true)
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.switch_focus();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn handle_scrolling_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::PageUp, _) => {
                // Always scroll diff for PageUp/PageDown regardless of focus
                self.ui_state.scroll_diff_page_up();
                Ok(true)
            }
            (KeyCode::PageDown, _) => {
                // Always scroll diff for PageUp/PageDown regardless of focus
                let max_lines = self.get_diff_line_count();
                self.ui_state.scroll_diff_page_down(max_lines);
                Ok(true)
            }
            // Mac-friendly vim-style navigation
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                // Ctrl+U = Page Up (vim-style)
                self.ui_state.scroll_diff_page_up();
                Ok(true)
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                // Ctrl+D = Page Down (vim-style)
                let max_lines = self.get_diff_line_count();
                self.ui_state.scroll_diff_page_down(max_lines);
                Ok(true)
            }
            // Mac-friendly emacs-style navigation
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                // Ctrl+B = Page Up (emacs-style)
                self.ui_state.scroll_diff_page_up();
                Ok(true)
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                // Ctrl+F = Page Down (emacs-style)
                let max_lines = self.get_diff_line_count();
                self.ui_state.scroll_diff_page_down(max_lines);
                Ok(true)
            }
            // Horizontal scrolling (but not when in copy mode)
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                // Don't handle 'a' for scrolling when in copy mode
                if self.copy_mode.is_some() {
                    return Ok(false); // Let copy handler deal with it
                }
                if let Some(focused_panel) = self.get_focused_panel() {
                    match focused_panel {
                        FocusedPanel::Commits => self.ui_state.scroll_commit_left(),
                        FocusedPanel::Diff => self.ui_state.scroll_diff_left(),
                    }
                }
                Ok(true)
            }
            (KeyCode::Char('s'), KeyModifiers::NONE) => {
                // Don't handle 's' for scrolling when in copy mode
                if self.copy_mode.is_some() {
                    return Ok(false); // Let copy handler deal with it
                }
                if let Some(focused_panel) = self.get_focused_panel() {
                    match focused_panel {
                        FocusedPanel::Commits => {
                            let max_width = self.calculate_max_commit_line_width();
                            self.ui_state.scroll_commit_right(max_width);
                        }
                        FocusedPanel::Diff => {
                            let max_width = self.calculate_max_diff_line_width();
                            self.ui_state.scroll_diff_right(max_width);
                        }
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn handle_ui_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                if self.show_commit_info {
                    self.hide_commit_info_popup();
                } else {
                    self.quit();
                }
                Ok(true)
            }
            (KeyCode::Esc, _) => {
                if self.ui_state.show_help {
                    self.ui_state.show_help = false;
                } else if self.show_commit_info {
                    self.hide_commit_info_popup();
                } else if self.diff_search_state.is_some() {
                    self.clear_diff_search();
                } else if self.copy_mode.is_some() {
                    self.cancel_copy_mode();
                } else if self.diff_range_start.is_some() {
                    self.clear_diff_range_selection();
                } else {
                    // HACK: revert the FilePicker context to Initial.
                    use crate::app::FilePickerState;
                    use crate::git::files::get_git_files;
                    let files = get_git_files(&self.repo_root)?;
                    self.mode = AppMode::FilePicker {
                        state: FilePickerState::new(files),
                        context: FilePickerContext::Initial,
                    };
                    // FIX: I wish it could be more like this tho.
                    // self.mode = AppMode::FilePicker {
                    //     state: mode.state,
                    //     context: FilePickerContext::Initial,
                    // };
                    if let Err(e) = self.switch_to_file_picker() {
                        self.error_message = Some(format!("Failed to open file picker: {}", e));
                    }
                }
                // else {
                //     self.quit();
                // }
                Ok(true)
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.ui_state.decrease_split_ratio();
                Ok(true)
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.ui_state.increase_split_ratio();
                Ok(true)
            }
            (KeyCode::Char('f'), KeyModifiers::NONE) => {
                // Open file picker to switch files
                if let Err(e) = self.switch_to_file_picker() {
                    self.error_message = Some(format!("Failed to open file picker: {}", e));
                }
                Ok(true)
            }
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                if !self.show_commit_info && self.get_focused_panel() == Some(FocusedPanel::Diff) {
                    self.start_diff_search();
                    Ok(true)
                } else {
                    Ok(false) // Let other handlers deal with it
                }
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                self.toggle_diff_range_selection()?;
                Ok(true)
            }
            (KeyCode::Char('?'), KeyModifiers::NONE) => {
                self.ui_state.toggle_help();
                Ok(true)
            }
            (KeyCode::Char('i'), KeyModifiers::NONE) | (KeyCode::Enter, KeyModifiers::NONE) => {
                // Show commit info popup (only in commits panel)
                if matches!(self.get_focused_panel(), Some(FocusedPanel::Commits)) {
                    self.show_commit_info_popup()?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn handle_copy_keys(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle copy keys in commits panel and history mode, or in commit info popup
        if !matches!(self.get_focused_panel(), Some(FocusedPanel::Commits))
            && !self.show_commit_info
        {
            return Ok(false);
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('y'), KeyModifiers::NONE) => {
                match self.copy_mode.as_ref() {
                    None => {
                        // First 'y' press - start copy mode
                        self.start_copy_mode();
                    }
                    Some(crate::copy::CopyMode::WaitingForTarget) => {
                        // Second 'y' press - copy full SHA
                        self.copy_commit_sha(false)?;
                    }
                }
                Ok(true)
            }
            (KeyCode::Char('Y'), KeyModifiers::SHIFT) => {
                // Capital Y - copy short SHA directly
                self.copy_commit_sha(true)?;
                Ok(true)
            }
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                // Direct copy of full SHA (especially useful in popup)
                if self.show_commit_info {
                    self.copy_commit_sha(false)?;
                } else {
                    // Start copy mode in normal view
                    self.start_copy_mode();
                }
                Ok(true)
            }
            // Note: 'm' key is only handled in copy mode section below
            _ => {
                // Handle copy mode targets
                if matches!(
                    self.copy_mode,
                    Some(crate::copy::CopyMode::WaitingForTarget)
                ) {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('s'), KeyModifiers::NONE) => {
                            self.copy_commit_sha(false)?;
                            Ok(true)
                        }
                        (KeyCode::Char('h'), KeyModifiers::NONE) => {
                            self.copy_commit_sha(true)?;
                            Ok(true)
                        }
                        (KeyCode::Char('m'), KeyModifiers::NONE) => {
                            self.copy_commit_message()?;
                            Ok(true)
                        }
                        (KeyCode::Char('a'), KeyModifiers::NONE) => {
                            self.copy_commit_author()?;
                            Ok(true)
                        }
                        (KeyCode::Char('d'), KeyModifiers::NONE) => {
                            self.copy_commit_date()?;
                            Ok(true)
                        }
                        (KeyCode::Char('u'), KeyModifiers::NONE) => {
                            self.copy_github_url()?;
                            Ok(true)
                        }
                        (KeyCode::Char('p'), KeyModifiers::NONE) => {
                            self.copy_file_relative_path()?;
                            Ok(true)
                        }

                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub fn handle_change_navigation_keys(&mut self, key: KeyEvent) -> Result<bool> {
        // Check if we're in active search mode first
        if let Some(ref search_state) = self.diff_search_state {
            if search_state.is_active
                && !search_state.is_input_mode
                && !search_state.results.is_empty()
            {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('n'), KeyModifiers::NONE) => {
                        self.navigate_to_next_search_result()?;
                        return Ok(true);
                    }
                    (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
                        self.navigate_to_previous_search_result()?;
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }

        // Existing hunk navigation logic
        match (key.code, key.modifiers) {
            (KeyCode::Char('n'), KeyModifiers::NONE) => {
                if self.copy_mode.is_some() {
                    // Don't conflict with copy mode
                    return Ok(false);
                }
                self.navigate_to_next_change()?;
                Ok(true)
            }
            (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
                self.navigate_to_previous_change()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn handle_search_input_keys(&mut self, key: KeyEvent) -> Result<bool> {
        if let Some(ref mut search_state) = self.diff_search_state {
            if !search_state.is_input_mode {
                return Ok(false);
            }

            match (key.code, key.modifiers) {
                (KeyCode::Char(c), KeyModifiers::NONE) => {
                    search_state.query.push(c);
                    self.update_search_results()?;
                    Ok(true)
                }
                (KeyCode::Backspace, KeyModifiers::NONE) => {
                    search_state.query.pop();
                    self.update_search_results()?;
                    Ok(true)
                }
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    search_state.is_input_mode = false;
                    if !search_state.results.is_empty() {
                        search_state.current_result = Some(0);
                        self.scroll_to_search_result(0)?;
                    }
                    Ok(true)
                }
                (KeyCode::Esc, _) => {
                    self.clear_diff_search();
                    Ok(true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}
