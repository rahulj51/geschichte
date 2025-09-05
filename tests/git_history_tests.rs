#[cfg(test)]
mod git_history_tests {
    use geschichte::commit::{Commit, PRStatus};
    use geschichte::git::history::detect_pr_info;

    #[test]
    fn test_pr_detection_from_subject() {
        let commit = Commit::new_enhanced(
            "abc123".to_string(),
            "abc123".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Fix authentication bug (#42)".to_string(),
            "".to_string(),
        );

        let pr_info = detect_pr_info(&commit);
        assert!(pr_info.is_some());

        let pr = pr_info.unwrap();
        assert_eq!(pr.number, 42);
        assert_eq!(pr.title, "Fix authentication bug (#42)");
        assert!(matches!(pr.status, PRStatus::Unknown));
    }

    #[test]
    fn test_pr_detection_merge_commit() {
        let commit = Commit::new_enhanced(
            "abc123".to_string(),
            "abc123".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Merge pull request #123 from feature/auth".to_string(),
            "".to_string(),
        );

        let pr_info = detect_pr_info(&commit);
        assert!(pr_info.is_some());

        let pr = pr_info.unwrap();
        assert_eq!(pr.number, 123);
        assert!(matches!(pr.status, PRStatus::Merged));
        assert!(pr.title.contains("Merge pull request"));
    }

    #[test]
    fn test_pr_detection_no_match() {
        let commit = Commit::new_enhanced(
            "abc123".to_string(),
            "abc123".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Regular commit message".to_string(),
            "".to_string(),
        );

        let pr_info = detect_pr_info(&commit);
        assert!(pr_info.is_none());
    }

    #[test]
    fn test_pr_number_extraction() {
        // Test cases for different PR number formats
        let test_cases = vec![
            ("Fix bug (#123)", Some(123)),
            ("Update docs #456", Some(456)),
            ("Feature PR-789", None), // Different format, not supported
            ("No PR number here", None),
            ("Multiple #123 numbers #456", Some(123)), // Should get first one
            ("At end #999", Some(999)),
        ];

        for (message, expected) in test_cases {
            let commit = Commit::new_enhanced(
                "hash".to_string(),
                "hash".to_string(),
                "Author".to_string(),
                "author@test.com".to_string(),
                "2023-01-01".to_string(),
                "Author".to_string(),
                "author@test.com".to_string(),
                "2023-01-01".to_string(),
                message.to_string(),
                "".to_string(),
            );

            let pr_info = detect_pr_info(&commit);
            match expected {
                Some(num) => {
                    assert!(pr_info.is_some(), "Should detect PR in: {}", message);
                    assert_eq!(pr_info.unwrap().number, num);
                }
                None => {
                    assert!(pr_info.is_none(), "Should not detect PR in: {}", message);
                }
            }
        }
    }

    #[test]
    fn test_commit_creation_backwards_compatibility() {
        // Test old constructor still works
        let commit = Commit::new(
            "abc123".to_string(),
            "abc123".to_string(),
            "2023-01-01 10:00:00".to_string(),
            "John Doe <john@example.com>".to_string(),
            "Test commit".to_string(),
        );

        assert_eq!(commit.hash, "abc123");
        assert_eq!(commit.short_hash, "abc123");
        assert_eq!(commit.author_name, "John Doe");
        assert_eq!(commit.author_email, "john@example.com");
        assert_eq!(commit.subject, "Test commit");
        assert_eq!(commit.date, "2023-01-01 10:00:00");
        assert!(commit.body.is_empty());
        assert!(!commit.is_working_directory);
    }

    #[test]
    fn test_enhanced_commit_creation() {
        let commit = Commit::new_enhanced(
            "def456".to_string(),
            "def456".to_string(),
            "Jane Smith".to_string(),
            "jane@example.com".to_string(),
            "2023-01-15 14:30:00".to_string(),
            "Jane Smith".to_string(),
            "jane@example.com".to_string(),
            "2023-01-15 14:30:00".to_string(),
            "Enhanced commit".to_string(),
            "This is the body\nof the commit".to_string(),
        );

        assert_eq!(commit.hash, "def456");
        assert_eq!(commit.author_name, "Jane Smith");
        assert_eq!(commit.author_email, "jane@example.com");
        assert_eq!(commit.committer_name, "Jane Smith");
        assert_eq!(commit.committer_email, "jane@example.com");
        assert_eq!(commit.author_date, "2023-01-15 14:30:00");
        assert_eq!(commit.committer_date, "2023-01-15 14:30:00");
        assert_eq!(commit.subject, "Enhanced commit");
        assert_eq!(commit.body, "This is the body\nof the commit");
        assert!(!commit.is_working_directory);
    }

    // Note: Most git command tests would require a real git repository
    // and are better suited for integration tests. Here we test the
    // parsing and data structure aspects.

    #[test]
    fn test_commit_author_formatting() {
        let commit_with_email = Commit::new_enhanced(
            "hash".to_string(),
            "hash".to_string(),
            "John Doe".to_string(),
            "john@example.com".to_string(),
            "2023-01-01".to_string(),
            "John Doe".to_string(),
            "john@example.com".to_string(),
            "2023-01-01".to_string(),
            "Test".to_string(),
            "".to_string(),
        );

        assert_eq!(commit_with_email.author(), "John Doe <john@example.com>");

        let commit_no_email = Commit::new_enhanced(
            "hash".to_string(),
            "hash".to_string(),
            "John Doe".to_string(),
            "".to_string(),
            "2023-01-01".to_string(),
            "John Doe".to_string(),
            "".to_string(),
            "2023-01-01".to_string(),
            "Test".to_string(),
            "".to_string(),
        );

        assert_eq!(commit_no_email.author(), "John Doe");
    }
}
