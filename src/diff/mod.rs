pub mod parser;
pub mod syntax;

use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Line, Span};
use std::path::Path;

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
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
}

/// Enhanced diff with syntax highlighting
pub struct HighlightedDiff {
    pub lines: Vec<DiffLine>,
    file_path: Option<std::path::PathBuf>,
}

impl HighlightedDiff {
    pub fn new(diff_text: &str, file_path: Option<&Path>) -> Self {
        let lines = parse_diff(diff_text);
        let file_path = file_path.map(|p| p.to_path_buf());
        
        Self { lines, file_path }
    }
    
    pub fn to_styled_lines(&self) -> Vec<Line<'static>> {
        self.lines.iter()
            .map(|line| self.style_diff_line(line))
            .collect()
    }
    
    fn style_diff_line(&self, line: &DiffLine) -> Line<'static> {
        match line.line_type {
            DiffLineType::Header => {
                // File headers in bold blue - no line numbers
                Line::from(vec![
                    Span::styled("         ".to_string(), Style::default()), // Space for line numbers (4+1+4+1=10 chars)
                    Span::styled(line.content.clone(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
                ])
            }
            DiffLineType::HunkHeader => {
                // Hunk headers in cyan - no line numbers
                Line::from(vec![
                    Span::styled("         ".to_string(), Style::default()), // Space for line numbers (4+1+4+1=10 chars)
                    Span::styled(line.content.clone(), Style::default().fg(Color::Cyan))
                ])
            }
            DiffLineType::Addition | DiffLineType::Deletion | DiffLineType::Context => {
                // Apply syntax highlighting to code content
                let mut spans = Vec::new();
                
                // Add line numbers (old file | new file)
                let old_num_str = match line.old_line_num {
                    Some(num) => format!("{:>4}", num),
                    None => "    ".to_string(),
                };
                let new_num_str = match line.new_line_num {
                    Some(num) => format!("{:>4}", num),
                    None => "    ".to_string(),
                };
                
                spans.push(Span::styled(
                    format!("{}│{} ", old_num_str, new_num_str),
                    Style::default().fg(Color::DarkGray)
                ));
                
                // Add the diff marker with appropriate color
                let (marker, marker_color, bg_color) = match line.line_type {
                    DiffLineType::Addition => ("+", Color::Green, Some(Color::Rgb(180, 235, 180))), // Medium light green
                    DiffLineType::Deletion => ("-", Color::Red, Some(Color::Rgb(235, 180, 180))),   // Medium light red
                    DiffLineType::Context => (" ", Color::Gray, None),
                    _ => unreachable!(),
                };
                
                spans.push(Span::styled(
                    marker.to_string(),
                    Style::default().fg(marker_color).add_modifier(Modifier::BOLD)
                ));
                
                // Get the code content (without the diff marker)
                let code_content = if line.content.len() > 1 {
                    line.content[1..].to_string()
                } else {
                    String::new()
                };
                
                // Apply syntax highlighting if available
                if let Some(ref file_path) = self.file_path {
                    let highlighted_spans = self::syntax::highlight_line(&code_content, file_path);
                    
                    // Apply background color for additions/deletions
                    for span in highlighted_spans {
                        let mut style = span.style;
                        if let Some(bg) = bg_color {
                            style = style.bg(bg);
                        }
                        spans.push(Span::styled(span.content, style));
                    }
                } else {
                    // No syntax highlighting, just use basic colors
                    let style = Style::default()
                        .fg(match line.line_type {
                            DiffLineType::Addition => Color::Green,
                            DiffLineType::Deletion => Color::Red,
                            DiffLineType::Context => Color::Gray,
                            _ => Color::White,
                        });
                    
                    let mut final_style = style;
                    if let Some(bg) = bg_color {
                        final_style = final_style.bg(bg);
                    }
                    
                    spans.push(Span::styled(code_content.clone(), final_style));
                }
                
                Line::from(spans)
            }
        }
    }
}


pub fn parse_diff(diff_text: &str) -> Vec<DiffLine> {
    let mut result = Vec::new();
    let mut old_line_num = 0;
    let mut new_line_num = 0;
    
    for line in diff_text.lines() {
        let line_type = if line.starts_with("diff --git") || line.starts_with("index ") {
            DiffLineType::Header
        } else if line.starts_with("@@") {
            // Parse hunk header to get line numbers
            if let Some((old_start, new_start)) = parse_hunk_header(line) {
                old_line_num = old_start;
                new_line_num = new_start;
            }
            DiffLineType::HunkHeader
        } else if line.starts_with('+') && !line.starts_with("+++") {
            new_line_num += 1;
            DiffLineType::Addition
        } else if line.starts_with('-') && !line.starts_with("---") {
            old_line_num += 1;
            DiffLineType::Deletion
        } else {
            // Context line - increment both
            old_line_num += 1;
            new_line_num += 1;
            DiffLineType::Context
        };
        
        let (old_num, new_num) = match line_type {
            DiffLineType::Header => (None, None),
            DiffLineType::HunkHeader => (None, None),
            DiffLineType::Addition => (None, Some(new_line_num)),
            DiffLineType::Deletion => (Some(old_line_num), None),
            DiffLineType::Context => (Some(old_line_num), Some(new_line_num)),
        };
        
        result.push(DiffLine {
            line_type,
            content: line.to_string(),
            old_line_num: old_num,
            new_line_num: new_num,
        });
    }
    
    result
}

/// Parse a hunk header like "@@ -24,6 +24,7 @@" to extract starting line numbers
fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
    use regex::Regex;
    static HUNK_REGEX: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"@@ -(\d+),?\d* \+(\d+),?\d* @@").unwrap()
    });
    
    HUNK_REGEX.captures(line).and_then(|caps| {
        let old_start = caps.get(1)?.as_str().parse::<usize>().ok()?;
        let new_start = caps.get(2)?.as_str().parse::<usize>().ok()?;
        Some((old_start, new_start))
    })
}