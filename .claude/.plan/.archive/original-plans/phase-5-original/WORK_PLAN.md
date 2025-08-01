# Phase 5: Advanced Features - Work Plan

## Prerequisites

Phase 5 builds advanced capabilities on top of the foundation from previous phases.

**Required from Previous Phases**:
- Profile management system (Phase 2)
- CLI and TUI interfaces (Phase 3)
- 1Password integration (Phase 4)
- Git configuration engine (Phase 2)
- Security primitives (Phase 1)

**Required Knowledge**:
- **Pattern Matching**: Glob patterns, path traversal (*critical*)
- **Git Internals**: Repository detection, configuration scopes (*critical*)
- **Signing Methods**: SSH, GPG, x509, gitsign protocols (*critical*)
- **System Diagnostics**: Health check patterns (*required*)
- **Network Programming**: HTTP/HTTPS for remote imports (*required*)

**Required Tools**:
- Git 2.25+ with signing support
- SSH agent running
- GPG configured (optional)
- Network access for remote features

💡 **Junior Dev Resources**:
- 📚 [Glob Pattern Guide](https://man7.org/linux/man-pages/man7/glob.7.html) - Pattern matching basics
- 🎥 [Git Internals](https://www.youtube.com/watch?v=P6jD966jzlk) - 30 min deep dive
- 📖 [Git Signing Guide](https://docs.github.com/en/authentication/managing-commit-signature-verification) - All signing methods
- 🔧 [Pattern Playground](https://globster.xyz/) - Test glob patterns
- 💻 [HTTP Client in Rust](https://rust-lang-nursery.github.io/rust-cookbook/web/clients.html) - Network basics

## Quick Reference - Essential Resources

### Pattern Matching & Detection
- [Glob Documentation](https://docs.rs/glob)
- [Git Directory Structure](https://git-scm.com/book/en/v2/Git-Internals)
- [Rust Path Handling](https://doc.rust-lang.org/std/path/)

### Signing Documentation
- [Git Signing Overview](https://git-scm.com/book/en/v2/Git-Tools-Signing-Your-Work)
- [SSH Signing](https://github.blog/changelog/2021-11-15-ssh-commit-verification-now-supported/)
- [Gitsign Documentation](https://github.com/sigstore/gitsign)
- [x509 Signing](https://git-scm.com/docs/git-config#Documentation/git-config.txt-gpgx509program)

### Project Resources
- **[SPEC.md](../../spec/SPEC.md)** - See FR-004, FR-006, FR-008
- **[Profile Manager](../phase-2/)** - Core profile operations
- **[1Password Integration](../phase-4/)** - Key discovery APIs

### Commands
- `git config --show-scope --list` - View all Git config
- `ssh-add -l` - List SSH keys in agent
- `gpg --list-secret-keys` - List GPG keys
- `gitsign verify` - Test gitsign setup

## Overview

Phase 5 adds intelligent automation and advanced signing capabilities to git-setup-rs. These features reduce manual configuration and support enterprise signing requirements.

**Key Deliverables**:
- Automatic profile detection based on repository context
- Multi-method signing configuration (SSH, GPG, x509, gitsign)
- Comprehensive health check system
- Remote profile import with verification
- Pattern-based profile matching

**Checkpoint Strategy**: 4 checkpoints for feature milestones

**Time Estimate**: 2 weeks (80 hours)

## Development Methodology: Test-Driven Development (TDD)

Special considerations for advanced features:
1. **Pattern Testing** - Edge cases for path matching
2. **Signing Tests** - Mock signing commands
3. **Health Checks** - Simulate various failure modes
4. **Network Tests** - Mock HTTP requests

## Done Criteria Checklist

Phase 5 is complete when:
- [ ] Profile detected in <100ms on directory change
- [ ] All 4 signing methods configurable
- [ ] Health checks complete in <1s total
- [ ] Remote profiles imported securely
- [ ] Pattern matching supports wildcards
- [ ] No signing method conflicts
- [ ] Comprehensive error messages
- [ ] All 4 checkpoints reviewed

## Work Breakdown with Review Checkpoints

### 5.1 Automatic Profile Detection (20 hours)

**Complexity**: High - Pattern matching and performance critical
**Files**: `src/detection/mod.rs`, `src/detection/patterns.rs`, `src/detection/cache.rs`

#### Task 5.1.1: Detection Engine Architecture (5 hours)

💡 **Junior Dev Concept**: Automatic Detection
**What it is**: Detect which Git profile to use based on repository location/remote
**Example**: All repos in ~/work/ use work profile, github.com/company/* uses company profile
**Key challenge**: Must be FAST - runs on every directory change

Build the detection framework:

```rust
pub struct ProfileDetector {
    patterns: Vec<DetectionPattern>,
    cache: DetectionCache,
    profile_manager: Arc<dyn ProfileManager>,
}

#[derive(Debug, Clone)]
pub struct DetectionPattern {
    pub name: String,
    pub priority: u8,
    pub matchers: Vec<Matcher>,
}

#[derive(Debug, Clone)]
pub enum Matcher {
    /// Match repository remote URL
    Remote { pattern: String },
    /// Match file path
    Path { glob: String },
    /// Match directory name
    Directory { pattern: String },
    /// Match Git config value
    GitConfig { key: String, value: String },
    /// Custom matcher function
    Custom(Arc<dyn Fn(&Path) -> bool>),
}

impl ProfileDetector {
    pub async fn detect(&self, path: &Path) -> Result<Option<String>> {
        // 1. Check cache first
        if let Some(profile) = self.cache.get(path) {
            return Ok(Some(profile));
        }
        
        // 2. Find Git repository root
        let repo_path = find_git_root(path)?;
        
        // 3. Evaluate patterns in priority order
        for pattern in &self.patterns {
            if self.matches_pattern(&pattern, &repo_path).await? {
                self.cache.set(path, &pattern.name);
                return Ok(Some(pattern.name.clone()));
            }
        }
        
        Ok(None)
    }
}
```

**Requirements**:
- Sub-100ms detection time
- Cache invalidation on config change
- Support nested Git repositories
- Handle submodules correctly

**Step-by-Step Implementation**:

1. **Design the pattern system** (1 hour)
   ```rust
   // Pattern priority: Higher number = higher precedence
   // This ensures specific patterns override general ones
   
   #[derive(Debug, Clone)]
   pub struct DetectionPattern {
       pub name: String,        // Profile name to apply
       pub priority: u8,        // 0-255, higher wins
       pub matchers: Vec<Matcher>,  // ALL must match
   }
   
   // Example patterns:
   // Pattern 1: "All work repos"
   // - priority: 10
   // - matcher: Path { glob: "~/work/**" }
   //
   // Pattern 2: "Specific work project"
   // - priority: 20 (overrides pattern 1)
   // - matcher: Remote { pattern: "github.com:company/important-*" }
   ```
   
   💡 **Priority System**: Specific patterns beat general ones

2. **Find Git repository root** (1.5 hours)
   ```rust
   use std::path::{Path, PathBuf};
   
   pub fn find_git_root(start_path: &Path) -> Result<PathBuf> {
       let mut current = start_path;
       
       loop {
           // Check if .git exists
           let git_dir = current.join(".git");
           
           if git_dir.exists() {
               // Could be a file (submodule) or directory
               if git_dir.is_dir() {
                   return Ok(current.to_path_buf());
               } else if git_dir.is_file() {
                   // Submodule: .git file contains path to real git dir
                   return handle_submodule(&git_dir);
               }
           }
           
           // Move up one directory
           match current.parent() {
               Some(parent) => current = parent,
               None => return Err(DetectionError::NotInGitRepo),
           }
       }
   }
   
   fn handle_submodule(git_file: &Path) -> Result<PathBuf> {
       // Read .git file which contains: gitdir: /path/to/.git/modules/name
       let content = fs::read_to_string(git_file)?;
       
       if let Some(gitdir) = content.strip_prefix("gitdir: ") {
           let gitdir = gitdir.trim();
           // Return the working directory, not the git dir
           Ok(git_file.parent().unwrap().to_path_buf())
       } else {
           Err(DetectionError::InvalidGitFile)
       }
   }
   ```
   
   ⚠️ **Common Mistake**: Forgetting about submodules
   ✅ **Solution**: Check if .git is a file (submodule) or directory

3. **Extract repository information** (1.5 hours)
   ```rust
   pub struct DetectionContext {
       pub path: PathBuf,
       pub repo_root: PathBuf,
       pub remotes: Vec<String>,
       pub branch: Option<String>,
   }
   
   impl DetectionContext {
       pub async fn from_path(path: &Path) -> Result<Self> {
           let repo_root = find_git_root(path)?;
           
           // Use git2 to read repository info
           let repo = git2::Repository::open(&repo_root)?;
           
           // Get all remotes
           let remotes = repo.remotes()?
               .iter()
               .filter_map(|name| {
                   name.and_then(|n| {
                       repo.find_remote(n).ok()
                           .and_then(|r| r.url().map(|u| u.to_string()))
                   })
               })
               .collect();
           
           // Get current branch
           let branch = repo.head().ok()
               .and_then(|h| h.shorthand().map(|s| s.to_string()));
           
           Ok(Self {
               path: path.to_path_buf(),
               repo_root,
               remotes,
               branch,
           })
       }
       
       pub fn git_config(&self, key: &str) -> Result<Option<String>> {
           let repo = git2::Repository::open(&self.repo_root)?;
           let config = repo.config()?;
           
           match config.get_string(key) {
               Ok(value) => Ok(Some(value)),
               Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
               Err(e) => Err(e.into()),
           }
       }
   }
   ```

4. **Build the complete detector** (1 hour)
   ```rust
   impl ProfileDetector {
       pub async fn detect(&self, path: &Path) -> Result<Option<String>> {
           let start = Instant::now();
           
           // Check cache first (critical for performance!)
           if let Some(profile) = self.cache.get(path) {
               return Ok(Some(profile));
           }
           
           // Build detection context
           let context = DetectionContext::from_path(path).await?;
           
           // Sort patterns by priority (highest first)
           let mut patterns = self.patterns.clone();
           patterns.sort_by_key(|p| std::cmp::Reverse(p.priority));
           
           // Find first matching pattern
           for pattern in patterns {
               let mut all_match = true;
               
               for matcher in &pattern.matchers {
                   if !matcher.evaluate(&context).await? {
                       all_match = false;
                       break;
                   }
               }
               
               if all_match {
                   // Cache the result
                   self.cache.set(&context.repo_root, &pattern.name);
                   
                   let elapsed = start.elapsed();
                   if elapsed > Duration::from_millis(100) {
                       warn!("Slow detection: {:?}ms", elapsed.as_millis());
                   }
                   
                   return Ok(Some(pattern.name));
               }
           }
           
           Ok(None)  // No pattern matched
       }
   }
   ```

#### Task 5.1.2: Pattern Matching Implementation (8 hours)

💡 **Junior Dev Concept**: Pattern Matching
**Glob patterns**: `*` matches any chars, `**` matches dirs recursively
**Examples**:
- `~/work/**` matches any path under ~/work/
- `github.com:company/*` matches company repos
- `*.rs` matches Rust files

Implement flexible matchers:

```rust
impl Matcher {
    pub fn evaluate(&self, context: &DetectionContext) -> Result<bool> {
        match self {
            Matcher::Remote { pattern } => {
                let remotes = context.git_remotes()?;
                Ok(remotes.iter().any(|r| glob_match(pattern, r)))
            }
            Matcher::Path { glob } => {
                Ok(glob_match(glob, context.path.to_str().unwrap()))
            }
            Matcher::Directory { pattern } => {
                if let Some(dir_name) = context.path.file_name() {
                    Ok(glob_match(pattern, dir_name.to_str().unwrap()))
                } else {
                    Ok(false)
                }
            }
            Matcher::GitConfig { key, value } => {
                let config_value = context.git_config(key)?;
                Ok(config_value.as_deref() == Some(value))
            }
            Matcher::Custom(func) => Ok(func(context.path)),
        }
    }
}
```

**Pattern Examples**:
```toml
[[detection_patterns]]
name = "work"
priority = 10
remote = "github.com:mycompany/*"

[[detection_patterns]]
name = "personal"
priority = 5
path = "~/personal/**"

[[detection_patterns]]
name = "opensource"
priority = 8
remote = "github.com:*"
directory = "oss-*"
```

**Step-by-Step Implementation**:

1. **Implement glob matching** (2 hours)
   ```rust
   use glob_match::glob_match;
   
   /// Match with glob pattern, handling home directory expansion
   pub fn glob_match_path(pattern: &str, path: &str) -> bool {
       // Expand ~ to home directory
       let pattern = expand_tilde(pattern);
       let path = expand_tilde(path);
       
       // Normalize paths for comparison
       let pattern = normalize_path(&pattern);
       let path = normalize_path(&path);
       
       glob_match(&pattern, &path)
   }
   
   fn expand_tilde(path: &str) -> String {
       if path.starts_with("~/") {
           if let Some(home) = dirs::home_dir() {
               return path.replacen("~", &home.to_string_lossy(), 1);
           }
       }
       path.to_string()
   }
   
   fn normalize_path(path: &str) -> String {
       // Convert backslashes to forward slashes
       path.replace('\\', "/")
   }
   
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_home_expansion() {
           // This will match paths like /home/user/work/project
           assert!(glob_match_path("~/work/**", "/home/user/work/project/src"));
       }
   }
   ```
   
   💡 **Path Normalization**: Critical for cross-platform!

2. **Implement remote matching** (2 hours)
   ```rust
   /// Match Git remote URLs with patterns
   pub fn match_remote(pattern: &str, remote_url: &str) -> bool {
       // Normalize different remote formats:
       // - https://github.com/user/repo.git
       // - git@github.com:user/repo.git
       // - ssh://git@github.com/user/repo.git
       
       let normalized = normalize_remote_url(remote_url);
       glob_match(pattern, &normalized)
   }
   
   fn normalize_remote_url(url: &str) -> String {
       // Remove protocol
       let url = url.trim_start_matches("https://")
           .trim_start_matches("http://")
           .trim_start_matches("ssh://")
           .trim_start_matches("git@");
       
       // Convert git@host:path to host/path
       let url = url.replace(':', "/");
       
       // Remove .git suffix
       url.trim_end_matches(".git").to_string()
   }
   
   #[test]
   fn test_remote_normalization() {
       assert_eq!(
           normalize_remote_url("git@github.com:rust-lang/rust.git"),
           "github.com/rust-lang/rust"
       );
       assert_eq!(
           normalize_remote_url("https://github.com/rust-lang/rust.git"),
           "github.com/rust-lang/rust"
       );
   }
   ```
   
   ⚠️ **Common Bug**: Different Git remote formats
   ✅ **Solution**: Normalize before matching

3. **Create pattern configuration** (2 hours)
   ```rust
   use serde::{Deserialize, Serialize};
   
   #[derive(Debug, Clone, Deserialize, Serialize)]
   pub struct PatternConfig {
       pub name: String,
       pub priority: Option<u8>,  // Default: 10
       
       // Matcher fields (at least one required)
       #[serde(skip_serializing_if = "Option::is_none")]
       pub remote: Option<String>,
       
       #[serde(skip_serializing_if = "Option::is_none")]
       pub path: Option<String>,
       
       #[serde(skip_serializing_if = "Option::is_none")]
       pub directory: Option<String>,
       
       #[serde(skip_serializing_if = "Option::is_none")]
       pub git_config: Option<HashMap<String, String>>,
   }
   
   impl PatternConfig {
       pub fn to_detection_pattern(&self) -> Result<DetectionPattern> {
           let mut matchers = Vec::new();
           
           if let Some(remote) = &self.remote {
               matchers.push(Matcher::Remote { 
                   pattern: remote.clone() 
               });
           }
           
           if let Some(path) = &self.path {
               matchers.push(Matcher::Path { 
                   glob: path.clone() 
               });
           }
           
           if let Some(dir) = &self.directory {
               matchers.push(Matcher::Directory { 
                   pattern: dir.clone() 
               });
           }
           
           if let Some(configs) = &self.git_config {
               for (key, value) in configs {
                   matchers.push(Matcher::GitConfig {
                       key: key.clone(),
                       value: value.clone(),
                   });
               }
           }
           
           if matchers.is_empty() {
               return Err(ConfigError::NoMatchers);
           }
           
           Ok(DetectionPattern {
               name: self.name.clone(),
               priority: self.priority.unwrap_or(10),
               matchers,
           })
       }
   }
   ```

4. **Load patterns from config** (2 hours)
   ```rust
   impl ProfileDetector {
       pub fn from_config(config: &AppConfig) -> Result<Self> {
           let mut patterns = Vec::new();
           
           // Load user-defined patterns
           for pattern_config in &config.detection_patterns {
               patterns.push(pattern_config.to_detection_pattern()?);
           }
           
           // Add default patterns if none defined
           if patterns.is_empty() {
               patterns.extend(Self::default_patterns());
           }
           
           Ok(Self {
               patterns,
               cache: DetectionCache::new(),
               profile_manager: Arc::new(ProfileManager::new()),
           })
       }
       
       fn default_patterns() -> Vec<DetectionPattern> {
           vec![
               // Work profile for work directory
               DetectionPattern {
                   name: "work".to_string(),
                   priority: 10,
                   matchers: vec![
                       Matcher::Path { glob: "~/work/**".to_string() }
                   ],
               },
               // Personal profile for personal directory
               DetectionPattern {
                   name: "personal".to_string(),
                   priority: 10,
                   matchers: vec![
                       Matcher::Path { glob: "~/personal/**".to_string() }
                   ],
               },
           ]
       }
   }
   ```

#### Task 5.1.3: Performance Optimization (5 hours)

Optimize for speed:

```rust
pub struct DetectionCache {
    entries: DashMap<PathBuf, CacheEntry>,
    ttl: Duration,
}

struct CacheEntry {
    profile: String,
    detected_at: Instant,
    repo_root: PathBuf,
}

impl DetectionCache {
    pub fn get(&self, path: &Path) -> Option<String> {
        // Find repo root for path
        let repo_root = find_git_root(path).ok()?;
        
        // Check if we have a valid cache entry
        if let Some(entry) = self.entries.get(&repo_root) {
            if entry.detected_at.elapsed() < self.ttl {
                return Some(entry.profile.clone());
            }
        }
        None
    }
}
```

**Optimization targets**:
- LRU cache with 1000 entry limit
- 5-minute TTL for entries
- Parallel pattern evaluation
- Early termination on match

#### Task 5.1.4: Integration with Shell (2 hours)

Shell hooks for automatic switching:

```bash
# Bash/Zsh integration
git_setup_auto_detect() {
    local profile=$(git-setup detect --path "$PWD")
    if [[ -n "$profile" ]]; then
        git-setup switch "$profile" --quiet
    fi
}

# Add to PROMPT_COMMAND or precmd
```

---

## 🛑 CHECKPOINT 1: Auto-Detection Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without performance verification.

### Pre-Checkpoint Checklist

- [ ] Detection completes in <100ms
- [ ] Pattern matching works for all types
- [ ] Cache invalidation tested
- [ ] Nested repositories handled
- [ ] Submodules detected correctly
- [ ] Shell integration documented
- [ ] Performance benchmarks passing

### Performance Testing

```bash
# Run detection benchmarks
cargo bench --bench detection_performance

# Test with complex repository
git clone --recursive https://github.com/rust-lang/rust
cd rust
time git-setup detect  # Should be <100ms

# Test cache effectiveness
for i in {1..100}; do
    time git-setup detect
done | grep real  # Should show cache working
```

### Review Requirements

#### Performance (Tech Lead)
- [ ] <100ms on all test cases
- [ ] Cache hit rate >90%
- [ ] Memory usage acceptable
- [ ] No performance regression

#### Functionality (Senior Dev)
- [ ] All pattern types working
- [ ] Priority system correct
- [ ] Edge cases handled
- [ ] Configuration flexible

### Consequences of Skipping

- Slow detection frustrates users
- Incorrect profile detection
- Shell integration unusable
- Performance degrades over time

---

### 5.2 Multi-Method Signing Configuration (25 hours)

**Complexity**: High - Multiple signing protocols
**Files**: `src/signing/mod.rs`, `src/signing/ssh.rs`, `src/signing/gpg.rs`, `src/signing/x509.rs`, `src/signing/gitsign.rs`

#### Task 5.2.1: Signing Architecture (5 hours)

💡 **Junior Dev Concept**: Git Commit Signing
**What it is**: Cryptographically prove YOU made a commit
**Methods**: SSH (newest), GPG (traditional), x509 (enterprise), gitsign (keyless)
**Why multiple**: Different organizations use different methods

Unified signing framework:

```rust
pub trait SigningMethod: Send + Sync {
    fn name(&self) -> &str;
    fn configure(&self, git_config: &mut GitConfig) -> Result<()>;
    fn verify_setup(&self) -> Result<SigningStatus>;
    fn find_keys(&self) -> Result<Vec<SigningKey>>;
}

pub struct SigningConfig {
    pub format: SigningFormat,
    pub key: SigningKeyRef,
    pub program: Option<String>,
}

pub enum SigningFormat {
    Ssh,
    Gpg,
    X509,
}

pub enum SigningKeyRef {
    /// Direct key ID/fingerprint
    Direct(String),
    /// Key file path
    File(PathBuf),
    /// 1Password reference
    OnePassword(String),
    /// Environment variable
    Environment(String),
}
```

#### Task 5.2.2: SSH Signing Implementation (6 hours)

💡 **Junior Dev Concept**: SSH Signing (New!)
**What it is**: Use your SSH key to sign commits (since Git 2.34)
**Advantages**: Most devs already have SSH keys, works with 1Password
**Setup**: Tell Git to use SSH format and which key

Modern SSH commit signing:

```rust
pub struct SshSigning {
    ssh_agent: SshAgent,
    op_integration: Option<Arc<OpCli>>,
}

impl SigningMethod for SshSigning {
    fn configure(&self, git_config: &mut GitConfig) -> Result<()> {
        git_config.set("gpg.format", "ssh")?;
        git_config.set("user.signingkey", &self.key_path())?;
        
        if let Some(allowed_signers) = self.find_allowed_signers()? {
            git_config.set("gpg.ssh.allowedSignersFile", &allowed_signers)?;
        }
        
        // Use 1Password for signing if available
        if self.op_integration.is_some() {
            git_config.set("gpg.ssh.program", "op-ssh-sign")?;
        }
        
        git_config.set("commit.gpgsign", "true")?;
        Ok(())
    }
    
    fn find_keys(&self) -> Result<Vec<SigningKey>> {
        let mut keys = vec![];
        
        // 1. Check SSH agent
        keys.extend(self.ssh_agent.list_keys()?);
        
        // 2. Check 1Password
        if let Some(op) = &self.op_integration {
            keys.extend(op.list_ssh_keys().await?);
        }
        
        // 3. Check ~/.ssh/
        keys.extend(self.scan_ssh_dir()?);
        
        Ok(keys)
    }
}
```

**Step-by-Step Implementation**:

1. **Understand SSH signing config** (1 hour)
   ```rust
   /// Git configuration for SSH signing:
   /// - gpg.format = "ssh" (use SSH instead of GPG)
   /// - user.signingkey = "/path/to/key.pub" or "ssh-rsa AAAA..."
   /// - gpg.ssh.allowedSignersFile = "/path/to/allowed_signers"
   /// - gpg.ssh.program = "ssh-keygen" (or "op-ssh-sign" for 1Password)
   /// - commit.gpgsign = true (sign all commits)
   
   pub struct SshSigningConfig {
       pub key_path: Option<PathBuf>,
       pub key_data: Option<String>,  // Raw public key
       pub allowed_signers_file: Option<PathBuf>,
       pub ssh_program: String,
       pub sign_commits: bool,
   }
   ```
   
   💡 **Config Priority**: key_path preferred over key_data

2. **Implement SSH agent integration** (2 hours)
   ```rust
   use std::process::Command;
   
   pub struct SshAgent;
   
   impl SshAgent {
       pub fn list_keys(&self) -> Result<Vec<SshKey>> {
           // Run ssh-add -L to list keys in agent
           let output = Command::new("ssh-add")
               .arg("-L")
               .output()?;
           
           if !output.status.success() {
               // No agent or no keys
               return Ok(Vec::new());
           }
           
           let stdout = String::from_utf8_lossy(&output.stdout);
           let mut keys = Vec::new();
           
           for line in stdout.lines() {
               if let Ok((key_type, fingerprint)) = parse_public_key(line) {
                   keys.push(SshKey {
                       source: KeySource::Agent,
                       key_type,
                       fingerprint,
                       public_key: line.to_string(),
                       path: None,
                   });
               }
           }
           
           Ok(keys)
       }
       
       pub fn key_in_agent(&self, fingerprint: &str) -> Result<bool> {
           let keys = self.list_keys()?;
           Ok(keys.iter().any(|k| k.fingerprint == fingerprint))
       }
   }
   ```
   
   ⚠️ **Common Issue**: ssh-agent not running
   ✅ **Handle Gracefully**: Return empty list, don't error

3. **Scan SSH directory** (2 hours)
   ```rust
   fn scan_ssh_dir(&self) -> Result<Vec<SshKey>> {
       let ssh_dir = dirs::home_dir()
           .ok_or(SigningError::NoHomeDir)?
           .join(".ssh");
       
       if !ssh_dir.exists() {
           return Ok(Vec::new());
       }
       
       let mut keys = Vec::new();
       
       // Common SSH key patterns
       let patterns = [
           "id_*.pub",
           "*_rsa.pub",
           "*_ed25519.pub",
           "*_ecdsa.pub",
       ];
       
       for pattern in &patterns {
           let glob_pattern = ssh_dir.join(pattern);
           
           for entry in glob::glob(&glob_pattern.to_string_lossy())? {
               let path = entry?;
               
               // Read public key
               let content = fs::read_to_string(&path)?;
               
               if let Ok((key_type, fingerprint)) = parse_public_key(&content) {
                   keys.push(SshKey {
                       source: KeySource::File(path.clone()),
                       key_type,
                       fingerprint,
                       public_key: content.trim().to_string(),
                       path: Some(path),
                   });
               }
           }
       }
       
       Ok(keys)
   }
   ```

4. **Configure Git for SSH signing** (1 hour)
   ```rust
   impl SigningMethod for SshSigning {
       fn configure(&self, git_config: &mut GitConfig) -> Result<()> {
           // Set SSH as signing format
           git_config.set("gpg.format", "ssh")?;
           
           // Set signing key
           match &self.selected_key {
               Some(key) => {
                   // Prefer file path if available
                   if let Some(path) = &key.path {
                       git_config.set("user.signingkey", path.to_str().unwrap())?;
                   } else {
                       // Use raw key data
                       git_config.set("user.signingkey", &key.public_key)?;
                   }
               }
               None => return Err(SigningError::NoKeySelected),
           }
           
           // Set up allowed signers file
           let allowed_signers = self.setup_allowed_signers()?;
           git_config.set("gpg.ssh.allowedSignersFile", &allowed_signers)?;
           
           // Use 1Password if available
           if self.op_integration.is_some() && self.selected_key.as_ref()
               .map(|k| k.source == KeySource::OnePassword)
               .unwrap_or(false) {
               git_config.set("gpg.ssh.program", "op-ssh-sign")?;
           }
           
           // Enable signing
           git_config.set("commit.gpgsign", "true")?;
           git_config.set("tag.gpgsign", "true")?;
           
           Ok(())
       }
   }
   ```

#### Task 5.2.3: GPG Signing Implementation (6 hours)

Traditional GPG signing:

```rust
pub struct GpgSigning {
    gpg_binary: PathBuf,
}

impl SigningMethod for GpgSigning {
    fn configure(&self, git_config: &mut GitConfig) -> Result<()> {
        git_config.set("gpg.format", "openpgp")?;
        git_config.set("user.signingkey", &self.key_id)?;
        git_config.set("gpg.program", &self.gpg_binary)?;
        git_config.set("commit.gpgsign", "true")?;
        Ok(())
    }
    
    fn find_keys(&self) -> Result<Vec<SigningKey>> {
        let output = Command::new(&self.gpg_binary)
            .args(&["--list-secret-keys", "--with-colons"])
            .output()?;
        
        self.parse_gpg_keys(&output.stdout)
    }
}
```

#### Task 5.2.4: x509 Signing Implementation (4 hours)

S/MIME certificate signing:

```rust
pub struct X509Signing {
    cert_path: PathBuf,
}

impl SigningMethod for X509Signing {
    fn configure(&self, git_config: &mut GitConfig) -> Result<()> {
        git_config.set("gpg.format", "x509")?;
        git_config.set("user.signingkey", &self.cert_path)?;
        git_config.set("gpg.x509.program", "smimesign")?;
        git_config.set("commit.gpgsign", "true")?;
        Ok(())
    }
}
```

#### Task 5.2.5: Gitsign Implementation (4 hours)

Sigstore keyless signing:

```rust
pub struct GitsignSigning {
    gitsign_binary: PathBuf,
    fulcio_url: Option<String>,
    rekor_url: Option<String>,
}

impl SigningMethod for GitsignSigning {
    fn configure(&self, git_config: &mut GitConfig) -> Result<()> {
        git_config.set("gpg.format", "gitsign")?;
        git_config.set("gpg.gitsign.program", &self.gitsign_binary)?;
        
        if let Some(fulcio) = &self.fulcio_url {
            git_config.set("gitsign.fulcio", fulcio)?;
        }
        
        git_config.set("commit.gpgsign", "true")?;
        git_config.set("tag.gpgsign", "true")?;
        Ok(())
    }
}
```

---

## 🛑 CHECKPOINT 2: Signing Methods Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without signing verification.

### Pre-Checkpoint Checklist

- [ ] SSH signing configuration working
- [ ] GPG signing configuration working
- [ ] x509 signing configuration working
- [ ] Gitsign configuration working
- [ ] No Git config conflicts
- [ ] Key discovery finds all keys
- [ ] Actual commit signing tested

### Signing Verification

```bash
# Test SSH signing
git-setup signing configure --method ssh
echo "test" > test.txt
git add test.txt
git commit -m "Test SSH signing"
git log --show-signature -1  # Should show SSH signature

# Test GPG signing
git-setup signing configure --method gpg
git commit --amend -m "Test GPG signing"
git log --show-signature -1  # Should show GPG signature

# Verify no conflicts
git config --list | grep -E "gpg\.|user\.signingkey|commit\.gpgsign"
```

### Review Requirements

#### Functionality (Senior Dev)
- [ ] All signing methods work
- [ ] Configuration is correct
- [ ] Key discovery complete
- [ ] Method switching clean

#### Security (Security Engineer)
- [ ] No private keys exposed
- [ ] Secure key selection
- [ ] Proper Git configuration
- [ ] No credential leaks

### Consequences of Skipping

- Broken commit signing
- Security audit failure
- User commits rejected
- Complex troubleshooting

---

### 5.3 Health Check System (20 hours)

**Complexity**: Medium - Comprehensive diagnostics
**Files**: `src/health/mod.rs`, `src/health/checks/*.rs`, `src/health/report.rs`

#### Task 5.3.1: Health Check Framework (5 hours)

💡 **Junior Dev Concept**: Health Checks
**What it is**: Automated diagnostics to find configuration problems
**Like**: `brew doctor` or `npm doctor` - finds and suggests fixes
**Goal**: Users can self-diagnose issues before asking for help

Extensible health check system:

```rust
#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> CheckCategory;
    async fn check(&self, context: &CheckContext) -> CheckResult;
}

pub enum CheckCategory {
    Profile,
    Git,
    Signing,
    System,
    Integration,
}

pub struct CheckResult {
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
    pub fix_hint: Option<String>,
}

pub enum CheckStatus {
    Ok,
    Warning(String),
    Error(String),
}

pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
    profile_manager: Arc<dyn ProfileManager>,
}
```

#### Task 5.3.2: Core Health Checks (8 hours)

💡 **Junior Dev Concept**: Check Categories
**System**: Is Git/SSH/GPG installed and working?
**Profile**: Are profiles valid and complete?
**Integration**: Does 1Password/signing work?
**Purpose**: Group related issues for easier fixing

Implement essential checks:

```rust
pub struct GitVersionCheck;

#[async_trait]
impl HealthCheck for GitVersionCheck {
    fn name(&self) -> &str { "Git Version" }
    fn category(&self) -> CheckCategory { CheckCategory::System }
    
    async fn check(&self, _: &CheckContext) -> CheckResult {
        match check_git_version() {
            Ok(version) if version >= Version::new(2, 25, 0) => {
                CheckResult::ok(format!("Git {} installed", version))
            }
            Ok(version) => {
                CheckResult::warning(
                    format!("Git {} is old", version),
                    "Consider upgrading to Git 2.25+"
                )
            }
            Err(e) => {
                CheckResult::error(
                    "Git not found",
                    "Install Git: https://git-scm.com"
                )
            }
        }
    }
}
```

**Check Categories**:
1. **System Checks**
   - Git version and configuration
   - SSH agent status
   - GPG availability
   - 1Password CLI status

2. **Profile Checks**
   - Profile validity
   - Configuration conflicts
   - Missing dependencies
   - Signing key availability

3. **Integration Checks**
   - 1Password connection
   - SSH key access
   - Network connectivity
   - Permission issues

**Step-by-Step Implementation**:

1. **Implement Git checks** (2 hours)
   ```rust
   use semver::Version;
   use std::process::Command;
   
   fn check_git_version() -> Result<Version> {
       let output = Command::new("git")
           .arg("--version")
           .output()
           .map_err(|_| HealthError::GitNotFound)?;
       
       if !output.status.success() {
           return Err(HealthError::GitError);
       }
       
       // Parse "git version 2.34.1" format
       let version_str = String::from_utf8_lossy(&output.stdout);
       let version_part = version_str
           .strip_prefix("git version ")
           .ok_or(HealthError::UnknownGitVersion)?
           .split_whitespace()
           .next()
           .ok_or(HealthError::UnknownGitVersion)?;
       
       // Handle version strings like "2.34.1.windows.1"
       let clean_version = version_part.split('.').take(3).collect::<Vec<_>>().join(".");
       
       Version::parse(&clean_version)
           .map_err(|_| HealthError::UnknownGitVersion)
   }
   
   pub struct GitConfigCheck;
   
   #[async_trait]
   impl HealthCheck for GitConfigCheck {
       fn name(&self) -> &str { "Git Configuration" }
       fn category(&self) -> CheckCategory { CheckCategory::System }
       
       async fn check(&self, ctx: &CheckContext) -> CheckResult {
           // Check if user.name and user.email are set
           let config = match git2::Config::open_default() {
               Ok(c) => c,
               Err(_) => return CheckResult::error(
                   "Cannot read Git config",
                   "Check Git installation"
               ),
           };
           
           let mut issues = Vec::new();
           
           if config.get_string("user.name").is_err() {
               issues.push("user.name not set");
           }
           
           if config.get_string("user.email").is_err() {
               issues.push("user.email not set");
           }
           
           if issues.is_empty() {
               CheckResult::ok("Git user configured")
           } else {
               CheckResult::warning(
                   format!("Git config incomplete: {}", issues.join(", ")),
                   "Run 'git-setup switch <profile>' to configure"
               )
           }
       }
   }
   ```

2. **Implement profile checks** (3 hours)
   ```rust
   pub struct ProfileValidityCheck {
       profile_manager: Arc<dyn ProfileManager>,
   }
   
   #[async_trait]
   impl HealthCheck for ProfileValidityCheck {
       fn name(&self) -> &str { "Profile Validity" }
       fn category(&self) -> CheckCategory { CheckCategory::Profile }
       
       async fn check(&self, ctx: &CheckContext) -> CheckResult {
           let profiles = match self.profile_manager.list().await {
               Ok(p) => p,
               Err(_) => return CheckResult::error(
                   "Cannot read profiles",
                   "Check ~/.config/git-setup/profiles/"
               ),
           };
           
           if profiles.is_empty() {
               return CheckResult::warning(
                   "No profiles found",
                   "Run 'git-setup new' to create your first profile"
               );
           }
           
           let mut invalid = Vec::new();
           
           for profile in profiles {
               if let Err(e) = profile.validate() {
                   invalid.push(format!("{}: {}", profile.name, e));
               }
           }
           
           if invalid.is_empty() {
               CheckResult::ok(format!("{} profiles valid", profiles.len()))
           } else {
               CheckResult::error(
                   format!("Invalid profiles: {}", invalid.join(", ")),
                   "Run 'git-setup edit <profile>' to fix"
               )
           }
       }
   }
   ```

3. **Implement signing checks** (2 hours)
   ```rust
   pub struct SigningKeyCheck {
       signing_manager: Arc<SigningManager>,
   }
   
   #[async_trait]
   impl HealthCheck for SigningKeyCheck {
       fn name(&self) -> &str { "Signing Keys" }
       fn category(&self) -> CheckCategory { CheckCategory::Integration }
       
       async fn check(&self, ctx: &CheckContext) -> CheckResult {
           // Check current Git signing configuration
           let config = git2::Config::open_default()?;
           
           let format = config.get_string("gpg.format")
               .unwrap_or_else(|_| "openpgp".to_string());
           
           let signing_key = match config.get_string("user.signingkey") {
               Ok(key) => key,
               Err(_) => return CheckResult::ok("Signing not configured"),
           };
           
           // Verify key is available
           match format.as_str() {
               "ssh" => self.check_ssh_key(&signing_key).await,
               "openpgp" | "gpg" => self.check_gpg_key(&signing_key).await,
               "x509" => self.check_x509_cert(&signing_key).await,
               _ => CheckResult::warning(
                   format!("Unknown signing format: {}", format),
                   "Check gpg.format configuration"
               ),
           }
       }
       
       async fn check_ssh_key(&self, key: &str) -> CheckResult {
           // Check if key exists (file or in agent)
           if key.starts_with("op://") {
               // 1Password reference
               CheckResult::ok("Using 1Password SSH key")
           } else if Path::new(key).exists() {
               CheckResult::ok("SSH key file found")
           } else if key.starts_with("ssh-") {
               // Raw key, check if in agent
               let agent = SshAgent::new();
               if agent.key_in_agent(key).unwrap_or(false) {
                   CheckResult::ok("SSH key in agent")
               } else {
                   CheckResult::warning(
                       "SSH key not in agent",
                       "Run 'ssh-add' to add key to agent"
                   )
               }
           } else {
               CheckResult::error(
                   "SSH key not found",
                   "Check user.signingkey configuration"
               )
           }
       }
   }
   ```

4. **Implement integration checks** (1 hour)
   ```rust
   pub struct OnePasswordCheck {
       op_cli: Option<Arc<OpCli>>,
   }
   
   #[async_trait]
   impl HealthCheck for OnePasswordCheck {
       fn name(&self) -> &str { "1Password Integration" }
       fn category(&self) -> CheckCategory { CheckCategory::Integration }
       
       async fn check(&self, _: &CheckContext) -> CheckResult {
           match OpCli::detect().await {
               Ok(cli) => {
                   if cli.authenticated {
                       CheckResult::ok(format!("1Password CLI {} (authenticated)", cli.version))
                   } else {
                       CheckResult::warning(
                           format!("1Password CLI {} (not authenticated)", cli.version),
                           "Run 'op signin' to authenticate"
                       )
                   }
               }
               Err(OnePasswordError::CliNotFound) => {
                   CheckResult::info(
                       "1Password CLI not installed",
                       "Optional: Install from https://1password.com/downloads/command-line/"
                   )
               }
               Err(OnePasswordError::VersionTooOld { found, required }) => {
                   CheckResult::warning(
                       format!("1Password CLI {} is old (need {}+)", found, required),
                       "Update 1Password CLI for full functionality"
                   )
               }
               Err(e) => CheckResult::error(
                   format!("1Password check failed: {}", e),
                   "Check 1Password CLI installation"
               ),
           }
       }
   }
   ```

#### Task 5.3.3: Health Report Generation (5 hours)

Beautiful health reports:

```rust
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub profile: Option<String>,
    pub results: Vec<CheckResult>,
}

impl HealthReport {
    pub fn render(&self) -> String {
        let mut output = String::new();
        
        // Summary
        let (ok, warn, err) = self.count_by_status();
        writeln!(output, "🏥 Health Check Report");
        writeln!(output, "   Profile: {}", self.profile.as_deref().unwrap_or("none"));
        writeln!(output, "   Status: {} ✓  {} ⚠  {} ✗", ok, warn, err);
        writeln!(output);
        
        // Group by category
        for category in CheckCategory::all() {
            let results = self.filter_by_category(category);
            if !results.is_empty() {
                writeln!(output, "{}", category.display_name());
                for result in results {
                    writeln!(output, "  {}", result.format());
                }
            }
        }
        
        output
    }
}
```

#### Task 5.3.4: Auto-fix Capabilities (2 hours)

Simple fixes for common issues:

```rust
pub trait AutoFixable: HealthCheck {
    async fn can_fix(&self, result: &CheckResult) -> bool;
    async fn fix(&self, context: &CheckContext) -> Result<()>;
}

impl HealthChecker {
    pub async fn fix_all(&self, context: &CheckContext) -> Result<Vec<FixResult>> {
        let mut fixes = vec![];
        
        for check in &self.checks {
            let result = check.check(context).await?;
            
            if let Some(fixable) = check.as_any().downcast_ref::<dyn AutoFixable>() {
                if fixable.can_fix(&result).await {
                    match fixable.fix(context).await {
                        Ok(()) => fixes.push(FixResult::success(check.name())),
                        Err(e) => fixes.push(FixResult::failed(check.name(), e)),
                    }
                }
            }
        }
        
        Ok(fixes)
    }
}
```

---

## 🛑 CHECKPOINT 3: Health System Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without health check validation.

### Pre-Checkpoint Checklist

- [ ] All check categories implemented
- [ ] Health report renders correctly
- [ ] Fix suggestions helpful and accurate
- [ ] Auto-fix works for simple issues
- [ ] Performance <1s for all checks
- [ ] No false positives in checks

### Health Check Testing

```bash
# Run full health check
git-setup doctor

# Test specific categories
git-setup doctor --category system
git-setup doctor --category profile
git-setup doctor --category integration

# Test auto-fix
git-setup doctor --fix

# Verify performance
time git-setup doctor --all
```

### Review Requirements

#### Usefulness (Product Owner)
- [ ] Diagnostics find real issues
- [ ] Fix suggestions work
- [ ] Output clear to users
- [ ] Reduces support burden

#### Technical (Tech Lead)
- [ ] Extensible architecture
- [ ] Performance acceptable
- [ ] No false positives
- [ ] Error handling robust

### Consequences of Skipping

- Users can't self-diagnose
- Support burden increases
- Configuration issues hidden
- Poor user experience

---

### 5.4 Remote Profile Import (15 hours)

**Complexity**: Medium - Security critical
**Files**: `src/import/remote.rs`, `src/import/verify.rs`

#### Task 5.4.1: Remote Import Framework (4 hours)

💡 **Junior Dev Concept**: Secure Remote Import
**What it is**: Download profiles from URLs (like GitHub)
**Security critical**: Must verify the profile is safe
**Key measures**: HTTPS only, hash verification, domain whitelist

Secure remote profile fetching:

```rust
pub struct RemoteImporter {
    http_client: reqwest::Client,
    profile_manager: Arc<dyn ProfileManager>,
}

impl RemoteImporter {
    pub async fn import(&self, url: &Url, options: ImportOptions) -> Result<Profile> {
        // 1. Validate URL
        self.validate_url(url)?;
        
        // 2. Fetch content
        let content = self.fetch_with_retry(url).await?;
        
        // 3. Verify integrity
        if let Some(expected_hash) = &options.sha256 {
            verify_sha256(&content, expected_hash)?;
        }
        
        // 4. Parse and validate
        let profile = Profile::from_toml(&content)?;
        profile.validate()?;
        
        // 5. Import with conflict resolution
        self.import_profile(profile, options).await
    }
}
```

#### Task 5.4.2: Security & Verification (6 hours)

Implement security measures:

```rust
pub struct ImportOptions {
    pub sha256: Option<String>,
    pub signature: Option<String>,
    pub trusted_domains: Vec<String>,
    pub conflict_resolution: ConflictResolution,
}

impl RemoteImporter {
    fn validate_url(&self, url: &Url) -> Result<()> {
        // HTTPS only
        if url.scheme() != "https" {
            return Err(ImportError::InsecureProtocol);
        }
        
        // Check trusted domains
        if !self.is_trusted_domain(url.host_str().unwrap()) {
            return Err(ImportError::UntrustedDomain);
        }
        
        Ok(())
    }
    
    async fn verify_signature(&self, content: &[u8], sig: &str) -> Result<()> {
        // GPG signature verification
        let temp_file = self.write_temp(content)?;
        let sig_file = self.write_temp(sig.as_bytes())?;
        
        let status = Command::new("gpg")
            .args(&["--verify", sig_file.path(), temp_file.path()])
            .status()?;
        
        if !status.success() {
            return Err(ImportError::InvalidSignature);
        }
        
        Ok(())
    }
}
```

#### Task 5.4.3: Import Formats Support (3 hours)

Support multiple formats:

```rust
pub enum ImportFormat {
    GitSetupToml,      // Native format
    OnePasswordAgent,  // agent.toml format
    GitConfig,         // .gitconfig format
    Json,              // JSON representation
}

impl RemoteImporter {
    pub async fn import_any(&self, url: &Url) -> Result<Profile> {
        let content = self.fetch(url).await?;
        
        // Detect format
        let format = match url.path() {
            p if p.ends_with(".toml") => self.detect_toml_format(&content)?,
            p if p.ends_with(".json") => ImportFormat::Json,
            p if p.ends_with(".gitconfig") => ImportFormat::GitConfig,
            _ => self.detect_format(&content)?,
        };
        
        // Convert to Profile
        match format {
            ImportFormat::GitSetupToml => Profile::from_toml(&content),
            ImportFormat::OnePasswordAgent => self.convert_op_agent(&content),
            ImportFormat::GitConfig => self.convert_gitconfig(&content),
            ImportFormat::Json => Profile::from_json(&content),
        }
    }
}
```

#### Task 5.4.4: Batch Import & Templates (2 hours)

Template repositories support:

```rust
pub async fn import_template_repo(&self, repo_url: &str) -> Result<Vec<Profile>> {
    // Clone or fetch template repository
    let temp_dir = self.clone_templates(repo_url).await?;
    
    // Find all profile files
    let profile_files = glob::glob(&format!("{}/**/*.toml", temp_dir.path()))?;
    
    // Import each profile
    let mut profiles = vec![];
    for path in profile_files {
        let content = std::fs::read_to_string(path?)?;
        if let Ok(profile) = Profile::from_toml(&content) {
            profiles.push(profile);
        }
    }
    
    Ok(profiles)
}
```

---

## 🛑 CHECKPOINT 4: Phase 5 Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** to Phase 6 without final review.

### Pre-Checkpoint Checklist

- [ ] Auto-detection <100ms verified
- [ ] All signing methods tested
- [ ] Health checks comprehensive
- [ ] Remote import security verified
- [ ] Documentation complete
- [ ] Integration tests passing

### Final Integration Testing

```bash
# Test detection with signing
cd ~/work/project
git-setup detect  # Should select work profile
git config user.signingkey  # Should be configured

# Test health check after changes
git-setup doctor  # All green

# Test secure import
git-setup import https://github.com/company/git-profiles/work.toml \
  --sha256 abc123...

# Full workflow test
./scripts/test-phase5-integration.sh
```

### Phase 5 Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Detection Time | <100ms | ___ | ⬜ |
| Health Check Time | <1s | ___ | ⬜ |
| Signing Success | 100% | ___ | ⬜ |
| Import Security | 100% | ___ | ⬜ |
| Test Coverage | ≥80% | ___ | ⬜ |

### Feature Matrix

| Feature | SSH | GPG | x509 | Gitsign |
|---------|-----|-----|------|------|
| Key Discovery | ✅ | ✅ | ✅ | ✅ |
| Configuration | ✅ | ✅ | ✅ | ✅ |
| 1Password | ✅ | ✅ | ❌ | ❌ |
| Verification | ✅ | ✅ | ✅ | ✅ |

### Sign-offs Required

- [ ] **Tech Lead**: Architecture sound
- [ ] **Security**: Import security verified
- [ ] **QA Lead**: All features tested
- [ ] **Product Owner**: Advanced features working
- [ ] **Performance**: Targets met

### Handoff to Phase 6

**What Phase 6 needs**:
1. Performance baselines for optimization
2. Cross-platform test cases
3. Feature flags for platform-specific code
4. Documentation for distribution

---

## Testing Strategy

### Pattern Testing
```rust
#[test]
fn test_pattern_matching() {
    let pattern = DetectionPattern {
        name: "work",
        matchers: vec![
            Matcher::Remote { pattern: "github.com:company/*" },
            Matcher::Path { glob: "**/work/**" },
        ],
    };
    
    assert!(pattern.matches("github.com:company/project"));
    assert!(pattern.matches("/home/user/work/project"));
    assert!(!pattern.matches("github.com:personal/project"));
}
```

### Signing Integration Tests
- Mock Git commands
- Test configuration generation
- Verify no conflicts
- Test key discovery

### Health Check Tests
- Simulate various failure modes
- Test fix suggestions
- Verify performance targets
- Test report formatting

### Security Tests
- Test URL validation
- Test signature verification
- Test hash validation
- Test untrusted domain rejection

## Common Issues & Solutions

### Issue: Detection Takes >200ms
**Symptom**: Shell feels sluggish when changing directories
**Cause**: No cache or inefficient patterns
**Solution**:
```rust
// Add caching layer
let cache_key = repo_root.to_string_lossy();
if let Some(cached) = cache.get(&cache_key) {
    return Ok(cached);  // <1ms
}

// Profile the slow part
let start = Instant::now();
let result = detect_profile(&context)?;
if start.elapsed() > Duration::from_millis(100) {
    warn!("Slow detection: {:?}ms", start.elapsed().as_millis());
}
```

### Issue: Signing Configuration Conflicts
**Symptom**: Git complains about invalid signing configuration
**Cause**: Leftover config from previous signing method
**Solution**:
```rust
// Clear previous signing config before setting new
fn clear_signing_config(git_config: &mut GitConfig) -> Result<()> {
    let keys_to_clear = [
        "gpg.format",
        "user.signingkey",
        "gpg.program",
        "gpg.ssh.program",
        "gpg.ssh.allowedSignersFile",
    ];
    
    for key in &keys_to_clear {
        git_config.unset(key, ConfigScope::Global).ok();
    }
    
    Ok(())
}
```

### Issue: Health Check False Positives
**Symptom**: Doctor reports issues that aren't real
**Cause**: Overly strict checks
**Solution**:
```rust
// Be permissive with optional features
if !feature_available {
    return CheckResult::info(  // Not warning or error!
        "Optional feature not configured",
        "This is fine unless you need this feature"
    );
}
```

### Issue: Remote Import SSL Errors
**Symptom**: "certificate verify failed" when importing
**Cause**: Corporate proxy or self-signed certs
**Solution**:
```rust
// Allow (but warn about) insecure mode
let client = if config.allow_insecure {
    warn!("INSECURE MODE: Certificate validation disabled!");
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
} else {
    reqwest::Client::new()
};
```

## Performance Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| Profile Detection | <100ms | <200ms |
| Pattern Evaluation | <10ms | <20ms |
| Health Check Suite | <1s | <2s |
| Remote Import | <5s | <10s |
| Signing Config | <50ms | <100ms |

## Security Considerations

- Pattern matching runs in sandbox
- No arbitrary code execution
- Remote imports require HTTPS
- Signature verification for imports
- No credential exposure in health checks
- Rate limiting for remote operations

## Junior Developer Tips

### Getting Started with Phase 5

1. **Understanding patterns**:
   - Play with glob patterns at https://globster.xyz/
   - Test regex at https://regex101.com/
   - Try detection on sample repos first

2. **Testing signing methods**:
   ```bash
   # Create test repo
   mkdir test-signing && cd test-signing
   git init
   
   # Test each method
   git-setup signing configure --method ssh
   echo "test" > file
   git add file
   git commit -m "Test"
   git log --show-signature
   ```

3. **Health check development**:
   - Start with simple checks (file exists?)
   - Test fix suggestions manually
   - Make error messages helpful

4. **Security mindset for imports**:
   - Never trust user input
   - Always validate downloaded content
   - Think: "How could this be attacked?"

### Debugging Advanced Features

```rust
// Enable detailed logging
std::env::set_var("RUST_LOG", "git_setup=debug");
env_logger::init();

// Time operations
let start = Instant::now();
// ... operation ...
debug!("Operation took {:?}", start.elapsed());
```

## Next Phase Preview

Phase 6 (Platform & Polish) will:
- Optimize for all platforms
- Create distribution packages
- Polish user experience
- Set up release automation
- Add telemetry (optional)

**What Phase 6 needs from Phase 5**:
- Performance baselines to beat
- Feature detection patterns
- Platform-specific test cases
- Polish opportunities identified

---

*Last updated: 2025-07-30*