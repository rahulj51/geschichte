use crate::app::{App, FocusedPanel};
use crate::diff::{DiffLine, DiffLineType};
use crate::ui::common::{
    commits::{draw_commits_panel, CommitsPanelLayout},
    draw_status_bar,
    utils::{create_border_style, create_side_by_side_title},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::path::PathBuf;

/// Draw the side-by-side layout (three-panel layout: top split panels for diffs, bottom panel for commits)
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    // Split main area vertically: 70% for diffs, 30% for commits
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);

    // Split top area horizontally for side-by-side diffs
    let diff_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);

    // Draw the three panels
    draw_old_file_panel(frame, app, diff_chunks[0]);
    draw_new_file_panel(frame, app, diff_chunks[1]);
    draw_commits_panel(frame, app, main_chunks[1], CommitsPanelLayout::Horizontal);

    // Draw status bar
    draw_status_bar(frame, app, chunks[1]);
}

fn draw_old_file_panel(frame: &mut Frame, app: &App, area: Rect) {
    let title = create_side_by_side_title(
        &app.commits,
        app.selected_index,
        app.current_diff_range,
        true, // is_old_file
    );

    let focused = app.get_focused_panel() == Some(FocusedPanel::Diff); // For now, both diff panels share focus
    let border_style = create_border_style(focused);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(border_style);

    if app.loading {
        let paragraph = Paragraph::new("Loading...")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    if let Some(ref side_by_side) = app.current_side_by_side_diff {
        // Render the old file content using the styled lines from HighlightedDiff
        let lines: Vec<Line> = side_by_side
            .old_lines
            .iter()
            .enumerate()
            .map(|(global_line_index, line_opt)| {
                let styled_line = match line_opt {
                    Some(line) => {
                        // Use the proper syntax highlighting and styling with search support
                        style_side_by_side_line(
                            line,
                            true,
                            app.get_file_path(),
                            global_line_index,
                            app.diff_search_state.as_ref(),
                        )
                        // true = old file
                    }
                    None => {
                        // Empty line for alignment with new file
                        Line::from(vec![Span::styled(
                            " ",
                            Style::default().fg(Color::DarkGray),
                        )])
                    }
                };

                // Apply cursor highlighting if this line is selected and panel is focused
                if global_line_index == app.ui_state.diff_cursor_line && focused {
                    apply_cursor_highlight(styled_line)
                } else {
                    styled_line
                }
            })
            .skip(app.ui_state.diff_scroll)
            .take(area.height.saturating_sub(2) as usize) // Account for borders
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((0, app.ui_state.diff_horizontal_scroll as u16));

        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No diff selected")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }
}

fn draw_new_file_panel(frame: &mut Frame, app: &App, area: Rect) {
    let title = create_side_by_side_title(
        &app.commits,
        app.selected_index,
        app.current_diff_range,
        false, // is_old_file
    );

    let focused = app.get_focused_panel() == Some(FocusedPanel::Diff); // For now, both diff panels share focus
    let border_style = create_border_style(focused);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(border_style);

    if app.loading {
        let paragraph = Paragraph::new("Loading...")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    if let Some(ref side_by_side) = app.current_side_by_side_diff {
        // Render the new file content using the styled lines from HighlightedDiff
        let lines: Vec<Line> = side_by_side
            .new_lines
            .iter()
            .enumerate()
            .map(|(global_line_index, line_opt)| {
                let styled_line = match line_opt {
                    Some(line) => {
                        // Use the proper syntax highlighting and styling with search support
                        style_side_by_side_line(
                            line,
                            false,
                            app.get_file_path(),
                            global_line_index,
                            app.diff_search_state.as_ref(),
                        )
                        // false = new file
                    }
                    None => {
                        // Empty line for alignment with old file
                        Line::from(vec![Span::styled(
                            " ",
                            Style::default().fg(Color::DarkGray),
                        )])
                    }
                };

                // Apply cursor highlighting if this line is selected and panel is focused
                if global_line_index == app.ui_state.diff_cursor_line && focused {
                    apply_cursor_highlight(styled_line)
                } else {
                    styled_line
                }
            })
            .skip(app.ui_state.diff_scroll)
            .take(area.height.saturating_sub(2) as usize) // Account for borders
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((0, app.ui_state.diff_horizontal_scroll as u16));

        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No diff selected")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }
}

/// Style a diff line for side-by-side view with proper syntax highlighting and line numbers
fn style_side_by_side_line(
    line: &DiffLine,
    is_old_file: bool,
    file_path: Option<&PathBuf>,
    line_index: usize,
    search_state: Option<&crate::app::DiffSearchState>,
) -> Line<'static> {
    match line.line_type {
        DiffLineType::Header => {
            // File headers in bold blue - no line numbers
            Line::from(vec![Span::styled(
                line.content.clone(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )])
        }
        DiffLineType::HunkHeader => {
            // Hunk headers in cyan - no line numbers
            Line::from(vec![Span::styled(
                line.content.clone(),
                Style::default().fg(Color::Cyan),
            )])
        }
        DiffLineType::Addition | DiffLineType::Deletion | DiffLineType::Context => {
            // For side-by-side, we need to show only relevant lines in each panel
            match (line.line_type, is_old_file) {
                (DiffLineType::Addition, true) => {
                    // Additions don't appear in old file - return empty line
                    return Line::from(vec![Span::styled(
                        " ",
                        Style::default().fg(Color::DarkGray),
                    )]);
                }
                (DiffLineType::Deletion, false) => {
                    // Deletions don't appear in new file - return empty line
                    return Line::from(vec![Span::styled(
                        " ",
                        Style::default().fg(Color::DarkGray),
                    )]);
                }
                _ => {}
            }

            let mut spans = Vec::new();

            // Add line number (only show relevant line number for this side)
            let line_num = if is_old_file {
                line.old_line_num
            } else {
                line.new_line_num
            };

            let num_str = match line_num {
                Some(num) => format!("{:>4} ", num),
                None => "     ".to_string(),
            };

            spans.push(Span::styled(num_str, Style::default().fg(Color::DarkGray)));

            // Add the diff marker with appropriate color (but only for relevant lines)
            let (marker, marker_color, bg_color) = match line.line_type {
                DiffLineType::Addition if !is_old_file => {
                    ("+", Color::Green, Some(Color::Rgb(180, 235, 180)))
                } // Medium light green - same as unified view
                DiffLineType::Deletion if is_old_file => {
                    ("-", Color::Red, Some(Color::Rgb(235, 180, 180)))
                } // Medium light red - same as unified view
                DiffLineType::Context => (" ", Color::Gray, None),
                _ => (" ", Color::Gray, None), // Fallback for mismatched lines
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
            if let Some(file_path) = file_path {
                let highlighted_spans =
                    crate::diff::syntax::highlight_line(&code_content, file_path);

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
                    DiffLineType::Addition if !is_old_file => Color::Green,
                    DiffLineType::Deletion if is_old_file => Color::Red,
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
                styled_line = apply_side_by_side_search_highlighting(
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

/// Apply cursor highlighting to a line by adding background color to all spans
fn apply_cursor_highlight(line: Line<'static>) -> Line<'static> {
    let highlighted_spans: Vec<Span> = line
        .spans
        .into_iter()
        .map(|span| {
            let mut style = span.style;
            // Use a subtle blue background for cursor highlighting
            style = style.bg(Color::Rgb(60, 80, 120)); // Dark blue background
            Span::styled(span.content, style)
        })
        .collect();

    Line::from(highlighted_spans)
}

/// Apply search highlighting to a side-by-side styled line
fn apply_side_by_side_search_highlighting(
    styled_line: Line<'static>,
    line_index: usize,
    search_state: &crate::app::DiffSearchState,
    original_line: &crate::diff::DiffLine,
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

    // Side-by-side structure: [line_number_span] [diff_marker_span] [code_content_spans...]
    // Similar to unified but with only one line number instead of two

    let mut result_spans = Vec::new();

    for (span_idx, span) in styled_line.spans.into_iter().enumerate() {
        if span_idx < 2 {
            // Line number and diff marker spans - keep unchanged
            result_spans.push(span);
            continue;
        }

        // This is a code content span - check if it needs highlighting
        let span_content = span.content.to_string();

        // Calculate this span's position within the code content
        let code_content_before: String = result_spans
            .iter()
            .skip(2) // Skip line number and diff marker spans
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
            // Apply highlighting to this span
            apply_side_by_side_highlighting_to_span(
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

/// Apply highlighting to a specific span in side-by-side view
fn apply_side_by_side_highlighting_to_span(
    span: Span<'static>,
    span_start_in_code: usize,
    overlapping_matches: &[&crate::app::SearchMatch],
    search_state: &crate::app::DiffSearchState,
    line_type: crate::diff::DiffLineType,
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

            // Use the same context-aware highlighting as unified view
            let highlight_style =
                get_side_by_side_search_highlight_style(is_current_match, line_type);

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

/// Get search highlight style for side-by-side view (reuse the same logic as unified view)
fn get_side_by_side_search_highlight_style(
    is_current_match: bool,
    line_type: crate::diff::DiffLineType,
) -> Style {
    // Reuse the same color logic from the unified view
    crate::diff::get_search_highlight_style(is_current_match, line_type)
}
