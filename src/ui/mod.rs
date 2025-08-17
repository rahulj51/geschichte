pub mod file_picker;
pub mod state;
pub mod commit_info;
mod unified;
mod side_by_side;
mod common;

use crate::app::App;
use crate::cli::LayoutMode;
use common::draw_help_overlay;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    match &app.mode {
        crate::app::AppMode::FilePicker { ref state, ref context } => {
            // In file picker mode, draw the file picker popup  
            file_picker::draw_file_picker(frame, state, context, frame.area());
        }
        crate::app::AppMode::History { .. } => {
            // In history mode, draw the normal UI
            draw_history_ui(frame, app);
        }
    }

    // Draw help overlay on top if shown
    if app.ui_state.show_help {
        draw_help_overlay(frame, app, frame.area());
    }
    
    // Draw commit info popup on top if shown
    if app.show_commit_info {
        if let Some(ref popup) = app.commit_info_popup {
            popup.render(frame, frame.area());
        }
    }
}

fn draw_history_ui(frame: &mut Frame, app: &App) {
    // Get the effective layout mode (handles Auto mode)
    let layout_mode = app.effective_layout();
    
    match layout_mode {
        LayoutMode::Unified => unified::draw(frame, app),
        LayoutMode::SideBySide => side_by_side::draw(frame, app),
        LayoutMode::Auto => {
            // This shouldn't happen since effective_layout() resolves Auto
            // But handle it anyway for completeness
            unified::draw(frame, app)
        }
    }
}