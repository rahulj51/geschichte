Single-File Git History & Diff TUI — Implementation Plan

A focused terminal UI that, when launched for a given path, shows commits touching that file on the left and the file’s diff for the selected commit on the right. Optimized for speed, simplicity, and rename awareness.

1) Scope & Goals

Primary goal

Launch with sfdiff <path/to/file> and get a two-pane view:

Left: commits that touched the file (rename-aware)

Right: unified diff for the file in the selected commit (vs its parent)

Non-goals (MVP)

Multi-file, staging, amending commits, or full repo management

True side-by-side diffs (can be a v2 feature)

Quality bar

Handles large histories smoothly, including renames

No surprises: what Git would show in CLI is what you see, filtered to one file

2) Tech Stack

Language: Rust (fast, single static binary, safe)

TUI: ratatui (layout, widgets) + crossterm (input)

Git integration: shell out to git (robust, respects user config)

Diff coloring: simple line-prefix colorization (+, -, @@) in MVP

Optional later: integrate delta output; ANSI translation module

Rationale: Shelling to Git avoids libgit edge cases and effortlessly inherits user config (rename detection thresholds, attributes, etc.).

3) UX at a Glance
┌───────────────────── Commits (for this file) ─────────────────────┐┌──────────────────────────── Diff ────────────────────────────┐
│  2025-08-12  3b7ad91  Fix: handle rename edge case               ││ @@ -45,7 +45,10 @@ fn render_diff(...)                    │
│  2025-08-10  9f1c003  Refactor: extract parser                   ││ - old line                                                      │
│  2025-08-08  a1c2e4f  Add basic TUI                               ││ + new line                                                      │
│  …                                                               ││ …                                                               │
└───────────────────────────────────────────────────────────────────┘└───────────────────────────────────────────────────────────────┘
 Status: repo=/path/repo  file=src/main.rs@a1c2e4f   ↑/↓ move  PgUp/PgDn scroll  / find  n/N next/prev  q quit


Key bindings (MVP)

↑/↓ or j/k: select commit

PgUp/PgDn / Ctrl+u/Ctrl+d: scroll diff

/ then text: search in diff; n/N next/prev

h/l or ←/→: resize split (optional)

q: quit

r: toggle rename awareness (fallback)

m: cycle merge parent (if commit is a merge)

4) CLI Interface
sfdiff <path> [options]

Options:
  -C, --repo <dir>        Explicit repo root (default: auto-discover)
  -L, --lines <n>         Diff context lines (default: 3)
  --first-parent          Linearize merges by first parent in log
  --no-follow             Disable rename tracking in history
  --word-diff             Show word-level changes (v2)
  --side-by-side          Side-by-side diff (v2)
  -v, --version           Version
  -h, --help              Help


Behavior

If <path> is relative, resolve against current working directory.

Auto-discover repo root via git rev-parse --show-toplevel.

5) Architecture Overview

Crates

[dependencies]
anyhow = "1"
ratatui = "0.28"
crossterm = "0.28"
parking_lot = "0.12"
regex = "1"
lazy_static = "1"


Module layout

src/
  main.rs            // arg parsing, app boot
  app.rs             // AppState, events, update loop
  ui.rs              // drawing, layout, widgets
  gitio.rs           // shell helpers for git log/show/diff, path-at-commit
  diff.rs            // parsing & colorizing unified diffs
  cache.rs           // LRU for per-commit diffs
  search.rs          // in-diff search (regex, case toggle later)


Core types

struct CommitRow {
    hash: String,         // full SHA
    short: String,        // short SHA
    date: String,         // yyyy-mm-dd
    author: String,
    subject: String,
    rename: Option<(String, String)>, // old->new at this commit, if any
}

struct AppState {
    repo_root: PathBuf,
    cli_path: PathBuf,        // path user passed
    path_at_head: PathBuf,    // normalized current path
    commits: Vec<CommitRow>,
    selected: usize,
    diff: DiffBuffer,         // current diff text + syntax spans
    scroll: usize,
    lru: DiffCache,           // commit-hash -> DiffBuffer
    rename_map: HashMap<String, PathBuf>, // commit hash -> path at that commit
    search: SearchState,      // query, matches, current idx
    first_parent: bool,
}

6) Git Integration (shell commands)

Discover repo root

git rev-parse --show-toplevel


Collect commits touching the file (rename-aware)

git log --follow --date=short \
  --format=%H%x09%ad%x09%an%x09%s -- path/to/file


Parse tab-separated fields into CommitRow.

Additionally, run once to build rename info:

git log --follow --name-status --format=%H -- path/to/file


Parse R<score>\told\tnew lines to track path changes per commit.

Parents for a commit (merge handling)

git rev-list --parents -n1 <hash>
# returns "<hash> <parent1> <parent2> ..."


File diff for selected commit vs parent

git diff --patch --unified=3 --find-renames <parent> <hash> -- <path-at-commit>


For root commit (no parent): git show --patch --unified=3 <hash> -- <path-at-commit>

Respect --lines <n> from CLI by swapping --unified=<n>.

Path at a specific commit

