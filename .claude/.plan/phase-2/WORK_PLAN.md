# Phase 2: Core Profile Management - Work Plan

## Prerequisites

Phase 2 builds upon the secure foundation from Phase 1.

**Required from Phase 1**:
- Atomic file operations module
- Secure memory types (SensitiveString, SecureBuffer)
- Configuration infrastructure (Figment)
- All Phase 1 tests passing

**Required Knowledge**:
- **Git Internals**: Config file format, scopes, precedence (*critical*)
- **Rust Patterns**: Builder pattern, error handling, traits (*critical*)
- **Profile Systems**: Multi-profile management, inheritance (*required*)
- **Validation**: Schema validation, business rules (*required*)

💡 **Junior Dev Resources**:
- 📚 [Git Config Deep Dive](https://git-scm.com/book/en/v2/Customizing-Git-Git-Configuration) - Read sections 8.1-8.3
- 🎥 [Rust Builder Pattern](https://www.youtube.com/watch?v=SskSx5dg0_s) - 20 min video
- 📖 [Trait Objects Explained](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) - Dynamic dispatch
- 🔧 [Git Config Playground](https://learngitbranching.js.org/) - Practice Git concepts
- 📚 [Phase 1 Rust Glossary](../../resources/rust-glossary.md) - Reference for Rust concepts

## Quick Reference - Essential Resources

### Git Documentation
- [Git Config Documentation](https://git-scm.com/docs/git-config)
- [Git Config File Format](https://git-scm.com/docs/git-config#_syntax)
- [git2-rs Documentation](https://docs.rs/git2)

### Project Resources
- **[SPEC.md](../../spec/SPEC.md)** - See FR-001, FR-002 requirements
- **[Phase 1 API](../phase-1/)** - Secure types and file operations
- **[Profile Examples](../../examples/profiles/)** - Sample profile formats

### Commands
- `cargo test profile::` - Run profile tests
- `cargo run -- profile list` - Test profile listing
- `git config --list --show-origin` - Inspect Git config

## Overview

Phase 2 implements the core profile management system, enabling users to create, manage, and apply Git configurations through profiles. This phase delivers the fundamental value proposition of git-setup-rs.

**Key Deliverables**:
- Profile CRUD operations with validation
- Git configuration read/write at all scopes
- Profile inheritance and templates
- Import from 1Password agent.toml format
- Comprehensive validation framework

**Checkpoint Strategy**: 4 checkpoints for major features

**Time Estimate**: 3 weeks (120 hours)

## Development Methodology: Test-Driven Development (TDD)

Continue strict TDD with focus on:
1. **Profile validation tests** - Invalid profiles never applied
2. **Git integration tests** - Verify actual Git behavior
3. **Security tests** - No credential leaks in profiles
4. **Performance tests** - <20ms profile operations

## Done Criteria Checklist

Phase 2 is complete when:
- [ ] Profile CRUD operations working with <20ms response
- [ ] Git configs apply correctly at all scopes
- [ ] Profile inheritance via `extends` field works
- [ ] Import from 1Password agent.toml succeeds
- [ ] Validation prevents all invalid configurations
- [ ] Profile switching completes in <3 seconds
- [ ] Test coverage ≥85% overall
- [ ] All 4 checkpoints reviewed and approved

## Work Breakdown with Review Checkpoints

### 2.1 Profile Data Model & Storage (30 hours)

**Complexity**: High - Core domain model
**Files**: `src/profile/mod.rs`, `src/profile/model.rs`, `src/profile/storage.rs`

#### Task 2.1.1: Design Profile Schema (6 hours)

💡 **Junior Dev Concept**: Data Modeling in Rust
**What it is**: Designing structs that represent your application's core data
**Why it matters**: The profile model is the heart of git-setup - get it wrong and everything else becomes harder
**Key decisions**: What fields to include, how to handle optional data, serialization format

Define the profile data model with all fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: Option<String>,
    pub extends: Option<String>, // Parent profile
    pub user: UserConfig,
    pub signing: Option<SigningConfig>,
    pub proxy: Option<ProxyConfig>,
    pub custom: HashMap<String, Value>,
    #[serde(skip)]
    pub metadata: ProfileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
    pub signingkey: Option<String>, // op:// reference
}
```

**Key Decisions**:
- TOML as primary format (YAML/JSON supported)
- Inheritance via `extends` field
- Metadata separate from user data
- Custom fields for extensibility

**Step-by-Step Implementation**:

1. **Create the module structure** (30 minutes)
   ```bash
   mkdir -p src/profile
   touch src/profile/mod.rs src/profile/model.rs src/profile/storage.rs
   ```

2. **Design the core data structures** (2 hours)
   
   💡 **Why these fields?**:
   - `name`: Unique identifier for the profile
   - `extends`: Enables profile inheritance (DRY principle)
   - `user`: Core Git configuration
   - `signing`: Optional but common signing setup
   - `custom`: Flexibility for future Git features
   
   ⚠️ **Common Mistake**: Making everything required
   ✅ **Instead**: Use `Option<T>` for optional fields

3. **Add serialization support** (1.5 hours)
   ```rust
   // Ensure all fields can be serialized/deserialized
   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub struct Profile { /* ... */ }
   ```
   
   📚 **Learn More**: [Serde Documentation](https://serde.rs/)

4. **Write comprehensive tests** (2 hours)
   ```rust
   #[test]
   fn test_profile_serialization() {
       let profile = Profile {
           name: "work".to_string(),
           // ... set up test data
       };
       
       // Test TOML round-trip
       let toml = toml::to_string(&profile).unwrap();
       let parsed: Profile = toml::from_str(&toml).unwrap();
       assert_eq!(profile, parsed);
   }
   ```

#### Task 2.1.2: Implement Profile Storage (12 hours)

💡 **Junior Dev Concept**: Secure File Storage
**What it is**: Saving profiles to disk safely using Phase 1's atomic operations
**Why it matters**: Profile corruption = lost Git configuration = angry developers
**Key pattern**: Always use atomic writes, never modify files in-place

Build profile storage using Phase 1 components:

**Requirements**:
- Use atomic file operations for all writes
- Set 600 permissions on profile files
- Support `.toml`, `.yaml`, `.json` extensions
- Profile directory: `~/.config/git-setup/profiles/`
- Automatic backups before modifications

**Step-by-Step Implementation**:

1. **Set up the storage module** (1 hour)
   ```rust
   use crate::fs::{AtomicWrite, SecurePermissions};
   use crate::config::AppConfig;
   
   pub struct ProfileStorage {
       profiles_dir: PathBuf,
       atomic_writer: AtomicFileWriter,
   }
   ```

2. **Implement profile path resolution** (2 hours)
   ```rust
   impl ProfileStorage {
       fn profile_path(&self, name: &str) -> Result<PathBuf> {
           // Validate profile name (no path traversal!)
           if name.contains('/') || name.contains('\\') {
               return Err(ProfileError::InvalidName);
           }
           
           Ok(self.profiles_dir.join(format!("{}.toml", name)))
       }
   }
   ```
   
   ⚠️ **Security Alert**: Always validate user input for path traversal

3. **Implement atomic save with backup** (4 hours)
   ```rust
   pub async fn save(&self, profile: &Profile) -> Result<()> {
       let path = self.profile_path(&profile.name)?;
       
       // Create backup if file exists
       if path.exists() {
           let backup_path = path.with_extension("toml.bak");
           self.atomic_writer.write_atomic(&backup_path, &fs::read(&path)?)?;
       }
       
       // Serialize and save atomically
       let content = toml::to_string_pretty(profile)?;
       self.atomic_writer.write_atomic(&path, content.as_bytes())?;
       
       // Set secure permissions
       self.set_permissions(&path, 0o600)?;
       
       Ok(())
   }
   ```

**TDD Focus**:
- Test concurrent profile access
- Test profile corruption recovery
- Test permission preservation
- Test format auto-detection

#### Task 2.1.3: Profile CRUD Operations (8 hours)

Implement the ProfileManager trait:

```rust
pub trait ProfileManager {
    fn create(&mut self, profile: Profile) -> Result<()>;
    fn read(&self, name: &str) -> Result<Profile>;
    fn update(&mut self, name: &str, profile: Profile) -> Result<()>;
    fn delete(&mut self, name: &str) -> Result<()>;
    fn list(&self) -> Result<Vec<ProfileSummary>>;
    fn exists(&self, name: &str) -> bool;
}
```

**Performance Requirements**:
- List 100 profiles in <20ms
- Load profile in <5ms
- Save with backup in <10ms

#### Task 2.1.4: Import/Export Functions (4 hours)

Support profile portability:
- Export single profile to file
- Export all profiles to archive
- Import with conflict resolution
- Import from 1Password agent.toml

---

## 🛑 CHECKPOINT 1: Profile Storage Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** past this checkpoint without completing the review process and obtaining all required approvals.

### Pre-Checkpoint Checklist

Before requesting review, ensure ALL items are complete:

- [ ] All code committed and pushed to feature branch
- [ ] All tests passing locally (`cargo test profile::`)
- [ ] Code coverage ≥ 90% for profile module: `cargo tarpaulin --packages git-setup --line --out Html`
- [ ] No compiler warnings: `cargo build --all-targets`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation complete for all public APIs
- [ ] Benchmarks show <20ms for profile operations
- [ ] Security: Path traversal tests passing

### Review Process

1. **Create Pull Request**
   ```bash
   git checkout -b checkpoint/phase-2-1-profile-storage
   git add .
   git commit -m "CHECKPOINT: Phase 2.1 - Profile Storage Complete"
   git push origin checkpoint/phase-2-1-profile-storage
   ```
   
   Use PR template: `.github/pull_request_template/checkpoint_review.md`

2. **Required Reviewers**
   - @tech-lead - Data model and API design
   - @senior-dev - Storage implementation
   - @qa-engineer - Test coverage and quality

### Review Criteria

#### Data Model (Tech Lead)
- [ ] Profile schema supports all requirements
- [ ] Inheritance model is sound
- [ ] Serialization format appropriate
- [ ] Extensibility considered

#### Storage Implementation (Senior Dev)
- [ ] Atomic operations used correctly
- [ ] Backup mechanism reliable
- [ ] Format detection works
- [ ] Performance acceptable

#### Test Quality (QA Engineer)
- [ ] CRUD operations fully tested
- [ ] Edge cases covered
- [ ] Concurrent access tested
- [ ] Import/export validated

### Consequences of Skipping

**If you proceed without approval**:
- Git integration will build on flawed foundation
- Data corruption issues in production
- Breaking changes require migration code
- 3-5 day rework minimum

---

### 2.2 Git Configuration Engine (35 hours)

**Complexity**: High - Git integration critical
**Files**: `src/git/mod.rs`, `src/git/config.rs`, `src/git/commands.rs`

#### Task 2.2.1: Git Config Abstraction (8 hours)

💡 **Junior Dev Concept**: Git Configuration Scopes
**What they are**: Git has 3 config levels - system (/etc), global (~/.gitconfig), local (.git/config)
**Precedence**: Local overrides global, global overrides system
**Why abstract**: The git2 crate API is low-level; we need a simpler interface

Create abstraction over git2 for config management:

```rust
pub trait GitConfig {
    fn get(&self, key: &str, scope: ConfigScope) -> Result<Option<String>>;
    fn set(&mut self, key: &str, value: &str, scope: ConfigScope) -> Result<()>;
    fn unset(&mut self, key: &str, scope: ConfigScope) -> Result<()>;
    fn list(&self, scope: ConfigScope) -> Result<HashMap<String, String>>;
}

pub enum ConfigScope {
    Local,   // .git/config
    Global,  // ~/.gitconfig
    System,  // /etc/gitconfig
}
```

#### Task 2.2.2: Profile to Git Mapping (10 hours)

💡 **Junior Dev Concept**: Configuration Mapping
**What it is**: Translating our profile format to Git's configuration format
**Why complex**: Git config has quirks - some keys are special, some allow multiple values
**Key insight**: We're building a "compiler" from Profile → Git Config

Implement profile application logic:

**Mapping Rules**:
- `user.name` → `user.name`
- `user.email` → `user.email`
- `signing.key` → `user.signingkey`
- `signing.format` → `gpg.format`
- Custom fields → direct mapping

**Special Handling**:
- Detect existing values before overwrite
- Support includeIf conditionals
- Handle multi-valued keys correctly

**Step-by-Step Implementation**:

1. **Define the mapping structure** (2 hours)
   ```rust
   pub struct ProfileMapper {
       // Track what we've changed for rollback
       changes: Vec<ConfigChange>,
   }
   
   struct ConfigChange {
       key: String,
       old_value: Option<String>,
       new_value: String,
       scope: ConfigScope,
   }
   ```

2. **Implement basic field mapping** (3 hours)
   ```rust
   impl ProfileMapper {
       pub fn map_profile(&mut self, profile: &Profile) -> Result<()> {
           // Map user fields
           self.map_field("user.name", &profile.user.name, ConfigScope::Global)?;
           self.map_field("user.email", &profile.user.email, ConfigScope::Global)?;
           
           // Map signing if present
           if let Some(signing) = &profile.signing {
               self.map_signing_config(signing)?;
           }
           
           Ok(())
       }
   }
   ```
   
   📚 **Git Config Keys**: [Complete list](https://git-scm.com/docs/git-config#_variables)

3. **Handle special Git behaviors** (3 hours)
   ```rust
   fn map_signing_config(&mut self, signing: &SigningConfig) -> Result<()> {
       // Git requires gpg.format before user.signingkey
       self.map_field("gpg.format", &signing.format, ConfigScope::Global)?;
       
       // Special handling for SSH signing
       if signing.format == "ssh" {
           self.map_field("gpg.ssh.allowedSignersFile", 
                         "~/.config/git/allowed_signers", 
                         ConfigScope::Global)?;
       }
       
       self.map_field("user.signingkey", &signing.key, ConfigScope::Global)?;
       Ok(())
   }
   ```
   
   ⚠️ **Common Mistake**: Setting signingkey before format
   ✅ **Correct Order**: Always set gpg.format first

4. **Add rollback capability** (2 hours)
   ```rust
   pub fn rollback(&self, git_config: &mut GitConfig) -> Result<()> {
       // Apply changes in reverse order
       for change in self.changes.iter().rev() {
           match &change.old_value {
               Some(value) => git_config.set(&change.key, value, change.scope)?,
               None => git_config.unset(&change.key, change.scope)?,
           }
       }
       Ok(())
   }
   ```

#### Task 2.2.3: Atomic Profile Switching (12 hours)

💡 **Junior Dev Concept**: Transactional Operations
**What it is**: All changes succeed together or all fail together (like database transactions)
**Why critical**: Half-applied profiles = broken Git setup = can't commit code
**How we do it**: Snapshot → Validate → Apply → Verify (with rollback on failure)

Implement transactional profile switching:

1. Snapshot current configuration
2. Validate target profile
3. Apply changes atomically
4. Rollback on any failure
5. Verify final state

**Performance**: Complete switch in <3 seconds

**Step-by-Step Implementation**:

1. **Design the switching flow** (2 hours)
   ```rust
   pub struct ProfileSwitcher {
       profile_manager: Arc<dyn ProfileManager>,
       git_config: GitConfig,
       mapper: ProfileMapper,
   }
   
   pub struct SwitchResult {
       pub previous_profile: Option<String>,
       pub new_profile: String,
       pub changes_applied: Vec<ConfigChange>,
       pub duration: Duration,
   }
   ```

2. **Implement configuration snapshot** (3 hours)
   ```rust
   async fn snapshot_current(&self) -> Result<ConfigSnapshot> {
       let start = Instant::now();
       
       // Capture all relevant Git config
       let snapshot = ConfigSnapshot {
           user_name: self.git_config.get("user.name", ConfigScope::Global)?,
           user_email: self.git_config.get("user.email", ConfigScope::Global)?,
           signing_key: self.git_config.get("user.signingkey", ConfigScope::Global)?,
           gpg_format: self.git_config.get("gpg.format", ConfigScope::Global)?,
           custom: self.capture_custom_fields()?,
           timestamp: Utc::now(),
       };
       
       debug!("Snapshot completed in {:?}", start.elapsed());
       Ok(snapshot)
   }
   ```
   
   💡 **Performance Tip**: Only snapshot fields we might change

3. **Implement the atomic switch** (5 hours)
   ```rust
   pub async fn switch(&mut self, profile_name: &str) -> Result<SwitchResult> {
       let start = Instant::now();
       
       // 1. Load and validate profile
       let profile = self.profile_manager.read(profile_name)
           .map_err(|e| SwitchError::ProfileNotFound(profile_name.to_string()))?;
       profile.validate()?;
       
       // 2. Snapshot current state
       let snapshot = self.snapshot_current().await?;
       
       // 3. Begin transaction
       let mut transaction = self.begin_transaction();
       
       // 4. Apply profile
       match self.mapper.map_profile(&profile) {
           Ok(changes) => {
               // 5. Verify changes
               if self.verify_changes(&changes).is_ok() {
                   transaction.commit()?;
                   Ok(SwitchResult {
                       previous_profile: snapshot.current_profile,
                       new_profile: profile_name.to_string(),
                       changes_applied: changes,
                       duration: start.elapsed(),
                   })
               } else {
                   transaction.rollback()?;
                   Err(SwitchError::VerificationFailed)
               }
           }
           Err(e) => {
               transaction.rollback()?;
               Err(e.into())
           }
       }
   }
   ```
   
   ⚠️ **Common Mistake**: Not verifying after apply
   ✅ **Always**: Read back config to ensure it was applied

4. **Add progress feedback** (2 hours)
   ```rust
   pub async fn switch_with_progress<F>(&mut self, 
                                       profile_name: &str, 
                                       mut progress: F) -> Result<SwitchResult>
   where
       F: FnMut(SwitchProgress),
   {
       progress(SwitchProgress::LoadingProfile);
       let profile = self.profile_manager.read(profile_name)?;
       
       progress(SwitchProgress::Validating);
       profile.validate()?;
       
       progress(SwitchProgress::ApplyingChanges);
       // ... rest of switch logic
   }
   ```

#### Task 2.2.4: Git Integration Tests (5 hours)

Comprehensive testing with real Git:
- Test all config scopes
- Verify `git config --list` output
- Test with actual commits
- Verify signing configuration

---

## 🛑 CHECKPOINT 2: Git Integration Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** past this checkpoint without completing the review process.

### Pre-Checkpoint Checklist

- [ ] All Git operations tested with real Git
- [ ] Profile switching is truly atomic
- [ ] Rollback mechanism verified
- [ ] Performance: Profile switch <3 seconds
- [ ] Integration tests with git2 passing
- [ ] Manual test: Can commit with switched profile

### Test Verification Commands

```bash
# Test profile switching
cargo run -- switch work
git config user.email  # Should show work email

# Test rollback
# Intentionally corrupt a profile and verify rollback works
```

### Review Requirements

**Demonstration Required**:
1. Live demo of profile switching
2. Show atomic rollback on failure
3. Verify with actual Git commands
4. Performance metrics

### Consequences of Skipping

- Users lose Git configuration
- Non-atomic switches corrupt setup
- Integration issues with Git tools
- Complete Phase 2 redo likely

---

### 2.3 Profile Validation Framework (25 hours)

**Complexity**: Medium - Critical for safety
**Files**: `src/validation/mod.rs`, `src/validation/rules.rs`, `src/validation/profile.rs`

#### Task 2.3.1: Validation Rule Engine (8 hours)

💡 **Junior Dev Concept**: Validation Architecture
**What it is**: A system to check if profiles are valid before applying them
**Why essential**: Invalid Git config = broken development environment
**Pattern**: Chain of Responsibility - each rule checks one thing

Build extensible validation system:

```rust
pub trait ValidationRule {
    fn validate(&self, profile: &Profile) -> ValidationResult;
    fn name(&self) -> &str;
    fn severity(&self) -> Severity;
}

pub struct ValidationEngine {
    rules: Vec<Box<dyn ValidationRule>>,
}
```

**Built-in Rules**:
- Email format validation
- Name not empty
- No credentials in values
- Valid Git config keys
- Circular inheritance detection

**Step-by-Step Implementation**:

1. **Design the rule system** (2 hours)
   ```rust
   use async_trait::async_trait;
   
   #[async_trait]
   pub trait ValidationRule: Send + Sync {
       /// Unique name for this rule
       fn name(&self) -> &str;
       
       /// How serious is a violation?
       fn severity(&self) -> Severity {
           Severity::Error  // Default: fail validation
       }
       
       /// Run the validation check
       async fn validate(&self, profile: &Profile) -> ValidationResult;
   }
   
   pub enum Severity {
       Error,    // Must fix
       Warning,  // Should fix
       Info,     // Nice to fix
   }
   ```
   
   💡 **Why async?** Some rules might need to check external resources

2. **Implement core validation rules** (3 hours)
   ```rust
   pub struct EmailFormatRule;
   
   #[async_trait]
   impl ValidationRule for EmailFormatRule {
       fn name(&self) -> &str { "email-format" }
       
       async fn validate(&self, profile: &Profile) -> ValidationResult {
           // RFC 5322 simplified regex
           let email_regex = Regex::new(
               r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
           ).unwrap();
           
           if email_regex.is_match(&profile.user.email) {
               ValidationResult::success()
           } else {
               ValidationResult::error(
                   format!("Invalid email format: '{}'", profile.user.email),
                   Some("Email should be like: user@example.com".to_string()),
               )
           }
       }
   }
   ```
   
   📚 **Email Validation**: [RFC 5322](https://tools.ietf.org/html/rfc5322)

3. **Create security-focused rules** (2 hours)
   ```rust
   pub struct NoCredentialsRule;
   
   impl ValidationRule for NoCredentialsRule {
       fn name(&self) -> &str { "no-credentials" }
       
       async fn validate(&self, profile: &Profile) -> ValidationResult {
           // Check for common credential patterns
           let suspicious_patterns = [
               r"password\s*=\s*",
               r"token\s*=\s*",
               r"secret\s*=\s*",
               r"BEGIN (RSA|DSA|EC) PRIVATE KEY",
           ];
           
           let profile_text = toml::to_string(profile)?;
           
           for pattern in &suspicious_patterns {
               if Regex::new(pattern)?.is_match(&profile_text) {
                   return ValidationResult::error(
                       "Profile appears to contain credentials",
                       Some("Use op:// references or SSH agent instead"),
                   );
               }
           }
           
           ValidationResult::success()
       }
   }
   ```
   
   ⚠️ **Security**: Never store actual credentials in profiles

4. **Build the validation engine** (1 hour)
   ```rust
   pub struct ValidationEngine {
       rules: Vec<Box<dyn ValidationRule>>,
   }
   
   impl ValidationEngine {
       pub async fn validate_profile(&self, profile: &Profile) -> Result<ValidationReport> {
           let mut results = Vec::new();
           
           // Run all rules in parallel for performance
           let futures: Vec<_> = self.rules.iter()
               .map(|rule| rule.validate(profile))
               .collect();
           
           let rule_results = futures::future::join_all(futures).await;
           
           // Collect results with rule names
           for (rule, result) in self.rules.iter().zip(rule_results) {
               results.push((rule.name().to_string(), result));
           }
           
           Ok(ValidationReport::new(results))
       }
   }
   ```

#### Task 2.3.2: Profile-Specific Validators (8 hours)

Implement profile validation:
- Required fields present
- Signing config consistency
- Parent profile exists
- No conflicting settings
- Security rules (no passwords)

#### Task 2.3.3: Git Config Validation (5 hours)

Validate Git-specific requirements:
- Key name format
- Value type correctness
- Scope appropriateness
- No breaking changes

#### Task 2.3.4: Validation Testing (4 hours)

Test all validation scenarios:
- Invalid email formats
- Missing required fields
- Circular inheritance
- Security violations
- Performance with 100 rules

---

### CHECKPOINT 3: Validation Framework Complete

**Review Focus**: Security and correctness

**Deliverables**:
- All validators implemented
- Security rules enforced
- Performance acceptable
- Clear error messages

---

### 2.4 Profile Templates & Inheritance (30 hours)

**Complexity**: Medium - User experience critical
**Files**: `src/template/mod.rs`, `src/profile/inheritance.rs`

💡 **Junior Dev Concept**: Template Systems & Inheritance
**What it is**: Reusable profile patterns and parent-child relationships
**Real-world analogy**: Like CSS classes - define once, use many times
**Why powerful**: Users can share common settings without repetition

#### Task 2.4.1: Template System Design (6 hours)

Design template system:
- Built-in templates (work, personal, oss)
- User-defined templates
- Template discovery
- Template validation

#### Task 2.4.2: Inheritance Engine (12 hours)

💡 **Junior Dev Concept**: Profile Inheritance
**What it is**: Profiles can extend other profiles, inheriting their settings
**Example**: "work-projectA" extends "work" extends "base"
**Precedence**: Child values override parent values (like CSS)

Implement profile inheritance:

```rust
pub fn resolve_profile(name: &str, profiles: &ProfileStore) -> Result<ResolvedProfile> {
    // 1. Load profile
    // 2. Resolve parent chain
    // 3. Merge in precedence order
    // 4. Validate result
}
```

**Features**:
- Multiple inheritance levels
- Field-level override
- Circular dependency detection
- Clear precedence rules

**Step-by-Step Implementation**:

1. **Design the inheritance resolver** (2 hours)
   ```rust
   pub struct InheritanceResolver {
       profile_store: Arc<dyn ProfileStore>,
       max_depth: usize,  // Prevent infinite recursion
   }
   
   pub struct ResolvedProfile {
       /// The final merged profile
       pub profile: Profile,
       /// Chain of profiles that contributed (for debugging)
       pub inheritance_chain: Vec<String>,
       /// Which profile each field came from
       pub field_sources: HashMap<String, String>,
   }
   ```

2. **Implement parent chain resolution** (4 hours)
   ```rust
   impl InheritanceResolver {
       pub async fn resolve(&self, name: &str) -> Result<ResolvedProfile> {
           let mut chain = Vec::new();
           let mut visited = HashSet::new();
           
           // Build inheritance chain
           self.build_chain(name, &mut chain, &mut visited)?;
           
           // Start with empty profile
           let mut merged = Profile::default();
           let mut field_sources = HashMap::new();
           
           // Apply profiles in order (parent to child)
           for profile_name in chain.iter().rev() {
               let profile = self.profile_store.read(profile_name)?;
               self.merge_profile(&mut merged, &profile, profile_name, &mut field_sources)?;
           }
           
           Ok(ResolvedProfile {
               profile: merged,
               inheritance_chain: chain,
               field_sources,
           })
       }
       
       fn build_chain(&self, 
                      name: &str, 
                      chain: &mut Vec<String>, 
                      visited: &mut HashSet<String>) -> Result<()> {
           // Circular dependency check
           if visited.contains(name) {
               return Err(ValidationError::CircularInheritance {
                   profile: name.to_string(),
                   chain: chain.clone(),
               });
           }
           
           visited.insert(name.to_string());
           chain.push(name.to_string());
           
           // Recursively add parents
           let profile = self.profile_store.read(name)?;
           if let Some(parent) = &profile.extends {
               self.build_chain(parent, chain, visited)?;
           }
           
           Ok(())
       }
   }
   ```
   
   ⚠️ **Common Bug**: Not checking for circular dependencies
   ✅ **Solution**: Track visited profiles during resolution

3. **Implement smart merging** (4 hours)
   ```rust
   fn merge_profile(&self,
                    base: &mut Profile,
                    overlay: &Profile,
                    source: &str,
                    field_sources: &mut HashMap<String, String>) -> Result<()> {
       // Merge user fields
       if !overlay.user.name.is_empty() {
           base.user.name = overlay.user.name.clone();
           field_sources.insert("user.name".to_string(), source.to_string());
       }
       
       if !overlay.user.email.is_empty() {
           base.user.email = overlay.user.email.clone();
           field_sources.insert("user.email".to_string(), source.to_string());
       }
       
       // Merge optional fields
       if overlay.signing.is_some() {
           base.signing = overlay.signing.clone();
           field_sources.insert("signing".to_string(), source.to_string());
       }
       
       // Merge custom fields (deep merge)
       for (key, value) in &overlay.custom {
           base.custom.insert(key.clone(), value.clone());
           field_sources.insert(format!("custom.{}", key), source.to_string());
       }
       
       Ok(())
   }
   ```
   
   💡 **Design Choice**: We use "last wins" merging - child overrides parent

4. **Add debugging support** (2 hours)
   ```rust
   impl ResolvedProfile {
       /// Show inheritance debug info
       pub fn debug_inheritance(&self) -> String {
           let mut output = String::new();
           
           writeln!(output, "Inheritance chain: {}", self.inheritance_chain.join(" → "));
           writeln!(output, "\nField sources:");
           
           for (field, source) in &self.field_sources {
               writeln!(output, "  {} ← {}", field, source);
           }
           
           output
       }
   }
   ```

#### Task 2.4.3: Built-in Templates (8 hours)

Create standard templates:

**Work Template**:
```toml
[user]
name = "{{ full_name }}"
email = "{{ email }}"

[signing]
format = "ssh"
key = "{{ ssh_key }}"
```

**Personal Template**:
```toml
[user]
name = "{{ name }}"
email = "{{ personal_email }}"

[commit]
gpgsign = false
```

#### Task 2.4.4: Template Testing (4 hours)

Test template functionality:
- Variable substitution
- Inheritance chains
- Override behavior
- Template validation

---

## 🛑 CHECKPOINT 4: Phase 2 Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** to Phase 3 without completing this final review.

### Pre-Checkpoint Checklist

- [ ] All 4 component checkpoints passed
- [ ] Integration tests passing between all components
- [ ] Performance benchmarks met:
  - [ ] Profile load <5ms
  - [ ] Profile switch <3s
  - [ ] Validation <10ms
- [ ] Documentation complete:
  - [ ] API documentation
  - [ ] Usage examples
  - [ ] Architecture diagrams
- [ ] Security review passed:
  - [ ] No credential storage
  - [ ] Path traversal prevented
  - [ ] Atomic operations verified

### Integration Test Suite

```bash
# Run full integration test
cargo test --test integration_phase2

# Manual verification
./scripts/test-profile-scenarios.sh
```

### Final Demo Requirements

1. **Profile Management Demo** (10 minutes)
   - Create profile with wizard
   - Edit existing profile
   - Show inheritance working
   - Import from 1Password agent.toml

2. **Git Integration Demo** (10 minutes)
   - Switch profiles
   - Show Git config changes
   - Demonstrate rollback
   - Make actual commit

3. **Validation Demo** (5 minutes)
   - Show invalid profile rejection
   - Demonstrate helpful error messages
   - Show security rule enforcement

### Phase 2 Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Test Coverage | ≥85% | ___ | ⬜ |
| Profile Operations | <20ms | ___ | ⬜ |
| Git Switch Time | <3s | ___ | ⬜ |
| Memory Usage | <30MB | ___ | ⬜ |
| Code Review | 100% | ___ | ⬜ |

### Sign-offs Required

- [ ] **Tech Lead**: Architecture approved
- [ ] **Security**: No vulnerabilities found
- [ ] **QA Lead**: Test coverage sufficient
- [ ] **Product Owner**: Features complete
- [ ] **Team Lead**: Ready for Phase 3

### Handoff to Phase 3

**What Phase 3 needs from us**:
1. Stable ProfileManager trait API
2. Comprehensive test fixtures
3. Performance baselines
4. Known limitations documented

**Breaking changes after this point** require:
- Architecture review
- Migration plan
- Update to all dependent phases

---

## Common Issues & Solutions

### Issue: "Profile not found" but file exists
**Symptom**: Profile file is in directory but manager can't find it
**Likely Cause**: Wrong file extension or permissions
**Solution**:
```rust
// Check the profile directory
ls -la ~/.config/git-setup/profiles/
// Ensure .toml extension and 600 permissions
```

### Issue: Git Config Formatting
**Symptom**: Git complains about config after switch
**Cause**: Quotes in values not escaped properly
**Solution**: Always use git2 for proper escaping:
```rust
// ❌ Wrong
format!("git config user.name '{}'", name)

// ✅ Right
git_config.set("user.name", &name, ConfigScope::Global)?;
```

### Issue: Profile Inheritance Loops
**Symptom**: Stack overflow or hang during resolution
**Cause**: A extends B extends A
**Solution**: Track visited profiles:
```rust
let mut visited = HashSet::new();
if !visited.insert(profile_name) {
    return Err(CircularDependency);
}
```

### Issue: Concurrent Profile Access
**Symptom**: Corrupted profiles when multiple git-setup instances run
**Cause**: Two processes modifying same file
**Solution**: Use file locking:
```rust
use fs2::FileExt;
let file = OpenOptions::new()
    .write(true)
    .open(&path)?;
file.lock_exclusive()?;  // Block until exclusive access
// ... do work ...
file.unlock()?;
```

### Issue: Slow Profile Switching
**Symptom**: Takes >3 seconds to switch
**Cause**: Too many Git config operations
**Solution**: Batch operations:
```rust
// ❌ Slow
for (key, value) in changes {
    git_config.set(key, value)?;
}

// ✅ Fast
git_config.batch_set(changes)?;
```

## Security Considerations

- Never store actual credentials
- Validate all user input
- Sanitize for command injection
- Log security violations
- Zeroize sensitive data

## Performance Targets

| Operation | Target | Maximum | Measurement Method |
|-----------|--------|---------|-------------------|
| Load Profile | <5ms | <20ms | `cargo bench profile_load` |
| Save Profile | <10ms | <50ms | `cargo bench profile_save` |
| List 100 Profiles | <20ms | <100ms | `cargo bench profile_list_100` |
| Switch Profile | <3s | <5s | End-to-end with real Git |
| Validate Profile | <10ms | <50ms | `cargo bench profile_validate` |

### Performance Testing

```bash
# Run all benchmarks
cargo bench --bench profile_benchmarks

# Generate flame graph for slow operations
cargo flamegraph --bench profile_benchmarks -- --bench switch_profile

# Profile memory usage
heaptrack target/release/git-setup switch work
```

### Optimization Checklist

- [ ] Profile operations use lazy loading
- [ ] Git config reads are batched
- [ ] Validation rules run in parallel
- [ ] File I/O is minimized
- [ ] Caching implemented where appropriate

## Junior Developer Tips

### Getting Started with Phase 2

1. **Before coding**:
   - Review the Rust glossary for unfamiliar terms
   - Run existing tests to understand expected behavior
   - Try the profile examples in `examples/profiles/`

2. **While coding**:
   - Start with tests (TDD!)
   - Use `cargo check` frequently
   - Ask questions early - don't struggle alone
   - Pair program on complex parts

3. **Common pitfalls**:
   - Forgetting to handle Option<T> cases
   - Not checking for path traversal attacks
   - Assuming Git config keys are simple
   - Skipping validation "for now"

### Resources for Success

- **Pair Programming**: Schedule with senior dev for:
  - Git configuration engine design
  - Inheritance resolver implementation
  - Performance optimization

- **Learning Path**:
  1. Complete Rust Book Chapter 17 (Trait Objects)
  2. Study git2 crate examples
  3. Understand Git config format
  4. Review Phase 1 implementation

## Next Phase Preview

Phase 3 (User Interfaces) will add:
- CLI commands for all operations
- Interactive TUI with profile wizard
- Fuzzy search for profiles
- Visual profile management

**What Phase 3 will need from Phase 2**:
- Stable async ProfileManager API
- Fast profile operations (<20ms)
- Clear error messages
- Mock implementations for testing

---

*Last updated: 2025-07-30*