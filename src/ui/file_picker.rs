use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::git::files::{format_file_size, format_modified_time, GitFile};

pub struct FilePickerState {
    pub files: Vec<GitFile>,
    pub filtered_files: Vec<(usize, Vec<usize>)>, // (file_index, highlight_indices)
    pub query: String,
    pub selected: usize,
    matcher: SkimMatcherV2,
}

impl std::fmt::Debug for FilePickerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilePickerState")
            .field("files", &self.files.len())
            .field("filtered_files", &self.filtered_files.len())
            .field("query", &self.query)
            .field("selected", &self.selected)
            .field("matcher", &"SkimMatcherV2")
            .finish()
    }
}

impl Clone for FilePickerState {
    fn clone(&self) -> Self {
        Self {
            files: self.files.clone(),
            filtered_files: self.filtered_files.clone(),
            query: self.query.clone(),
            selected: self.selected,
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl FilePickerState {
    pub fn new(files: Vec<GitFile>) -> Self {
        let mut state = Self {
            files,
            filtered_files: Vec::new(),
            query: String::new(),
            selected: 0,
            matcher: SkimMatcherV2::default(),
        };
        
        // Initially show all files
        state.update_filter();
        state
    }

    #[allow(dead_code)]
    pub fn update_query(&mut self, query: String) {
        self.query = query;
        self.selected = 0;
        self.update_filter();
    }

    pub fn append_char(&mut self, c: char) {
        self.query.push(c);
        self.selected = 0;
        self.update_filter();
    }

    pub fn delete_char(&mut self) {
        self.query.pop();
        self.selected = 0;
        self.update_filter();
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.selected = 0;
        self.update_filter();
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected < self.filtered_files.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn get_selected_file(&self) -> Option<&GitFile> {
        if let Some((file_index, _)) = self.filtered_files.get(self.selected) {
            self.files.get(*file_index)
        } else {
            None
        }
    }

    fn update_filter(&mut self) {
        self.filtered_files.clear();

        if self.query.is_empty() {
            // Show all files when no query
            self.filtered_files = self.files
                .iter()
                .enumerate()
                .map(|(i, _)| (i, Vec::new()))
                .collect();
        } else {
            // Fuzzy match against display path
            let mut matches: Vec<_> = self.files
                .iter()
                .enumerate()
                .filter_map(|(i, file)| {
                    if let Some((score, indices)) = self.matcher.fuzzy_indices(&file.display_path, &self.query) {
                        Some((score, i, indices))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by score (higher is better)
            matches.sort_by(|a, b| b.0.cmp(&a.0));

            // Take the best matches
            self.filtered_files = matches
                .into_iter()
                .map(|(_, file_index, indices)| (file_index, indices))
                .collect();
        }

    }
}

pub fn draw_file_picker(frame: &mut Frame, state: &FilePickerState, context: &crate::app::FilePickerContext, area: Rect) {
    // Calculate popup size (80% of screen, but at least 60x20)
    let popup_width = (area.width as f32 * 0.8).max(60.0) as u16;
    let popup_height = (area.height as f32 * 0.8).max(20.0) as u16;
    
    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Clear the background
    frame.render_widget(Clear, popup_area);

    // Main popup block with context-specific title
    let title = match context {
        crate::app::FilePickerContext::Initial => " Select File ",
        crate::app::FilePickerContext::SwitchFile { .. } => " Switch to File ",
    };
    
    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Black));

    frame.render_widget(popup_block, popup_area);

    // Split popup into search, list, and status areas
    let inner_area = popup_area.inner(Margin::new(1, 1));
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(0),    // File list
            Constraint::Length(1), // Status line
        ])
        .split(inner_area);

    // Search box
    draw_search_box(frame, state, popup_chunks[0]);

    // File list
    draw_file_list(frame, state, popup_chunks[1]);

    // Status line
    draw_status_line(frame, state, context, popup_chunks[2]);
}

fn draw_search_box(frame: &mut Frame, state: &FilePickerState, area: Rect) {
    let search_block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let search_area = area.inner(Margin::new(1, 1));
    
    // Show search icon and query with cursor
    let search_text = format!("🔍 {}", state.query);
    let search_paragraph = Paragraph::new(search_text)
        .style(Style::default().fg(Color::White))
        .block(search_block);

    frame.render_widget(search_paragraph, area);

    // Position cursor at end of query
    let cursor_x = search_area.x + 3 + state.query.len() as u16; // 3 for "🔍 "
    let cursor_y = search_area.y;
    if cursor_x < area.x + area.width.saturating_sub(1) {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_file_list(frame: &mut Frame, state: &FilePickerState, area: Rect) {
    let list_items: Vec<ListItem> = state.filtered_files
        .iter()
        .enumerate()
        .map(|(_i, (file_index, highlight_indices))| {
            let file = &state.files[*file_index];
            
            // Create highlighted file path
            let highlighted_path = create_highlighted_text(&file.display_path, highlight_indices);
            
            // File status symbol and metadata
            let status_symbol = file.status.symbol();
            let status_color = file.status.style_color();
            let modified = format_modified_time(file.modified);
            let _size = format_file_size(file.size);

            // Create the line with proper spacing
            let mut spans = vec![
                Span::styled(
                    format!("{} ", status_symbol),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ];
            spans.extend(highlighted_path);
            
            // Add metadata if there's space (simplified for now)
            let metadata = format!(" {}", modified);
            spans.push(Span::styled(
                metadata,
                Style::default().fg(Color::Gray),
            ));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let files_list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // Create list state on the fly based on current selection
    let mut list_state = ListState::default();
    if !state.filtered_files.is_empty() && state.selected < state.filtered_files.len() {
        list_state.select(Some(state.selected));
    }

    frame.render_stateful_widget(files_list, area, &mut list_state);
}

fn draw_status_line(frame: &mut Frame, state: &FilePickerState, context: &crate::app::FilePickerContext, area: Rect) {
    let total_files = state.files.len();
    let filtered_count = state.filtered_files.len();
    
    // Add context information
    let context_info = match context {
        crate::app::FilePickerContext::Initial => "",
        crate::app::FilePickerContext::SwitchFile { previous_file } => {
            &format!(" • Current: {}", previous_file.file_name().unwrap_or_default().to_string_lossy())
        }
    };
    
    // Context-aware escape message
    let esc_action = match context {
        crate::app::FilePickerContext::Initial => "quit",
        crate::app::FilePickerContext::SwitchFile { .. } => "return",
    };
    
    let status_text = if state.query.is_empty() {
        format!("📁 {} files{} • ↑↓/^P^N: navigate • Enter: select • Esc: {} • Type to search", total_files, context_info, esc_action)
    } else {
        format!("📁 {} files • {} matches{} • ↑↓/^P^N: navigate • Enter: select • Esc: {}", total_files, filtered_count, context_info, esc_action)
    };

    let status_paragraph = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(status_paragraph, area);
}

fn create_highlighted_text<'a>(text: &'a str, highlight_indices: &[usize]) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut last_idx = 0;

    for &idx in highlight_indices {
        if idx < chars.len() {
            // Add non-highlighted text before this character
            if last_idx < idx {
                let segment: String = chars[last_idx..idx].iter().collect();
                if !segment.is_empty() {
                    spans.push(Span::styled(segment, Style::default().fg(Color::White)));
                }
            }

            // Add highlighted character
            let highlighted_char = chars[idx].to_string();
            spans.push(Span::styled(
                highlighted_char,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));

            last_idx = idx + 1;
        }
    }

    // Add remaining non-highlighted text
    if last_idx < chars.len() {
        let segment: String = chars[last_idx..].iter().collect();
        if !segment.is_empty() {
            spans.push(Span::styled(segment, Style::default().fg(Color::White)));
        }
    }

    // If no highlights were added, return the original text
    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), Style::default().fg(Color::White)));
    }

    spans
}