git ls-tree --name-only <hash> -- <path>   # verify presence; fallback to rename map

7) Diff Parsing & Rendering

MVP approach

Treat diff as plain text but color by prefix:

Lines starting with + → green

Lines starting with - → red

Lines starting with @@ (hunk headers) → bold/blue

Other lines (context/file headers) → dim

Use regex to identify @@ -a,b +c,d @@ hunk headers for quick hunk nav.

Wrap long lines at viewport width (ratatui handles wrapping in Paragraph).

Later

Word-diff mode: run git diff --word-diff=plain and parse markers {+ +}, [- -].

Side-by-side: compute alignment per hunk (non-trivial; v2).

8) Rename Awareness

Goal: When a commit renames the file, show the right old/new path and still render the diff.

Strategy

Build rename_map by scanning git log --follow --name-status once.

For each commit H, resolve path_at_commit(H):

If H has an R* old -> new, use old for the diff target before the rename, new after.

When missing, fallback to previous known path (walk history).

Toggle

r key toggles follow-renames off if user wants raw path history only.

9) TUI Rendering & Interaction

Layout

Horizontal split: left fixed width (e.g. 40), right flexible.

Bottom status line: repo root, file, current commit short SHA, help hints.

Widgets

Left: List with highlighted selection; show date short subject

Right: Paragraph of styled Spans (diff lines), scrollable

Event loop

Poll input @ ~50–100ms

On selection change:

Check LRU; if missing, spawn blocking diff fetch (simple MVP) → store in cache → redraw

Scroll management: keep cursor in view; PgUp/PgDn adjust scroll by page size

10) Performance & Caching

Lazy diff loading: compute only for the selected commit

LRU cache: keep last N (e.g., 20) diffs; evict oldest

Virtualization: store diff as Vec<Line>; render only visible slice

Fast search: pre-index hunk starts; regex over visible page + next chunk (simple MVP)

Optimization later:

Use a worker thread for diff; cancel if selection changes quickly (debounce ~100ms)

11) Error Handling & Edge Cases

Not a git repo → friendly message with detected CWD

Path not found at commit → show “file absent in this commit”

Binary diffs → detect GIT binary patch; show placeholder with size/SHAs

Merges → default to first parent; allow cycling parent with m

Huge diffs → cap initial render length with “load more” (or just paginate)

No commits → graceful “no history for file”

12) Testing Strategy

Unit

Parse git log rows into CommitRow

Diff colorization by prefix (sample inputs → styled output markers)

Rename map construction from --name-status sample

Integration (golden snapshots)

A tiny fixture repo with scripted history:
add → edit → rename → edit → delete

Snapshot expected left-pane rows

Snapshot a few diffs (use stable context lines)

Manual

Run against a real repo; confirm parity with git log --follow and git diff

Tooling

Consider insta for snapshot tests

CI builds for macOS/Linux; run fixture tests

13) Build, Packaging, & Release

Build

cargo build --release
# target/release/sfdiff


Install

Copy binary to PATH (/usr/local/bin) or provide Homebrew tap later

Versioning

Semantic versioning: 0.1.0 for MVP

Changelog generation (e.g., git-cliff)

Homebrew (later)

Create a tap repo with formula pointing to GitHub release tarball

14) Milestones (Checklist)

M0 — Skeleton

 Parse args; repo root discovery

 ratatui app with two panes + static sample data

M1 — History & Selection

 git log --follow for commits → left pane

 Selection updates state; bottom status shows SHA

M2 — Diff Rendering

 Fetch diff for selected commit (vs parent)

 Colorize by prefix, scrolling, paging

M3 — Rename Awareness

 Build rename map; resolve path-at-commit

 Show old → new marker on rename commits

M4 — Polishing

 Search /, n/N

 Merge parent cycling m

 Errors, empty states, binary + huge diffs

M5 — Packaging

 Release binary; README with demo gif

 Optional: Homebrew formula

15) Minimal Pseudocode
main():
  args = parse_cli()
  repo = git_root(args.repo or cwd)
  file = resolve_path(args.path)
  commits = git_log_follow(file)
  rename_map = build_rename_map(file)

  state = AppState{...}
  tui::run(state)

on_commit_select(hash):
  if cache.contains(hash):
    state.diff = cache.get(hash)
  else:
    parent = first_parent(hash)
    path_at = path_at_commit(hash, rename_map, file)
    diff_txt = git_diff(parent, hash, path_at, context=lines)
    state.diff = colorize(diff_txt)
    cache.put(hash, state.diff)

draw(frame, state):
  left = render_commit_list(state.commits, state.selected)
  right = render_diff(state.diff, state.scroll)
  status = render_status(...)

16) Future Upgrades (Post-MVP)

Side-by-side diffs with intra-line highlights

Word-diff toggle; whitespace-ignore toggle

Blame overlay for current commit (b)

Copy hunk to clipboard; save patch

Render ANSI from git delta output (if present)

Perf: async worker with cancel; batched rendering

Mouse support for scrolling and pane resizing

Config file (~/.config/sfdiff/config.toml) for colors/keys
