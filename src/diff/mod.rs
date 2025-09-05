pub mod parser;
pub mod side_by_side;
pub mod syntax;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn to_styled_lines_with_search(
        &self,
        search_state: Option<&crate::app::DiffSearchState>,
    ) -> Vec<Line<'static>> {
        self.lines
            .iter()
            .enumerate()
            .map(|(index, line)| self.style_diff_line(line, index, search_state))
            .collect()
    }

    /// Find all line indices that represent changes (additions or deletions)
    /// Returns a sorted vector of line indices for efficient binary search
    pub fn find_changes(&self) -> Vec<usize> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| match line.line_type {
                DiffLineType::Addition | DiffLineType::Deletion => Some(i),
                _ => None,
            })
            .collect()
    }

    fn style_diff_line(
        &self,
        line: &DiffLine,
        line_index: usize,
        search_state: Option<&crate::app::DiffSearchState>,
    ) -> Line<'static> {
        match line.line_type {
            DiffLineType::Header => {
                // File headers in bold blue - no line numbers
                Line::from(vec![
                    Span::styled("         ".to_string(), Style::default()), // Space for line numbers (4+1+4+1=10 chars)
                    Span::styled(
                        line.content.clone(),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            }
            DiffLineType::HunkHeader => {
                // Hunk headers in cyan - no line numbers
                Line::from(vec![
                    Span::styled("         ".to_string(), Style::default()), // Space for line numbers (4+1+4+1=10 chars)
                    Span::styled(line.content.clone(), Style::default().fg(Color::Cyan)),
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
                    Style::default().fg(Color::DarkGray),
                ));

                // Add the diff marker with appropriate color
                let (marker, marker_color, bg_color) = match line.line_type {
                    DiffLineType::Addition => ("+", Color::Green, Some(Color::Rgb(180, 235, 180))), // Medium light green
                    DiffLineType::Deletion => ("-", Color::Red, Some(Color::Rgb(235, 180, 180))), // Medium light red
                    DiffLineType::Context => (" ", Color::Gray, None),
                    _ => unreachable!(),
                };

                spans.push(Span::styled(
                    marker.to_string(),
                    Style::default()
                        .fg(marker_color)
                        .add_modifier(Modifier::BOLD),
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
                    let style = Style::default().fg(match line.line_type {
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

                let mut styled_line = Line::from(spans);

                // Apply search highlighting if active - only for code lines
                if let Some(search_state) = search_state {
                    styled_line = apply_search_highlighting_to_code_content(
                        styled_line,
                        line_index,
                        search_state,
                        line,
                    );
                }

                styled_line
            }
        }
    }
}

/// Apply search highlighting specifically to code content, respecting the line structure
fn apply_search_highlighting_to_code_content(
    styled_line: Line<'static>,
    line_index: usize,
    search_state: &crate::app::DiffSearchState,
    original_line: &DiffLine,
) -> Line<'static> {
    // Find matches for this line
    let line_matches: Vec<&crate::app::SearchMatch> = search_state
        .results
        .iter()
        .filter(|m| m.line_index == line_index)
        .collect();

    if line_matches.is_empty() {
        return styled_line;
    }

    // Note: We'll work with the styled spans to apply highlighting

    let mut result_spans = Vec::new();

    // The styled line structure is: [line_numbers_span] [diff_marker_span] [code_content_spans...]
    // We keep the first two spans as-is and apply highlighting to code content spans

    for (span_idx, span) in styled_line.spans.into_iter().enumerate() {
        if span_idx < 2 {
            // Line numbers and diff marker spans - keep unchanged
            result_spans.push(span);
            continue;
        }

        // This is a code content span - check if it needs highlighting
        let span_content = span.content.to_string();

        // Calculate this span's position within the code content
        let code_content_before: String = result_spans
            .iter()
            .skip(2) // Skip line numbers and diff marker spans
            .map(|s| s.content.as_ref())
            .collect();

        let span_start_in_code = code_content_before.len();
        let span_end_in_code = span_start_in_code + span_content.len();

        // Find matches that overlap with this span's content
        let overlapping_matches: Vec<&crate::app::SearchMatch> = line_matches
            .iter()
            .filter(|m| {
                // SearchMatch positions are relative to the original line content (including diff marker)
                // So we need to subtract 1 to get positions relative to code content only
                let match_start_in_code = m.char_start.saturating_sub(1);
                let match_end_in_code = m.char_end.saturating_sub(1);

                match_start_in_code < span_end_in_code && match_end_in_code > span_start_in_code
            })
            .copied()
            .collect();

        if overlapping_matches.is_empty() {
            // No matches in this span
            result_spans.push(span);
        } else {
            // Apply highlighting to this span with context about the diff line type
            apply_highlighting_to_span(
                span,
                span_start_in_code,
                &overlapping_matches,
                search_state,
                original_line.line_type,
                &mut result_spans,
            );
        }
    }

    Line::from(result_spans)
}

/// Apply highlighting to a specific span based on overlapping matches
fn apply_highlighting_to_span(
    span: Span<'static>,
    span_start_in_code: usize,
    overlapping_matches: &[&crate::app::SearchMatch],
    search_state: &crate::app::DiffSearchState,
    line_type: DiffLineType,
    result_spans: &mut Vec<Span<'static>>,
) {
    let span_content = span.content.to_string();
    let mut processed_chars = 0;

    for &search_match in overlapping_matches {
        // Convert match positions from original line to code-only coordinates
        let match_start_in_code = search_match.char_start.saturating_sub(1);
        let match_end_in_code = search_match.char_end.saturating_sub(1);

        // Calculate positions within this specific span
        let match_start_in_span = match_start_in_code.saturating_sub(span_start_in_code);
        let match_end_in_span =
            (match_end_in_code.saturating_sub(span_start_in_code)).min(span_content.len());

        // Skip if match doesn't actually overlap this span
        if match_start_in_span >= span_content.len() || match_end_in_span == 0 {
            continue;
        }

        // Add text before the match
        if match_start_in_span > processed_chars {
            let pre_text = span_content
                .chars()
                .skip(processed_chars)
                .take(match_start_in_span - processed_chars)
                .collect::<String>();

            if !pre_text.is_empty() {
                result_spans.push(Span::styled(pre_text, span.style));
            }
        }

        // Add the highlighted match
        let match_text = span_content
            .chars()
            .skip(match_start_in_span)
            .take(match_end_in_span - match_start_in_span)
            .collect::<String>();

        if !match_text.is_empty() {
            let is_current_match = search_state.current_result.is_some_and(|idx| {
                idx < search_state.results.len() && &search_state.results[idx] == search_match
            });

            // Choose highlight colors based on diff line type for optimal contrast
            let highlight_style = get_search_highlight_style(is_current_match, line_type);

            result_spans.push(Span::styled(match_text, highlight_style));
        }

        processed_chars = match_end_in_span;
    }

    // Add remaining text after all matches
    if processed_chars < span_content.len() {
        let remaining_text = span_content
            .chars()
            .skip(processed_chars)
            .collect::<String>();

        if !remaining_text.is_empty() {
            result_spans.push(Span::styled(remaining_text, span.style));
        }
    }
}

/// Get search highlight style based on match type and diff line context
pub fn get_search_highlight_style(is_current_match: bool, line_type: DiffLineType) -> Style {
    match (is_current_match, line_type) {
        // Current match styles - need to be highly visible against any background
        (true, DiffLineType::Addition) => {
            // Against light green background: Use dark purple for maximum contrast
            Style::default()
                .bg(Color::Rgb(75, 0, 130)) // Indigo
                .fg(Color::Rgb(255, 255, 255)) // Bright white
                .add_modifier(Modifier::BOLD)
        }
        (true, DiffLineType::Deletion) => {
            // Against light red background: Use dark blue for maximum contrast
            Style::default()
                .bg(Color::Rgb(0, 0, 139)) // Dark blue
                .fg(Color::Rgb(255, 255, 255)) // Bright white
                .add_modifier(Modifier::BOLD)
        }
        (true, _) => {
            // Against no background (context): Use bright yellow
            Style::default()
                .bg(Color::Rgb(255, 215, 0)) // Gold
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        }

        // Non-current match styles - should be visible but less prominent
        (false, DiffLineType::Addition) => {
            // Against light green background: Use dark navy
            Style::default()
                .bg(Color::Rgb(25, 25, 112)) // Midnight blue
                .fg(Color::Rgb(200, 200, 200)) // Light gray
                .add_modifier(Modifier::BOLD)
        }
        (false, DiffLineType::Deletion) => {
            // Against light red background: Use dark green
            Style::default()
                .bg(Color::Rgb(0, 100, 0)) // Dark green
                .fg(Color::Rgb(200, 200, 200)) // Light gray
                .add_modifier(Modifier::BOLD)
        }
        (false, _) => {
            // Against no background (context): Use medium blue
            Style::default()
                .bg(Color::Rgb(70, 130, 180)) // Steel blue
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
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
    static HUNK_REGEX: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"@@ -(\d+),?\d* \+(\d+),?\d* @@").unwrap());

    HUNK_REGEX.captures(line).and_then(|caps| {
        let old_start = caps.get(1)?.as_str().parse::<usize>().ok()?;
        let new_start = caps.get(2)?.as_str().parse::<usize>().ok()?;
        Some((old_start, new_start))
    })
}
