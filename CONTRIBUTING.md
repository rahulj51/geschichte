# Contributing to Geschichte

Welcome! We're excited you're interested in contributing to Geschichte. This guide will help you get started.

## 🚀 Quick Start

### Prerequisites
- Rust 1.70 or later
- Git
- A terminal that supports ANSI colors

### Development Setup
```bash
# Clone the repository
git clone https://github.com/yourusername/geschichte.git
cd geschichte

# Build the project
cargo build

# Run tests
cargo test

# Try it out
cargo run -- src/main.rs
```

## 🔧 Development Workflow

### Code Style
We use standard Rust formatting and linting:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run tests
cargo test

# Build release version
cargo build --release
```

### Before Submitting
1. **Format your code**: `cargo fmt`
2. **Fix clippy warnings**: `cargo clippy`
3. **Run tests**: `cargo test`
4. **Test manually**: Try your changes with real git repositories

## 📁 Project Structure

```
src/
├── main.rs           # Entry point and CLI parsing
├── app.rs            # Core application state and event handling
├── ui/mod.rs         # TUI components and rendering
├── git/              # Git integration layer
│   ├── mod.rs        # Repository discovery
│   ├── commands.rs   # Git command execution
│   ├── history.rs    # Commit history parsing
│   ├── diff.rs       # Diff fetching and path resolution
│   └── working.rs    # Working directory status
├── diff/             # Diff processing and parsing
├── cache.rs          # LRU diff cache
├── commit.rs         # Commit data structures
├── error.rs          # Error types
└── terminal.rs       # Terminal setup/teardown
```

## 🎯 Areas for Contribution

### High-Priority Features
1. **File picker popup** - Allow running `geschichte` without file argument
2. **In-diff search** - Search within diff content with `/` key
3. **Merge parent cycling** - Navigate through merge parents with `m` key
4. **Copy commit hash** - Copy hash to clipboard with `c` key

### Medium-Priority Features
- Configuration file support
- Syntax highlighting integration
- Performance optimizations
- Additional keybinding options

### Documentation & Polish
- Improve error messages
- Add more comprehensive tests
- Performance benchmarks
- User documentation improvements

## 🧪 Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Manual Testing
Always test your changes manually:

```bash
# Test with different file types
cargo run -- README.md
cargo run -- src/main.rs
cargo run -- Cargo.toml

# Test with CLI options
cargo run -- -L 10 src/main.rs
cargo run -- --no-follow src/main.rs
cargo run -- --first-parent src/main.rs

# Test edge cases
cargo run -- nonexistent.txt        # Should show error
cargo run -- .                      # Should show error
```

### Test Repositories
Consider testing with repositories that have:
- File renames
- Merge commits
- Large files
- Binary files
- Very long commit histories
- Empty commits

## 📝 Code Guidelines

### Rust Best Practices
- Use `anyhow::Result` for error handling
- Prefer explicit error messages over generic ones
- Use `log::debug!` for debugging information
- Document complex algorithms or git interactions
- Keep functions focused and testable

### TUI Guidelines
- Use ratatui's built-in styling consistently
- Ensure UI remains responsive during long operations
- Handle terminal resize events gracefully
- Provide clear visual feedback for user actions

### Git Integration
- Always handle edge cases (empty repos, detached HEAD, etc.)
- Use appropriate git commands for better performance
- Cache expensive operations when possible
- Test with different git versions

## 🐛 Bug Reports

When reporting bugs, please include:

1. **Environment**:
   - Operating system and version
   - Terminal emulator
   - Rust version (`rustc --version`)
   - Geschichte version

2. **Reproduction steps**:
   - Exact commands run
   - Repository state (if relevant)
   - Expected vs actual behavior

3. **Logs**:
   - Run with `--debug` flag for verbose output
   - Include any error messages

## 💡 Feature Requests

When suggesting features:

1. **Describe the use case**: What problem does it solve?
2. **Provide examples**: How would it work in practice?
3. **Consider alternatives**: Are there existing ways to achieve this?
4. **Think about edge cases**: How should it handle unusual situations?

## 🔄 Pull Request Process

1. **Fork the repository** and create a feature branch
2. **Make your changes** following the code guidelines
3. **Write tests** for new functionality
4. **Update documentation** if needed
5. **Submit a pull request** with a clear description

### PR Description Template
```markdown
## What
Brief description of what this PR does.

## Why
Explanation of why this change is needed.

## How
Technical details of the implementation.

## Testing
How you tested the changes.

## Checklist
- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] Tests pass
- [ ] Manual testing done
- [ ] Documentation updated (if needed)
```

## 🎨 Code Organization

### Adding New Features
1. **Start with the data structures** - Define what you need to track
2. **Add UI components** - How will users interact with it?
3. **Implement the logic** - Connect git commands to UI updates
4. **Add keybindings** - Make it accessible to users
5. **Update help text** - Document the new functionality

### Git Command Integration
When adding new git operations:

1. **Add to `git/commands.rs`** - Raw command execution
2. **Add to `git/parser.rs`** - Parse command output
3. **Update `app.rs`** - Integrate with application state
4. **Add error handling** - Handle failure cases gracefully

## 🚦 Release Process

We follow semantic versioning:
- **Patch** (0.1.1): Bug fixes
- **Minor** (0.2.0): New features, backwards compatible
- **Major** (1.0.0): Breaking changes

## ❓ Questions?

- Check existing [issues](https://github.com/yourusername/geschichte/issues)
- Start a [discussion](https://github.com/yourusername/geschichte/discussions)
- Look at the [project board](https://github.com/yourusername/geschichte/projects) for current priorities

## 🙏 Recognition

All contributors will be acknowledged in the README and release notes. Thank you for helping make Geschichte better!

---

**Happy contributing!** 🎉