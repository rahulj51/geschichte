use tempfile::TempDir;
use std::path::Path;
use std::fs;
use std::process::Command;

pub struct TestRepo {
    temp_dir: TempDir,
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRepo {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        Command::new("git").args(["init"]).current_dir(repo_path).output().unwrap();
        Command::new("git").args(["config", "user.name", "Test User"]).current_dir(repo_path).output().unwrap();
        Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(repo_path).output().unwrap();
        
        fs::write(repo_path.join("test.txt"), "Initial content").unwrap();
        Command::new("git").args(["add", "."]).current_dir(repo_path).output().unwrap();
        Command::new("git").args(["commit", "-m", "Initial commit"]).current_dir(repo_path).output().unwrap();
        
        Self { temp_dir }
    }
    
    pub fn new_with_commits(count: usize) -> Self {
        let repo = Self::new();
        
        for i in 1..count {
            let content = format!("Content version {}", i);
            fs::write(repo.path().join("test.txt"), content).unwrap();
            Command::new("git").args(["add", "."]).current_dir(repo.path()).output().unwrap();
            Command::new("git").args(["commit", "-m", &format!("Commit {}", i)]).current_dir(repo.path()).output().unwrap();
        }
        
        repo
    }
    
    pub fn new_with_renames() -> Self {
        let repo = Self::new();
        let repo_path = repo.path();
        
        fs::write(repo_path.join("original.rs"), "fn main() {}").unwrap();
        Command::new("git").args(["add", "."]).current_dir(repo_path).output().unwrap();
        Command::new("git").args(["commit", "-m", "Add original file"]).current_dir(repo_path).output().unwrap();
        
        fs::write(repo_path.join("original.rs"), "fn main() {\n    println!(\"Hello\");\n}").unwrap();
        Command::new("git").args(["add", "."]).current_dir(repo_path).output().unwrap();
        Command::new("git").args(["commit", "-m", "Modify file"]).current_dir(repo_path).output().unwrap();
        
        Command::new("git").args(["mv", "original.rs", "new_name.rs"]).current_dir(repo_path).output().unwrap();
        Command::new("git").args(["commit", "-m", "Rename file"]).current_dir(repo_path).output().unwrap();
        
        repo
    }
    
    pub fn new_with_many_files(count: usize) -> Self {
        let repo = Self::new();
        let repo_path = repo.path();
        
        fs::create_dir_all(repo_path.join("src")).unwrap();
        fs::create_dir_all(repo_path.join("tests")).unwrap();
        fs::create_dir_all(repo_path.join("docs")).unwrap();
        
        for i in 0..count {
            let dir = match i % 3 {
                0 => "src",
                1 => "tests", 
                _ => "docs",
            };
            
            let filename = format!("file_{}.rs", i);
            let content = format!("// File number {}\nfn function_{}() {{}}", i, i);
            fs::write(repo_path.join(dir).join(filename), content).unwrap();
        }
        
        Command::new("git").args(["add", "."]).current_dir(repo_path).output().unwrap();
        Command::new("git").args(["commit", "-m", "Add many files"]).current_dir(repo_path).output().unwrap();
        
        repo
    }
    
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}