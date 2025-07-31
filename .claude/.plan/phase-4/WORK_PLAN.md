# Phase 4: 1Password Integration - Work Plan

## Prerequisites

Phase 4 integrates 1Password CLI for secure credential management.

**Required from Previous Phases**:
- Secure memory types (Phase 1)
- Profile management system (Phase 2)
- TUI key selection UI (Phase 3)
- Secure credential store (Phase 1)

**Required Knowledge**:
- **1Password CLI**: op command, authentication flows, vault structure (*critical*)
- **Process Management**: Rust Command API, stdout parsing (*critical*)
- **Security**: Credential handling, biometric auth (*critical*)
- **JSON Parsing**: Custom 1Password formats (*required*)

**Required Tools**:
- 1Password CLI v2.0+ installed
- 1Password account (for testing)
- Multiple test vaults setup
- SSH keys in 1Password

💡 **Junior Dev Resources**:
- 📚 [1Password CLI Guide](https://developer.1password.com/docs/cli/get-started/) - Complete tutorial
- 🎥 [Process Management in Rust](https://www.youtube.com/watch?v=QCIU24XH_vw) - 25 min video
- 📖 [Security Best Practices](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html) - OWASP guide
- 🔧 [op CLI Playground](https://developer.1password.com/docs/cli/reference/) - Try commands
- 💻 [JSON Parsing with Serde](https://serde.rs/json.html) - Official guide

## Quick Reference - Essential Resources

### 1Password Documentation
- [1Password CLI Reference](https://developer.1password.com/docs/cli)
- [op CLI Commands](https://developer.1password.com/docs/cli/reference)
- [1Password SSH Agent](https://developer.1password.com/docs/ssh)
- [Biometric Unlock](https://developer.1password.com/docs/cli/biometric-unlock)

### Project Resources
- **[SPEC.md](../../spec/SPEC.md)** - See FR-003 1Password CLI Integration
- **[Security Module](../phase-1/)** - SensitiveString, SecureBuffer
- **[1Password Examples](../../examples/1password/)** - Sample interactions

### Commands
- `op --version` - Check CLI version
- `op vault list` - List accessible vaults
- `op item list --categories "SSH Key"` - List SSH keys
- `op whoami` - Check authentication status

## Overview

Phase 4 integrates 1Password CLI to enable secure credential management without storing actual keys. This integration is critical for the security value proposition of git-setup-rs.

**Key Deliverables**:
- 1Password CLI wrapper with version detection
- Vault and item browsing capabilities
- SSH key discovery and metadata extraction
- Biometric authentication support
- Graceful degradation when op unavailable

**Checkpoint Strategy**: 4 checkpoints for integration milestones

**Time Estimate**: 2 weeks (80 hours)

## Development Methodology: Test-Driven Development (TDD)

Special considerations for external CLI integration:
1. **Mock op CLI** for unit tests
2. **Integration tests** with real 1Password (optional)
3. **Security tests** - No credential leakage
4. **Error handling** - Every CLI failure mode

## Done Criteria Checklist

Phase 4 is complete when:
- [ ] op CLI detected and version verified
- [ ] List vaults and items in <2s
- [ ] SSH keys discovered with metadata
- [ ] Biometric auth works (where available)
- [ ] op:// references handled correctly
- [ ] No actual keys stored in memory
- [ ] Graceful fallback without op CLI
- [ ] Custom GPG format supported
- [ ] All 4 checkpoints reviewed

## Work Breakdown with Review Checkpoints

### 4.1 1Password CLI Wrapper Foundation (20 hours)

**Complexity**: High - External process management
**Files**: `src/onepassword/mod.rs`, `src/onepassword/cli.rs`, `src/onepassword/error.rs`

#### Task 4.1.1: CLI Detection and Version Check (5 hours)

💡 **Junior Dev Concept**: External Process Management
**What it is**: Running other programs (like `op`) from your Rust code
**Key challenges**: Finding the program, handling output, managing errors
**Security critical**: Never leak credentials through command arguments or environment

Implement robust op CLI detection:

```rust
pub struct OpCli {
    path: PathBuf,
    version: Version,
    authenticated: bool,
}

impl OpCli {
    pub async fn detect() -> Result<Self> {
        // 1. Find op in PATH
        let path = which::which("op")
            .map_err(|_| OnePasswordError::CliNotFound)?;
        
        // 2. Check version
        let output = Command::new(&path)
            .arg("--version")
            .output()
            .await?;
        
        let version = parse_version(&output.stdout)?;
        if version < Version::new(2, 0, 0) {
            return Err(OnePasswordError::VersionTooOld(version));
        }
        
        // 3. Check auth status
        let authenticated = Self::check_auth(&path).await?;
        
        Ok(Self { path, version, authenticated })
    }
}
```

**Requirements**:
- Support op 2.0.0+
- Clear error for missing/old CLI
- <100ms detection time
- Handle PATH edge cases

**Step-by-Step Implementation**:

1. **Find op in the system PATH** (1 hour)
   ```rust
   use which::which;
   use std::process::Command;
   
   pub fn find_op_cli() -> Result<PathBuf> {
       // Try to find 'op' in PATH
       match which("op") {
           Ok(path) => {
               debug!("Found op CLI at: {:?}", path);
               Ok(path)
           }
           Err(_) => {
               // Check common installation locations
               let common_paths = [
                   "/usr/local/bin/op",
                   "/opt/1Password/op",
                   "C:\\Program Files\\1Password CLI\\op.exe",
               ];
               
               for path in &common_paths {
                   let path = Path::new(path);
                   if path.exists() {
                       return Ok(path.to_path_buf());
                   }
               }
               
               Err(OnePasswordError::CliNotFound)
           }
       }
   }
   ```
   
   💡 **Why check common paths?** Users might install outside PATH

2. **Parse version safely** (1.5 hours)
   ```rust
   use semver::Version;
   
   async fn get_op_version(path: &Path) -> Result<Version> {
       let output = Command::new(path)
           .arg("--version")
           .output()
           .await
           .map_err(|e| OnePasswordError::CliExecution(e))?;
       
       if !output.status.success() {
           return Err(OnePasswordError::CliError {
               stderr: String::from_utf8_lossy(&output.stderr).to_string(),
           });
       }
       
       // Parse version from output like "2.7.1"
       let version_str = String::from_utf8_lossy(&output.stdout);
       let version_str = version_str.trim();
       
       // Handle different formats: "2.7.1" or "op version 2.7.1"
       let version_part = version_str
           .split_whitespace()
           .last()
           .ok_or(OnePasswordError::InvalidVersionFormat)?;
       
       Version::parse(version_part)
           .map_err(|_| OnePasswordError::InvalidVersionFormat)
   }
   ```
   
   ⚠️ **Common Mistake**: Not handling different version output formats
   ✅ **Solution**: Extract just the version number part

3. **Check authentication status** (1.5 hours)
   ```rust
   async fn check_auth_status(path: &Path) -> Result<bool> {
       // Run 'op whoami' to check if authenticated
       let output = Command::new(path)
           .arg("whoami")
           .output()
           .await?;
       
       // If successful, we're authenticated
       if output.status.success() {
           // Parse account info if needed
           let account_info = String::from_utf8_lossy(&output.stdout);
           debug!("Authenticated as: {}", account_info.trim());
           Ok(true)
       } else {
           // Check if it's an auth error or other error
           let stderr = String::from_utf8_lossy(&output.stderr);
           if stderr.contains("not currently signed in") {
               Ok(false)
           } else {
               Err(OnePasswordError::CliError { stderr: stderr.to_string() })
           }
       }
   }
   ```

4. **Put it all together** (1 hour)
   ```rust
   impl OpCli {
       pub async fn detect() -> Result<Self> {
           let start = Instant::now();
           
           // Step 1: Find the CLI
           let path = find_op_cli()?;
           
           // Step 2: Check version
           let version = get_op_version(&path).await?;
           if version < Version::new(2, 0, 0) {
               return Err(OnePasswordError::VersionTooOld { 
                   found: version,
                   required: Version::new(2, 0, 0),
               });
           }
           
           // Step 3: Check auth
           let authenticated = check_auth_status(&path).await?;
           
           let detection_time = start.elapsed();
           if detection_time > Duration::from_millis(100) {
               warn!("Slow op CLI detection: {:?}", detection_time);
           }
           
           Ok(Self {
               path,
               version,
               authenticated,
           })
       }
   }
   ```

#### Task 4.1.2: Process Execution Framework (8 hours)

💡 **Junior Dev Concept**: Secure Process Execution
**What it is**: Running external commands without leaking secrets
**Key risks**: Credentials in command args, environment variables, logs
**Golden rule**: Never put secrets where they can be seen (ps, logs, etc.)

Build secure command execution:

```rust
pub struct OpCommand {
    cli: Arc<OpCli>,
    args: Vec<String>,
    timeout: Duration,
}

impl OpCommand {
    pub async fn execute(&self) -> Result<OpOutput> {
        let mut cmd = Command::new(&self.cli.path);
        cmd.args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null()); // Never interactive
        
        // Security: No environment variable leakage
        cmd.env_clear();
        cmd.env("PATH", std::env::var("PATH")?);
        
        let output = timeout(self.timeout, cmd.output()).await??;
        
        // Parse and sanitize output
        self.parse_output(output)
    }
}
```

**Security Requirements**:
- Clear environment variables
- No credential logging
- Timeout all operations
- Sanitize error messages

**Step-by-Step Implementation**:

1. **Create the command builder** (2 hours)
   ```rust
   pub struct OpCommandBuilder {
       cli: Arc<OpCli>,
       args: Vec<String>,
       timeout: Duration,
       env_vars: HashMap<String, String>,
   }
   
   impl OpCommandBuilder {
       pub fn new(cli: Arc<OpCli>) -> Self {
           Self {
               cli,
               args: Vec::new(),
               timeout: Duration::from_secs(30),
               env_vars: HashMap::new(),
           }
       }
       
       pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
           self.args.push(arg.into());
           self
       }
       
       pub fn args<I, S>(mut self, args: I) -> Self 
       where
           I: IntoIterator<Item = S>,
           S: Into<String>,
       {
           self.args.extend(args.into_iter().map(Into::into));
           self
       }
       
       pub fn timeout(mut self, timeout: Duration) -> Self {
           self.timeout = timeout;
           self
       }
       
       pub fn build(self) -> OpCommand {
           OpCommand {
               cli: self.cli,
               args: self.args,
               timeout: self.timeout,
           }
       }
   }
   ```
   
   💡 **Builder Pattern**: Makes it easy to construct commands safely

2. **Implement secure execution** (3 hours)
   ```rust
   impl OpCommand {
       pub async fn execute(&self) -> Result<OpOutput> {
           // Log command (but never log sensitive args!)
           debug!("Executing op command: op {}", self.safe_args_string());
           
           let mut cmd = Command::new(&self.cli.path);
           cmd.args(&self.args)
               .stdout(Stdio::piped())
               .stderr(Stdio::piped())
               .stdin(Stdio::null())  // Never interactive
               .kill_on_drop(true);   // Clean up if we're dropped
           
           // Security: Clear environment
           cmd.env_clear();
           
           // Only pass through safe environment variables
           let safe_env_vars = ["PATH", "HOME", "LANG", "LC_ALL"];
           for var in &safe_env_vars {
               if let Ok(value) = std::env::var(var) {
                   cmd.env(var, value);
               }
           }
           
           // Add 1Password-specific env vars if needed
           if let Ok(token) = std::env::var("OP_SERVICE_ACCOUNT_TOKEN") {
               // Service account mode
               cmd.env("OP_SERVICE_ACCOUNT_TOKEN", token);
           }
           
           // Execute with timeout
           let child = cmd.spawn()
               .map_err(|e| OnePasswordError::CliExecution(e))?;
           
           match timeout(self.timeout, child.wait_with_output()).await {
               Ok(Ok(output)) => self.parse_output(output),
               Ok(Err(e)) => Err(OnePasswordError::CliExecution(e)),
               Err(_) => Err(OnePasswordError::Timeout(self.timeout)),
           }
       }
       
       fn safe_args_string(&self) -> String {
           // Never log sensitive arguments
           self.args.iter()
               .map(|arg| {
                   if arg.starts_with("op://") || arg.contains("password") {
                       "<redacted>"
                   } else {
                       arg.as_str()
                   }
               })
               .collect::<Vec<_>>()
               .join(" ")
       }
   }
   ```
   
   ⚠️ **Security Critical**: Never log passwords or secrets!

3. **Parse output safely** (2 hours)
   ```rust
   fn parse_output(&self, output: Output) -> Result<OpOutput> {
       let stdout = String::from_utf8_lossy(&output.stdout).to_string();
       let stderr = String::from_utf8_lossy(&output.stderr).to_string();
       
       // Sanitize any accidental credential leaks
       let stdout = self.sanitize_output(stdout);
       let stderr = self.sanitize_output(stderr);
       
       if output.status.success() {
           Ok(OpOutput {
               stdout,
               stderr,
               exit_code: 0,
           })
       } else {
           // Parse known error patterns
           let error = self.parse_error(&stderr)?;
           Err(error)
       }
   }
   
   fn sanitize_output(&self, output: String) -> String {
       // Remove any potential secrets
       let patterns = [
           (r"password\s*:\s*\S+", "password: <redacted>"),
           (r"token\s*:\s*\S+", "token: <redacted>"),
           (r"BEGIN [A-Z]+ PRIVATE KEY[\s\S]+END [A-Z]+ PRIVATE KEY", "<private key redacted>"),
       ];
       
       let mut sanitized = output;
       for (pattern, replacement) in &patterns {
           let re = Regex::new(pattern).unwrap();
           sanitized = re.replace_all(&sanitized, *replacement).to_string();
       }
       
       sanitized
   }
   ```

4. **Add retry logic** (1 hour)
   ```rust
   pub async fn execute_with_retry(&self, max_retries: u32) -> Result<OpOutput> {
       let mut attempts = 0;
       let mut last_error = None;
       
       while attempts <= max_retries {
           match self.execute().await {
               Ok(output) => return Ok(output),
               Err(e) => {
                   attempts += 1;
                   last_error = Some(e);
                   
                   if attempts <= max_retries {
                       // Exponential backoff
                       let delay = Duration::from_millis(100 * 2u64.pow(attempts));
                       tokio::time::sleep(delay).await;
                   }
               }
           }
       }
       
       Err(last_error.unwrap())
   }
   ```

#### Task 4.1.3: Authentication Management (5 hours)

Handle authentication flows:

```rust
pub enum AuthMethod {
    Biometric,
    Password(SensitiveString),
    EnvironmentAuth, // OP_SERVICE_ACCOUNT_TOKEN
}

impl OpCli {
    pub async fn authenticate(&mut self, method: AuthMethod) -> Result<()> {
        match method {
            AuthMethod::Biometric => {
                // Trigger biometric prompt
                self.run_command(&["signin", "--raw"]).await?;
            }
            AuthMethod::Password(pwd) => {
                // Use expect for password
                self.run_with_stdin(&["signin"], pwd.as_bytes()).await?;
            }
            AuthMethod::EnvironmentAuth => {
                // Verify env auth working
                self.run_command(&["whoami"]).await?;
            }
        }
        self.authenticated = true;
        Ok(())
    }
}
```

#### Task 4.1.4: Error Handling & Logging (2 hours)

Comprehensive error handling:
- Parse op CLI error formats
- Distinguish auth errors from others
- Secure logging (no secrets)
- Helpful error messages

---

## 🛑 CHECKPOINT 1: CLI Wrapper Foundation Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without security review.

### Pre-Checkpoint Checklist

- [ ] op CLI detection working on all platforms
- [ ] Version checking accurate and robust
- [ ] Command execution never leaks credentials
- [ ] All error messages sanitized
- [ ] Timeout handling tested
- [ ] Authentication status detection working
- [ ] No sensitive data in logs

### Security Verification

```bash
# Test credential safety
cargo test --test security_cli_wrapper

# Check for credential leaks
grep -r "password\|token\|secret" logs/

# Verify process cleanup
ps aux | grep "op " # Should be empty after tests
```

### Review Requirements

#### Security Review (Security Engineer)
- [ ] Environment variables cleared
- [ ] No credentials in command arguments
- [ ] Output sanitization working
- [ ] Process cleanup verified

#### Code Review (Senior Dev)
- [ ] Error handling comprehensive
- [ ] Async/await used correctly
- [ ] Resource cleanup guaranteed
- [ ] API design extensible

### Consequences of Skipping

- Credential leaks in production
- Security vulnerabilities
- Complete security audit required
- Potential data breach liability

---

### 4.2 Vault and Item Management (25 hours)

**Complexity**: Medium - API design critical
**Files**: `src/onepassword/vault.rs`, `src/onepassword/item.rs`, `src/onepassword/models.rs`

#### Task 4.2.1: Data Models (5 hours)

💡 **Junior Dev Concept**: Modeling External APIs
**What it is**: Creating Rust structs that match 1Password's JSON responses
**Key tool**: Serde for automatic JSON deserialization
**Pro tip**: Start with the actual JSON, then build structs to match

Define 1Password data structures:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: String,
    pub name: String,
    pub vault_type: VaultType,
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub vault: VaultReference,
    pub category: Category,
    pub urls: Vec<Url>,
    #[serde(skip)]
    pub fields: Vec<Field>, // Loaded separately
}

#[derive(Debug, Clone)]
pub enum Category {
    Login,
    SshKey,
    Document,
    Custom(String),
}
```

#### Task 4.2.2: Vault Operations (8 hours)

💡 **Junior Dev Concept**: Working with External APIs
**Pattern**: Command → JSON → Rust Struct
**Error handling**: External APIs can fail in many ways
**Testing**: Mock the CLI responses for unit tests

Implement vault browsing:

```rust
impl OpCli {
    pub async fn list_vaults(&self) -> Result<Vec<Vault>> {
        let output = self.run_command(&[
            "vault", "list", 
            "--format", "json"
        ]).await?;
        
        let vaults: Vec<Vault> = serde_json::from_str(&output)?;
        Ok(vaults)
    }
    
    pub async fn get_vault(&self, id: &str) -> Result<Vault> {
        let output = self.run_command(&[
            "vault", "get", id,
            "--format", "json"
        ]).await?;
        
        serde_json::from_str(&output)
            .map_err(Into::into)
    }
}
```

**Step-by-Step Implementation**:

1. **Understand the JSON structure** (1 hour)
   ```bash
   # First, see what op returns
   op vault list --format json | jq .
   
   # Sample output:
   # [
   #   {
   #     "id": "abc123",
   #     "name": "Personal",
   #     "type": "PERSONAL",
   #     "created_at": "2023-01-01T00:00:00Z"
   #   }
   # ]
   ```
   
   💡 **Always**: Check real API responses before coding

2. **Create data models** (2 hours)
   ```rust
   use chrono::{DateTime, Utc};
   use serde::{Deserialize, Serialize};
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Vault {
       pub id: String,
       pub name: String,
       
       #[serde(rename = "type")]
       pub vault_type: VaultType,
       
       #[serde(rename = "created_at")]
       pub created_at: DateTime<Utc>,
       
       // Optional fields that might not always be present
       #[serde(skip_serializing_if = "Option::is_none")]
       pub description: Option<String>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
   pub enum VaultType {
       Personal,
       Shared,
       Private,
       #[serde(other)]
       Unknown,
   }
   ```
   
   ⚠️ **Common Mistake**: Not handling unknown enum variants
   ✅ **Solution**: Use #[serde(other)] for forward compatibility

3. **Implement vault operations** (3 hours)
   ```rust
   impl OpCli {
       pub async fn list_vaults(&self) -> Result<Vec<Vault>> {
           let cmd = OpCommandBuilder::new(self.clone())
               .args(["vault", "list", "--format", "json"])
               .timeout(Duration::from_secs(10))
               .build();
           
           let output = cmd.execute().await?;
           
           // Parse JSON response
           let vaults: Vec<Vault> = serde_json::from_str(&output.stdout)
               .map_err(|e| OnePasswordError::JsonParse {
                   context: "vault list".to_string(),
                   error: e,
               })?;
           
           debug!("Found {} vaults", vaults.len());
           Ok(vaults)
       }
       
       pub async fn get_vault(&self, vault_id: &str) -> Result<Vault> {
           // Validate input
           if vault_id.is_empty() {
               return Err(OnePasswordError::InvalidInput("Vault ID cannot be empty"));
           }
           
           let cmd = OpCommandBuilder::new(self.clone())
               .args(["vault", "get", vault_id, "--format", "json"])
               .timeout(Duration::from_secs(5))
               .build();
           
           let output = cmd.execute().await?;
           
           serde_json::from_str(&output.stdout)
               .map_err(|e| OnePasswordError::JsonParse {
                   context: format!("vault get {}", vault_id),
                   error: e,
               })
       }
       
       pub async fn get_vault_by_name(&self, name: &str) -> Result<Option<Vault>> {
           let vaults = self.list_vaults().await?;
           Ok(vaults.into_iter().find(|v| v.name == name))
       }
   }
   ```

4. **Add caching for performance** (2 hours)
   ```rust
   use std::sync::RwLock;
   use std::time::{Duration, Instant};
   
   pub struct CachedOpCli {
       inner: OpCli,
       vault_cache: RwLock<Option<CachedData<Vec<Vault>>>>,
   }
   
   struct CachedData<T> {
       data: T,
       fetched_at: Instant,
       ttl: Duration,
   }
   
   impl CachedOpCli {
       pub async fn list_vaults(&self) -> Result<Vec<Vault>> {
           // Check cache first
           if let Some(cached) = self.vault_cache.read().unwrap().as_ref() {
               if cached.fetched_at.elapsed() < cached.ttl {
                   return Ok(cached.data.clone());
               }
           }
           
           // Fetch fresh data
           let vaults = self.inner.list_vaults().await?;
           
           // Update cache
           *self.vault_cache.write().unwrap() = Some(CachedData {
               data: vaults.clone(),
               fetched_at: Instant::now(),
               ttl: Duration::from_secs(300), // 5 minutes
           });
           
           Ok(vaults)
       }
   }
   ```

#### Task 4.2.3: Item Discovery (8 hours)

List and filter items efficiently:

```rust
pub struct ItemQuery {
    pub vault: Option<String>,
    pub category: Option<Category>,
    pub tags: Vec<String>,
}

impl OpCli {
    pub async fn list_items(&self, query: ItemQuery) -> Result<Vec<Item>> {
        let mut args = vec!["item", "list", "--format", "json"];
        
        if let Some(vault) = &query.vault {
            args.push("--vault");
            args.push(vault);
        }
        
        if let Some(category) = &query.category {
            args.push("--categories");
            args.push(category.as_str());
        }
        
        let output = self.run_command(&args).await?;
        let items: Vec<Item> = serde_json::from_str(&output)?;
        
        // Client-side tag filtering if needed
        Ok(self.filter_by_tags(items, &query.tags))
    }
}
```

#### Task 4.2.4: Item Detail Loading (4 hours)

Load full item details on demand:

```rust
impl OpCli {
    pub async fn get_item(&self, id: &str) -> Result<ItemDetails> {
        // Load without revealing fields
        let output = self.run_command(&[
            "item", "get", id,
            "--format", "json",
            "--reveal" // Only when needed
        ]).await?;
        
        let mut details: ItemDetails = serde_json::from_str(&output)?;
        
        // Immediately wrap sensitive fields
        details.secure_fields();
        
        Ok(details)
    }
}
```

---

## 🛑 CHECKPOINT 2: Vault Management Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without API review.

### Pre-Checkpoint Checklist

- [ ] Vault operations tested with real 1Password
- [ ] JSON parsing handles all field variations
- [ ] Performance <2s for 100 items
- [ ] Error messages helpful and specific
- [ ] Caching implemented and tested
- [ ] No credential data exposed

### API Testing

```rust
#[tokio::test]
async fn test_vault_operations() {
    let cli = OpCli::detect().await.unwrap();
    
    // List vaults
    let vaults = cli.list_vaults().await.unwrap();
    assert!(!vaults.is_empty());
    
    // Get specific vault
    let vault = cli.get_vault(&vaults[0].id).await.unwrap();
    assert_eq!(vault.id, vaults[0].id);
    
    // Performance test
    let start = Instant::now();
    let _ = cli.list_items(ItemQuery::default()).await.unwrap();
    assert!(start.elapsed() < Duration::from_secs(2));
}
```

### Review Criteria

#### API Design (Tech Lead)
- [ ] Intuitive method names
- [ ] Consistent error types
- [ ] Async/await patterns correct
- [ ] Extensible for future needs

#### Performance (Senior Dev)
- [ ] Caching strategy sound
- [ ] No unnecessary API calls
- [ ] Memory usage reasonable
- [ ] Concurrent operations safe

### Consequences of Skipping

- Poor API design affects all users
- Performance issues compound
- Security vulnerabilities missed
- Major refactoring needed later

---

### 4.3 SSH Key Integration (20 hours)

**Complexity**: High - Critical feature
**Files**: `src/onepassword/ssh.rs`, `src/onepassword/keys.rs`

#### Task 4.3.1: SSH Key Discovery (6 hours)

💡 **Junior Dev Concept**: SSH Key Management
**What it is**: SSH keys are used for authentication and signing
**In 1Password**: Keys stored securely, accessed via op:// references
**Our goal**: Find keys, extract metadata, never touch private keys

Find and parse SSH keys:

```rust
#[derive(Debug, Clone)]
pub struct SshKey {
    pub id: String,
    pub title: String,
    pub vault: String,
    pub fingerprint: String,
    pub key_type: SshKeyType,
    pub public_key: String,
    pub op_reference: String, // op://vault/item/field
}

impl OpCli {
    pub async fn list_ssh_keys(&self) -> Result<Vec<SshKey>> {
        // Get all SSH key items
        let items = self.list_items(ItemQuery {
            category: Some(Category::SshKey),
            ..Default::default()
        }).await?;
        
        // Convert to SshKey with metadata
        let mut keys = Vec::new();
        for item in items {
            if let Ok(key) = self.parse_ssh_key(item).await {
                keys.push(key);
            }
        }
        
        Ok(keys)
    }
}
```

#### Task 4.3.2: Key Metadata Extraction (6 hours)

💡 **Junior Dev Concept**: Safe Key Handling
**Golden rule**: Never load private keys into memory
**Instead**: Use op:// references that 1Password resolves
**Metadata only**: We only need public key, fingerprint, type

Extract key information safely:

```rust
impl OpCli {
    async fn parse_ssh_key(&self, item: Item) -> Result<SshKey> {
        // Get public key without private
        let details = self.get_item_field(&item.id, "public_key").await?;
        
        // Parse key type and fingerprint
        let (key_type, fingerprint) = parse_public_key(&details)?;
        
        // Build op:// reference
        let op_reference = format!(
            "op://{}/{}/private_key",
            item.vault.name,
            item.title
        );
        
        Ok(SshKey {
            id: item.id,
            title: item.title,
            vault: item.vault.name,
            fingerprint,
            key_type,
            public_key: details,
            op_reference,
        })
    }
}
```

**Step-by-Step Implementation**:

1. **Understand SSH key formats** (1 hour)
   ```rust
   // SSH public keys look like:
   // ssh-rsa AAAAB3NzaC1yc2EA... user@host
   // ssh-ed25519 AAAAC3NzaC1lZDI1NTE5... user@host
   
   #[derive(Debug, Clone, PartialEq)]
   pub enum SshKeyType {
       Rsa,
       Ed25519,
       Ecdsa,
       Dsa,  // Deprecated but still found
   }
   
   impl SshKeyType {
       fn from_string(s: &str) -> Option<Self> {
           match s {
               "ssh-rsa" => Some(Self::Rsa),
               "ssh-ed25519" => Some(Self::Ed25519),
               "ecdsa-sha2-nistp256" | "ecdsa-sha2-nistp384" | "ecdsa-sha2-nistp521" => Some(Self::Ecdsa),
               "ssh-dss" => Some(Self::Dsa),
               _ => None,
           }
       }
   }
   ```

2. **Parse public keys safely** (2 hours)
   ```rust
   use base64::{Engine as _, engine::general_purpose};
   use sha2::{Sha256, Digest};
   
   pub fn parse_public_key(public_key: &str) -> Result<(SshKeyType, String)> {
       let parts: Vec<&str> = public_key.split_whitespace().collect();
       
       if parts.len() < 2 {
           return Err(OnePasswordError::InvalidKeyFormat("Missing key type or data"));
       }
       
       // Parse key type
       let key_type = SshKeyType::from_string(parts[0])
           .ok_or_else(|| OnePasswordError::InvalidKeyFormat("Unknown key type"))?;
       
       // Decode key data
       let key_data = general_purpose::STANDARD
           .decode(parts[1])
           .map_err(|_| OnePasswordError::InvalidKeyFormat("Invalid base64"))?;
       
       // Calculate fingerprint (SHA256)
       let mut hasher = Sha256::new();
       hasher.update(&key_data);
       let hash = hasher.finalize();
       
       // Format as SSH fingerprint
       let fingerprint = format!("SHA256:{}", 
           general_purpose::STANDARD_NO_PAD.encode(hash)
       );
       
       Ok((key_type, fingerprint))
   }
   ```
   
   ⚠️ **Common Mistake**: Not handling all key formats
   ✅ **Solution**: Support RSA, Ed25519, ECDSA at minimum

3. **Get item fields securely** (2 hours)
   ```rust
   impl OpCli {
       async fn get_item_field(&self, item_id: &str, field_name: &str) -> Result<String> {
           // Use op item get with specific field
           let cmd = OpCommandBuilder::new(self.clone())
               .args([
                   "item", "get", item_id,
                   "--fields", &format!("label={}", field_name),
                   "--format", "json"
               ])
               .timeout(Duration::from_secs(5))
               .build();
           
           let output = cmd.execute().await?;
           
           // Parse field value from JSON
           let field_data: serde_json::Value = serde_json::from_str(&output.stdout)?;
           
           field_data["value"].as_str()
               .ok_or_else(|| OnePasswordError::FieldNotFound {
                   item_id: item_id.to_string(),
                   field: field_name.to_string(),
               })
               .map(|s| s.to_string())
       }
       
       // Never use this for private keys!
       async fn get_public_key_only(&self, item_id: &str) -> Result<String> {
           // Some items might have the public key in different fields
           let field_names = ["public_key", "public key", "publicKey"];
           
           for field in &field_names {
               match self.get_item_field(item_id, field).await {
                   Ok(key) => return Ok(key),
                   Err(_) => continue,
               }
           }
           
           Err(OnePasswordError::PublicKeyNotFound(item_id.to_string()))
       }
   }
   ```

4. **Build op:// references** (1 hour)
   ```rust
   /// Build op:// reference for SSH key
   /// Format: op://vault/item/field
   pub fn build_op_reference(vault: &str, item: &str, field: &str) -> String {
       // Handle special characters in names
       let vault = urlencoding::encode(vault);
       let item = urlencoding::encode(item);
       let field = urlencoding::encode(field);
       
       format!("op://{}/{}/{}", vault, item, field)
   }
   
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_op_reference_encoding() {
           let reference = build_op_reference(
               "My Vault",
               "SSH Key (Work)",
               "private key"
           );
           
           assert_eq!(
               reference, 
               "op://My%20Vault/SSH%20Key%20%28Work%29/private%20key"
           );
       }
   }
   ```

#### Task 4.3.3: Git Configuration Integration (6 hours)

Configure Git to use 1Password keys:

```rust
pub fn configure_git_signing(key: &SshKey) -> Result<GitConfig> {
    let config = GitConfig {
        user_signingkey: key.op_reference.clone(),
        gpg_format: "ssh".to_string(),
        gpg_ssh_program: Some("op-ssh-sign".to_string()),
        // Additional SSH signing config
    };
    
    Ok(config)
}
```

#### Task 4.3.4: Key Selection UI Integration (2 hours)

Integrate with TUI from Phase 3:
- Provide key list to TUI
- Show key metadata in selection
- Handle async loading
- Cache results appropriately

---

## 🛑 CHECKPOINT 3: SSH Key Integration Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** without security verification.

### Pre-Checkpoint Checklist

- [ ] SSH key discovery finds all key types
- [ ] Public keys parsed correctly
- [ ] Fingerprints match ssh-keygen output
- [ ] op:// references properly encoded
- [ ] NO private keys in memory/logs
- [ ] Git signing configuration tested

### Security Verification

```bash
# Verify no private keys in memory
cargo test --test ssh_key_security

# Check logs for key material
grep -E "BEGIN.*PRIVATE KEY" logs/ # Should be empty

# Test op:// reference
echo "test" | op run --no-masking -- ssh-keygen -Y sign -f "op://Personal/SSH Key/private key" -n git
```

### Review Requirements

#### Security (Security Engineer)
- [ ] No private key exposure
- [ ] op:// references secure
- [ ] Public key only operations
- [ ] Fingerprint verification correct

#### Integration (Senior Dev)
- [ ] Works with all SSH key types
- [ ] Git configuration correct
- [ ] Performance acceptable
- [ ] Error handling comprehensive

### Consequences of Skipping

- Private key exposure risk
- Git signing failures
- Security audit failure
- User trust destroyed

---

### 4.4 Advanced Features & Polish (15 hours)

**Complexity**: Medium - Enhancement phase
**Files**: `src/onepassword/gpg.rs`, `src/onepassword/cache.rs`

#### Task 4.4.1: Custom GPG Key Support (6 hours)

💡 **Junior Dev Concept**: GPG Key Handling
**What it is**: GPG keys for signing/encryption (more complex than SSH)
**Challenge**: 1Password stores GPG in various formats
**Approach**: Parse multiple formats, extract only public data

Handle 1Password's GPG storage format:

```rust
#[derive(Debug, Deserialize)]
pub struct GpgKeyCustom {
    #[serde(rename = "Private Key")]
    private_key: String, // Never store!
    #[serde(rename = "Public Key")]
    public_key: String,
    #[serde(rename = "Key ID")]
    key_id: String,
    fingerprint: Option<String>,
}

impl OpCli {
    pub async fn get_gpg_key(&self, item_id: &str) -> Result<GpgKey> {
        // Special handling for GPG stored as secure note
        let item = self.get_item(item_id).await?;
        
        if let Some(notes) = item.notes {
            // Try to parse as JSON
            if let Ok(custom) = serde_json::from_str::<GpgKeyCustom>(&notes) {
                return Ok(self.convert_custom_gpg(custom));
            }
        }
        
        // Fallback to standard field parsing
        self.parse_standard_gpg(item).await
    }
}
```

#### Task 4.4.2: Response Caching (4 hours)

Implement secure, time-limited caching:

```rust
pub struct OpCache {
    vaults: TimedCache<Vec<Vault>>,
    items: TimedCache<String, Vec<Item>>,
    ttl: Duration,
}

impl OpCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            vaults: TimedCache::new(ttl),
            items: TimedCache::new(ttl),
            ttl,
        }
    }
    
    pub async fn get_or_fetch_vaults<F>(&self, fetch: F) -> Result<Vec<Vault>>
    where
        F: Future<Output = Result<Vec<Vault>>>,
    {
        if let Some(cached) = self.vaults.get() {
            return Ok(cached);
        }
        
        let vaults = fetch.await?;
        self.vaults.set(vaults.clone());
        Ok(vaults)
    }
}
```

#### Task 4.4.3: Performance Optimization (3 hours)

Optimize for responsive UI:
- Parallel vault queries
- Lazy item detail loading
- Progress callbacks for long operations
- Cancelable operations

#### Task 4.4.4: Documentation & Examples (2 hours)

Create comprehensive docs:
- 1Password setup guide
- Troubleshooting common issues
- Example configurations
- Security best practices

---

## 🛑 CHECKPOINT 4: Phase 4 Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** to Phase 5 without final security audit.

### Pre-Checkpoint Checklist

- [ ] All 1Password operations tested
- [ ] Performance <2s for all operations
- [ ] Security audit passed
- [ ] No credential leaks verified
- [ ] Biometric auth working (where available)
- [ ] Documentation complete
- [ ] Integration tests passing

### Final Security Audit

```bash
# Run security test suite
cargo test --test security_audit --features onepassword

# Memory leak detection
valgrind --leak-check=full target/release/git-setup

# Check for secrets in binary
strings target/release/git-setup | grep -E "password|secret|token"
```

### Phase 4 Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| op Detection | <100ms | ___ | ⬜ |
| Vault List | <500ms | ___ | ⬜ |
| Key Discovery | <1s | ___ | ⬜ |
| Memory Leaks | 0 | ___ | ⬜ |
| Security Issues | 0 | ___ | ⬜ |

### Integration Testing

**Test Scenarios**:
1. Fresh 1Password install
2. Multiple vaults with 100+ items
3. Biometric and password auth
4. Network interruptions
5. op CLI not installed

### Sign-offs Required

- [ ] **Security Lead**: No vulnerabilities found
- [ ] **1Password Team**: Integration approved
- [ ] **Tech Lead**: Code quality acceptable
- [ ] **QA Lead**: Testing complete
- [ ] **Product Owner**: Features working

### Handoff to Phase 5

**What Phase 5 needs**:
1. Stable key discovery API
2. Fast item searching
3. Secure credential handling patterns
4. Mock 1Password for testing

---

## Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        OpCliExecutor {
            async fn execute(&self, args: &[&str]) -> Result<String>;
        }
    }
    
    #[tokio::test]
    async fn test_list_vaults() {
        let mut mock = MockOpCliExecutor::new();
        mock.expect_execute()
            .with(eq(&["vault", "list", "--format", "json"]))
            .returning(|_| Ok(SAMPLE_VAULTS_JSON.to_string()));
        
        // Test vault parsing
    }
}
```

### Integration Testing
- Optional: Test with real 1Password
- Use test vault with known items
- Verify op:// references work
- Test auth timeout scenarios

### Security Testing
- Verify no credentials in memory
- Check process environment cleared
- Validate error message sanitization
- Test timeout enforcement

## Common Issues & Solutions

### Issue: "op: command not found"
**Symptom**: OpCli::detect() fails with CliNotFound
**Cause**: 1Password CLI not installed or not in PATH
**Solution**:
```rust
// Provide helpful error message
match OpCli::detect().await {
    Err(OnePasswordError::CliNotFound) => {
        eprintln!("1Password CLI not found!");
        eprintln!("Install from: https://developer.1password.com/docs/cli/get-started/");
        eprintln!("Or add to PATH if already installed");
    }
    // ...
}
```

### Issue: Authentication Timeout
**Symptom**: Commands fail with "not signed in"
**Cause**: 1Password session expired (30 min default)
**Solution**:
```rust
// Auto-retry with re-auth
loop {
    match cli.list_vaults().await {
        Ok(vaults) => return Ok(vaults),
        Err(OnePasswordError::NotAuthenticated) => {
            cli.authenticate(AuthMethod::Biometric).await?;
        }
        Err(e) => return Err(e),
    }
}
```

### Issue: Biometric Prompt Not Appearing
**Symptom**: Auth hangs or fails immediately
**Cause**: Terminal environment doesn't support Touch ID
**Solution**:
```rust
// Detect and fallback
let auth_method = if supports_biometric() {
    AuthMethod::Biometric
} else {
    println!("Touch ID not available in this terminal");
    println!("Please enter your 1Password password:");
    let password = rpassword::prompt_password("> ")?;
    AuthMethod::Password(SensitiveString::new(password))
};
```

### Issue: JSON Parse Errors
**Symptom**: "error parsing item: missing field 'vault'"
**Cause**: 1Password CLI output format changed
**Solution**:
```rust
// Use permissive parsing
#[derive(Deserialize)]
#[serde(deny_unknown_fields = false)]  // Allow new fields
pub struct Item {
    pub id: String,
    pub title: String,
    
    #[serde(default)]  // Handle missing fields
    pub vault: VaultReference,
}
```

### Issue: Performance Degradation
**Symptom**: Commands taking >5 seconds
**Cause**: No caching, repeated API calls
**Solution**: Implement caching (already shown in code)

## Performance Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| op CLI Detection | <100ms | <200ms |
| List Vaults | <500ms | <2s |
| List Items (100) | <1s | <2s |
| Get Item Details | <200ms | <500ms |
| Biometric Auth | <500ms | <2s |

## Security Considerations

- Never store private keys
- Use SensitiveString for passwords
- Clear process environment
- Timeout all operations
- Sanitize all error messages
- No credential logging
- Secure cache expiration

## Junior Developer Tips

### Getting Started with Phase 4

1. **Setup for development**:
   - Install 1Password CLI locally
   - Create test vault with dummy SSH keys
   - Never use production credentials for testing
   - Use `op inject` for local testing

2. **Understanding op:// references**:
   ```bash
   # See how op:// works
   echo "op://Personal/GitHub SSH/private key" | op inject
   # (Don't run with real keys!)
   ```

3. **Debugging external commands**:
   ```rust
   // Log commands for debugging
   std::env::set_var("RUST_LOG", "onepassword=debug");
   env_logger::init();
   ```

4. **Security mindset**:
   - Always ask: "Could this leak credentials?"
   - When in doubt, don't log it
   - Test with fake data first
   - Review security checklist before commits

### Common Pitfalls

1. **Logging sensitive data**: Always sanitize
2. **Storing credentials**: Use op:// references only
3. **Timeout handling**: External commands can hang
4. **Error messages**: May contain secrets

## Next Phase Preview

Phase 5 (Advanced Features) will add:
- Automatic profile detection using patterns
- Multi-method signing configuration
- Health check system
- Remote profile import with security

**What Phase 5 needs from Phase 4**:
- Key discovery API for signing setup
- Secure patterns for external commands
- Performance baselines
- Error handling patterns

---

*Last updated: 2025-07-30*