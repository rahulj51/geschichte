use crate::fixtures::TestRepo;
use std::path::PathBuf;

pub fn sample_git_files() -> Vec<geschichte::git::files::GitFile> {
    vec![
        geschichte::git::files::GitFile {
            path: PathBuf::from("src/main.rs"),
            display_path: "src/main.rs".to_string(),
            status: geschichte::git::files::FileStatus::Clean,
            modified: None,
            size: Some(1024),
        },
        geschichte::git::files::GitFile {
            path: PathBuf::from("src/app.rs"),
            display_path: "src/app.rs".to_string(),
            status: geschichte::git::files::FileStatus::Modified,
            modified: None,
            size: Some(2048),
        },
        geschichte::git::files::GitFile {
            path: PathBuf::from("tests/test.rs"),
            display_path: "tests/test.rs".to_string(),
            status: geschichte::git::files::FileStatus::Clean,
            modified: None,
            size: Some(512),
        },
    ]
}

pub fn create_test_app() -> geschichte::app::App {
    let test_repo = TestRepo::new();
    geschichte::app::App::new_history(
        test_repo.path().to_path_buf(),
        PathBuf::from("test.txt"),
        3,
        false,
        false,
        geschichte::cli::LayoutMode::Auto,
    )
}

pub fn create_test_app_with_commits() -> geschichte::app::App {
    let test_repo = TestRepo::new_with_commits(5);
    let mut app = geschichte::app::App::new_history(
        test_repo.path().to_path_buf(),
        PathBuf::from("test.txt"),
        3,
        false,
        false,
        geschichte::cli::LayoutMode::Auto,
    );

    let commits = geschichte::git::history::fetch_commit_history(
        test_repo.path(),
        &PathBuf::from("test.txt"),
        false,
        false,
    )
    .unwrap();

    app.commits = commits;
    app
}
