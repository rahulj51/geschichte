use crate::app::{App, FocusedPanel};
use crate::diff::HighlightedDiff;
use crate::ui::common::{
    commits::{draw_commits_panel, CommitsPanelLayout},
    draw_status_bar,
    utils::{apply_horizontal_scroll, create_border_style, create_diff_title},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draw the unified layout (traditional two-panel layout)
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let left_percent = (app.ui_state.split_ratio * 100.0) as u16;
    let right_percent = 100 - left_percent;

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_percent),
            Constraint::Percentage(right_percent),
        ])
        .split(chunks[0]);

    draw_commits_panel(frame, app, main_chunks[0], CommitsPanelLayout::Vertical);
    draw_diff_panel(frame, app, main_chunks[1]);
    draw_status_bar(frame, app, chunks[1]);
}

fn draw_diff_panel(frame: &mut Frame, app: &App, area: Rect) {
    let title = create_diff_title(
        &app.commits,
        app.selected_index,
        app.current_diff_range,
        app.diff_range_start,
        app.ui_state.diff_horizontal_scroll,
    );

    let focused = app.get_focused_panel() == Some(FocusedPanel::Diff);
    let border_style = create_border_style(focused);

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
    let all_styled_lines =
        highlighted_diff.to_styled_lines_with_search(app.diff_search_state.as_ref());

    // Apply both vertical AND horizontal scrolling with cursor highlighting
    let styled_lines: Vec<Line> = all_styled_lines
        .into_iter()
        .enumerate()
        .map(|(global_line_index, line)| {
            if global_line_index == app.ui_state.diff_cursor_line && focused {
                // Apply cursor highlighting - add background color to all spans
                apply_cursor_highlight(line)
            } else {
                line
            }
        })
        .skip(app.ui_state.diff_scroll) // Vertical scroll
        .take(area.height.saturating_sub(2) as usize) // Account for borders
        .map(|line| {
            apply_horizontal_scroll(
                line,
                app.ui_state.diff_horizontal_scroll,
                area.width as usize,
            )
        })
        .collect();

    let paragraph = Paragraph::new(styled_lines).block(block);
    frame.render_widget(paragraph, area);
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
