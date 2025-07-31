# Just Quick Reference for Git-Setup Development

## Essential Commands

### Development Flow
```bash
just                    # Show all available commands
just setup              # One-time setup of dev tools
just dev <args>         # Quick build and run
just watch              # Auto-run tests on file changes
```

### TDD Commands
```bash
just new-module <name>  # Create module with test structure
just tdd-new <module>   # Create test file for existing module
just tdd-help           # Show TDD workflow help
just test-one <name>    # Run specific test
just watch-test <name>  # Watch and run specific test
```

### Testing
```bash
just test               # Run all tests
just test-quick         # Fast unit tests only
just test-verbose       # Tests with output
just test-unit          # Unit tests only
just test-integration   # Integration tests only
just test-coverage      # Generate coverage report
just test-coverage-open # Coverage with browser
just test-proptest      # Property tests (1000 cases)
```

### Code Quality
```bash
just fmt                # Format code
just fmt-check          # Check formatting
just lint               # Run clippy
just lint-fix           # Auto-fix clippy issues
just lint-pedantic      # Strict linting
just pre-commit         # Format, lint, and test
just ci                 # Full CI pipeline
```

### Building
```bash
just build              # Debug build
just build-release      # Release build
just check              # Fast compile check
just clean              # Clean build artifacts
```

### Debugging
```bash
just debug <args>       # Run with debug logging
just trace <args>       # Run with trace logging
just expand <module>    # Expand macros
just todo               # Find TODO/FIXME items
```

### Analysis
```bash
just audit              # Security audit
just outdated           # Check dependencies
just size               # Binary size info
just build-time         # Build performance
just bench              # Run benchmarks
```

## Common Workflows

### Starting a New Feature
```bash
just new-module my_feature
just watch-test my_feature
# Write tests and code...
just pre-commit
```

### Before Committing
```bash
just pre-commit         # Or individually:
just fmt
just lint
just test
```

### Investigating Issues
```bash
just test-verbose       # See test output
just debug list         # Debug logging
just expand config      # See expanded macros
```

### Checking Project Health
```bash
just ci                 # Run everything
just test-coverage-open # Check coverage
just audit              # Security check
just outdated           # Update check
```

## Tips

1. **Use `just watch` during development** - Instant feedback on file save
2. **Run `just pre-commit` before git commit** - Catches issues early
3. **Use `just test-quick` for rapid iteration** - Faster than full test suite
4. **Check `just todo` regularly** - Track work items in code
5. **Run `just ci` before pushing** - Ensures CI will pass

## Customization

Edit the Justfile to:
- Add project-specific commands
- Modify default behaviors
- Create command aliases
- Add new workflows

Remember: Just commands are composable - chain them for custom workflows!
