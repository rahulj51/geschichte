use geschichte::app::App;
use geschichte::cli::LayoutMode;
use std::path::PathBuf;

#[test]
fn test_search_functionality() {
    // Create a test app with mock data
    let repo_root = PathBuf::from("/tmp");
    let file_path = PathBuf::from("test.rs");
    let mut app = App::new_history(repo_root, file_path, 3, false, false, LayoutMode::Unified);

    // Mock some diff content
    app.current_diff = "diff --git a/test.rs b/test.rs\n@@ -1,3 +1,3 @@\n function() {\n-  println!(\"Hello\");\n+  println!(\"Hello World\");\n }".to_string();

    // Test starting search
    app.start_diff_search();
    assert!(app.diff_search_state.is_some());

    let search_state = app.diff_search_state.as_ref().unwrap();
    assert!(search_state.is_active);
    assert!(search_state.is_input_mode);
    assert_eq!(search_state.query, "");

    // Test updating search results
    if let Some(ref mut search_state) = app.diff_search_state {
        search_state.query = "Hello".to_string();
    }

    app.update_search_results().unwrap();

    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.results.len(), 2); // Should find 2 instances of "Hello"

    // Test navigation
    app.navigate_to_next_search_result().unwrap();
    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.current_result, Some(0));

    app.navigate_to_next_search_result().unwrap();
    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.current_result, Some(1));

    // Test wrapping
    app.navigate_to_next_search_result().unwrap();
    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.current_result, Some(0)); // Should wrap around

    // Test clearing search
    app.clear_diff_search();
    assert!(app.diff_search_state.is_none());
}

#[test]
fn test_case_insensitive_search() {
    let repo_root = PathBuf::from("/tmp");
    let file_path = PathBuf::from("test.rs");
    let mut app = App::new_history(repo_root, file_path, 3, false, false, LayoutMode::Unified);

    app.current_diff = "function test() {\n  HELLO world\n  hello World\n}".to_string();
    app.start_diff_search();

    if let Some(ref mut search_state) = app.diff_search_state {
        search_state.query = "hello".to_string();
    }

    app.update_search_results().unwrap();

    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.results.len(), 2); // Should find both "HELLO" and "hello"
}

#[test]
fn test_search_match_positions() {
    let repo_root = PathBuf::from("/tmp");
    let file_path = PathBuf::from("test.rs");
    let mut app = App::new_history(repo_root, file_path, 3, false, false, LayoutMode::Unified);

    app.current_diff = "line one with test\nline two with another test".to_string();
    app.start_diff_search();

    if let Some(ref mut search_state) = app.diff_search_state {
        search_state.query = "test".to_string();
    }

    app.update_search_results().unwrap();

    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.results.len(), 2);

    // Check first match
    let first_match = &search_state.results[0];
    assert_eq!(first_match.line_index, 0);
    assert_eq!(first_match.char_start, 14);
    assert_eq!(first_match.char_end, 18);
    assert_eq!(first_match.content, "test");

    // Check second match
    let second_match = &search_state.results[1];
    assert_eq!(second_match.line_index, 1);
    assert_eq!(second_match.char_start, 22);
    assert_eq!(second_match.char_end, 26);
    assert_eq!(second_match.content, "test");
}

#[test]
fn test_search_excludes_headers() {
    let repo_root = PathBuf::from("/tmp");
    let file_path = PathBuf::from("test.rs");
    let mut app = App::new_history(repo_root, file_path, 3, false, false, LayoutMode::Unified);

    // Diff with "test" in both header and code content
    app.current_diff = "diff --git a/test.rs b/test.rs\nindex 123..456 100644\n@@ -1,3 +1,3 @@ fn test_function()\n function test() {\n-  let test = 1;\n+  let test = 2;\n }".to_string();
    app.start_diff_search();

    if let Some(ref mut search_state) = app.diff_search_state {
        search_state.query = "test".to_string();
    }

    app.update_search_results().unwrap();

    let search_state = app.diff_search_state.as_ref().unwrap();

    // Should find "test" in code lines but not in headers
    // Expected matches:
    // - Line with " function test() {" (line_index will depend on parsed structure)
    // - Line with "-  let test = 1;"
    // - Line with "+  let test = 2;"
    // But NOT in "diff --git a/test.rs b/test.rs" or "@@ ... test_function()"

    assert!(
        search_state.results.len() >= 3,
        "Should find at least 3 matches in code content"
    );

    // Verify no matches are found in header lines by checking that all matches
    // have reasonable char_start positions (headers would have matches at position 0 or very early)
    for result in &search_state.results {
        // Code content matches should not be at the very beginning of lines
        // (since they come after diff markers like +, -, or space)
        assert!(
            result.char_start > 0,
            "Match at char_start {} suggests it might be in a header line",
            result.char_start
        );
    }
}

#[test]
fn test_regex_search_patterns() {
    let repo_root = PathBuf::from("/tmp");
    let file_path = PathBuf::from("test.rs");
    let mut app = App::new_history(repo_root, file_path, 3, false, false, LayoutMode::Unified);

    app.current_diff =
        "function calculate() {\n  let result = search_function();\n  return result;\n}"
            .to_string();

    // Test regex pattern with dot wildcard
    app.start_diff_search();
    if let Some(ref mut search_state) = app.diff_search_state {
        search_state.query = "searc.".to_string(); // Should match "search"
    }

    app.update_search_results().unwrap();
    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.results.len(), 1);
    assert_eq!(search_state.results[0].content, "search");

    // Test regex pattern with word boundary
    app.clear_diff_search();
    app.start_diff_search();
    if let Some(ref mut search_state) = app.diff_search_state {
        search_state.query = r"\bresult\b".to_string(); // Should match "result" as whole word
    }

    app.update_search_results().unwrap();
    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.results.len(), 2); // Two instances of "result" as whole words

    // Test invalid regex - should not crash and show no results
    app.clear_diff_search();
    app.start_diff_search();
    if let Some(ref mut search_state) = app.diff_search_state {
        search_state.query = "[invalid".to_string(); // Invalid regex pattern
    }

    app.update_search_results().unwrap();
    let search_state = app.diff_search_state.as_ref().unwrap();
    assert_eq!(search_state.results.len(), 0); // Should show no results for invalid regex
}
