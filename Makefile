# Geschichte Makefile
# Common development tasks for the Geschichte project

.PHONY: all build test clean install lint fmt check clippy release help dev-install

# Default target
all: build

# Build the project in debug mode
build:
	@echo "Building geschichte (debug)..."
	cargo build

# Build optimized release version
release:
	@echo "Building geschichte (release)..."
	cargo build --release

# Run all tests
test:
	@echo "Running tests..."
	cargo test

# Run tests with output
test-verbose:
	@echo "Running tests (verbose)..."
	cargo test -- --nocapture

# Run clippy for linting
clippy:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt

# Check formatting
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt -- --check

# Full lint check (clippy + format check)
lint: clippy fmt-check
	@echo "Linting complete!"

# Quick check (faster than full build)
check:
	@echo "Running cargo check..."
	cargo check

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Install from source to ~/.cargo/bin
install:
	@echo "Installing geschichte..."
	cargo install --path . --force

# Development install (debug build with faster compilation)
dev-install:
	@echo "Installing geschichte (debug, faster compilation)..."
	cargo install --path . --debug --force

# Run the binary with a test file (requires install first)
demo: install
	@echo "Running demo on src/main.rs..."
	geschichte src/main.rs || echo "Demo requires a git repository"

# Full development workflow: format, lint, test, build
dev: fmt lint test build
	@echo "Development workflow complete!"

# Continuous integration workflow
ci: fmt-check clippy test build
	@echo "CI workflow complete!"

# Release workflow: full checks plus optimized build
pre-release: fmt-check clippy test release
	@echo "Pre-release checks complete!"
	@echo "Release binary available at: target/release/geschichte"

# Show project information
info:
	@echo "Geschichte - Git File History Viewer"
	@echo "===================================="
	@cargo --version
	@rustc --version
	@echo ""
	@echo "Project structure:"
	@find src -name "*.rs" | head -10
	@echo ""
	@echo "Available targets:"
	@echo "  build, test, lint, install, clean, demo, dev, ci, pre-release"

# Update dependencies
update:
	@echo "Updating dependencies..."
	cargo update

# Audit dependencies for security issues
audit:
	@echo "Auditing dependencies..."
	@command -v cargo-audit >/dev/null 2>&1 || { echo "Installing cargo-audit..."; cargo install cargo-audit; }
	cargo audit

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --open

# Benchmark (if we had benchmarks)
bench:
	@echo "Running benchmarks..."
	cargo bench || echo "No benchmarks configured yet"

# Show help
help:
	@echo "Geschichte Development Makefile"
	@echo "=============================="
	@echo ""
	@echo "Common tasks:"
	@echo "  make build       - Build debug version"
	@echo "  make release     - Build optimized release"
	@echo "  make test        - Run all tests"
	@echo "  make lint        - Run clippy + format check"
	@echo "  make fmt         - Format all code"
	@echo "  make install     - Install to ~/.cargo/bin"
	@echo "  make clean       - Clean build artifacts"
	@echo ""
	@echo "Development workflows:"
	@echo "  make dev         - Format + lint + test + build"
	@echo "  make ci          - CI workflow (checks + test + build)"
	@echo "  make pre-release - Full release preparation"
	@echo ""
	@echo "Utilities:"
	@echo "  make demo        - Install and run demo"
	@echo "  make docs        - Generate and open documentation"
	@echo "  make update      - Update dependencies"
	@echo "  make audit       - Security audit dependencies"
	@echo "  make info        - Show project information"
	@echo ""
	@echo "Use 'make <target>' to run a specific task"