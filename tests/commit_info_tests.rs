#[cfg(test)]
mod commit_info_tests {
    use geschichte::commit::{Commit, CommitStats, PRStatus, PullRequestInfo};
    use geschichte::ui::commit_info::CommitInfoPopup;

    fn create_enhanced_commit() -> Commit {
        let mut commit = Commit::new_enhanced(
            "abc123def456789".to_string(),
            "abc123d".to_string(),
            "Jane Developer".to_string(),
            "jane@company.com".to_string(),
            "2023-01-15 10:30:00".to_string(),
            "Jane Developer".to_string(),
            "jane@company.com".to_string(),
            "2023-01-15 10:30:00".to_string(),
            "Implement user authentication".to_string(),
            "Added JWT token support and password hashing.\n\nThis commit includes:\n- JWT token generation\n- Password validation\n- Session management".to_string(),
        );

        // Add some metadata
        commit.refs = vec!["branch:main".to_string(), "tag:v1.0.0".to_string()];
        commit.stats = Some(CommitStats {
            files_changed: 5,
            insertions: 120,
            deletions: 30,
        });
        commit.pr_info = Some(PullRequestInfo {
            number: 42,
            title: "Add authentication system".to_string(),
            url: "https://github.com/company/repo/pull/42".to_string(),
            status: PRStatus::Merged,
        });

        commit
    }

    #[test]
    fn test_commit_info_popup_creation() {
        let commit = create_enhanced_commit();
        let popup = CommitInfoPopup::new(commit.clone());

        assert_eq!(popup.commit.hash, commit.hash);
        assert_eq!(popup.scroll_position, 0);
    }

    #[test]
    fn test_commit_info_popup_scrolling() {
        let commit = create_enhanced_commit();
        let mut popup = CommitInfoPopup::new(commit);

        // Test initial state
        assert_eq!(popup.scroll_position, 0);

        // Test scrolling down
        popup.scroll_down(20, 10);
        assert!(popup.scroll_position > 0);

        // Test scrolling up
        let old_offset = popup.scroll_position;
        popup.scroll_up();
        assert!(popup.scroll_position < old_offset);

        // Test boundary - can't scroll above 0
        popup.scroll_position = 0;
        popup.scroll_up();
        assert_eq!(popup.scroll_position, 0);
    }

    #[test]
    fn test_commit_info_popup_line_counting() {
        let commit = create_enhanced_commit();
        let popup = CommitInfoPopup::new(commit.clone());

        let total_lines = popup.get_total_lines();
        // get_total_lines only counts the body lines, not the entire popup
        let expected_lines = if commit.body.is_empty() {
            1
        } else {
            commit.body.lines().count()
        };
        assert_eq!(total_lines, expected_lines);

        // Test with empty body
        let mut empty_commit = commit;
        empty_commit.body = String::new();
        let empty_popup = CommitInfoPopup::new(empty_commit);
        assert_eq!(empty_popup.get_total_lines(), 1);
    }

    #[test]
    fn test_commit_author_parsing() {
        let commit = Commit::new(
            "hash123".to_string(),
            "hash123".to_string(),
            "2023-01-01".to_string(),
            "John Doe <john@example.com>".to_string(),
            "Test commit".to_string(),
        );

        assert_eq!(commit.author_name, "John Doe");
        assert_eq!(commit.author_email, "john@example.com");
    }

    #[test]
    fn test_commit_author_parsing_fallback() {
        let commit = Commit::new(
            "hash123".to_string(),
            "hash123".to_string(),
            "2023-01-01".to_string(),
            "john@example.com".to_string(), // Just email, no brackets
            "Test commit".to_string(),
        );

        assert_eq!(commit.author_name, "john@example.com");
        assert_eq!(commit.author_email, "");
    }

    #[test]
    fn test_commit_stats_display() {
        let stats = CommitStats {
            files_changed: 3,
            insertions: 45,
            deletions: 12,
        };

        // Test that stats can be accessed (basic functionality)
        assert_eq!(stats.files_changed, 3);
        assert_eq!(stats.insertions, 45);
        assert_eq!(stats.deletions, 12);
    }

    #[test]
    fn test_pr_info_display() {
        let pr_info = PullRequestInfo {
            number: 123,
            title: "Fix critical bug".to_string(),
            url: "https://github.com/repo/pull/123".to_string(),
            status: PRStatus::Merged,
        };

        assert_eq!(pr_info.number, 123);
        assert_eq!(pr_info.title, "Fix critical bug");
        assert_eq!(pr_info.url, "https://github.com/repo/pull/123");
        assert!(matches!(pr_info.status, PRStatus::Merged));
    }

    #[test]
    fn test_commit_body_handling() {
        let commit_with_body = create_enhanced_commit();
        assert!(!commit_with_body.body.is_empty());
        assert!(commit_with_body.body.contains("JWT token support"));

        let commit_no_body = Commit::new_enhanced(
            "hash".to_string(),
            "hash".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Author".to_string(),
            "author@test.com".to_string(),
            "2023-01-01".to_string(),
            "Simple commit".to_string(),
            "".to_string(),
        );
        assert!(commit_no_body.body.is_empty());
    }

    #[test]
    fn test_refs_handling() {
        let mut commit = create_enhanced_commit();

        // Test with refs
        assert!(!commit.refs.is_empty());
        assert!(commit.refs.iter().any(|r| r.contains("main")));
        assert!(commit.refs.iter().any(|r| r.contains("v1.0.0")));

        // Test without refs
        commit.refs.clear();
        assert!(commit.refs.is_empty());
    }

    #[test]
    fn test_working_directory_commit_special_case() {
        let wd_commit = Commit::new_working_directory("Modified + Staged".to_string());

        assert!(wd_commit.is_working_directory);
        assert_eq!(wd_commit.hash, "WORKING_DIR");
        assert_eq!(wd_commit.short_hash, "WD");
        assert_eq!(wd_commit.subject, "Modified + Staged");
        assert_eq!(wd_commit.author_name, "Working");
        assert_eq!(wd_commit.committer_name, "Directory");

        // Working directory commits should not have stats or PR info
        assert!(wd_commit.stats.is_none());
        assert!(wd_commit.pr_info.is_none());
    }
}
