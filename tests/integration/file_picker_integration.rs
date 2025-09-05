use std::time::Duration;

mod test {
    use super::*;
    use crate::common::*;

    #[test]
    fn test_file_picker_with_large_repository() {
        let test_repo = TestRepo::new_with_many_files(1000);
        let files = geschichte::git::files::get_git_files(test_repo.path()).unwrap();

        let mut picker = geschichte::ui::file_picker::FilePickerState::new(files);

        let start = std::time::Instant::now();
        picker.append_char('t');
        picker.append_char('e');
        picker.append_char('s');
        picker.append_char('t');
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(100));
        assert!(!picker.filtered_files.is_empty());
    }

    #[test]
    fn test_file_picker_performance_search() {
        let test_repo = TestRepo::new_with_many_files(500);
        let files = geschichte::git::files::get_git_files(test_repo.path()).unwrap();

        let mut picker = geschichte::ui::file_picker::FilePickerState::new(files);

        let start = std::time::Instant::now();
        for ch in "file_1".chars() {
            picker.append_char(ch);
        }
        let duration = start.elapsed();

        assert!(duration < Duration::from_millis(50));
        assert!(!picker.filtered_files.is_empty());
    }

    #[test]
    fn test_file_picker_memory_usage() {
        let test_repo = TestRepo::new_with_many_files(100);
        let files = geschichte::git::files::get_git_files(test_repo.path()).unwrap();

        let picker = geschichte::ui::file_picker::FilePickerState::new(files);

        assert!(!picker.filtered_files.is_empty());
        assert!(picker.filtered_files.len() <= 200); // Allow some tolerance
    }

    #[test]
    fn test_navigation_with_many_files() {
        let test_repo = TestRepo::new_with_many_files(100);
        let files = geschichte::git::files::get_git_files(test_repo.path()).unwrap();

        let mut picker = geschichte::ui::file_picker::FilePickerState::new(files);
        let original_count = picker.filtered_files.len();

        for _ in 0..10 {
            picker.move_down();
        }

        assert!(picker.selected < original_count);

        for _ in 0..5 {
            picker.move_up();
        }

        assert!(picker.selected < original_count);
    }
}
