#[derive(Debug)]
pub struct UIState {
    pub split_ratio: f32,
    pub show_help: bool,
    pub terminal_height: u16,
    pub terminal_width: u16,
    pub diff_scroll: usize,
    pub diff_horizontal_scroll: usize,
    pub commit_horizontal_scroll: usize,
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
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}