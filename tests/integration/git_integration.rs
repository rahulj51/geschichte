use std::path::PathBuf;
use std::time::Duration;

mod test {
    use super::*;
    use crate::common::*;

    #[test]
    fn test_with_real_git_repo() {
        let test_repo = TestRepo::new();

        let result = geschichte::git::history::fetch_commit_history(
            test_repo.path(),
            &PathBuf::from("test.txt"),
            false,
            false,
        );
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_rename_tracking_integration() {
        let test_repo = TestRepo::new_with_renames();

        let commits_with_follow = geschichte::git::history::fetch_commit_history(
            test_repo.path(),
            &PathBuf::from("new_name.rs"),
            true,
            false,
        )
        .unwrap();

        let commits_without_follow = geschichte::git::history::fetch_commit_history(
            test_repo.path(),
            &PathBuf::from("new_name.rs"),
            false,
            false,
        )
        .unwrap();

        assert!(commits_with_follow.len() > commits_without_follow.len());
    }

    #[test]
    fn test_large_repository_performance() {
        let test_repo = TestRepo::new_with_commits(100);

        let start = std::time::Instant::now();
        let commits = geschichte::git::history::fetch_commit_history(
            test_repo.path(),
            &PathBuf::from("test.txt"),
            false,
            false,
        )
        .unwrap();
        let duration = start.elapsed();

        assert!(commits.len() <= 100);
        assert!(duration < Duration::from_secs(5));
    }

    #[test]
    fn test_git_files_listing() {
        let test_repo = TestRepo::new_with_many_files(50);

        let start = std::time::Instant::now();
        let files = geschichte::git::files::get_git_files(test_repo.path()).unwrap();
        let duration = start.elapsed();

        assert!(!files.is_empty());
        assert!(duration < Duration::from_secs(2));
    }

    #[test]
    fn test_diff_generation() {
        let test_repo = TestRepo::new_with_commits(3);
        let commits = geschichte::git::history::fetch_commit_history(
            test_repo.path(),
            &PathBuf::from("test.txt"),
            false,
            false,
        )
        .unwrap();

        assert!(commits.len() >= 2);

        let diff = geschichte::git::diff::get_diff_between_commits(
            test_repo.path(),
            &commits[1].hash,
            &commits[0].hash,
            &PathBuf::from("test.txt"),
            3,
        )
        .unwrap();

        assert!(!diff.is_empty());
        assert!(diff.contains("@@"));
    }

    #[test]
    fn test_working_directory_changes() {
        let test_repo = TestRepo::new();

        std::fs::write(test_repo.path().join("test.txt"), "New content").unwrap();

        let diff = geschichte::git::working::fetch_working_directory_diff(
            test_repo.path(),
            &PathBuf::from("test.txt"),
            3,
        )
        .unwrap();

        assert!(!diff.is_empty());
        assert!(diff.contains("New content"));
    }
}
