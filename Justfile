# Git-Setup Rust Project Justfile
# This file defines common development tasks and workflows

# Default recipe - show available commands
default:
    @just --list

# Project setup - install dependencies and tools
setup:
    @echo "Installing development dependencies..."
    cargo install cargo-tarpaulin
    cargo install cargo-watch
    cargo install cargo-audit
    cargo install cargo-outdated
    cargo install cargo-expand
    @echo "Setup complete!"

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the project (debug mode)
run *ARGS:
    cargo run -- {{ARGS}}

# Run the project (release mode)
run-release *ARGS:
    cargo run --release -- {{ARGS}}

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run specific test
test-one TEST:
    cargo test {{TEST}} -- --nocapture

# Run only unit tests
test-unit:
    cargo test --lib

# Run only integration tests
test-integration:
    cargo test --test '*'

# Run tests with coverage
test-coverage:
    cargo tarpaulin --out Html --output-dir target/coverage

# Run tests with coverage and open report
test-coverage-open: test-coverage
    @echo "Opening coverage report..."
    @open target/coverage/tarpaulin-report.html || xdg-open target/coverage/tarpaulin-report.html

# Run property-based tests with more cases
test-proptest:
    PROPTEST_CASES=1000 cargo test

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run specific test
watch-test TEST:
    cargo watch -x "test {{TEST}} -- --nocapture"

# Format code
fmt:
    cargo fmt

# Format and check
fmt-check:
    cargo fmt -- --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run clippy with pedantic lints
lint-pedantic:
    cargo clippy -- -W clippy::pedantic

# Fix clippy suggestions
lint-fix:
    cargo clippy --fix -- -D warnings

# Check for security vulnerabilities
audit:
    cargo audit

# Check for outdated dependencies
outdated:
    cargo outdated

# Update dependencies
update:
    cargo update

# Clean build artifacts
clean:
    cargo clean

# Deep clean including all target directories
clean-all:
    cargo clean
    rm -rf target/
    rm -rf Cargo.lock

# Check code (fast compile check)
check:
    cargo check

# Check all targets
check-all:
    cargo check --all-targets

# Generate documentation
doc:
    cargo doc --no-deps

# Generate and open documentation
doc-open:
    cargo doc --no-deps --open

# Expand macros for debugging
expand MODULE:
    cargo expand {{MODULE}}

# Run benchmarks
bench:
    cargo bench

# Create a new release build for all platforms
release-build:
    @echo "Building for current platform..."
    cargo build --release
    @echo "Build complete: target/release/git-setup"

# Cross-compile for multiple platforms (requires cross)
release-cross:
    @echo "Building for multiple platforms..."
    cross build --release --target x86_64-unknown-linux-gnu
    cross build --release --target x86_64-apple-darwin
    cross build --release --target x86_64-pc-windows-gnu
    @echo "Cross-compilation complete!"

# Run pre-commit checks
pre-commit: fmt lint test
    @echo "All pre-commit checks passed!"

# Run full CI pipeline locally
ci: clean fmt-check lint check-all test test-integration audit
    @echo "CI pipeline passed!"

# TDD workflow - create a new test file
tdd-new MODULE:
    @mkdir -p src/{{MODULE}}
    @echo "#[cfg(test)]" > src/{{MODULE}}/tests.rs
    @echo "mod tests {" >> src/{{MODULE}}/tests.rs
    @echo "    use super::*;" >> src/{{MODULE}}/tests.rs
    @echo "" >> src/{{MODULE}}/tests.rs
    @echo "    #[test]" >> src/{{MODULE}}/tests.rs
    @echo "    fn test_example() {" >> src/{{MODULE}}/tests.rs
    @echo "        // Write your test first!" >> src/{{MODULE}}/tests.rs
    @echo "        assert_eq!(2 + 2, 4);" >> src/{{MODULE}}/tests.rs
    @echo "    }" >> src/{{MODULE}}/tests.rs
    @echo "}" >> src/{{MODULE}}/tests.rs
    @echo "Created test file: src/{{MODULE}}/tests.rs"

