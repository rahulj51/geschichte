mod app;
mod cache;
mod cli;
mod commit;
mod copy;
mod diff;
mod error;
mod git;
mod terminal;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseButton, MouseEvent,
    MouseEventKind,
};
use std::time::Duration;

fn main() -> Result<()> {
    // Set up panic handler to restore terminal on crash
    let original_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to restore terminal
        let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);

        // Call the original panic handler
        original_panic_hook(panic_info);
    }));

    // Parse command line arguments
    let args = cli::Args::parse();

    // Initialize logging
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Run the application
    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn run(args: cli::Args) -> Result<()> {
    // Discover git repository
    let start_path = if let Some(ref repo) = args.repo_path {
        repo.clone()
    } else {
        std::env::current_dir()?
    };

    let repo_root = git::discover_repository(&start_path).inspect_err(|_e| {
        eprintln!(
            "Failed to find git repository from: {}",
            start_path.display()
        );
    })?;

    log::debug!("Found git repository at: {}", repo_root.display());

    // Get the effective layout mode
    let layout_mode = args.effective_layout();
    let effective_context_lines = args.effective_context_lines();

    // Create application state based on whether file argument was provided
    let mut app = if let Some(file_path_arg) = args.file_path {
        // File argument provided - use history mode
        let file_path = if file_path_arg.is_absolute() {
            file_path_arg
        } else {
            std::env::current_dir()?.join(&file_path_arg)
        };

        // Verify file exists in git
        let relative_path = git::files::verify_file_in_repo(&repo_root, &file_path)?;
        log::debug!("Viewing history for: {}", relative_path.display());

        let mut app = app::App::new_history(
            repo_root,
            relative_path,
            effective_context_lines,
            !args.no_follow,
            args.first_parent,
            layout_mode,
        );

        // Load git data
        if let Err(e) = app.load_git_data() {
            eprintln!("Failed to load git data: {}", e);
            std::process::exit(1);
        }

        app
    } else {
        // No file argument - use file picker mode
        match app::App::new_file_picker(
            repo_root,
            effective_context_lines,
            !args.no_follow,
            args.first_parent,
            layout_mode,
        ) {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Failed to initialize file picker: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Setup terminal
    let mut terminal = terminal::setup_terminal()?;

    // Enable mouse capture
    crossterm::execute!(std::io::stdout(), EnableMouseCapture)?;

    // Run the UI loop
    let result = run_ui(&mut terminal, &mut app);

    // Cleanup: disable mouse capture
    crossterm::execute!(std::io::stdout(), DisableMouseCapture)?;

    // Restore terminal
    terminal::restore_terminal(&mut terminal)?;

    result
}

fn run_ui(terminal: &mut terminal::AppTerminal, app: &mut app::App) -> Result<()> {
    loop {
        // Draw the UI
        if app.redraw_tui {
            terminal.clear()?;
            app.redraw_tui = false;
        }
        terminal.draw(|frame| {
            // Update terminal dimensions before drawing
            app.handle_resize(frame.area().width, frame.area().height);
            ui::draw(frame, app);
        })?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            // Add error recovery for malformed terminal input
            match event::read() {
                Ok(Event::Key(key)) => {
                    // HACK: The following line needs to be amended if and when enabling the
                    // `KeyboardEnhancementFlags::REPORT_EVENT_TYPES` flag on unix.
                    let event_kind_enabled = cfg!(target_family = "windows");
                    let process_event = !event_kind_enabled || key.kind != KeyEventKind::Release;
                    if process_event {
                        app.handle_key(key)?;
                    }
                }
                Ok(Event::Mouse(mouse_event)) => {
                    handle_mouse_event(app, mouse_event)?;
                }
                Ok(Event::Resize(width, height)) => {
                    app.handle_resize(width, height);
                }
                Ok(_) => {
                    // Ignore other event types
                }
                Err(_) => {
                    // Skip malformed/unparseable terminal input to prevent crashes
                    // This can happen with rapid arrow key presses or terminal compatibility issues
                    continue;
                }
            }
        }

        // Check for message timeout
        app.check_message_timeout();

        // Check if we should quit
        if app.should_quit {
            terminal.clear()?;
            break;
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
enum PanelType {
    Commits,
    Diff,
}

fn get_panel_at_position(app: &app::App, col: u16, _row: u16) -> Option<PanelType> {
    // Calculate panel boundaries based on split ratio and actual terminal width
    let split_ratio = app.ui_state.split_ratio;
    let terminal_width = app.ui_state.terminal_width;
    let split_point = (terminal_width as f32 * split_ratio) as u16;

    if col < split_point {
        Some(PanelType::Commits)
    } else {
        Some(PanelType::Diff)
    }
}

fn get_commit_at_row(app: &app::App, row: u16) -> Option<usize> {
    // Calculate which commit corresponds to the clicked row
    // Account for:
    // - Panel borders (typically 1 row at top)
    // - Title row is inside the border

    if row <= 1 {
        return None; // Clicked on border or title
    }

    let commit_row = row.saturating_sub(2); // Account for border and title
    let commit_index = commit_row as usize;

    if commit_index < app.commits.len() {
        Some(commit_index)
    } else {
        None
    }
}

fn handle_mouse_event(app: &mut app::App, mouse_event: MouseEvent) -> Result<()> {
    // Only handle mouse events in history mode
    if !matches!(app.mode, app::AppMode::History { .. }) {
        return Ok(());
    }

    match mouse_event.kind {
        MouseEventKind::ScrollUp => {
            match get_panel_at_position(app, mouse_event.column, mouse_event.row) {
                Some(PanelType::Diff) => {
                    app.ui_state.scroll_diff_up();
                }
                Some(PanelType::Commits) => {
                    if app.selected_index > 0 {
                        app.move_selection_up()?;
                    }
                }
                None => {}
            }
        }
        MouseEventKind::ScrollDown => {
            match get_panel_at_position(app, mouse_event.column, mouse_event.row) {
                Some(PanelType::Diff) => {
                    let max_lines = app.get_diff_line_count();
                    app.ui_state.scroll_diff_down(max_lines);
                }
                Some(PanelType::Commits) => {
                    if app.selected_index + 1 < app.commits.len() {
                        app.move_selection_down()?;
                    }
                }
                None => {}
            }
        }
        MouseEventKind::ScrollLeft => {
            // Horizontal scrolling (if terminal supports it)
            match get_panel_at_position(app, mouse_event.column, mouse_event.row) {
                Some(PanelType::Diff) => {
                    app.ui_state.scroll_diff_left();
                }
                Some(PanelType::Commits) => {
                    app.ui_state.scroll_commit_left();
                }
                None => {}
            }
        }
        MouseEventKind::ScrollRight => {
            // Horizontal scrolling (if terminal supports it)
            match get_panel_at_position(app, mouse_event.column, mouse_event.row) {
                Some(PanelType::Diff) => {
                    let max_width = app.calculate_max_diff_line_width();
                    app.ui_state.scroll_diff_right(max_width);
                }
                Some(PanelType::Commits) => {
                    let max_width = app.calculate_max_commit_line_width();
                    app.ui_state.scroll_commit_right(max_width);
                }
                None => {}
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            handle_mouse_click(app, mouse_event.column, mouse_event.row)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_mouse_click(app: &mut app::App, col: u16, row: u16) -> Result<()> {
    match get_panel_at_position(app, col, row) {
        Some(PanelType::Commits) => {
            // Switch focus to commits panel
            if let app::AppMode::History {
                ref mut focused_panel,
                ..
            } = app.mode
            {
                *focused_panel = app::FocusedPanel::Commits;
            }

            // Click-to-select commit
            if let Some(commit_index) = get_commit_at_row(app, row) {
                if commit_index != app.selected_index {
                    app.selected_index = commit_index;
                    app.load_diff_for_selected_commit()?;
                }
            }
        }
        Some(PanelType::Diff) => {
            // Switch focus to diff panel
            if let app::AppMode::History {
                ref mut focused_panel,
                ..
            } = app.mode
            {
                *focused_panel = app::FocusedPanel::Diff;
            }
        }
        None => {}
    }
    Ok(())
}
