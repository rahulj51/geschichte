use crate::app::{App, FocusedPanel};
use crate::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub fn handle_navigation_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                if let Some(focused_panel) = self.get_focused_panel() {
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
                if let Some(focused_panel) = self.get_focused_panel() {
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
            _ => Ok(false)
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
                self.ui_state.scroll_diff_page_down();
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
                self.ui_state.scroll_diff_page_down();
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
                self.ui_state.scroll_diff_page_down();
                Ok(true)
            }
            // Horizontal scrolling
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                if let Some(focused_panel) = self.get_focused_panel() {
                    match focused_panel {
                        FocusedPanel::Commits => self.ui_state.scroll_commit_left(),
                        FocusedPanel::Diff => self.ui_state.scroll_diff_left(),
                    }
                }
                Ok(true)
            }
            (KeyCode::Char('s'), KeyModifiers::NONE) => {
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
            _ => Ok(false)
        }
    }

    pub fn handle_ui_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.quit();
                Ok(true)
            }
            (KeyCode::Esc, _) => {
                if self.ui_state.show_help {
                    self.ui_state.show_help = false;
                } else if self.diff_range_start.is_some() {
                    self.clear_diff_range_selection();
                } else {
                    self.quit();
                }
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
                // TODO: Start search
                Ok(true)
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                self.toggle_diff_range_selection()?;
                Ok(true)
            }
            (KeyCode::Char('?'), KeyModifiers::NONE) => {
                self.ui_state.toggle_help();
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}