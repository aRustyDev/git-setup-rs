# Rust Patterns and Best Practices for Git-Setup

## Error Handling Patterns

### Custom Error Types with thiserror
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitSetupError {
    #[error("Configuration file not found: {path}")]
    ConfigNotFound { path: String },

    #[error("Profile '{name}' already exists")]
    DuplicateProfile { name: String },

    #[error("Failed to execute {command}: {reason}")]
    ExternalCommand {
        command: String,
        reason: String,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),
}

// Use Result type alias for cleaner signatures
pub type Result<T> = std::result::Result<T, GitSetupError>;
```

### Error Context with anyhow
```rust
use anyhow::{Context, Result};

pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Invalid TOML in {}", path.display()))?;

    Ok(config)
}
```

### Early Returns and ? Operator
```rust
// Good - use ? for early returns
pub fn apply_profile(name: &str) -> Result<()> {
    let profile = load_profile(name)?;
    validate_profile(&profile)?;
    apply_git_config(&profile)?;
    Ok(())
}

// FORBIDDEN - NEVER use unwrap in ANY code
pub fn apply_profile_bad(name: &str) {
    let profile = load_profile(name).unwrap(); // WILL FAIL CODE REVIEW
    // Use .expect() ONLY in tests
    // Use ? or match in production code
}
```

## Builder Pattern

### ONLY Use When Required (3+ optional fields)
**DO NOT** create builders "just in case" - YAGNI applies
```rust
#[derive(Default)]
pub struct ProfileBuilder {
    name: Option<String>,
    email: Option<String>,
    key_type: Option<KeyType>,
    signing_key: Option<String>,
    scope: Option<Scope>,
}

impl ProfileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn key_type(mut self, key_type: KeyType) -> Self {
        self.key_type = Some(key_type);
        self
    }

    pub fn build(self) -> Result<Profile> {
        let name = self.name.ok_or(GitSetupError::Validation(
            "Profile name is required".to_string()
        ))?;

        let email = self.email.ok_or(GitSetupError::Validation(
            "Email is required".to_string()
        ))?;

        Ok(Profile {
            name,
            git_user_email: email,
            key_type: self.key_type.unwrap_or(KeyType::Ssh),
            signing_key: self.signing_key.unwrap_or_default(),
            scope: self.scope.unwrap_or(Scope::Local),
            ..Default::default()
        })
    }
}

// Usage
let profile = ProfileBuilder::new()
    .name("work")
    .email("work@example.com")
    .key_type(KeyType::Ssh)
    .build()?;
```

## Type State Pattern

### For Multi-Step Processes
```rust
// Type states for profile creation wizard
pub struct NameStep;
pub struct EmailStep;
pub struct KeyTypeStep;
pub struct Complete;

pub struct ProfileWizard<State> {
    data: ProfileData,
    _state: PhantomData<State>,
}

impl ProfileWizard<NameStep> {
    pub fn new() -> Self {
        ProfileWizard {
            data: ProfileData::default(),
            _state: PhantomData,
        }
    }

    pub fn with_name(mut self, name: String) -> Result<ProfileWizard<EmailStep>> {
        validate_name(&name)?;
        self.data.name = name;
        Ok(ProfileWizard {
            data: self.data,
            _state: PhantomData,
        })
    }
}

impl ProfileWizard<EmailStep> {
    pub fn with_email(mut self, email: String) -> Result<ProfileWizard<KeyTypeStep>> {
        validate_email(&email)?;
        self.data.email = email;
        Ok(ProfileWizard {
            data: self.data,
            _state: PhantomData,
        })
    }
}

// Can only call finish() on Complete state
impl ProfileWizard<Complete> {
    pub fn finish(self) -> Profile {
        self.data.into()
    }
}
```

## Smart Pointer Patterns

### Arc for Shared State
```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct App {
    profile_manager: Arc<Mutex<ProfileManager>>,
    op_client: Arc<dyn OnePasswordOperations>,
    cache: Arc<SecureCache>,
}

impl App {
    pub async fn handle_command(&self, cmd: Command) -> Result<()> {
        match cmd {
            Command::AddProfile(name) => {
                let mut manager = self.profile_manager.lock().await;
                manager.add_profile(name).await?;
            }
            // ...
        }
        Ok(())
    }
}
```

### Cow for Efficient String Handling
```rust
use std::borrow::Cow;

