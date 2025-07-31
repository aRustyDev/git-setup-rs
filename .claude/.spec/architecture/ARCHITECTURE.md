# Git-Setup Rust Architecture

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Interface                         │
│                    (Clap-based parsing)                      │
└─────────────────┬───────────────────────┬───────────────────┘
                  │                       │
                  ▼                       ▼
┌─────────────────────────┐     ┌─────────────────────────────┐
│    Command Handler      │     │      TUI Application        │
│  (Direct CLI commands)  │     │    (Ratatui-based UI)       │
└───────────┬─────────────┘     └──────────┬──────────────────┘
            │                              │
            ▼                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Profile Manager                          │
│              (Core business logic layer)                    │
└─────────────────┬───────────────────────┬───────────────────┘
                  │                       │
                  ▼                       ▼
┌─────────────────────────┐     ┌─────────────────────────────┐
│   Configuration Layer   │     │   External Tools Layer      │
│    (TOML persistence)   │     │  (Git, 1Password, GPG)      │
└─────────────────────────┘     └─────────────────────────────┘
```

## Module Structure

### Core Modules

#### 1. Error Module (`src/error.rs`)
Central error handling using `thiserror`:
- `GitSetupError` enum for all application errors
- Automatic error conversions from external errors
- Context-rich error messages

#### 2. Configuration Module (`src/config/`)
Handles all configuration and profile data:
- `Config` struct - main configuration container
- `Profile` struct - individual profile data
- `ConfigPaths` - git config file locations
- TOML serialization/deserialization
- Validation logic

#### 3. Profile Management (`src/profile/`)
Business logic for profile operations:
- `ProfileManager` - CRUD operations (MUST use FileProfileManager, NOT in-memory)
- `ProfileMatcher` - fuzzy matching (MUST implement SPEC Section 12.1 algorithm)
- `ScopeResolver` - determines effective scope (local > global > system)
- Profile import/export functionality (agent.toml ONLY for v1.0)

#### 4. External Tools (`src/external/`)
Abstractions for external tool integration:
- Trait definitions for testability (REQUIRED - no direct process calls)
- Concrete implementations (MUST follow SPEC Section 13 exactly)
- Mock implementations for testing (REQUIRED for all traits)
- Platform-specific adaptations (MUST use paths from SPEC Section 7)

```rust
// Trait hierarchy
pub trait GitOperations {
    async fn get_config(&self, key: &str, scope: GitScope) -> Result<Option<String>>;
    async fn set_config(&self, key: &str, value: &str, scope: GitScope) -> Result<()>;
}

pub trait OnePasswordOperations {
    async fn list_ssh_keys(&self) -> Result<Vec<SshKey>>;
    async fn create_ssh_key(&self, vault: &str, title: &str) -> Result<String>;
}

pub trait GpgOperations {
    async fn list_keys(&self) -> Result<Vec<GpgKey>>;
    async fn import_key(&self, key_data: &[u8], password: &str) -> Result<String>;
}
```

### CLI Layer (`src/cli/`)

#### Command Structure
```rust
pub enum Commands {
    // No subcommand - apply profile
    Apply { profile: String, scope: Scope },

    // Profile management
    Add { name: String },
    Edit { name: String },
    Delete { name: String },
    List { format: OutputFormat },
    Import,

    // Interactive mode
    Interactive,
}
```

#### Argument Parsing
- Uses Clap's derive API
- Maintains backward compatibility with Go version
- Environment variable support
- Hidden debug flags

### TUI Layer (`src/tui/`)

#### State Management
```rust
pub struct App {
    state: AppState,
    navigation_stack: Vec<AppState>,
    shared_state: SharedState,
}

pub enum AppState {
    MainMenu(MainMenuState),
    ProfileList(ProfileListState),
    AddProfile(AddProfileWizard),
    EditProfile(EditProfileState),
    ProfileSelector(ProfileSelectorState),
    InspectProfile(InspectProfileState),
}
```

#### Component Architecture
- Each screen is a separate component
- Shared navigation handling
- Modal dialog system
- Consistent styling through shared styles module

#### Navigation Flow
```
MainMenu
├── ProfileList
│   └── InspectProfile (modal)
├── ProfileSelector
│   └── InspectProfile (modal)
│       └── Apply Profile
├── AddProfile (wizard)
│   ├── Step 1: Name
│   ├── Step 2: Git Config
│   ├── Step 3: Key Type
│   ├── Step 4: Key Source
│   ├── Step 5: Additional
│   └── Step 6: Confirm
└── EditProfile
    └── Field Editor
