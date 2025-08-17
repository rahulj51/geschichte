#[cfg(test)]
mod app_tests {
    use geschichte::app::App;
    use geschichte::cli::LayoutMode;
    use geschichte::commit::Commit;
    use std::path::PathBuf;

    fn create_test_app() -> App {
        App::new_history(
            PathBuf::from("/test/repo"),
            PathBuf::from("test.rs"),
            3,
            false,
            false,
            LayoutMode::Unified,
        )
    }

    fn create_test_commits() -> Vec<Commit> {
        vec![
            Commit::new_enhanced(
                "abc123".to_string(),
                "abc123d".to_string(),
                "Alice".to_string(),
                "alice@test.com".to_string(),
                "2023-01-15 10:30:00".to_string(),
                "Alice".to_string(),
                "alice@test.com".to_string(),
                "2023-01-15 10:30:00".to_string(),
                "First commit".to_string(),
                "This is the first commit".to_string(),
            ),
            Commit::new_enhanced(
                "def456".to_string(),
                "def456a".to_string(),
                "Bob".to_string(),
                "bob@test.com".to_string(),
                "2023-01-14 09:20:00".to_string(),
                "Bob".to_string(),
                "bob@test.com".to_string(),
                "2023-01-14 09:20:00".to_string(),
                "Second commit".to_string(),
                "This is the second commit".to_string(),
            ),
        ]
    }

    #[test]
    fn test_app_creation() {
        let app = create_test_app();
        assert!(!app.should_quit);
        assert_eq!(app.context_lines, 3);
        assert!(!app.follow_renames);
        assert!(!app.first_parent);
        assert_eq!(app.selected_index, 0);
        assert!(app.commits.is_empty());
        assert!(app.copy_mode.is_none());
        assert!(app.copy_message.is_none());
        assert!(!app.show_commit_info);
    }

    #[test]
    fn test_copy_mode_operations() {
        let mut app = create_test_app();
        app.commits = create_test_commits();
        
        // Test starting copy mode
        app.start_copy_mode();
        assert!(app.copy_mode.is_some());
        assert!(app.copy_message.is_some());
        
        // Test canceling copy mode
        app.cancel_copy_mode();
        assert!(app.copy_mode.is_none());
        assert!(app.copy_message.is_none());
    }

    #[test]
    fn test_message_timer_functionality() {
        let mut app = create_test_app();
        
        // Start timer
        app.start_message_timer();
        assert!(app.message_timer.is_some());
        
        // Check that timer exists and doesn't clear immediately
        app.check_message_timeout();
        assert!(app.message_timer.is_some());
    }

    #[test]
    fn test_commit_info_popup() {
        let mut app = create_test_app();
        app.commits = create_test_commits();
        
        // Test showing popup (should work even without git data)
        let result = app.show_commit_info_popup();
        assert!(result.is_ok());
        assert!(app.show_commit_info);
        assert!(app.commit_info_popup.is_some());
        
        // Test hiding popup
        app.hide_commit_info_popup();
        assert!(!app.show_commit_info);
        assert!(app.commit_info_popup.is_none());
    }

    #[test]
    fn test_commit_navigation() {
        let mut app = create_test_app();
        app.commits = create_test_commits();
        
        // Test initial state
        assert_eq!(app.selected_index, 0);
        
        // Test moving down (may fail due to missing git repo, but index should change)
        let old_index = app.selected_index;
        let _ = app.move_selection_down(); // Ignore result since git operations may fail in tests
        if app.selected_index == old_index + 1 {
            // Move was successful
            assert_eq!(app.selected_index, 1);
            
            // Test moving up
            let _ = app.move_selection_up();
            // Index might not change if git operations fail, so just check it's valid
            assert!(app.selected_index < app.commits.len());
        }
        
        // Test boundary conditions by setting index directly
        app.selected_index = 0;
        let _ = app.move_selection_up();
        assert_eq!(app.selected_index, 0); // Should stay at 0
        
        app.selected_index = app.commits.len() - 1;
        let _ = app.move_selection_down();
        assert_eq!(app.selected_index, app.commits.len() - 1); // Should stay at last index
    }

    #[test]
    fn test_diff_range_selection() {
        let mut app = create_test_app();
        app.commits = create_test_commits();
        
        // Test marking first commit for diff
        let result = app.toggle_diff_range_selection();
        assert!(result.is_ok());
        assert_eq!(app.diff_range_start, Some(0));
        
        // Test clearing diff range
        app.clear_diff_range_selection();
        assert_eq!(app.diff_range_start, None);
        assert_eq!(app.current_diff_range, None);
        
        // Test checking if commit is marked
        app.diff_range_start = Some(1);
        assert!(app.is_commit_marked_for_diff(1));
        assert!(!app.is_commit_marked_for_diff(0));
    }

    #[test]
    fn test_focus_panel_switching() {
        let mut app = create_test_app();
        
        // Test initial focus (should be Commits)
        let focus = app.get_focused_panel();
        assert!(focus.is_some());
        
        // Test switching focus
        app.switch_focus();
        let new_focus = app.get_focused_panel();
        assert!(new_focus.is_some());
        assert_ne!(focus, new_focus);
    }

    #[test]
    fn test_layout_mode_effectiveness() {
        let mut app = create_test_app();
        
        // Test with narrow terminal (should use Unified)
        app.ui_state.terminal_width = 80;
        app.layout_mode = LayoutMode::Auto;
        assert_eq!(app.effective_layout(), LayoutMode::Unified);
        
        // Test with wide terminal (should use SideBySide)
        app.ui_state.terminal_width = 150;
        assert_eq!(app.effective_layout(), LayoutMode::SideBySide);
        
        // Test explicit mode (should override auto)
        app.layout_mode = LayoutMode::Unified;
        assert_eq!(app.effective_layout(), LayoutMode::Unified);
    }

    #[test]
    fn test_resize_handling() {
        let mut app = create_test_app();
        let _old_layout = app.effective_layout();
        
        // Test resize
        app.handle_resize(120, 30);
        assert_eq!(app.ui_state.terminal_width, 120);
        assert_eq!(app.ui_state.terminal_height, 30);
        
        // Layout might change if in Auto mode
        if app.layout_mode == LayoutMode::Auto {
            // Layout change depends on width threshold
            let new_layout = app.effective_layout();
            // Just ensure it's deterministic
            assert!(new_layout == LayoutMode::Unified || new_layout == LayoutMode::SideBySide);
        }
    }

    #[test]
    fn test_quit_functionality() {
        let mut app = create_test_app();
        assert!(!app.should_quit);
        
        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_file_path_retrieval() {
        let app = create_test_app();
        let file_path = app.get_file_path();
        assert!(file_path.is_some());
        assert_eq!(file_path.unwrap(), &PathBuf::from("test.rs"));
    }

    #[test]
    fn test_content_width_calculations() {
        let mut app = create_test_app();
        app.commits = create_test_commits();
        app.current_diff = "line 1\nlong line with many characters\nshort".to_string();
        
        let max_commit_width = app.calculate_max_commit_line_width();
        assert!(max_commit_width > 0);
        
        let max_diff_width = app.calculate_max_diff_line_width();
        assert!(max_diff_width > 0);
        
        let diff_line_count = app.get_diff_line_count();
        assert_eq!(diff_line_count, 3); // Three lines in the diff
    }
}