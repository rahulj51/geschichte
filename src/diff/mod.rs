pub mod parser;

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

#[derive(Debug, Clone)]
pub enum DiffLineType {
    Header,
    HunkHeader,
    Addition,
    Deletion,
    Context,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
}

impl DiffLine {
    pub fn new(content: String) -> Self {
        let line_type = if content.starts_with("diff --git") || content.starts_with("index ") {
            DiffLineType::Header
        } else if content.starts_with("@@") {
            DiffLineType::HunkHeader
        } else if content.starts_with('+') && !content.starts_with("+++") {
            DiffLineType::Addition
        } else if content.starts_with('-') && !content.starts_with("---") {
            DiffLineType::Deletion
        } else {
            DiffLineType::Context
        };

        Self { line_type, content }
    }

    pub fn to_styled_line(&self) -> Line {
        let style = match self.line_type {
            DiffLineType::Header => Style::default().fg(Color::Blue),
            DiffLineType::HunkHeader => Style::default().fg(Color::Cyan),
            DiffLineType::Addition => Style::default().fg(Color::Green),
            DiffLineType::Deletion => Style::default().fg(Color::Red),
            DiffLineType::Context => Style::default().fg(Color::Gray),
        };

        Line::from(vec![Span::styled(&self.content, style)])
    }
}

pub fn parse_diff(diff_text: &str) -> Vec<DiffLine> {
    diff_text
        .lines()
        .map(|line| DiffLine::new(line.to_string()))
        .collect()
}