pub fn expand_path(path: &str) -> Cow<str> {
    if path.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_default();
        Cow::Owned(path.replacen('~', &home, 1))
    } else {
        Cow::Borrowed(path)
    }
}
```

## Trait Patterns

### Extension Traits
```rust
pub trait ProfileExt {
    fn display_name(&self) -> String;
    fn is_onepassword_enabled(&self) -> bool;
}

impl ProfileExt for Profile {
    fn display_name(&self) -> String {
        format!("{} <{}>", self.name, self.git_user_email)
    }

    fn is_onepassword_enabled(&self) -> bool {
        self.one_password && !self.vault_name.is_empty()
    }
}
```

### Trait Objects for Polymorphism
```rust
pub struct ExternalToolManager {
    git: Box<dyn GitOperations>,
    op: Box<dyn OnePasswordOperations>,
    gpg: Box<dyn GpgOperations>,
}

impl ExternalToolManager {
    pub fn new_real() -> Self {
        Self {
            git: Box::new(RealGit::new()),
            op: Box::new(RealOnePassword::new()),
            gpg: Box::new(RealGpg::new()),
        }
    }

    #[cfg(test)]
    pub fn new_mock(
        git: Box<dyn GitOperations>,
        op: Box<dyn OnePasswordOperations>,
        gpg: Box<dyn GpgOperations>,
    ) -> Self {
        Self { git, op, gpg }
    }
}
```

## Iterator Patterns

### Custom Iterators
```rust
pub struct ProfileIterator<'a> {
    profiles: &'a [Profile],
    index: usize,
}

impl<'a> Iterator for ProfileIterator<'a> {
    type Item = &'a Profile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.profiles.len() {
            let profile = &self.profiles[self.index];
            self.index += 1;
            Some(profile)
        } else {
            None
        }
    }
}

// Iterator combinators
pub fn find_matching_profiles<'a>(
    profiles: &'a [Profile],
    pattern: &str,
) -> impl Iterator<Item = &'a Profile> {
    profiles.iter()
        .filter(move |p| p.name.contains(pattern))
        .take(10)
}
```

## Async Patterns

### Concurrent Operations
```rust
use futures::future::try_join_all;

pub async fn validate_all_profiles(profiles: &[Profile]) -> Result<()> {
    let validation_futures = profiles.iter()
        .map(|p| validate_profile_async(p));

    try_join_all(validation_futures).await?;
    Ok(())
}

// Select pattern for timeouts
use tokio::time::{timeout, Duration};

pub async fn fetch_with_timeout(
    op: &dyn OnePasswordOperations,
    vault: &str,
    item: &str,
) -> Result<String> {
    timeout(
        Duration::from_secs(30),
        op.get_public_key(vault, item)
    )
    .await
    .map_err(|_| GitSetupError::Timeout)?
}
```

## Memory Safety Patterns

### String Handling Rules
```rust
// REQUIRED - use &str for validation
pub fn validate_email(email: &str) -> Result<()> {
    // MUST use regex from validation module, NOT simple contains check
    validation::validate_email(email)
        .map_err(|e| GitSetupError::Validation(e))
}

// FORBIDDEN - taking String when &str works
pub fn validate_email_bad(email: String) -> Result<()> {
    // WILL FAIL CODE REVIEW
}
```

### Zero-Copy Parsing (ONLY if proven necessary)
**WARNING**: Do NOT use unless profiling shows allocation bottleneck
```rust
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(borrow)]
pub struct ConfigRef<'a> {
    #[serde(borrow)]
    pub profiles: Vec<ProfileRef<'a>>,
}

#[derive(Deserialize)]
pub struct ProfileRef<'a> {
    pub name: &'a str,
    pub email: &'a str,
}
```

## Testing Patterns

### Test Fixtures
```rust
#[cfg(test)]
pub mod fixtures {
    use super::*;

    pub struct TestContext {
        pub temp_dir: TempDir,
        pub config_path: PathBuf,
        pub profiles: Vec<Profile>,
    }

    impl TestContext {
        pub fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.toml");

            Self {
                temp_dir,
                config_path,
                profiles: vec![
                    Profile::test_default("work"),
                    Profile::test_default("personal"),
                ],
            }
        }

