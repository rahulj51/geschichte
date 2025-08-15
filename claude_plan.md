# Geschichte - Git File History TUI Implementation Plan

## Project Overview

**Name**: Geschichte (German for "history")  
**Purpose**: A blazingly fast terminal UI for viewing git history and diffs for a single file  
**Core Value**: Instant, rename-aware file history visualization with keyboard-driven navigation

## Technical Architecture

### Core Stack
- **Language**: Rust (performance, safety, single binary distribution)
- **TUI Framework**: ratatui 0.28 + crossterm 0.28
- **Git Integration**: Shell commands via std::process::Command
- **Async Runtime**: tokio (for non-blocking git operations)
- **Error Handling**: anyhow + thiserror for typed errors

### Key Dependencies
```toml
[dependencies]
anyhow = "1"
ratatui = "0.28"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
once_cell = "1"
chrono = "0.4"
regex = "1"
lru = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Module Structure

```
src/
├── main.rs           # Entry point, CLI parsing, app initialization
├── app.rs            # Core application state and event handling
├── ui/
│   ├── mod.rs        # UI orchestration
│   ├── commits.rs    # Commit list widget
│   ├── diff.rs       # Diff viewer widget
│   └── status.rs     # Status bar widget
├── git/
│   ├── mod.rs        # Git abstraction layer
│   ├── commands.rs   # Git command execution
│   ├── parser.rs     # Output parsing utilities
│   └── rename.rs     # Rename tracking logic
├── diff/
│   ├── mod.rs        # Diff processing
│   ├── parser.rs     # Unified diff parser
│   └── highlight.rs  # Syntax highlighting
├── cache.rs          # LRU cache for diffs
├── search.rs         # In-diff search implementation
├── config.rs         # Configuration management
└── error.rs          # Custom error types
```

## Core Data Structures

```rust
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub date: chrono::NaiveDate,
    pub author: String,
    pub subject: String,
    pub rename_info: Option<RenameInfo>,
}

pub struct RenameInfo {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub similarity: u8,
}

pub struct AppState {
    // Repository info
    pub repo_root: PathBuf,
    pub target_file: PathBuf,
    pub current_path: PathBuf,
    
    // Commit data
    pub commits: Vec<Commit>,
    pub selected_index: usize,
    pub rename_map: HashMap<String, PathBuf>,
    
    // Diff display
    pub current_diff: DiffContent,
    pub diff_scroll: usize,
    pub diff_cache: LruCache<String, DiffContent>,
    
    // Search
    pub search_query: Option<String>,
    pub search_matches: Vec<SearchMatch>,
    pub search_index: usize,
    
    // UI state
    pub split_ratio: f32,
    pub show_help: bool,
    pub loading: bool,
    pub error_message: Option<String>,
    
    // Configuration
    pub config: Config,
}

pub struct DiffContent {
    pub lines: Vec<DiffLine>,
    pub hunks: Vec<HunkInfo>,
    pub stats: DiffStats,
}

pub enum DiffLine {
    Header(String),
    HunkHeader(String, HunkRange),
    Addition(String),
    Deletion(String),
    Context(String),
}
```

## Implementation Phases

### Phase 1: Foundation (Days 1-2)
- [x] Project setup with Cargo.toml
- [ ] CLI argument parsing with clap
- [ ] Basic TUI scaffold with ratatui
- [ ] Git repository discovery
- [ ] Error handling framework

### Phase 2: Git Integration (Days 3-4)
- [ ] Implement git command wrapper
- [ ] Parse commit history with --follow
- [ ] Build rename tracking map
- [ ] Handle merge commits properly
- [ ] Implement path resolution at specific commits

### Phase 3: Diff Processing (Days 5-6)
- [ ] Parse unified diff output
- [ ] Implement diff colorization
- [ ] Add LRU cache for diffs
- [ ] Handle binary files gracefully
- [ ] Support configurable context lines

### Phase 4: UI Components (Days 7-8)
- [ ] Commit list with selection
- [ ] Scrollable diff viewer
- [ ] Status bar with context
- [ ] Split pane resizing
- [ ] Help overlay

### Phase 5: Features (Days 9-10)
- [ ] In-diff search with regex
- [ ] Navigation shortcuts
- [ ] Merge parent cycling
- [ ] Toggle rename awareness
- [ ] Copy commit hash to clipboard

### Phase 6: Polish (Days 11-12)
- [ ] Performance optimization
- [ ] Comprehensive error messages
- [ ] Configuration file support
- [ ] Documentation and help text
- [ ] Release build and packaging

## Git Command Interface

### Core Commands

```bash
# Repository discovery
git rev-parse --show-toplevel

# Commit history with renames
git log --follow --format='%H%x00%h%x00%ad%x00%an%x00%s' \
        --date=format:'%Y-%m-%d' -- <file>

# Rename tracking
git log --follow --name-status --format='%H' -- <file>

# Get diff for commit
git diff --unified=<n> --find-renames \
         <parent> <commit> -- <path-at-commit>

# Handle root commit
git show --unified=<n> <commit> -- <path>

# Get parents
git rev-list --parents -n1 <commit>

