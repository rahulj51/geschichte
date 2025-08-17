pub mod commits;
pub mod utils;

use crate::app::{App, FocusedPanel};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draw the status bar at the bottom of the screen
pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let focus_hint = match app.get_focused_panel() {
        Some(FocusedPanel::Commits) => "↑↓/jk: select commit | a/s: h-scroll | mouse: scroll/click",
        Some(FocusedPanel::Diff) => "↑↓/jk: move cursor | PgUp/PgDn: scroll | a/s: h-scroll",
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

/// Draw the help overlay popup
pub fn draw_help_overlay(frame: &mut Frame, _app: &App, area: Rect) {
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
            Span::raw("    Navigate commits OR move cursor in diff"),
        ]),
        Line::from(vec![
            Span::styled("h/l", Style::default().fg(Color::Yellow)),
            Span::raw("      Resize split pane"),
        ]),
        Line::from(vec![
            Span::styled("PgUp/Dn", Style::default().fg(Color::Yellow)),
            Span::raw("  Scroll diff by page (always)"),
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

