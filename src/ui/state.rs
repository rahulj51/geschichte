#[derive(Debug)]
pub struct UIState {
    pub split_ratio: f32,
    pub show_help: bool,
    pub terminal_height: u16,
    pub terminal_width: u16,
    pub diff_scroll: usize,
    pub diff_horizontal_scroll: usize,
    pub commit_horizontal_scroll: usize,
    pub diff_cursor_line: usize,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            split_ratio: 0.4,
            show_help: false,
            terminal_height: 24,
            terminal_width: 80,
            diff_scroll: 0,
            diff_horizontal_scroll: 0,
            commit_horizontal_scroll: 0,
            diff_cursor_line: 0,
        }
    }

    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_height = height;
        self.terminal_width = width;

        // Recalculate scroll bounds if window got smaller
        if self.diff_horizontal_scroll > width as usize {
            self.diff_horizontal_scroll = (width as usize).saturating_sub(10);
        }
        if self.commit_horizontal_scroll > width as usize {
            self.commit_horizontal_scroll = (width as usize).saturating_sub(10);
        }
    }

    pub fn reset_diff_scroll(&mut self) {
        self.diff_scroll = 0;
        self.diff_horizontal_scroll = 0;
        self.diff_cursor_line = 0;
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

    pub fn get_page_scroll_size(&self) -> usize {
        // Calculate scroll size based on visible diff area
        // Accounting for borders (2 lines) and status bar (1 line)
        let visible_height = self.terminal_height.saturating_sub(3) as usize;
        // Use 60% of the visible area for diff (based on split_ratio)
        let diff_height = ((visible_height as f32) * (1.0 - self.split_ratio)) as usize;
        // Scroll by half a page for better readability
        diff_height.saturating_sub(2) / 2
    }

    // Scrolling methods
    pub fn scroll_diff_up(&mut self) {
        if self.diff_scroll > 0 {
            self.diff_scroll -= 1;
        }
    }

    pub fn scroll_diff_down(&mut self, max_lines: usize) {
        // Ensure we don't scroll past the content
        // Account for viewport height to prevent scrolling too far
        let viewport_height = self.get_visible_lines(&crate::cli::LayoutMode::Unified);
        let max_scroll = max_lines.saturating_sub(viewport_height);

        if self.diff_scroll < max_scroll {
            self.diff_scroll += 1;
        }
    }

    pub fn scroll_diff_page_up(&mut self) {
        let page_size = self.get_page_scroll_size();
        self.diff_scroll = self.diff_scroll.saturating_sub(page_size);
    }

    pub fn scroll_diff_page_down(&mut self, max_lines: usize) {
        let page_size = self.get_page_scroll_size();
        let viewport_height = self.get_visible_lines(&crate::cli::LayoutMode::Unified);
        let max_scroll = max_lines.saturating_sub(viewport_height);

        // Ensure we don't scroll past the content
        self.diff_scroll = (self.diff_scroll + page_size).min(max_scroll);
    }

    pub fn scroll_diff_left(&mut self) {
        self.diff_horizontal_scroll = self.diff_horizontal_scroll.saturating_sub(4);
    }

    pub fn scroll_diff_right(&mut self, max_width: usize) {
        if self.diff_horizontal_scroll + 4 < max_width {
            self.diff_horizontal_scroll += 4;
        }
    }

    pub fn scroll_commit_left(&mut self) {
        self.commit_horizontal_scroll = self.commit_horizontal_scroll.saturating_sub(4);
    }

    pub fn scroll_commit_right(&mut self, max_width: usize) {
        if self.commit_horizontal_scroll + 4 < max_width {
            self.commit_horizontal_scroll += 4;
        }
    }

    // Cursor navigation methods
    pub fn move_cursor_up(&mut self, layout_mode: &crate::cli::LayoutMode) {
        if self.diff_cursor_line > 0 {
            self.diff_cursor_line -= 1;
            self.ensure_cursor_visible(layout_mode);
        }
    }

    pub fn move_cursor_down(&mut self, max_lines: usize, layout_mode: &crate::cli::LayoutMode) {
        if max_lines > 0 && self.diff_cursor_line < max_lines - 1 {
            self.diff_cursor_line += 1;
            self.ensure_cursor_visible(layout_mode);
        }
    }

    pub fn get_visible_lines(&self, layout_mode: &crate::cli::LayoutMode) -> usize {
        // Calculate how many lines are visible in the diff area
        let visible_height = self.terminal_height.saturating_sub(1) as usize; // Account for status bar

        match layout_mode {
            crate::cli::LayoutMode::SideBySide => {
                // In side-by-side mode, diff takes 70% of height, minus borders
                let diff_height = ((visible_height as f32) * 0.7) as usize;
                diff_height.saturating_sub(2) // Account for panel borders
            }
            _ => {
                // In unified mode, diff area uses split ratio, minus borders
                let diff_height = ((visible_height as f32) * (1.0 - self.split_ratio)) as usize;
                diff_height.saturating_sub(2) // Account for panel borders
            }
        }
    }

    pub fn ensure_cursor_visible(&mut self, layout_mode: &crate::cli::LayoutMode) {
        let visible_lines = self.get_visible_lines(layout_mode);

        // If cursor is above the current scroll, scroll up
        if self.diff_cursor_line < self.diff_scroll {
            self.diff_scroll = self.diff_cursor_line;
        }
        // If cursor is below the visible area, scroll down
        else if self.diff_cursor_line >= self.diff_scroll + visible_lines {
            self.diff_scroll = self
                .diff_cursor_line
                .saturating_sub(visible_lines.saturating_sub(1));
        }
    }

    pub fn ensure_diff_line_visible(
        &mut self,
        target_line: usize,
        layout_mode: &crate::cli::LayoutMode,
    ) {
        let visible_lines = self.get_visible_lines(layout_mode);

        // If target line is above the current scroll, scroll up
        if target_line < self.diff_scroll {
            self.diff_scroll = target_line;
        }
        // If target line is below the visible area, scroll down to center it
        else if target_line >= self.diff_scroll + visible_lines {
            // Try to center the target line in the viewport
            let half_viewport = visible_lines / 2;
            self.diff_scroll = target_line.saturating_sub(half_viewport);
        }

        // Also update cursor position to the target line
        self.diff_cursor_line = target_line;
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}
