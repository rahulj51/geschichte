# Geschichte

**A blazingly fast terminal UI for viewing git file history and diffs**

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Built with ratatui](https://img.shields.io/badge/built%20with-ratatui-blue)](https://github.com/ratatui-org/ratatui)

Geschichte (German for "history") is a fast, keyboard-driven terminal UI for exploring git file history. Navigate through commits, view diffs, and understand how your files evolved over time—all without leaving your terminal.

Unlike full-featured git clients, Geschichte focuses specifically on **single-file history exploration**. It's designed for developers who want to quickly understand how a particular file changed over time, not for managing branches, commits, or other git operations. Think of it as a specialized tool that does one thing exceptionally well: showing you the story of your file.

I wrote this because I was badly missing this feature in 'Zed', my primary IDE. Geschichte can be opened in a terminal window in your favorite IDE. 

![Main screen](screenshots/geschichte-main.png)

## ✨ Features

### Core Functionality
- 📁 **Interactive file picker** - Fuzzy search and select any tracked file with popup interface
- 🔍 **File history visualization** with commit dates, hashes, authors, and subjects
- 🔄 **Working directory support** - See uncommitted changes as the top entry
- 🎯 **Interactive navigation** with vim-style keybindings and focus-aware panels
- 🎨 **Colorized diffs** with visual highlighting for additions, deletions, and context
- 🔀 **Rename tracking** - Follow files across renames and moves (with `--follow`)
- ⚡ **Performance optimized** with LRU caching for instant diff switching
- 🖥️ **Split-pane interface** with resizable panels and help overlay

### User Experience
- 🔄 **Seamless file switching** - Switch between files without losing context using 'f' key
- 🍎 **Mac-friendly navigation** - Multiple scroll options (PageUp/Down, Ctrl+D/U, Ctrl+F/B)
- 📱 **Focus-aware controls** - Arrow keys work differently based on active panel
- 🎹 **Comprehensive keybindings** - Vim, emacs, and traditional navigation styles
- 🔧 **Merge commit handling** - Proper parent resolution for complex histories
- ⚙️ **Configurable context** - Adjust diff context lines via CLI arguments

## 🚀 Installation

### From Source (Current)
```bash
git clone https://github.com/yourusername/geschichte.git
cd geschichte
cargo build --release
cargo install --path .
```

### Using Cargo (Coming Soon)
```bash
cargo install geschichte
```

### Homebrew (Planned)
```bash
brew install geschichte
```

## 📖 Usage

### Basic Usage
```bash
# Open file picker to browse and select any tracked file
geschichte

# View history for a specific file
geschichte src/main.rs
geschichte README.md
geschichte path/to/any/file.txt
```

### Command Line Options
```bash
geschichte [OPTIONS] [FILE]

Arguments:
  [FILE]  Path to the file to view history for (optional - opens file picker if not provided)

Options:
  -C, --repo <DIR>             Repository root directory (auto-discovered if not specified)
  -L, --lines <CONTEXT_LINES>  Number of context lines in diffs [default: 3]
      --first-parent           Show only first-parent commits (linearize merges)
      --no-follow              Disable rename tracking
      --debug                  Enable debug logging
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples
```bash
# Open file picker to browse all tracked files
geschichte

# More context in diffs
geschichte -L 10 src/main.rs

# Disable rename tracking for performance
geschichte --no-follow large-file.txt

# Linear history only (ignore merge commits)
geschichte --first-parent main.rs
```

## ⌨️ Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `Tab` | Switch between commit list and diff panels |
| `↑↓` / `j/k` | Navigate commits OR scroll diff (focus-aware) |
| `h/l` | Resize split panes |

### Scrolling (Multiple Options)
| Key | Action | Style |
|-----|--------|-------|
| `PageUp/PageDown` | Scroll diff | Traditional |
| `Ctrl+U/Ctrl+D` | Scroll diff | Vim-style |
| `Ctrl+B/Ctrl+F` | Scroll diff | Emacs-style |

### File Switching
| Key | Action |
|-----|--------|
| `f` | Open file picker to switch to another file |

### File Picker (when open)
| Key | Action |
|-----|--------|
| `↑↓` / `Ctrl+P/N` | Navigate file list |
| `Enter` | Select file and view history |
| `Esc` | Return to previous file (or quit if no previous file) |
| Type characters | Fuzzy search files |
| `Ctrl+U` | Clear search |

### General
| Key | Action |
|-----|--------|
| `?` | Show/hide help overlay |
| `q` / `Esc` | Quit (context-aware) |

### Coming Soon
| Key | Action |
|-----|--------|
| `/` | Search in diff |
| `c` | Copy commit hash |
| `m` | Cycle merge parents |

## 🎨 Interface

### File Picker Mode
```
                   ┌─ Select File ─────────────────────────┐
                   │ ┌─ Search ─────────────────────────┐ │
                   │ │ 🔍 main                          │ │
                   │ └─────────────────────────────────────┘ │
                   │ ▲ src/main.rs        Modified     │
                   │ > src/app.rs         2024-08-15   │
                   │   src/cli.rs         2024-08-14   │
                   │   README.md          2024-08-13   │
                   │   Cargo.toml         2024-08-12   │
                   │   ...                              │
                   │ 📁 42 files • 4 matches • ↑↓: navigate • Enter: select • Esc: quit
                   └───────────────────────────────────────┘
```

### History View Mode
```
┌─ Commits ─────────────────────┐┌─ Diff ─────────────────────────┐
│ > Working Dir Modified        ││ diff --git a/src/main.rs       │
│   2025-08-15 77942bc Latest   ││ @@ -10,6 +10,8 @@ fn main() { │
│   2025-08-14 3f30143 Feature  ││  fn main() {                   │
│   2025-08-13 603c9b0 Phase-2  ││ -    println!("Hello");       │
│   ...                         ││ +    println!("Hello, world!");│
└───────────────────────────────┘└─────────────────────────────────┘
[main.rs@77942bc] Tab: switch panels | f: switch file | ?: help | q: quit
```

## 🎯 Why Geschichte?

**Fast & Focused**: Unlike heavyweight GUI tools, Geschichte is built for speed and terminal workflows.

**Rename-Aware**: Tracks files across renames and moves, showing the complete evolution of your code.

**Working Directory Integration**: See your uncommitted changes alongside git history in one unified view.

**Keyboard-Driven**: Efficient navigation with vim-style keybindings, plus Mac-friendly alternatives.

**Developer-Friendly**: Built by developers, for developers who live in the terminal.

## 🛣️ Roadmap

### Upcoming Features
- 🔍 **In-diff search** - Search within diff content with regex support
- 🔄 **Merge parent cycling** - Navigate through merge commit parents
- 📋 **Copy commit hash** - Quick clipboard integration

### Future Enhancements
- 🎨 **Syntax highlighting** - Code-aware diff visualization
- ⚙️ **Configuration files** - Customizable themes and keybindings
- 📊 **Performance optimizations** - Handle massive repositories efficiently
- 📱 **Side-by-side diff view** - Alternative layout option

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
```bash
git clone https://github.com/yourusername/geschichte.git
cd geschichte
cargo build
cargo run -- src/main.rs
```

### Running Tests
```bash
cargo test
cargo clippy
cargo fmt
```

## 📊 Performance

- **Startup time**: < 500ms for typical repositories
- **Memory usage**: < 20MB for 1000+ commits
- **Diff caching**: LRU cache holds 50 diffs for instant navigation
- **Large repositories**: Tested with 10k+ commit histories

## 🔧 Dependencies

Built with these excellent Rust crates:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [clap](https://github.com/clap-rs/clap) - Command line parsing
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by `tig`, `lazygit`, and other excellent terminal git tools
- Built with the amazing Rust ecosystem
- Special thanks to the `ratatui` community for the excellent TUI framework

---

**Etymology**: *Geschichte* is German for "history" or "story" - fitting for a tool that helps you explore the story of your code.