```

## Data Flow

### Profile Creation Flow
1. User initiates profile creation (CLI or TUI)
2. Input validation at UI layer
3. ProfileManager validates business rules
4. If 1Password enabled:
   - Check/create SSH key in 1Password
   - Retrieve public key
5. Save profile to configuration
6. Update agent.toml if needed

### Profile Application Flow
1. User selects profile to apply
2. ProfileManager loads profile
3. Determine effective scope (local/global/system)
4. For each configuration value:
   - Call appropriate Git command
   - Handle platform-specific paths
5. Update allowed_signers file
6. Create includeIf configurations if needed

## Async Architecture

### Tokio Runtime
- Main runtime for async operations
- External tool calls are async
- TUI runs in separate thread to maintain responsiveness

### Concurrency Patterns
```rust
// Parallel vault listing
let (vaults, ssh_keys) = tokio::join!(
    op_client.list_vaults(),
    op_client.list_ssh_keys()
);

// Timeout handling
let result = tokio::time::timeout(
    Duration::from_secs(30),
    op_client.create_ssh_key(vault, title)
).await??;
```

## Security Architecture

### Principle of Least Privilege
- Never store private keys
- Tokens only from environment
- Minimal file permissions (0600)

### Data Protection
```rust
// Encrypted memory cache for 1Password data
pub struct SecureCache {
    data: Arc<Mutex<HashMap<String, SecureString>>>,
}

impl SecureCache {
    pub fn store(&self, key: String, value: String) {
        let encrypted = self.encrypt(value);
        self.data.lock().unwrap().insert(key, encrypted);
    }
}
```

### Input Validation
- Profile names: alphanumeric + dash/underscore
- Email validation
- Path traversal prevention
- Command injection prevention

## Testing Architecture

### Test Pyramid
```
         /\
        /  \    E2E Tests (少)
       /    \   - Complete workflows
      /──────\  - Real external tools
     /        \
    /          \  Integration Tests (中)
   /            \ - Module boundaries
  /──────────────\- File operations
 /                \
/                  \ Unit Tests (多)
────────────────────- Pure functions
                    - Mock externals
```

### Mock Strategy
- Traits for all external operations
- MockAll for automatic mock generation
- Fixture files for test data
- TempDir for file operations

## Performance Considerations

### Lazy Loading
```rust
impl Config {
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn load_profile(&self, name: &str) -> Result<Profile> {
        // Only parse when needed
        self.profiles.iter()
            .find(|p| p.name == name)
            .cloned()
            .ok_or_else(|| GitSetupError::ProfileNotFound(name.to_string()))
    }
}
```

### Caching Strategy
- 1Password results cached per session
- Git config values not cached (may change externally)
- Profile list cached until modification

## Platform Abstractions

### Platform Traits
```rust
pub trait Platform {
    fn home_dir(&self) -> Result<PathBuf>;
    fn config_dir(&self) -> Result<PathBuf>;
    fn ssh_agent_path(&self) -> &str;
    fn gpg_program(&self) -> &str;
}

#[cfg(target_os = "windows")]
pub struct WindowsPlatform;

#[cfg(target_os = "macos")]
pub struct MacOSPlatform;

#[cfg(target_os = "linux")]
pub struct LinuxPlatform;
```

### Conditional Compilation
```rust
#[cfg(unix)]
fn set_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(windows)]
fn set_permissions(_path: &Path) -> Result<()> {
    // Windows permissions handled differently
    Ok(())
}
```

## Extensibility Points

### Adding New Key Types
1. Add variant to `KeyType` enum
2. Implement configuration logic in `apply_profile`
3. Add external tool trait if needed
4. Update validation logic
5. Add tests

### Adding New Commands
1. Add variant to `Commands` enum
2. Implement handler function
3. Update CLI help text
4. Add integration tests

### Adding New TUI Screens
1. Create new state variant
2. Implement Component trait
3. Add navigation logic
4. Update help text

## Dependencies and Versioning

### Core Dependencies
- `clap` - CLI parsing (^4.0)
- `tokio` - Async runtime (^1.0)
- `ratatui` - TUI framework (^0.26)
- `serde` - Serialization (^1.0)
- `toml` - Configuration format (^0.8)

### Development Dependencies
- `mockall` - Mocking framework
- `proptest` - Property testing
- `tempfile` - Test file handling
- `assert_cmd` - CLI testing

### Version Compatibility
- MSRV: 1.70.0 (for async traits)
- Target latest stable Rust
- Follow semantic versioning
