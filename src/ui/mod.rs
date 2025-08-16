pub mod file_picker;

use crate::app::{App, FocusedPanel};
use crate::diff::HighlightedDiff;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, ListState},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    match &app.mode {
        crate::app::AppMode::FilePicker { ref state, ref context } => {
            // In file picker mode, draw the file picker popup  
            file_picker::draw_file_picker(frame, state, context, frame.area());
        }
        crate::app::AppMode::History { .. } => {
            // In history mode, draw the normal UI
            draw_history_ui(frame, app);
        }
    }

    // Draw help overlay on top if shown
    if app.show_help {
        draw_help_overlay(frame, app, frame.area());
    }
}

fn draw_history_ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let left_percent = (app.split_ratio * 100.0) as u16;
    let right_percent = 100 - left_percent;
    
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(left_percent), Constraint::Percentage(right_percent)])
        .split(chunks[0]);

    draw_commits_panel(frame, app, main_chunks[0]);
    draw_diff_panel(frame, app, main_chunks[1]);
    draw_status_bar(frame, app, chunks[1]);
}

fn draw_commits_panel(frame: &mut Frame, app: &App, area: Rect) {
    let mut title = if app.loading {
        " Commits (Loading...) ".to_string()
    } else {
        format!(" Commits ({}) ", app.commits.len())
    };
    
    // Add horizontal scroll indicator
    if app.commit_horizontal_scroll > 0 {
        title = format!("{} ←→", title.trim_end());
    }

    let focused = app.get_focused_panel() == Some(FocusedPanel::Commits);
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(border_style);

    if app.commits.is_empty() {
        let message = if app.loading {
            "Loading commits..."
        } else {
            "No commits found for this file"
        };
        
        let paragraph = Paragraph::new(message)
            .block(block)
            .style(Style::default().fg(Color::Gray));
        
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app.commits
        .iter()
        .enumerate()
        .map(|(index, commit)| {
            let marker = if app.is_commit_marked_for_diff(index) {
                "► "
            } else {
                ""
            };

            let line = if commit.is_working_directory {
                // Special styling for working directory
                Line::from(vec![
                    Span::styled(marker.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled("Working".to_string(), Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled("Dir".to_string(), Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled(
                        commit.subject.clone(), // Don't truncate here, let horizontal scroll handle it
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    ),
                ])
            } else {
                // Regular commit styling
                Line::from(vec![
                    Span::styled(marker.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(commit.date.clone(), Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::styled(commit.short_hash.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::raw(commit.subject.clone()), // Don't truncate here, let horizontal scroll handle it
                ])
            };
            
            // Apply horizontal scrolling to commit line
            let scrolled_line = apply_horizontal_scroll(line, app.commit_horizontal_scroll, area.width.saturating_sub(2) as usize);
            ListItem::new(scrolled_line)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn draw_diff_panel(frame: &mut Frame, app: &App, area: Rect) {
    let mut title = if app.commits.is_empty() {
        " Diff ".to_string()
    } else if app.selected_index < app.commits.len() {
        // Check if we're showing a range diff
        if let Some((older_idx, newer_idx)) = app.current_diff_range {
            if older_idx < app.commits.len() && newer_idx < app.commits.len() {
                format!(" Diff ({}..{}) ", 
                    app.commits[older_idx].short_hash,
                    app.commits[newer_idx].short_hash)
            } else {
                format!(" Diff ({}) ", app.commits[app.selected_index].short_hash)
            }
        } else if let Some(_start_index) = app.diff_range_start {
            // Show that we're in selection mode
            format!(" Diff ({}) [Selecting...] ", app.commits[app.selected_index].short_hash)
        } else {
            format!(" Diff ({}) ", app.commits[app.selected_index].short_hash)
        }
    } else {
        " Diff ".to_string()
    };
    
    // Add horizontal scroll indicator
    if app.diff_horizontal_scroll > 0 {
        title = format!("{} ←→", title.trim_end());
    }

    let focused = app.get_focused_panel() == Some(FocusedPanel::Diff);
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(border_style);

    if app.current_diff.is_empty() {
        let message = if app.loading {
            "Loading diff..."
        } else if app.commits.is_empty() {
            "No commits to show diff for"
        } else {
            "No diff available"
        };
        
        let paragraph = Paragraph::new(message)
            .block(block)
            .style(Style::default().fg(Color::Gray));
        
        frame.render_widget(paragraph, area);
        return;
    }

    // Create a highlighted diff with syntax highlighting based on the file path
    let file_path = app.get_file_path().map(|p| p.as_path());
    
    let highlighted_diff = HighlightedDiff::new(&app.current_diff, file_path);
    let all_styled_lines = highlighted_diff.to_styled_lines();
    
    // Apply both vertical AND horizontal scrolling
    let styled_lines: Vec<Line> = all_styled_lines
        .into_iter()
        .skip(app.diff_scroll) // Vertical scroll
        .take(area.height.saturating_sub(2) as usize) // Account for borders
        .map(|line| apply_horizontal_scroll(line, app.diff_horizontal_scroll, area.width as usize))
        .collect();

    let paragraph = Paragraph::new(styled_lines).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let focus_hint = match app.get_focused_panel() {
        Some(FocusedPanel::Commits) => "↑↓/jk: select commit | a/s: h-scroll",
        Some(FocusedPanel::Diff) => "↑↓/jk: scroll diff | a/s: h-scroll",
        None => "Type to search files",
    };

    let file_display = match app.get_file_path() {
        Some(path) => path.display().to_string(),
        None => "File Picker".to_string(),
    };

    let status = format!(
        " {} | {} | Tab: panel | {} | h/l: resize | ?: help | q: quit ",
        app.repo_root.display(),
        file_display,
        focus_hint
    );

    let status_bar = Paragraph::new(Line::from(vec![Span::styled(
        status,
        Style::default().fg(Color::Gray).bg(Color::Black),
    )]));

    frame.render_widget(status_bar, area);
}

fn draw_help_overlay(frame: &mut Frame, _app: &App, area: Rect) {
    // Calculate popup size - center it
    let popup_width = 50;
    let popup_height = 18;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    
    let popup_area = Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    };
    
    // Clear the background
    frame.render_widget(Clear, popup_area);
    
    let help_text = vec![
        Line::from(vec![Span::styled(
            "Geschichte - Git File History Viewer",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw("      Switch between panels"),
        ]),
        Line::from(vec![
            Span::styled("↑↓/jk", Style::default().fg(Color::Yellow)),
            Span::raw("    Navigate commits OR scroll diff"),
        ]),
        Line::from(vec![
            Span::styled("h/l", Style::default().fg(Color::Yellow)),
            Span::raw("      Resize split pane"),
        ]),
        Line::from(vec![
            Span::styled("PgUp/Dn", Style::default().fg(Color::Yellow)),
            Span::raw("  Scroll diff (always)"),
        ]),
        Line::from(vec![
            Span::styled("^U/^D", Style::default().fg(Color::Yellow)),
            Span::raw("    Scroll diff (vim-style)"),
        ]),
        Line::from(vec![
            Span::styled("^B/^F", Style::default().fg(Color::Yellow)),
            Span::raw("    Scroll diff (emacs-style)"),
        ]),
        Line::from(vec![
            Span::styled("a/s", Style::default().fg(Color::Yellow)),
            Span::raw("      Horizontal scroll (left/right)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("f", Style::default().fg(Color::Green)),
            Span::raw("        Switch to another file"),
        ]),
        Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Green)),
            Span::raw("        Search in diff (TODO)"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Green)),
            Span::raw("        Copy commit hash (TODO)"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(Color::Green)),
            Span::raw("        Mark/diff between commits"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw("        Quit"),
        ]),
        Line::from(vec![
            Span::styled("?", Style::default().fg(Color::Magenta)),
            Span::raw("        Show/hide this help"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ? or Esc to close",
            Style::default().fg(Color::Gray),
        )]),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White).bg(Color::Black)),
        )
        .alignment(Alignment::Left);
    
    frame.render_widget(help_paragraph, popup_area);
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

/// Apply horizontal scrolling to a line
fn apply_horizontal_scroll(line: Line<'static>, horizontal_offset: usize, viewport_width: usize) -> Line<'static> {
    // Calculate total line width in characters
    let total_width: usize = line.spans.iter()
        .map(|span| span.content.chars().count())
        .sum();
    
    // If no horizontal offset, return original line
    if horizontal_offset == 0 {
        return line;
    }
    
    // Always apply horizontal scrolling regardless of line length
    // This ensures visual alignment of all lines
    
    // If the horizontal offset is greater than the total line width,
    // return an empty line (the line is scrolled completely out of view)
    if horizontal_offset >= total_width {
        return Line::from(vec![]);
    }
    
    // Apply horizontal offset by trimming characters from the start
    let mut char_count = 0;
    let mut new_spans = Vec::new();
    let mut remaining_offset = horizontal_offset;
    
    for span in line.spans {
        let span_char_count = span.content.chars().count();
        
        if remaining_offset >= span_char_count {
            // Skip this entire span
            remaining_offset -= span_char_count;
            continue;
        }
        
        // Partial span - trim from the start
        let trimmed_content: String = span.content
            .chars()
            .skip(remaining_offset)
            .take(viewport_width.saturating_sub(char_count))
            .collect();
        
        if !trimmed_content.is_empty() {
            new_spans.push(Span::styled(trimmed_content.clone(), span.style));
            char_count += trimmed_content.chars().count();
            
            // Stop if we've filled the viewport
            if char_count >= viewport_width {
                break;
            }
        }
        
        remaining_offset = 0; // Used up the offset
    }
    
    Line::from(new_spans)
}