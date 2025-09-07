use crate::error::{GeschichteError, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout, Write};

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

/// Force a complete terminal reset after external editor usage
pub fn force_terminal_reset(terminal: &mut AppTerminal) -> Result<()> {
    // Clear the entire screen and reset cursor
    execute!(
        terminal.backend_mut(),
        Clear(ClearType::All),
        Clear(ClearType::Purge)
    )
    .map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    // Force terminal to redraw everything
    terminal
        .clear()
        .map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    // Flush the backend to ensure all changes are applied
    let backend = terminal.backend_mut();
    backend
        .flush()
        .map_err(|e| GeschichteError::TerminalError(e.to_string()))?;

    Ok(())
}
