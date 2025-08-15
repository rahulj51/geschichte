use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    draw_commits_panel(frame, app, main_chunks[0]);
    draw_diff_panel(frame, app, main_chunks[1]);
    draw_status_bar(frame, app, chunks[1]);
}

fn draw_commits_panel(frame: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Commits ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("2025-08-14 ", Style::default().fg(Color::Yellow)),
            Span::styled("abc123 ", Style::default().fg(Color::Cyan)),
            Span::raw("Initial implementation"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("2025-08-13 ", Style::default().fg(Color::Yellow)),
            Span::styled("def456 ", Style::default().fg(Color::Cyan)),
            Span::raw("Add feature X"),
        ])),
    ];

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(list, area);
}

fn draw_diff_panel(frame: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Diff ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let text = vec![
        Line::from(vec![Span::styled(
            "diff --git a/src/main.rs b/src/main.rs",
            Style::default().fg(Color::Blue),
        )]),
        Line::from(vec![Span::styled(
            "@@ -1,5 +1,7 @@",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::raw(" fn main() {")]),
        Line::from(vec![Span::styled(
            "-    println!(\"Hello\");",
            Style::default().fg(Color::Red),
        )]),
        Line::from(vec![Span::styled(
            "+    println!(\"Hello, world!\");",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            "+    // Added comment",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::raw("     process::exit(0);")]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = format!(
        " {} | {} | ↑↓/jk: select | /: search | q: quit | ?: help ",
        app.repo_root.display(),
        app.file_path.display()
    );

    let status_bar = Paragraph::new(Line::from(vec![Span::styled(
        status,
        Style::default().fg(Color::Gray).bg(Color::Black),
    )]));

    frame.render_widget(status_bar, area);
}