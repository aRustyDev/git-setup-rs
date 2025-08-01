# Examples Directory

This directory contains working examples for git-setup-rs development. Each subdirectory focuses on a specific aspect of the project.

## Directory Structure

- **profiles/** - Profile configuration examples (TOML, YAML, JSON)
- **1password/** - 1Password integration examples and mocks
- **tui/** - Terminal UI component examples
- **security/** - Security implementations (atomic writes, zeroization)

## Usage

Each example is designed to be run independently:

```bash
# Run a specific example
cargo run --example basic_profile

# Run all examples
cargo test --examples
```

## For Junior Developers

Start with the examples in this order:
1. `profiles/basic_profile.toml` - Simplest configuration
2. `security/atomic_writes.rs` - File safety basics
3. `tui/simple_menu.rs` - Basic TUI interaction
4. `1password/mock_integration.rs` - Testing without 1Password

Each example includes extensive comments explaining the concepts.