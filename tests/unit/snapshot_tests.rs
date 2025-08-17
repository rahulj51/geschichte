use insta::assert_debug_snapshot;
use std::path::Path;

mod syntax_highlighting_snapshots {
    use super::*;
    use geschichte::diff::syntax::highlight_line;
    
    #[test]
    fn test_rust_syntax_highlighting() {
        let rust_code = "fn main() { println!(\"Hello, world!\"); }";
        let file_path = Path::new("main.rs");
        
        let highlighted = highlight_line(rust_code, file_path);
        
        // Convert the highlighted spans to a more snapshot-friendly format
        let snapshot_data: Vec<(String, String)> = highlighted
            .into_iter()
            .map(|span| {
                let content = span.content.to_string();
                let style = format!("{:?}", span.style);
                (content, style)
            })
            .collect();
        
        assert_debug_snapshot!(snapshot_data);
    }
    
    #[test]
    fn test_json_syntax_highlighting() {
        let json_code = r#"{"name": "test", "value": 42, "enabled": true}"#;
        let file_path = Path::new("config.json");
        
        let highlighted = highlight_line(json_code, file_path);
        
        // Convert to snapshot-friendly format
        let snapshot_data: Vec<(String, String)> = highlighted
            .into_iter()
            .map(|span| {
                let content = span.content.to_string();
                let style = format!("{:?}", span.style);
                (content, style)
            })
            .collect();
        
        assert_debug_snapshot!(snapshot_data);
    }
}

mod app_state_snapshots {
    use super::*;
    use crate::common::helpers::create_test_app;
    use geschichte::app::{App, AppMode, FocusedPanel};
    use geschichte::cli::LayoutMode;
    use std::path::PathBuf;
    
    // Helper to create a snapshot-friendly version of App state
    #[derive(Debug)]
    #[allow(dead_code)] // Fields are used by Debug derive but linter doesn't see it
    struct AppSnapshot {
        repo_root: PathBuf,
        should_quit: bool,
        context_lines: u32,
        follow_renames: bool,
        first_parent: bool,
        mode: String, // Simplified representation
        commits_count: usize,
        selected_index: usize,
        layout_mode: LayoutMode,
        loading: bool,
        error_message: Option<String>,
        focused_panel: Option<FocusedPanel>,
        terminal_width: u16,
        terminal_height: u16,
    }
    
    impl From<&App> for AppSnapshot {
        fn from(app: &App) -> Self {
            let mode = match &app.mode {
                AppMode::FilePicker { context, .. } => {
                    format!("FilePicker({:?})", context)
                }
                AppMode::History { file_path, focused_panel } => {
                    format!("History(file: {:?}, focus: {:?})", file_path.file_name(), focused_panel)
                }
            };
            
            // Normalize the repo_root path to avoid snapshot differences due to temp directories
            let path_str = app.repo_root.to_string_lossy();
            let normalized_repo_root = if path_str.contains("/tmp") || path_str.contains("/var/folders") {
                PathBuf::from("/tmp/test_repo")
            } else {
                app.repo_root.clone()
            };
            
            AppSnapshot {
                repo_root: normalized_repo_root,
                should_quit: app.should_quit,
                context_lines: app.context_lines,
                follow_renames: app.follow_renames,
                first_parent: app.first_parent,
                mode,
                commits_count: app.commits.len(),
                selected_index: app.selected_index,
                layout_mode: app.layout_mode,
                loading: app.loading,
                error_message: app.error_message.clone(),
                focused_panel: app.get_focused_panel(),
                terminal_width: app.ui_state.terminal_width,
                terminal_height: app.ui_state.terminal_height,
            }
        }
    }
    
    #[test]
    fn test_initial_app_state() {
        let app = create_test_app();
        let snapshot = AppSnapshot::from(&app);
        
        assert_debug_snapshot!(snapshot);
    }
    
    #[test]
    fn test_app_state_after_resize() {
        let mut app = create_test_app();
        app.handle_resize(120, 40);
        
        let snapshot = AppSnapshot::from(&app);
        
        assert_debug_snapshot!(snapshot);
    }
}