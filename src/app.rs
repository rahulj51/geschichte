use crate::error::Result;
use std::path::PathBuf;

pub struct App {
    pub repo_root: PathBuf,
    pub file_path: PathBuf,
    pub should_quit: bool,
    pub context_lines: u32,
    pub follow_renames: bool,
    pub first_parent: bool,
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
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Up | KeyCode::Char('k') => {
                // TODO: Move selection up
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // TODO: Move selection down
            }
            KeyCode::PageUp => {
                // TODO: Scroll diff up
            }
            KeyCode::PageDown => {
                // TODO: Scroll diff down
            }
            KeyCode::Char('/') => {
                // TODO: Start search
            }
            KeyCode::Char('?') => {
                // TODO: Show help
            }
            _ => {}
        }

        Ok(())
    }
}