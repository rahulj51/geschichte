use crate::app::{App, FocusedPanel};
use crate::diff::{DiffLine, DiffLineType};
use crate::ui::common::{
    commits::{draw_commits_panel, CommitsPanelLayout},
    utils::{create_border_style, create_side_by_side_title},
    draw_status_bar,
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
        let lines: Vec<Line> = side_by_side.old_lines
            .iter()
            .skip(app.ui_state.diff_scroll)
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
        let lines: Vec<Line> = side_by_side.new_lines
            .iter()
            .skip(app.ui_state.diff_scroll)
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