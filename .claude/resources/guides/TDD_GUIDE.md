# Test-Driven Development Guide for Git-Setup

## TDD Philosophy

Test-Driven Development is not just about testing - it's about **design**. By writing tests first, you:
1. Define the interface before implementation
2. Think about edge cases upfront
3. Create better, more modular designs
4. Build confidence in your code

## The TDD Cycle

### Red → Green → Refactor

1. **RED**: Write a test that fails
   - The test describes what you want the code to do
   - Run it and see it fail (compilation failure counts!)

2. **GREEN**: Write the minimum code to pass
   - Write ONLY enough code to make the test pass
   - Do NOT add extra functionality
   - Do NOT write code for future tests

3. **REFACTOR**: Improve the code
   - Remove duplication
   - Improve naming
   - Extract functions/modules
   - The tests ensure you don't break anything

## MANDATORY TDD RULES

**VIOLATIONS WILL CAUSE PHASE FAILURE:**
1. Writing implementation code before tests
2. Writing more code than needed to pass current test
3. Skipping the RED phase (test must fail first)
4. Not running tests after each change
5. Commenting out failing tests instead of fixing them

## Test Categories for Git-Setup

### 1. Unit Tests
Location: `src/<module>/<file>_test.rs` or `src/<module>/tests.rs`

#### Configuration Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_empty_config_creates_default() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Act
        let config = Config::load_or_create(&config_path).unwrap();

        // Assert
        assert_eq!(config.defaults.email, "you@example.com");
        assert_eq!(config.profiles.len(), 0);
        assert!(config_path.exists());
    }

    #[test]
    fn test_profile_validate_missing_name() {
        // Arrange
        let profile = Profile {
            name: "".to_string(),
            git_user_email: "test@example.com".to_string(),
            key_type: KeyType::Ssh,
            ..Default::default()
        };

        // Act
        let result = profile.validate();

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Profile validation failed: name is required"
        );
    }

    #[test]
    fn test_add_duplicate_profile_fails() {
        // Arrange
        let mut config = Config::default();
        let profile = create_test_profile("work");
        config.add_profile(profile.clone()).unwrap();

        // Act
        let result = config.add_profile(profile);

        // Assert
        assert!(matches!(
            result.unwrap_err(),
            GitSetupError::DuplicateProfile(_)
        ));
    }
}
```

#### External Tool Tests (with Mocks)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_apply_ssh_profile_sets_git_config() {
        // Arrange
        let mut mock_git = MockGitOperations::new();
        mock_git
            .expect_set_config()
            .with(eq("user.name"), eq("John Doe"), eq(GitScope::Local))
            .times(1)
            .returning(|_, _, _| Ok(()));
        mock_git
            .expect_set_config()
            .with(eq("user.email"), eq("john@example.com"), eq(GitScope::Local))
            .times(1)
            .returning(|_, _, _| Ok(()));
        mock_git
            .expect_set_config()
            .with(eq("gpg.format"), eq("ssh"), eq(GitScope::Local))
            .times(1)
            .returning(|_, _, _| Ok(()));

        let profile = Profile {
            name: "work".to_string(),
            git_user_name: "John Doe".to_string(),
            git_user_email: "john@example.com".to_string(),
            key_type: KeyType::Ssh,
            ..Default::default()
        };

        let manager = ProfileManager::new(Box::new(mock_git));

        // Act
        let result = manager.apply_profile(&profile, GitScope::Local).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_onepassword_key_not_found() {
        // Arrange
        let mut mock_op = MockOnePasswordOperations::new();
        mock_op
            .expect_get_public_key()
            .with(eq("Personal"), eq("My SSH Key"))
            .times(1)
            .returning(|_, _| {
                Err(GitSetupError::OnePassword(
                    "Item not found in vault".to_string()
                ))
            });

        // Act
        let result = mock_op.get_public_key("Personal", "My SSH Key").await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }
}
```

### 2. Integration Tests
Location: `tests/integration/`

