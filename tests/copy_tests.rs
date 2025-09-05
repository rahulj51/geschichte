#[cfg(test)]
mod copy_tests {
    use geschichte::commit::Commit;
    use geschichte::copy::{CommitCopier, CopyFormat};

    fn create_test_commit() -> Commit {
        Commit::new_enhanced(
            "abc123def456".to_string(),
            "abc123d".to_string(),
            "John Doe".to_string(),
            "john@example.com".to_string(),
            "2023-01-15 10:30:00".to_string(),
            "John Doe".to_string(),
            "john@example.com".to_string(),
            "2023-01-15 10:30:00".to_string(),
            "Add new feature".to_string(),
            "This is the commit body\nwith multiple lines".to_string(),
        )
    }

    #[test]
    fn test_copy_full_sha() {
        let mut copier = CommitCopier::new();
        let commit = create_test_commit();

        let result = copier.copy_commit_info(&commit, CopyFormat::FullSha);
        // Note: This might fail in headless environments without clipboard access
        match result {
            Ok(content) => assert_eq!(content, "abc123def456"),
            Err(_) => {
                // Expected in CI/headless environments
                println!("Clipboard not available for testing");
            }
        }
    }

    #[test]
    fn test_copy_short_sha() {
        let mut copier = CommitCopier::new();
        let commit = create_test_commit();

        let result = copier.copy_commit_info(&commit, CopyFormat::ShortSha);
        match result {
            Ok(content) => assert_eq!(content, "abc123d"),
            Err(_) => println!("Clipboard not available for testing"),
        }
    }

    #[test]
    fn test_copy_message_with_body() {
        let mut copier = CommitCopier::new();
        let commit = create_test_commit();

        let result = copier.copy_commit_info(&commit, CopyFormat::Message);
        match result {
            Ok(content) => {
                assert_eq!(
                    content,
                    "Add new feature\n\nThis is the commit body\nwith multiple lines"
                );
            }
            Err(_) => println!("Clipboard not available for testing"),
        }
    }

    #[test]
    fn test_copy_message_without_body() {
        let mut copier = CommitCopier::new();
        let mut commit = create_test_commit();
        commit.body = String::new();

        let result = copier.copy_commit_info(&commit, CopyFormat::Message);
        match result {
            Ok(content) => assert_eq!(content, "Add new feature"),
            Err(_) => println!("Clipboard not available for testing"),
        }
    }

    #[test]
    fn test_copy_author() {
        let mut copier = CommitCopier::new();
        let commit = create_test_commit();

        let result = copier.copy_commit_info(&commit, CopyFormat::Author);
        match result {
            Ok(content) => assert_eq!(content, "John Doe <john@example.com>"),
            Err(_) => println!("Clipboard not available for testing"),
        }
    }

    #[test]
    fn test_copy_date() {
        let mut copier = CommitCopier::new();
        let commit = create_test_commit();

        let result = copier.copy_commit_info(&commit, CopyFormat::Date);
        match result {
            Ok(content) => assert_eq!(content, "2023-01-15 10:30:00"),
            Err(_) => println!("Clipboard not available for testing"),
        }
    }

    #[test]
    fn test_copy_github_url_default() {
        let mut copier = CommitCopier::new();
        let commit = create_test_commit();

        let result = copier.copy_commit_info(&commit, CopyFormat::GitHubUrl);
        match result {
            Ok(content) => assert_eq!(content, "https://github.com/repo/commit/abc123def456"),
            Err(_) => println!("Clipboard not available for testing"),
        }
    }

    #[test]
    fn test_commit_author_format() {
        let commit = create_test_commit();
        assert_eq!(commit.author(), "John Doe <john@example.com>");
    }

    #[test]
    fn test_commit_author_format_no_email() {
        let mut commit = create_test_commit();
        commit.author_email = String::new();
        assert_eq!(commit.author(), "John Doe");
    }

    #[test]
    fn test_working_directory_commit() {
        let wd_commit = Commit::new_working_directory("Modified".to_string());
        assert!(wd_commit.is_working_directory);
        assert_eq!(wd_commit.hash, "WORKING_DIR");
        assert_eq!(wd_commit.short_hash, "WD");
        assert_eq!(wd_commit.subject, "Modified");
    }
}
