use crate::error::{GeschichteError, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn setup_terminal() -> Result<AppTerminal> {
    enable_raw_mode().map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let terminal =
        Terminal::new(backend).map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut AppTerminal) -> Result<()> {
    disable_raw_mode().map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    terminal
        .show_cursor()
        .map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    Ok(())
}