#### CLI Integration Tests
```rust
// tests/integration/cli_test.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_cli_add_profile() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.toml");

    Command::cargo_bin("git-setup")
        .unwrap()
        .args(&[
            "--file", config_file.to_str().unwrap(),
            "--add", "work"
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Profile 'work' created successfully"));

    // Verify config file was created
    assert!(config_file.exists());
    let content = std::fs::read_to_string(&config_file).unwrap();
    assert!(content.contains("name = \"work\""));
}

#[test]
fn test_cli_list_profiles_json_output() {
    let temp_dir = setup_test_config_with_profiles();

    Command::cargo_bin("git-setup")
        .unwrap()
        .args(&[
            "--file", temp_dir.path().join("config.toml").to_str().unwrap(),
            "--list",
            "--output", "json"
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_json());
}

#[test]
fn test_cli_apply_profile_not_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    Command::cargo_bin("git-setup")
        .unwrap()
        .arg("work")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in a git repository"));
}
```

#### File Permission Tests
```rust
#[test]
#[cfg(unix)]
fn test_config_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create config through the application
    let config = Config::default();
    config.save(&config_path).unwrap();

    // Check permissions
    let metadata = std::fs::metadata(&config_path).unwrap();
    let permissions = metadata.permissions();
    assert_eq!(permissions.mode() & 0o777, 0o600);
}
```

### 3. TUI Component Tests
Location: `tests/tui/`

```rust
// tests/tui/main_menu_test.rs
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_main_menu_navigation() {
    // Arrange
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = MainMenu::new();

    // Act - simulate key presses
    app.handle_key(KeyCode::Down);
    app.handle_key(KeyCode::Down);
    app.handle_key(KeyCode::Enter);

    // Assert
    assert_eq!(app.selected_index(), 2);
    assert_eq!(app.selected_action(), Some(MenuAction::AddProfile));
}

#[test]
fn test_profile_selector_fuzzy_search() {
    // Arrange
    let profiles = vec![
        create_test_profile("work"),
        create_test_profile("personal"),
        create_test_profile("opensource"),
    ];
    let mut selector = ProfileSelector::new(profiles);

    // Act - type "wor"
    selector.handle_input('w');
    selector.handle_input('o');
    selector.handle_input('r');

    // Assert
    let filtered = selector.filtered_profiles();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "work");
}
```

### 4. Property-Based Tests
Location: Alongside unit tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_profile_name_validation_accepts_valid_chars(
        name in "[a-zA-Z][a-zA-Z0-9_-]*"
    ) {
        let profile = Profile {
            name,
            git_user_email: "test@example.com".to_string(),
            ..Default::default()
        };
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_fuzzy_match_score_properties(
        input in "\\w{1,20}",
        pattern in "\\w{1,10}"
    ) {
        let score = fuzzy_match(&input, &pattern);

        // Properties that should always hold
        assert!(score >= 0.0 && score <= 100.0);

        if input == pattern {
            assert_eq!(score, 100.0, "Exact match should score 100");
        }

        if pattern.len() > input.len() {
            assert_eq!(score, 0.0, "Pattern longer than input should score 0");
        }
    }

    #[test]
    fn test_config_roundtrip(
        profiles in prop::collection::vec(arb_profile(), 0..10)
    ) {
        let config = Config {
            profiles,
            ..Default::default()
        };

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }
}

// Arbitrary profile generator for property tests
fn arb_profile() -> impl Strategy<Value = Profile> {
    (
        "[a-z]{3,10}",
        "[a-z]+@[a-z]+\\.[a-z]{2,3}",
        prop_oneof![
            Just(KeyType::Ssh),
            Just(KeyType::Gpg),
            Just(KeyType::X509),
        ],
    ).prop_map(|(name, email, key_type)| Profile {
        name,
        git_user_email: email,
        key_type,
        ..Default::default()
    })
}
```

## Testing Best Practices

### 1. Test Naming
Use descriptive names that explain:
- What is being tested
- Under what conditions
- What the expected outcome is

```rust
// Good
#[test]
fn test_apply_profile_with_missing_ssh_key_returns_error() { }

