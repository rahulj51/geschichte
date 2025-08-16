pub mod file_picker;

use crate::app::{App, FocusedPanel};
use crate::cli::LayoutMode;
use crate::diff::{DiffLine, DiffLineType, HighlightedDiff};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, ListState},
    Frame,
};
use std::path::PathBuf;

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
    // Get the effective layout mode (handles Auto mode)
    let layout_mode = app.effective_layout();
    
    match layout_mode {
        LayoutMode::Unified => draw_unified_layout(frame, app),
        LayoutMode::SideBySide => draw_side_by_side_layout(frame, app),
        LayoutMode::Auto => {
            // This shouldn't happen since effective_layout() resolves Auto
            // But handle it anyway for completeness
            draw_unified_layout(frame, app)
        }
    }
}

fn draw_unified_layout(frame: &mut Frame, app: &App) {
    // Traditional two-panel layout
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

fn draw_side_by_side_layout(frame: &mut Frame, app: &App) {
    // Three-panel layout: top split panels for diffs, bottom panel for commits
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
    draw_bottom_commits_panel(frame, app, main_chunks[1]);
    
    // Draw status bar
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
        Some(FocusedPanel::Commits) => "↑↓/jk: select commit | a/s: h-scroll | mouse: scroll/click",
        Some(FocusedPanel::Diff) => "↑↓/jk: scroll diff | a/s: h-scroll | mouse: scroll",
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
    let popup_height = 19;
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
        Line::from(vec![
            Span::styled("Mouse", Style::default().fg(Color::Yellow)),
            Span::raw("     Wheel scroll, click to focus/select"),
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

// Three-panel layout functions

fn draw_old_file_panel(frame: &mut Frame, app: &App, area: Rect) {
    // Build title with commit hash, similar to unified layout
    let title = if !app.commits.is_empty() {
        // Check if we're showing a range diff
        if let Some((older_idx, newer_idx)) = app.current_diff_range {
            if older_idx < app.commits.len() && newer_idx < app.commits.len() {
                format!(" Old ({}) ", app.commits[older_idx].short_hash)
            } else {
                " Old File ".to_string()
            }
        } else {
            // For single commit diff, show the same commit hash as unified layout does
            // This represents "the diff OF this commit" not "diff FROM parent TO commit"
            format!(" Old ({}) ", app.commits[app.selected_index].short_hash)
        }
    } else {
        " Old File ".to_string()
    };
    
    let focused = app.get_focused_panel() == Some(FocusedPanel::Diff); // For now, both diff panels share focus
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };
    
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
        let lines: Vec<Line> = side_by_side.old_lines
            .iter()
            .skip(app.diff_scroll)
            .take(area.height.saturating_sub(2) as usize) // Account for borders
            .map(|line_opt| {
                match line_opt {
                    Some(line) => {
                        // Use the proper syntax highlighting and styling
                        style_side_by_side_line(line, true, app.get_file_path()) // true = old file
                    }
                    None => {
                        // Empty line for alignment with new file
                        Line::from(vec![Span::styled(
                            " ",
                            Style::default().fg(Color::DarkGray)
                        )])
                    }
                }
            })
            .collect();
        
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((0, app.diff_horizontal_scroll as u16));
        
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No diff selected")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }
}

fn draw_new_file_panel(frame: &mut Frame, app: &App, area: Rect) {
    // Build title with commit hash, similar to unified layout
    let title = if !app.commits.is_empty() {
        // Check if we're showing a range diff
        if let Some((older_idx, newer_idx)) = app.current_diff_range {
            if older_idx < app.commits.len() && newer_idx < app.commits.len() {
                format!(" New ({}) ", app.commits[newer_idx].short_hash)
            } else {
                " New File ".to_string()
            }
        } else {
            format!(" New ({}) ", app.commits[app.selected_index].short_hash)
        }
    } else {
        " New File ".to_string()
    };
    
    let focused = app.get_focused_panel() == Some(FocusedPanel::Diff); // For now, both diff panels share focus
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };
    
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
        let lines: Vec<Line> = side_by_side.new_lines
            .iter()
            .skip(app.diff_scroll)
            .take(area.height.saturating_sub(2) as usize) // Account for borders
            .map(|line_opt| {
                match line_opt {
                    Some(line) => {
                        // Use the proper syntax highlighting and styling
                        style_side_by_side_line(line, false, app.get_file_path()) // false = new file
                    }
                    None => {
                        // Empty line for alignment with old file
                        Line::from(vec![Span::styled(
                            " ",
                            Style::default().fg(Color::DarkGray)
                        )])
                    }
                }
            })
            .collect();
        
        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((0, app.diff_horizontal_scroll as u16));
        
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No diff selected")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }
}

fn draw_bottom_commits_panel(frame: &mut Frame, app: &App, area: Rect) {
    // This is essentially the same as draw_commits_panel but adapted for horizontal layout
    let mut title = if app.loading {
        " Commits (Loading...) ".to_string()
    } else {
        format!(" Commits ({}) ", app.commits.len())
    };
    
    // Add horizontal scroll indicator if needed
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

    // For horizontal layout, we might want to show commits in a more compact way
    // For now, let's show them as a horizontal list or a compact vertical list
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
                format!("{}[Working Directory] {}", marker, commit.subject)
            } else {
                // Don't truncate - let horizontal scrolling handle long messages
                format!("{}{} {} {}",
                    marker,
                    &commit.short_hash,
                    &commit.date[..10.min(commit.date.len())], // Take first 10 chars (date part)
                    commit.subject
                )
            };
            
            let style = if index == app.selected_index {
                if focused {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                }
            } else if app.is_commit_marked_for_diff(index) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    
    // Create a list state
    let mut state = ListState::default();
    state.select(Some(app.selected_index));
    
    frame.render_stateful_widget(list, area, &mut state);
}


fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

/// Style a diff line for side-by-side view with proper syntax highlighting and line numbers
fn style_side_by_side_line(line: &DiffLine, is_old_file: bool, file_path: Option<&PathBuf>) -> Line<'static> {
    match line.line_type {
        DiffLineType::Header => {
            // File headers in bold blue - no line numbers
            Line::from(vec![
                Span::styled(line.content.clone(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            ])
        }
        DiffLineType::HunkHeader => {
            // Hunk headers in cyan - no line numbers
            Line::from(vec![
                Span::styled(line.content.clone(), Style::default().fg(Color::Cyan))
            ])
        }
        DiffLineType::Addition | DiffLineType::Deletion | DiffLineType::Context => {
            // For side-by-side, we need to show only relevant lines in each panel
            match (line.line_type, is_old_file) {
                (DiffLineType::Addition, true) => {
                    // Additions don't appear in old file - return empty line
                    return Line::from(vec![Span::styled(" ", Style::default().fg(Color::DarkGray))]);
                }
                (DiffLineType::Deletion, false) => {
                    // Deletions don't appear in new file - return empty line
                    return Line::from(vec![Span::styled(" ", Style::default().fg(Color::DarkGray))]);
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
            
            spans.push(Span::styled(
                num_str,
                Style::default().fg(Color::DarkGray)
            ));
            
            // Add the diff marker with appropriate color (but only for relevant lines)
            let (marker, marker_color, bg_color) = match line.line_type {
                DiffLineType::Addition if !is_old_file => ("+", Color::Green, Some(Color::Rgb(180, 235, 180))), // Medium light green - same as unified view
                DiffLineType::Deletion if is_old_file => ("-", Color::Red, Some(Color::Rgb(235, 180, 180))),   // Medium light red - same as unified view
                DiffLineType::Context => (" ", Color::Gray, None),
                _ => (" ", Color::Gray, None), // Fallback for mismatched lines
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
            if let Some(file_path) = file_path {
                let highlighted_spans = crate::diff::syntax::highlight_line(&code_content, file_path);
                
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
            
            Line::from(spans)
        }
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