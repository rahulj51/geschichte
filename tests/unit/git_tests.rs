use std::path::PathBuf;

mod test {
    use super::*;
    use crate::common::*;

    #[test]
    fn test_fetch_diff_with_context() {
        let test_repo = TestRepo::new_with_commits(3);
        let commits = geschichte::git::history::fetch_commit_history(
            test_repo.path(),
            &PathBuf::from("test.txt"),
            false,
            false,
        ).unwrap();
        
        assert!(!commits.is_empty());
        
        let result = geschichte::git::diff::fetch_diff(
            test_repo.path(),
            &commits[0].hash,
            None,
            &PathBuf::from("test.txt"),
            5,
        );
        
        assert!(result.is_ok());
        let diff = result.unwrap();
        assert!(!diff.is_empty());
    }
    
    #[test]
    fn test_get_diff_between_commits() {
        let test_repo = TestRepo::new_with_commits(3);
        let commits = geschichte::git::history::fetch_commit_history(
            test_repo.path(),
            &PathBuf::from("test.txt"),
            false,
            false,
        ).unwrap();
        
        assert!(commits.len() >= 2);
        
        let result = geschichte::git::diff::get_diff_between_commits(
            test_repo.path(),
            &commits[1].hash,
            &commits[0].hash,
            &PathBuf::from("test.txt"),
            3,
        );
        
        assert!(result.is_ok());
        let diff = result.unwrap();
        assert!(!diff.is_empty());
    }
    
    #[test] 
    fn test_rename_tracking() {
        let test_repo = TestRepo::new_with_renames();
        
        let commits_with_follow = geschichte::git::history::fetch_commit_history(
            test_repo.path(), 
            &PathBuf::from("new_name.rs"), 
            true,
            false
        ).unwrap();
        
        let commits_without_follow = geschichte::git::history::fetch_commit_history(
            test_repo.path(), 
            &PathBuf::from("new_name.rs"), 
            false,
            false
        ).unwrap();
        
        assert!(commits_with_follow.len() > commits_without_follow.len());
    }
    
    #[test]
    fn test_working_directory_diff() {
        let test_repo = TestRepo::new();
        
        std::fs::write(test_repo.path().join("test.txt"), "Modified content").unwrap();
        
        let result = geschichte::git::working::fetch_working_directory_diff(
            test_repo.path(),
            &PathBuf::from("test.txt"),
            3,
        );
        
        assert!(result.is_ok());
        let diff = result.unwrap();
        assert!(!diff.is_empty());
        assert!(diff.contains("Modified content"));
    }
}