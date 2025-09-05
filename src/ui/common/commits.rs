use crate::app::{App, FocusedPanel};
use crate::ui::common::utils::{
    apply_horizontal_scroll, create_border_style, create_commits_title,
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub enum CommitsPanelLayout {
    Vertical,
    Horizontal,
}

/// Draw commits panel that works for both unified and side-by-side layouts
pub fn draw_commits_panel(frame: &mut Frame, app: &App, area: Rect, layout: CommitsPanelLayout) {
    let title = create_commits_title(
        app.commits.len(),
        app.loading,
        app.ui_state.commit_horizontal_scroll,
    );

    let focused = app.get_focused_panel() == Some(FocusedPanel::Commits);
    let border_style = create_border_style(focused);

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

    let items: Vec<ListItem> = match layout {
        CommitsPanelLayout::Vertical => create_vertical_commit_items(app, area),
        CommitsPanelLayout::Horizontal => create_horizontal_commit_items(app),
    };

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Create commit items for vertical layout (unified view)
fn create_vertical_commit_items(app: &App, area: Rect) -> Vec<ListItem<'_>> {
    app.commits
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
                    Span::styled(
                        marker.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Working".to_string(), Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled("Dir".to_string(), Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled(
                        commit.subject.clone(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                // Regular commit styling
                Line::from(vec![
                    Span::styled(
                        marker.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(commit.date.clone(), Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::styled(commit.short_hash.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::raw(commit.subject.clone()),
                ])
            };

            // Apply horizontal scrolling to commit line
            let scrolled_line = apply_horizontal_scroll(
                line,
                app.ui_state.commit_horizontal_scroll,
                area.width.saturating_sub(2) as usize,
            );
            ListItem::new(scrolled_line)
        })
        .collect()
}

/// Create commit items for horizontal layout (side-by-side view)
fn create_horizontal_commit_items(app: &App) -> Vec<ListItem<'_>> {
    app.commits
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
                format!(
                    "{}{} {} {}",
                    marker,
                    &commit.short_hash,
                    &commit.date[..10.min(commit.date.len())], // Take first 10 chars (date part)
                    commit.subject
                )
            };

            let style = if index == app.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else if app.is_commit_marked_for_diff(index) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect()
}
