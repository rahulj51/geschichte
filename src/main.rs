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

    // Resolve file path
    let file_path = if args.file_path.is_absolute() {
        args.file_path.clone()
    } else {
        std::env::current_dir()?.join(&args.file_path)
    };

    // Verify file exists in git
    let relative_path = git::verify_file_in_repo(&repo_root, &file_path)?;
    log::debug!("Viewing history for: {}", relative_path.display());

    // Create application state
    let mut app = app::App::new(
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