# Install git hooks
install-hooks:
    @echo "#!/bin/sh" > .git/hooks/pre-commit
    @echo "just pre-commit" >> .git/hooks/pre-commit
    @chmod +x .git/hooks/pre-commit
    @echo "Git hooks installed!"

# Profile the application (requires perf on Linux)
profile ARGS:
    cargo build --release
    perf record --call-graph=dwarf target/release/git-setup {{ARGS}}
    perf report

# Check project size and dependencies
size:
    @echo "=== Dependency tree ==="
    cargo tree
    @echo ""
    @echo "=== Binary size ==="
    @ls -lh target/release/git-setup 2>/dev/null || echo "Release binary not built yet"

# Run the application with debug logging
debug *ARGS:
    RUST_LOG=debug cargo run -- {{ARGS}}

# Run the application with trace logging
trace *ARGS:
    RUST_LOG=trace cargo run -- {{ARGS}}

# Test a specific integration scenario
test-scenario SCENARIO:
    cargo test --test integration -- {{SCENARIO}} --nocapture

# Generate test coverage badge (requires grcov)
coverage-badge:
    cargo tarpaulin --out Xml
    @echo "Coverage report generated: cobertura.xml"

# Quick development build and run
dev *ARGS: build
    ./target/debug/git-setup {{ARGS}}

# Verify all links in documentation
check-links:
    @echo "Checking documentation links..."
    @grep -r "http\|\.md" .claude/ --include="*.md" || true

# Platform-specific test
test-platform:
    @echo "Running platform-specific tests..."
    cargo test --features "$(rustc --print cfg | grep target_os | cut -d'"' -f2)"

# Create a new module with TDD structure
new-module NAME:
    @mkdir -p src/{{NAME}}
    @echo "pub mod {{NAME}};" >> src/lib.rs
    @echo "// {{NAME}} module" > src/{{NAME}}/mod.rs
    @just tdd-new {{NAME}}
    @echo "Module {{NAME}} created with test file!"

# Run minimal tests (for quick feedback during development)
test-quick:
    cargo test --lib -- --test-threads=4

# Analyze build times
build-time:
    cargo build --release --timings
    @echo "Build timing report: target/cargo-timings/cargo-timing.html"

# Show TODO items in code
todo:
    @grep -r "TODO\|FIXME\|HACK\|XXX" src/ --include="*.rs" || echo "No TODOs found!"

# Validate TOML files
validate-toml:
    @echo "Validating Cargo.toml..."
    @cargo verify-project
    @echo "Validating config files..."
    @find . -name "*.toml" -exec echo "Checking {}" \; -exec toml lint {} \; 2>/dev/null || true

# Test 1Password module specifically
test-onepassword:
    cargo test onepassword

# TDD workflow for 1Password module
tdd-onepassword:
    cargo watch -x "test onepassword"

# Test GPG module specifically
test-gpg:
    cargo test gpg

# TDD workflow for GPG module
tdd-gpg:
    cargo watch -x "test gpg"

# Help for TDD workflow
tdd-help:
    @echo "TDD Workflow Commands:"
    @echo "  just tdd-new MODULE     - Create a new test file"
    @echo "  just watch             - Watch and run all tests"
    @echo "  just watch-test TEST   - Watch and run specific test"
    @echo "  just test-one TEST     - Run a specific test"
    @echo "  just test-quick        - Run quick unit tests"
    @echo "  just test-onepassword  - Test OnePassword module"
    @echo "  just tdd-onepassword   - Watch OnePassword tests"
    @echo "  just test-gpg          - Test GPG module"
    @echo "  just tdd-gpg           - Watch GPG tests"
    @echo ""
    @echo "TDD Cycle:"
    @echo "  1. Write a failing test"
    @echo "  2. Run: just test-one <test_name>"
    @echo "  3. Write minimal code to pass"
    @echo "  4. Run: just test"
    @echo "  5. Refactor"
    @echo "  6. Run: just lint"
