mod app;
mod cache;
mod cli;
mod commit;
mod diff;
mod error;
mod git;
mod terminal;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event};
use std::time::Duration;

fn main() -> Result<()> {
    // Parse command line arguments
    let args = cli::Args::parse();
    
    // Initialize logging
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
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

    let repo_root = git::discover_repository(&start_path).map_err(|e| {
        eprintln!("Failed to find git repository from: {}", start_path.display());
        e
    })?;

    log::debug!("Found git repository at: {}", repo_root.display());

    // Create application state based on whether file argument was provided
    let mut app = if let Some(file_path_arg) = args.file_path {
        // File argument provided - use history mode
        let file_path = if file_path_arg.is_absolute() {
            file_path_arg
        } else {
            std::env::current_dir()?.join(&file_path_arg)
        };

        // Verify file exists in git
        let relative_path = git::verify_file_in_repo(&repo_root, &file_path)?;
        log::debug!("Viewing history for: {}", relative_path.display());

        let mut app = app::App::new_history(
            repo_root,
            relative_path,
            args.context_lines,
            !args.no_follow,
            args.first_parent,
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
            args.context_lines,
            !args.no_follow,
            args.first_parent,
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

    // Run the UI loop
    let result = run_ui(&mut terminal, &mut app);

    // Restore terminal
    terminal::restore_terminal(&mut terminal)?;

    result
}

fn run_ui(terminal: &mut terminal::AppTerminal, app: &mut app::App) -> Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            // Update terminal height before drawing
            app.update_terminal_height(frame.area().height);
            ui::draw(frame, app);
        })?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