// Bad
#[test]
fn test_apply() { }
```

### 2. Arrange-Act-Assert Pattern
Structure tests clearly:

```rust
#[test]
fn test_example() {
    // Arrange - Set up test data and mocks
    let profile = create_test_profile();
    let mut mock = MockGitOperations::new();
    mock.expect_something().returning(|_| Ok(()));

    // Act - Perform the action being tested
    let result = do_something(&profile, &mock);

    // Assert - Verify the outcome
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}
```

### 3. Test Fixtures and Builders
Create helper functions for common test data ONLY when:
- The same data is used in 3+ tests
- The data setup is complex (>5 lines)
- Never create "just in case" helpers

```rust
// tests/common/mod.rs
pub fn create_test_profile(name: &str) -> Profile {
    Profile {
        name: name.to_string(),
        git_user_name: format!("{} User", name),
        git_user_email: format!("{}@example.com", name),
        key_type: KeyType::Ssh,
        signing_key: "ssh-ed25519 AAAAC3...".to_string(),
        ..Default::default()
    }
}

pub fn setup_test_config_dir() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        profiles: vec![
            create_test_profile("work"),
            create_test_profile("personal"),
        ],
        ..Default::default()
    };
    let config_path = temp_dir.path().join("config.toml");
    config.save(&config_path).unwrap();
    temp_dir
}
```

### 4. Testing Error Conditions
Always test the unhappy path:

```rust
#[test]
fn test_profile_validation_errors() {
    let test_cases = vec![
        (Profile { name: "".to_string(), ..default() }, "name is required"),
        (Profile { git_user_email: "".to_string(), ..default() }, "email is required"),
        (Profile { key_type: KeyType::Ssh, ssh_key_source: SshKeySource::File, ssh_key_path: "".to_string(), ..default() }, "SSH key path required for file source"),
    ];

    for (profile, expected_error) in test_cases {
        let result = profile.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(expected_error));
    }
}
```

### 5. Async Testing
Use tokio::test for async functions:

```rust
#[tokio::test]
async fn test_async_operation() {
    let mock = create_async_mock();
    let result = perform_async_operation(&mock).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_operations() {
    // MUST test actual concurrent behavior, not just compile
    let shared = Arc::new(Mutex::new(ProfileManager::new()));
    let mut handles = vec![];
    
    for i in 0..10 {
        let manager = shared.clone();
        handles.push(tokio::spawn(async move {
            manager.lock().await.create_profile(/* ... */).await
        }));
    }
    
    // Verify no data corruption
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

## Test Organization

### Directory Structure
```
git-setup-rs/
├── src/
│   ├── config/
│   │   ├── mod.rs
│   │   └── tests.rs         # Unit tests for config module
│   └── profile/
│       ├── mod.rs
│       └── manager.rs
│           └── #[cfg(test)] mod tests { }  # Inline tests
├── tests/
│   ├── common/
│   │   └── mod.rs           # Shared test utilities
│   ├── integration/
│   │   ├── cli_test.rs      # CLI integration tests
│   │   ├── config_test.rs   # Config file tests
│   │   └── workflow_test.rs # End-to-end workflows
│   └── tui/
│       └── components_test.rs
└── benches/
    └── fuzzy_match.rs       # Performance benchmarks
```

## Running Tests

### Using Just Commands
The project includes a Justfile with convenient test commands:

```bash
# Run all tests
just test

# Run tests with output
just test-verbose

# Run specific test
just test-one test_profile_validation

# Run only unit tests
just test-unit

# Run only integration tests
just test-integration

# Run with coverage and open report
just test-coverage-open

# Run property tests with more cases
just test-proptest

# Watch for changes and run tests
just watch

# Watch and run specific test
just watch-test test_profile_validation

# Run quick unit tests (parallel, faster)
just test-quick

# Run full CI pipeline locally
just ci
```

### Direct Cargo Commands
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_profile_validation

# Run tests in specific module
cargo test config::

# Run only integration tests
cargo test --test integration

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Run property tests with more cases
PROPTEST_CASES=1000 cargo test
```

### TDD Workflow with Just
```bash
# Create a new module with test structure
just new-module profile

# Start TDD workflow
just tdd-help  # Show TDD commands

# Create new test file
just tdd-new config

# Watch and develop
just watch-test my_new_test
```

### Continuous Integration
Ensure tests run on:
- Multiple OS (Linux, macOS, Windows)
- Multiple Rust versions (MSRV + stable + nightly)
- With different feature flags

## Remember

1. **Test behavior, not implementation**
2. **Each test should test one thing**
3. **Tests should be independent**
4. **Fast tests encourage running them often**
5. **A failing test should clearly indicate what's wrong**
6. **Refactor tests just like production code**

When you write tests first, you're designing the API. Make it nice!
