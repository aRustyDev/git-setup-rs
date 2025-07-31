# Profile Module

This module implements the Profile Manager (C1) component for git-setup-rs, providing CRUD operations for managing git configuration profiles.

## Structure

- `mod.rs` - Module exports and the `ProfileManager` trait definition
- `manager.rs` - In-memory implementation of `ProfileManager` (`ProfileManagerImpl`)
- `mock.rs` - Mock implementation for testing (`MockProfileManager`)

## Features

### ProfileManager Trait

The trait defines the following operations:
- `create()` - Create a new profile with validation
- `read()` - Read a profile by name
- `update()` - Update an existing profile (supports renaming)
- `delete()` - Delete a profile by name
- `list()` - List all profiles sorted by name
- `exists()` - Check if a profile exists

### ProfileManagerImpl

The in-memory implementation provides:
- Thread-safe storage using `Arc<Mutex<HashMap>>`
- Profile validation (name format, email validation)
- Default profile management
- Profile search/filtering by pattern
- Support for profile renaming with automatic default profile updates

### Validation Rules

- Profile names must be non-empty and ≤ 100 characters
- Names can only contain alphanumeric characters, dashes, and underscores
- Names cannot contain path separators (/, \)
- Email addresses must contain @ and have valid format
- Case-sensitive profile names

### MockProfileManager

The mock implementation provides:
- All ProfileManager trait methods
- Configurable failure modes for testing error handling
- Pre-population with test data
- Thread-safe operation

## Usage

```rust
use git_setup_rs::profile::{ProfileManager, manager::ProfileManagerImpl};
use git_setup_rs::config::types::{Profile, KeyType};

// Create a new profile manager
let manager = ProfileManagerImpl::new();

// Create a profile
let profile = Profile {
    name: "work".to_string(),
    git_user_email: "work@example.com".to_string(),
    key_type: KeyType::Ssh,
    // ... other fields
};

manager.create(profile)?;

// List all profiles
let profiles = manager.list()?;

// Find profiles by pattern
let work_profiles = manager.find("work")?;

// Set default profile
manager.set_default("work")?;
```

## Testing

The module includes 40+ tests covering:
- All CRUD operations
- Validation edge cases
- Thread safety
- Error conditions
- Profile renaming
- Default profile management

Run tests with: `cargo test profile::`

## Future Enhancements

- File-based persistence (FileProfileManager)
- Profile templates
- Profile groups/tags
- Import/export functionality
- Migration support for schema changes
