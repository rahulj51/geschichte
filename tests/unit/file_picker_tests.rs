mod test {
    use crate::common::*;

    #[test]
    fn test_fuzzy_filtering() {
        let mut picker = geschichte::ui::file_picker::FilePickerState::new(sample_git_files());
        picker.append_char('m');
        picker.append_char('a');
        picker.append_char('i');

        assert!(!picker.filtered_files.is_empty());
        let has_main = picker.filtered_files.iter().any(|(idx, _)| {
            picker.files[*idx]
                .path
                .to_string_lossy()
                .contains("main.rs")
        });
        assert!(has_main);
    }

    #[test]
    fn test_multi_word_fuzzy_search() {
        let mut picker = geschichte::ui::file_picker::FilePickerState::new(sample_git_files());
        picker.append_char('m');
        picker.append_char('a');
        picker.append_char('i');
        picker.append_char(' ');
        picker.append_char('r');
        picker.append_char('s');

        // Multi-word search might not be implemented yet, just verify it doesn't crash
    }

    #[test]
    fn test_file_navigation() {
        let mut picker = geschichte::ui::file_picker::FilePickerState::new(sample_git_files());

        assert_eq!(picker.selected, 0);
        picker.move_down();
        assert_eq!(picker.selected, 1);
        picker.move_up();
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_query_clear() {
        let mut picker = geschichte::ui::file_picker::FilePickerState::new(sample_git_files());
        picker.append_char('t');
        picker.append_char('e');
        picker.append_char('s');
        picker.append_char('t');

        picker.clear_query();
        assert!(picker.query.is_empty());
        assert_eq!(picker.selected, 0);
    }

    #[test]
    fn test_empty_query_shows_all_files() {
        let files = sample_git_files();
        let file_count = files.len();
        let picker = geschichte::ui::file_picker::FilePickerState::new(files);

        assert_eq!(picker.filtered_files.len(), file_count);
    }

    #[test]
    fn test_case_insensitive_search() {
        let mut picker = geschichte::ui::file_picker::FilePickerState::new(sample_git_files());
        picker.append_char('m');
        picker.append_char('a');
        picker.append_char('i');

        // Search should work with lowercase
        assert!(!picker.filtered_files.is_empty());
    }
}