        pub fn with_config(self) -> Self {
            let config = Config {
                profiles: self.profiles.clone(),
                ..Default::default()
            };
            config.save(&self.config_path).unwrap();
            self
        }
    }
}
```

### Parameterized Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("work", true ; "valid name")]
    #[test_case("my-profile", true ; "name with dash")]
    #[test_case("", false ; "empty name")]
    #[test_case("profile!", false ; "invalid character")]
    fn test_validate_profile_name(name: &str, expected: bool) {
        let result = validate_profile_name(name);
        assert_eq!(result.is_ok(), expected);
    }
}
```

## Performance Patterns

### Lazy Initialization
```rust
use once_cell::sync::Lazy;

static DEFAULT_SSH_PROGRAM: Lazy<String> = Lazy::new(|| {
    which::which("op-ssh-sign")
        .or_else(|_| which::which("ssh-keygen"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "ssh-keygen".to_string())
});
```

### String Interning for Repeated Values
```rust
use string_cache::DefaultAtom;

#[derive(Clone)]
pub struct Profile {
    pub name: DefaultAtom,
    pub key_type: KeyTypeAtom,
    // ...
}

// Define atoms for common strings
string_cache::atom!(KeyTypeAtom: "ssh", "gpg", "x509", "gitsign");
```

## Documentation Patterns

### Module Documentation
```rust
//! # Profile Management
//!
//! This module handles all profile-related operations including:
//! - CRUD operations on profiles
//! - Profile validation
//! - Profile matching and auto-detection
//!
//! ## Examples
//!
//! ```rust
//! use git_setup::profile::ProfileManager;
//!
//! let manager = ProfileManager::new(config)?;
//! manager.add_profile(profile)?;
//! ```

/// Manages git configuration profiles.
///
/// The `ProfileManager` is responsible for all profile operations
/// including creation, modification, deletion, and application.
pub struct ProfileManager {
    // ...
}
```

### Function Documentation
```rust
/// Applies a profile to the git configuration.
///
/// This function will:
/// 1. Load the specified profile
/// 2. Validate all settings
/// 3. Apply the configuration to git
/// 4. Update auxiliary files (allowed_signers, etc.)
///
/// # Arguments
///
/// * `name` - The profile name to apply
/// * `scope` - The git configuration scope
///
/// # Errors
///
/// Returns an error if:
/// - Profile not found
/// - Invalid configuration
/// - Git command fails
///
/// # Examples
///
/// ```rust
/// manager.apply_profile("work", GitScope::Local)?;
/// ```
pub fn apply_profile(&self, name: &str, scope: GitScope) -> Result<()> {
    // ...
}
```

## Development Workflow with Just

### TDD Workflow
```bash
# Start new feature with TDD
just new-module feature_name     # Creates module with test structure
just watch-test feature_name     # Watch and run tests as you code

# Run tests at different granularities
just test-quick                  # Fast feedback during development
just test                        # Run all tests
just test-coverage-open          # Check coverage with visual report

# Code quality checks
just pre-commit                  # Run before committing
just ci                          # Run full CI pipeline locally
```

### Debugging Workflow
```bash
# Different log levels
just debug list                  # Run with debug logging
just trace apply work            # Run with trace logging

# Analyze issues
just expand profile::manager     # Expand macros in a module
just todo                        # Find TODO items
just check-links                 # Verify documentation links
```

### Performance Analysis
```bash
# Build and analyze
just build-release               # Optimized build
just build-time                  # Analyze build performance
just size                        # Check binary size
just bench                       # Run benchmarks
just profile "list --output json"  # Profile with perf (Linux)
```

## Common Antipatterns to Avoid

### Don't Use `.unwrap()` in Production
```rust
// Bad
let config = load_config().unwrap();

// Good
let config = load_config()?;
```

### Don't Clone Unnecessarily
```rust
// Bad - unnecessary clone
fn process(profiles: Vec<Profile>) {
    for profile in profiles {
        // ...
    }
}

// Good - borrow instead
fn process(profiles: &[Profile]) {
    for profile in profiles {
        // ...
    }
}
```

### Don't Ignore Errors
```rust
// Bad
let _ = std::fs::remove_file(path);

// Good
if let Err(e) = std::fs::remove_file(path) {
    tracing::warn!("Failed to remove file: {}", e);
}
```

Remember: Rust's ownership system is your friend. Work with it, not against it!
