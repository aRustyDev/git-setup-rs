# Phase 2: Core Profile Management - Work Plan

## Prerequisites

Before starting Phase 2, ensure Phase 1 is solid and all concepts are understood.

**Required from Phase 1**:
- ✅ Atomic file operations working
- ✅ Secure memory types (SensitiveString)
- ✅ Configuration system (Figment)
- ✅ All Phase 1 checkpoints approved

**Required Knowledge**:
- **Rust Fundamentals**: Structs, traits, error handling (*critical*)
- **Git Config**: How Git stores settings (*critical*)
- **Serialization**: Converting data to/from files (*required*)
- **Validation**: Ensuring data correctness (*required*)

💡 **Junior Dev Resources**:
- 📚 [Git Config Explained](https://www.atlassian.com/git/tutorials/setting-up-a-repository/git-config) - Start here (15 min)
- 📖 [Serde Guide](https://serde.rs/data-model.html) - Quick serialization intro
- 📖 [The Rust Book Ch. 10](https://doc.rust-lang.org/book/ch10-00-generics.html) - Traits refresher
- 🔧 Practice: Complete `examples/profiles/` exercises first
- 📝 [Project Glossary](../../resources/rust-glossary.md) - Quick concept lookup
- ✅ [Profile Validation Framework](PROFILE_VALIDATION_FRAMEWORK.md) - **Essential** for Task 2.1.3
- 🔐 [Security Implementation Examples](../SECURITY_IMPLEMENTATION_EXAMPLES.md) - File security patterns

## Quick Reference - Essential Resources

### Git Commands to Know
```bash
# See all Git configs and where they come from
git config --list --show-origin

# Set a Git config value
git config --local user.name "Your Name"

# Unset a value
git config --unset user.email
```

### Key Documentation
- **[Profile Examples](../../examples/profiles/)** - Study these first!
- **[SPEC Requirements](../../.spec/SPEC.md#fr-001-profile-management-system)** - What we're building
- **[Git Config Docs](https://git-scm.com/docs/git-config)** - Official reference

### Visual Overview - Profile System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   git-setup-rs                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Profile Storage                Git Configuration       │
│  ───────────────                ─────────────────       │
│                                                         │
│  ~/.config/git-setup/           git config --local     │
│  ├── profiles/                  user.name = "Work"     │
│  │   ├── work.toml       ───>   user.email = "..."     │
│  │   ├── personal.toml          commit.gpgsign = true  │
│  │   └── client.toml                                   │
│  └── config.toml                git config --global    │
│                                 (optional)              │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Profile Structure Visualization

```
┌─── Profile: "work" ─────────────────────────────────┐
│                                                     │
│  Basic Info                 Git Config              │
│  ──────────                ──────────               │
│  name: "work"              user.name: "Alice"      │
│  description: "Company"     user.email: "a@co.com" │
│                                                     │
│  Signing Config            Remote Patterns          │
│  ──────────────           ────────────────         │
│  method: SSH               github.com/company/*    │
│  key: ~/.ssh/work_ed25519  Priority: 10           │
│                                                     │
│  Extensions                Metadata                 │
│  ──────────               ────────                 │
│  parent: "base"           created: 2024-01-20      │
│  overrides: [...]         modified: 2024-01-21     │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Testing Commands
```bash
# Run only profile tests
cargo test profile:: -- --nocapture

# Test a specific profile operation
cargo run --example profile_crud

# Check serialization works
cargo run --example profile_formats
```

## Overview

Phase 2 implements the core value of git-setup-rs: managing Git configurations through profiles. Users can create different profiles for work, personal, and client projects, switching between them instantly.

**What You'll Build**:
1. Profile data structures with validation
2. CRUD operations (Create, Read, Update, Delete)
3. Git configuration management
4. Profile templates and inheritance
5. Import/export functionality

**Success Looks Like**:
- User creates a "work" profile with corporate email
- User switches to "personal" profile for open source
- Configurations apply instantly and correctly

**Time Estimate**: 3 weeks (120 hours)
- Week 1: Data model and basic operations (40h)
- Week 2: Git integration and validation (40h)
- Week 3: Advanced features and polish (40h)

## Development Methodology

Continue TDD from Phase 1, with extra focus on:
- **Integration tests**: Actually test with Git
- **Validation tests**: Catch bad data early
- **Performance tests**: Keep operations fast

## Done Criteria Checklist

Before moving to Phase 3, ensure:
- [ ] All CRUD operations work correctly
- [ ] Profiles validate before saving
- [ ] Git configs apply at correct scope
- [ ] Profile inheritance works
- [ ] Import from 1Password succeeds
- [ ] Operations complete in <20ms
- [ ] Test coverage ≥85%
- [ ] All checkpoints approved

## Work Breakdown

### 2.1 Profile Data Model & Storage (32 hours)

**Complexity**: Medium - Critical foundation
**Focus**: Design right, implement carefully

#### Task 2.1.1: Design Profile Data Model (8 hours)

💡 **Junior Dev Concept**: Data Modeling with Serde
**What it is**: Creating Rust structures that can be saved to files
**Why we use it**: Users need to edit profiles in TOML/YAML, we need to load them
**Real Example**: User edits `work.toml`, our code loads it into a Profile struct

**Prerequisites**:
- [ ] Read: [Serde Guide](https://serde.rs/data-model.html) - First 2 pages
- [ ] Study: `examples/profiles/basic_profile.toml`
- [ ] Understand: Option<T> for optional fields

**Step-by-Step Implementation**:

1. **Create the Profile Structure** (2 hours)
   
   First, create the basic structure:
   ```rust
   // src/profile/model.rs
   
   use serde::{Deserialize, Serialize};
   use std::collections::HashMap;
   
   /// A Git profile containing user configuration
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Profile {
       /// Unique identifier (e.g., "work", "personal")
       pub name: String,
       
       /// Human-readable description
       #[serde(skip_serializing_if = "Option::is_none")]
       pub description: Option<String>,
       
       /// Parent profile to inherit from
       #[serde(skip_serializing_if = "Option::is_none")]
       pub extends: Option<String>,
       
       /// Git configuration
       pub git: GitConfig,
       
       /// Signing configuration
       #[serde(skip_serializing_if = "Option::is_none")]
       pub signing: Option<SigningConfig>,
       
       /// Remote URL patterns for auto-detection
       #[serde(default, skip_serializing_if = "Vec::is_empty")]
       pub remotes: Vec<RemotePattern>,
   }
   ```
   
   💡 **Tip**: `skip_serializing_if` prevents `field: null` in output files

2. **Design Git Configuration** (2 hours)
   ```rust
   /// Git-specific configuration
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct GitConfig {
       /// Git user.name
       pub user_name: String,
       
       /// Git user.email
       pub user_email: String,
       
       /// Additional Git config keys
       #[serde(default)]
       pub extra: HashMap<String, String>,
   }
   
   /// Configuration scope for Git
   #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
   #[serde(rename_all = "lowercase")]
   pub enum ConfigScope {
       Local,   // Repository-specific
       Global,  // User-wide
       System,  // System-wide
   }
   ```
   
   ⚠️ **Common Mistake**: Forgetting `#[serde(default)]` on collections
   ✅ **Instead**: Always use `default` for Vec, HashMap to handle missing fields

3. **Add Signing Configuration** (2 hours)
   ```rust
   /// Signing configuration for commits/tags
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct SigningConfig {
       /// Signing method
       pub method: SigningMethod,
       
       /// SSH key path or 1Password reference
       #[serde(skip_serializing_if = "Option::is_none")]
       pub ssh_key: Option<String>,
       
       /// GPG key ID
       #[serde(skip_serializing_if = "Option::is_none")]
       pub gpg_key: Option<String>,
       
       /// Auto-sign commits
       #[serde(default)]
       pub sign_commits: bool,
       
       /// Auto-sign tags
       #[serde(default)]
       pub sign_tags: bool,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "lowercase")]
   pub enum SigningMethod {
       Ssh,
       Gpg,
       X509,
       Sigstore,
   }
   ```

4. **Create Builder Pattern** (2 hours)
   ```rust
   /// Builder for creating profiles programmatically
   pub struct ProfileBuilder {
       name: String,
       description: Option<String>,
       extends: Option<String>,
       git: Option<GitConfig>,
       signing: Option<SigningConfig>,
       remotes: Vec<RemotePattern>,
   }
   
   impl ProfileBuilder {
       pub fn new(name: impl Into<String>) -> Self {
           Self {
               name: name.into(),
               description: None,
               extends: None,
               git: None,
               signing: None,
               remotes: Vec::new(),
           }
       }
       
       pub fn description(mut self, desc: impl Into<String>) -> Self {
           self.description = Some(desc.into());
           self
       }
       
       pub fn git_config(mut self, name: impl Into<String>, email: impl Into<String>) -> Self {
           self.git = Some(GitConfig {
               user_name: name.into(),
               user_email: email.into(),
               extra: HashMap::new(),
           });
           self
       }
       
       pub fn build(self) -> Result<Profile, ProfileError> {
           let git = self.git.ok_or(ProfileError::MissingGitConfig)?;
           
           Ok(Profile {
               name: self.name,
               description: self.description,
               extends: self.extends,
               git,
               signing: self.signing,
               remotes: self.remotes,
           })
       }
   }
   ```
   
   💡 **Builder Pattern**: Makes creating complex objects easier and safer

**Testing Your Work**:
```bash
# Run model tests
cargo test profile::model::tests

# Try the builder
cargo run --example profile_builder

# Test serialization
cargo run --example profile_serde
```

**Debugging Guide**:

**Error**: `missing field 'git'`
**Solution**: Make sure required fields are present in TOML/YAML

**Error**: `unknown field 'extra_field'`
**Solution**: Add `#[serde(flatten)]` or use `#[serde(deny_unknown_fields)]`

**Error**: Builder doesn't compile
**Solution**: Check all `self` parameters are `mut self`

**Visual Data Flow**:
```
User's TOML File          Profile Struct           Git Config
────────────────         ──────────────          ───────────
[profile]                Profile {               git config
name = "work"      ──→     name: "work",   ──→   user.name Alice
[profile.git]              git: GitConfig,       user.email alice@
user_name = "Alice"        ...                   work.com
```

**When You're Stuck**:
1. Check examples: `examples/profiles/basic_profile.toml`
2. Run specific test: `cargo test test_profile_builder`
3. Use `dbg!()` macro to inspect values
4. Ask in Slack: #rust-beginners (tag @serde-help)

#### Task 2.1.2: Implement Profile Storage (8 hours)

💡 **Junior Dev Concept**: File-Based Storage
**What it is**: Saving profiles as individual files in a directory
**Why this way**: Simple, portable, easy to backup, Git-friendly
**Real Example**: `~/.config/git-setup/profiles/work.toml`

**Prerequisites**:
- [ ] Understand: Phase 1 atomic file operations
- [ ] Read: `examples/security/atomic_writes.rs`
- [ ] Know: Path handling across platforms

**Step-by-Step Implementation**:

1. **Create Storage Trait** (2 hours)
   ```rust
   // src/profile/storage.rs
   
   use crate::profile::{Profile, ProfileError};
   use std::path::Path;
   
   /// Trait for profile storage backends
   pub trait ProfileStorage: Send + Sync {
       /// List all profile names
       fn list(&self) -> Result<Vec<String>, ProfileError>;
       
       /// Load a profile by name
       fn load(&self, name: &str) -> Result<Profile, ProfileError>;
       
       /// Save a profile
       fn save(&self, profile: &Profile) -> Result<(), ProfileError>;
       
       /// Delete a profile
       fn delete(&self, name: &str) -> Result<(), ProfileError>;
       
       /// Check if profile exists
       fn exists(&self, name: &str) -> Result<bool, ProfileError>;
   }
   ```

2. **Implement File Storage** (3 hours)
   ```rust
   use crate::fs::AtomicWrite;
   use std::path::PathBuf;
   
   /// File-based profile storage
   pub struct FileStorage {
       /// Directory containing profiles
       profiles_dir: PathBuf,
       /// File operations handler
       atomic_writer: Box<dyn AtomicWrite>,
   }
   
   impl FileStorage {
       pub fn new(profiles_dir: PathBuf) -> Result<Self, ProfileError> {
           // Create directory if it doesn't exist
           if !profiles_dir.exists() {
               std::fs::create_dir_all(&profiles_dir)
                   .map_err(|e| ProfileError::Storage(e.to_string()))?;
           }
           
           Ok(Self {
               profiles_dir,
               atomic_writer: Box::new(crate::fs::AtomicFileWriter::new()),
           })
       }
       
       /// Get path for a profile file
       fn profile_path(&self, name: &str) -> PathBuf {
           self.profiles_dir.join(format!("{}.toml", name))
       }
   }
   ```

3. **Implement Storage Methods** (3 hours)
   ```rust
   impl ProfileStorage for FileStorage {
       fn list(&self) -> Result<Vec<String>, ProfileError> {
           let mut profiles = Vec::new();
           
           // Read directory entries
           let entries = std::fs::read_dir(&self.profiles_dir)
               .map_err(|e| ProfileError::Storage(e.to_string()))?;
           
           for entry in entries {
               let entry = entry.map_err(|e| ProfileError::Storage(e.to_string()))?;
               let path = entry.path();
               
               // Check if it's a TOML file
               if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                   if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                       profiles.push(name.to_string());
                   }
               }
           }
           
           profiles.sort(); // Consistent ordering
           Ok(profiles)
       }
       
       fn load(&self, name: &str) -> Result<Profile, ProfileError> {
           let path = self.profile_path(name);
           
           // Read file contents
           let contents = std::fs::read_to_string(&path)
               .map_err(|e| ProfileError::NotFound(name.to_string()))?;
           
           // Parse TOML
           toml::from_str(&contents)
               .map_err(|e| ProfileError::ParseError(e.to_string()))
       }
       
       fn save(&self, profile: &Profile) -> Result<(), ProfileError> {
           let path = self.profile_path(&profile.name);
           
           // Serialize to TOML
           let contents = toml::to_string_pretty(profile)
               .map_err(|e| ProfileError::SerializeError(e.to_string()))?;
           
           // Write atomically
           self.atomic_writer.write_atomic(&path, contents.as_bytes())
               .map_err(|e| ProfileError::Storage(e.to_string()))?;
           
           Ok(())
       }
       
       fn delete(&self, name: &str) -> Result<(), ProfileError> {
           let path = self.profile_path(name);
           
           if !path.exists() {
               return Err(ProfileError::NotFound(name.to_string()));
           }
           
           // Create backup before deletion
           let backup_path = self.profiles_dir.join(format!("{}.toml.deleted", name));
           std::fs::copy(&path, &backup_path)
               .map_err(|e| ProfileError::Storage(e.to_string()))?;
           
           // Delete original
           std::fs::remove_file(&path)
               .map_err(|e| ProfileError::Storage(e.to_string()))?;
           
           Ok(())
       }
       
       fn exists(&self, name: &str) -> Result<bool, ProfileError> {
           Ok(self.profile_path(name).exists())
       }
   }
   ```

**Testing Your Implementation**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Create test profile
        let profile = ProfileBuilder::new("test")
            .git_config("Test User", "test@example.com")
            .build()
            .unwrap();
        
        // Save
        storage.save(&profile).unwrap();
        
        // Load
        let loaded = storage.load("test").unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.git.user_email, "test@example.com");
    }
    
    #[test]
    fn test_list_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Save multiple profiles
        for name in &["personal", "work", "client-a"] {
            let profile = ProfileBuilder::new(*name)
                .git_config("User", "user@example.com")
                .build()
                .unwrap();
            storage.save(&profile).unwrap();
        }
        
        // List should be sorted
        let profiles = storage.list().unwrap();
        assert_eq!(profiles, vec!["client-a", "personal", "work"]);
    }
}
```

**Debugging Guide**:

**Error**: `Permission denied`
**Solution**: Check directory permissions, use temp directory for tests

**Error**: `Profile not found`
**Solution**: Verify file exists, check name doesn't include `.toml`

**Error**: `Parse error`
**Solution**: Validate TOML syntax, check for missing quotes

#### Task 2.1.3: Profile Validation Framework (8 hours)

💡 **Junior Dev Concept**: Validation
**What it is**: Checking data is correct before using it
**Why critical**: Bad Git config can break repositories
**Real Example**: Ensuring email contains @ symbol

📖 **Complete Implementation Available**: See [Profile Validation Framework](PROFILE_VALIDATION_FRAMEWORK.md) for the full implementation with examples, error handling, and testing patterns.

---

## 🛑 CHECKPOINT 2.1: Profile Foundation Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without review and approval.

### Pre-Checkpoint Checklist

Complete ALL items before requesting review:

- [ ] Profile model handles all required fields
- [ ] Storage implementation with atomic writes
- [ ] Validation catches invalid data
- [ ] Tests cover happy path and errors
- [ ] Examples compile and run
- [ ] Documentation complete

### Review Process

1. **Create Pull Request**
   ```bash
   git checkout -b checkpoint/phase-2-1-profile-foundation
   git add .
   git commit -m "CHECKPOINT 2.1: Profile foundation complete"
   git push origin checkpoint/phase-2-1-profile-foundation
   ```

2. **Required Reviews**
   - Tech Lead: Data model design
   - Security: Storage implementation
   - QA: Test coverage

### Success Criteria
- Profile CRUD operations work
- Atomic file operations used
- Validation comprehensive
- Performance <20ms per operation

---

[Content continues with remaining tasks in same detailed format...]