# Verify file existence
git ls-tree --name-only <commit> -- <path>
```

## UI Layout

```
┌─ Commits (main.rs) ──────────────────┐┌─ Diff ─────────────────────────────┐
│ > 2025-08-14 a3b4c5d Fix memory leak ││ diff --git a/src/main.rs b/src/... │
│   2025-08-13 d6e7f8g Add feature X   ││ @@ -10,6 +10,8 @@ fn main() {       │
│   2025-08-12 9a0b1c2 [R] Rename file ││  fn main() {                        │
│   2025-08-11 3d4e5f6 Initial impl    ││ -    println!("Hello");            │
│                                      ││ +    println!("Hello, world!");    │
│                                      ││ +    // Added comment               │
│                                      ││      process::exit(0);             │
└──────────────────────────────────────┘└─────────────────────────────────────┘
[main.rs@a3b4c5d] [↑↓/jk: select] [/: search] [q: quit] [?: help]
```

## Key Bindings

### Navigation
- `↑/↓`, `j/k`: Select commit
- `PgUp/PgDn`, `Ctrl-u/Ctrl-d`: Scroll diff
- `g/G`: Jump to first/last commit
- `Home/End`: Jump to diff start/end

### Features
- `/`: Start search in diff
- `n/N`: Next/previous search match
- `r`: Toggle rename tracking
- `m`: Cycle merge parents
- `c`: Copy commit hash
- `d`: Show commit details
- `?`: Show help overlay
- `q`, `Esc`: Quit

### UI Control
- `h/l`, `←/→`: Adjust split ratio
- `f`: Toggle fullscreen diff
- `t`: Toggle timestamps
- `w`: Toggle word wrap

## Performance Optimizations

### Caching Strategy
- LRU cache for last 50 diffs
- Pre-fetch adjacent commits
- Cancel in-flight requests on selection change
- Virtualized rendering for long lists

### Async Operations
- Non-blocking git commands via tokio
- Debounced selection changes (100ms)
- Progressive diff loading for large files
- Background rename map building

## Error Handling

### User-Friendly Messages
- "Not a git repository" → Show initialization help
- "File not found" → Suggest similar files
- "Binary file" → Show size and type info
- "Network error" → Offer offline mode
- "Large diff" → Provide truncation options

### Recovery Strategies
- Fallback to basic git log without --follow
- Cache partial results on failure
- Provide manual refresh option
- Save state for crash recovery

## Configuration

### Config File Location
- `~/.config/geschichte/config.toml`
- `$XDG_CONFIG_HOME/geschichte/config.toml`
- `./.geschichte.toml` (project-specific)

### Configuration Options
```toml
[ui]
theme = "nord"
split_ratio = 0.4
show_author = true
date_format = "%Y-%m-%d"

[git]
follow_renames = true
context_lines = 3
first_parent = false
find_renames_threshold = 50

[keybindings]
quit = ["q", "Esc"]
search = ["/"]
help = ["?", "h"]

[performance]
cache_size = 50
prefetch_count = 2
debounce_ms = 100
```

## Testing Strategy

### Unit Tests
- Git output parsing
- Diff colorization logic
- Rename map construction
- Search functionality
- Cache eviction

### Integration Tests
- Full workflow with test repository
- Rename handling scenarios
- Merge commit navigation
- Large file performance
- Edge cases (empty history, deleted files)

### Test Repository Structure
```
test-repo/
├── linear-history/
├── with-renames/
├── with-merges/
├── binary-files/
└── large-diffs/
```

## Release Checklist

### Build Artifacts
- [ ] Linux x86_64 binary
- [ ] macOS arm64 binary
- [ ] macOS x86_64 binary
- [ ] Windows x86_64 binary

### Distribution
- [ ] GitHub releases with checksums
- [ ] Homebrew formula
- [ ] Cargo crate publication
- [ ] AUR package (community)

### Documentation
- [ ] README with demo GIF
- [ ] Man page
- [ ] Interactive tutorial
- [ ] Changelog

## Future Enhancements

### Version 2.0
- Side-by-side diff view
- Syntax highlighting via tree-sitter
- Git delta integration
- Blame overlay mode
- Multi-file history view

### Version 3.0
- Plugin system
- Custom themes
- Export to various formats
- Integration with git-absorb
- Collaborative features

## Success Metrics

### Performance Targets
- < 100ms to display initial commits
- < 50ms to switch between commits
- < 200ms to search large diffs
- < 10MB memory for typical usage

### Quality Metrics
- 100% parity with git CLI output
- Zero panics in production
- < 1s startup time
- Handles 10k+ commit histories smoothly

## Development Workflow

### Branch Strategy
- `main`: stable releases
- `develop`: integration branch
- `feature/*`: feature branches
- `fix/*`: bug fixes
- `perf/*`: performance improvements

### Commit Convention
```
type(scope): description

feat(ui): add split pane resizing
fix(git): handle renamed files correctly
perf(cache): optimize LRU implementation
docs(readme): add installation instructions
```

### CI/CD Pipeline
1. Run clippy and rustfmt
2. Execute test suite
3. Build for all platforms
4. Generate checksums
5. Create draft release
6. Update homebrew formula

This plan provides a comprehensive roadmap for building Geschichte, a focused and efficient git file history viewer that prioritizes speed, usability, and correctness.