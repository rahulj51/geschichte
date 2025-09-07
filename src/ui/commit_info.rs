use crate::commit::Commit;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct CommitInfoPopup {
    pub commit: Commit,
    pub scroll_position: usize,
}

impl CommitInfoPopup {
    pub fn new(commit: Commit) -> Self {
        Self {
            commit,
            scroll_position: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
        }
    }

    pub fn scroll_down(&mut self, max_lines: usize, viewport_height: usize) {
        if self.scroll_position + viewport_height < max_lines {
            self.scroll_position += 1;
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Create centered popup area
        let popup_area = self.centered_rect(80, 60, area);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        // Create main border
        let block = Block::default()
            .title(" Commit Details ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        frame.render_widget(block, popup_area);

        // Inner area for content
        let inner_area = popup_area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        });

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Metadata section
                Constraint::Min(5),    // Message section
                Constraint::Length(1), // Help line
            ])
            .split(inner_area);

        // Render metadata section
        self.render_metadata(frame, chunks[0]);

        // Render message section
        self.render_message(frame, chunks[1]);

        // Render help line
        self.render_help(frame, chunks[2]);
    }

    fn render_metadata(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();

        // Hash
        lines.push(Line::from(vec![
            Span::styled(
                "Hash:      ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&self.commit.hash, Style::default().fg(Color::Cyan)),
        ]));

        // Author
        lines.push(Line::from(vec![
            Span::styled(
                "Author:    ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(self.commit.author()),
        ]));

        // Author Date
        lines.push(Line::from(vec![
            Span::styled(
                "Date:      ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&self.commit.author_date),
        ]));

        // Committer (if different from author)
        if self.commit.committer_name != self.commit.author_name
            || self.commit.committer_email != self.commit.author_email
        {
            lines.push(Line::from(vec![
                Span::styled(
                    "Committer: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "{} <{}>",
                    self.commit.committer_name, self.commit.committer_email
                )),
            ]));

            if !self.commit.committer_date.is_empty()
                && self.commit.committer_date != self.commit.author_date
            {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Commit Date:",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&self.commit.committer_date),
                ]));
            }
        }

        // Refs (branches/tags)
        if !self.commit.refs.is_empty() {
            let refs_text = self.commit.refs.join(", ");
            lines.push(Line::from(vec![
                Span::styled(
                    "Refs:      ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({})", refs_text),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }

        // PR info
        if let Some(ref pr_info) = self.commit.pr_info {
            lines.push(Line::from(vec![
                Span::styled(
                    "PR:        ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("#{} - {}", pr_info.number, pr_info.title),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }

        // Stats
        if let Some(ref stats) = self.commit.stats {
            lines.push(Line::from(vec![
                Span::styled(
                    "Stats:     ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "{} files, +{} -{} lines",
                        stats.files_changed, stats.insertions, stats.deletions
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines).style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    fn render_message(&self, frame: &mut Frame, area: Rect) {
        // Message section with border
        let message_block = Block::default()
            .title(" Full Message ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Blue));

        let message_area = area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        frame.render_widget(message_block, area);

        // Prepare message content
        let mut message_lines = Vec::new();

        if !self.commit.body.is_empty() {
            // Only show body lines (subject is already shown in metadata above)
            for line in self.commit.body.lines() {
                message_lines.push(Line::from(line.to_string()));
            }
        } else {
            // Show a message when there's no additional body content
            message_lines.push(Line::from(vec![Span::styled(
                "(No additional message content)",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        // Calculate visible lines based on scroll position
        let visible_height = message_area.height as usize;
        let start_line = self.scroll_position;
        let end_line = (start_line + visible_height).min(message_lines.len());

        let visible_lines = if start_line < message_lines.len() {
            message_lines[start_line..end_line].to_vec()
        } else {
            Vec::new()
        };

        let paragraph = Paragraph::new(visible_lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, message_area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = "[↑↓/jk] Scroll  [c] Copy hash  [m] Copy message  [q] Close";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(help, area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    pub fn get_total_lines(&self) -> usize {
        if !self.commit.body.is_empty() {
            self.commit.body.lines().count()
        } else {
            1 // "(No additional message content)" line
        }
    }
}
