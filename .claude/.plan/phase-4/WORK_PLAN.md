# Phase 4: 1Password Integration - Work Plan

## Prerequisites

Before starting Phase 4, understand that we're integrating with an external security tool - precision is critical.

**Required from Previous Phases**:
- ✅ Secure memory handling (Phase 1 - SensitiveString)
- ✅ Profile storage system (Phase 2)
- ✅ UI for credential selection (Phase 3)
- ✅ Error handling patterns established

**Required Knowledge**:
- **Process Execution**: Running external commands safely (*critical*)
- **JSON Parsing**: Handling 1Password's output (*required*)
- **Security Principles**: Never log secrets (*critical*)
- **Error Recovery**: External tools can fail (*important*)

💡 **Junior Dev Resources**:
- 📚 [1Password CLI Basics](https://support.1password.com/command-line-getting-started/) - Start here! (30 min)
- 📖 [Rust Process Management](https://doc.rust-lang.org/std/process/index.html) - Official guide
- 📖 [Secure Coding Guide](https://wiki.sei.cmu.edu/confluence/display/seccode) - Read security section
- 🔧 Practice: Try `examples/1password/mock_integration.rs` first
- 📝 [Security Checklist](../../resources/security-checklist.md) - Keep handy
- 🧪 [Mock Testing Guide](MOCK_TESTING_GUIDE.md) - **Complete mock implementation patterns**
- 🔐 [Security Implementation Examples](../SECURITY_IMPLEMENTATION_EXAMPLES.md) - Secure external calls
- ⚡ [Async/Await Examples](../ASYNC_AWAIT_EXAMPLES.md) - For async 1Password operations

## Quick Reference - Essential Resources

### 1Password CLI Commands to Know
```bash
# Check if authenticated
op account list

# List SSH keys
op item list --categories "SSH Key" --format json

# Get specific item (DO NOT log output!)
op item get "GitHub SSH Key" --format json

# Read a secret reference
op read "op://Personal/GitHub SSH Key/private key"
```

### Security Golden Rules
1. **NEVER** log credential values
2. **NEVER** pass secrets as command arguments
3. **ALWAYS** use SensitiveString for credential data
4. **ALWAYS** clear credentials from memory after use
5. **NEVER** store op:// references with actual values

## Overview

Phase 4 integrates 1Password CLI to provide secure credential management. Users can reference credentials without git-setup-rs ever storing the actual secret values.

### Visual Integration Overview

```
┌─────────────────────────────────────────────────────────────┐
│                  1Password Integration Flow                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  git-setup Profile              1Password Vault            │
│  ─────────────────              ───────────────            │
│                                                             │
│  signing.ssh_key: ──────►  Fetch via CLI  ──────►  SSH Key │
│    "op://vault/ssh/key"         (op)              (actual) │
│                                  │                          │
│  git.password: ─────────►       ▼                 Password │
│    "op://vault/git/pat"    Biometric Auth        (actual)  │
│                                  │                          │
│                                  ▼                          │
│                             Secure Return                   │
│                                  │                          │
│  Profile uses secret ◄───────────┘                         │
│  (never stored locally)                                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Authentication Flow

```
┌──── 1Password Authentication Process ─────┐
│                                           │
│  1. User runs git-setup                   │
│     └─► Needs credential from 1Password   │
│                                           │
│  2. Check auth status                     │
│     └─► op whoami                         │
│           ├─► Authenticated: Continue     │
│           └─► Not auth: Prompt signin     │
│                                           │
│  3. Biometric prompt                      │
│     └─► Touch ID / Windows Hello         │
│                                           │
│  4. Session established                   │
│     └─► Valid for 30 minutes             │
│                                           │
│  5. Fetch secrets as needed              │
│     └─► Each fetch may require touch     │
│                                           │
└───────────────────────────────────────────┘
```

**What You'll Build**:
1. Safe wrapper around 1Password CLI
2. Credential discovery and listing
3. Biometric authentication support
4. Secure credential retrieval
5. Graceful degradation without 1Password

**Success Looks Like**:
- User's SSH keys stay in 1Password
- Git-setup only stores references like `op://vault/item/field`
- Biometric unlock works seamlessly
- Zero credential leaks

**Time Estimate**: 2 weeks (80 hours)
- Week 1: CLI wrapper and basic operations (40h)
- Week 2: Advanced features and security (40h)

## Done Criteria Checklist

Phase 4 is complete when:
- [ ] 1Password CLI detected correctly
- [ ] Credential listing works
- [ ] Biometric auth supported
- [ ] No secrets in logs/memory
- [ ] Graceful degradation
- [ ] Mock testing complete
- [ ] Security review passed
- [ ] Documentation complete

## Week 1: CLI Wrapper Foundation

### 4.1 1Password CLI Wrapper (32 hours)

#### Task 4.1.1: CLI Detection and Setup (8 hours)

💡 **Junior Dev Concept**: External Tool Integration
**What it is**: Using another program (1Password CLI) from our Rust code
**Why carefully**: Security tools require extra care - one mistake leaks credentials
**Real Example**: Like how VS Code uses Git - it runs git commands for you

**Prerequisites**:
- [ ] Install 1Password CLI (for testing)
- [ ] Read: [std::process::Command docs](https://doc.rust-lang.org/std/process/struct.Command.html)
- [ ] Understand: How PATH works on your OS

**Visual Architecture**:
```
git-setup-rs                    1Password CLI (op)
────────────                    ─────────────────
   │                                    │
   ├─ "op --version" ──────────────────>│
   │<────────────────── "2.24.0" ───────┤
   │                                    │
   ├─ "op item list" ──────────────────>│
   │<──────────────── JSON items ──────┤
   │                                    │
   └─ Never stores actual secrets!      └─ Holds the secrets
```

**Step-by-Step Implementation**:

1. **Create 1Password Module Structure** (1 hour)
   ```rust
   // src/onepassword/mod.rs
   
   pub mod cli;
   pub mod error;
   pub mod types;
   pub mod mock;  // For testing!
   
   pub use cli::OnePasswordCli;
   pub use error::OnePasswordError;
   pub use types::{Vault, Item, ItemType};
   
   /// Minimum supported version of 1Password CLI
   pub const MIN_OP_VERSION: &str = "2.0.0";
   ```

2. **Define Error Types** (1 hour)
   ```rust
   // src/onepassword/error.rs
   
   use thiserror::Error;
   
   #[derive(Debug, Error)]
   pub enum OnePasswordError {
       #[error("1Password CLI not found. Please install from https://1password.com/downloads/command-line/")]
       CliNotFound,
       
       #[error("1Password CLI version {0} is too old. Minimum required: {}", crate::MIN_OP_VERSION)]
       VersionTooOld(String),
       
       #[error("Not authenticated to 1Password. Run: op signin")]
       NotAuthenticated,
       
       #[error("Command failed: {0}")]
       CommandFailed(String),
       
       #[error("Invalid JSON response: {0}")]
       InvalidJson(String),
       
       #[error("Item not found: {0}")]
       ItemNotFound(String),
       
       #[error("Access denied to vault: {0}")]
       AccessDenied(String),
   }
   ```
   
   💡 **Good Errors**: Tell users exactly what went wrong and how to fix it

3. **Implement CLI Detection** (3 hours)
   ```rust
   // src/onepassword/cli.rs
   
   use std::process::Command;
   use crate::onepassword::error::OnePasswordError;
   
   /// Wrapper around 1Password CLI
   pub struct OnePasswordCli {
       /// Path to op executable
       op_path: String,
       /// Detected version
       version: Option<String>,
   }
   
   impl OnePasswordCli {
       /// Create new instance and detect op CLI
       pub fn new() -> Result<Self, OnePasswordError> {
           // Step 1: Find op in PATH
           let op_path = Self::find_op_executable()?;
           
           // Step 2: Check version
           let version = Self::get_op_version(&op_path)?;
           
           // Step 3: Verify minimum version
           Self::check_version_compatibility(&version)?;
           
           Ok(Self {
               op_path,
               version: Some(version),
           })
       }
       
       /// Find op executable in system PATH
       fn find_op_executable() -> Result<String, OnePasswordError> {
           // Try to run 'op' directly
           match Command::new("op").arg("--version").output() {
               Ok(output) if output.status.success() => {
                   // Found in PATH
                   Ok("op".to_string())
               }
               _ => {
                   // Try common installation paths
                   let common_paths = if cfg!(target_os = "windows") {
                       vec![
                           "C:\\Program Files\\1Password CLI\\op.exe",
                           "C:\\Program Files (x86)\\1Password CLI\\op.exe",
                       ]
                   } else if cfg!(target_os = "macos") {
                       vec![
                           "/usr/local/bin/op",
                           "/opt/homebrew/bin/op",
                           "/usr/bin/op",
                       ]
                   } else {
                       vec![
                           "/usr/local/bin/op",
                           "/usr/bin/op",
                           "/snap/bin/op",
                       ]
                   };
                   
                   for path in common_paths {
                       if std::path::Path::new(path).exists() {
                           return Ok(path.to_string());
                       }
                   }
                   
                   Err(OnePasswordError::CliNotFound)
               }
           }
       }
       
       /// Get version from op CLI
       fn get_op_version(op_path: &str) -> Result<String, OnePasswordError> {
           let output = Command::new(op_path)
               .arg("--version")
               .output()
               .map_err(|e| OnePasswordError::CommandFailed(e.to_string()))?;
           
           if !output.status.success() {
               return Err(OnePasswordError::CommandFailed(
                   "Failed to get version".to_string()
               ));
           }
           
           let version_output = String::from_utf8_lossy(&output.stdout);
           // Output is like "2.24.0\n"
           Ok(version_output.trim().to_string())
       }
       
       /// Check if version meets minimum requirement
       fn check_version_compatibility(version: &str) -> Result<(), OnePasswordError> {
           // Simple version comparison (you might want semver crate for production)
           let version_parts: Vec<u32> = version
               .split('.')
               .filter_map(|s| s.parse().ok())
               .collect();
           
           let min_parts: Vec<u32> = MIN_OP_VERSION
               .split('.')
               .filter_map(|s| s.parse().ok())
               .collect();
           
           if version_parts.len() < 3 || min_parts.len() < 3 {
               return Err(OnePasswordError::VersionTooOld(version.to_string()));
           }
           
           // Compare major.minor.patch
           for i in 0..3 {
               if version_parts[i] < min_parts[i] {
                   return Err(OnePasswordError::VersionTooOld(version.to_string()));
               } else if version_parts[i] > min_parts[i] {
                   break; // Newer version is OK
               }
           }
           
           Ok(())
       }
   }
   ```
   
   ⚠️ **Common Mistake**: Not handling PATH differences across OS
   ✅ **Solution**: Check platform-specific locations

4. **Create Authentication Check** (3 hours)
   ```rust
   impl OnePasswordCli {
       /// Check if user is authenticated
       pub fn is_authenticated(&self) -> Result<bool, OnePasswordError> {
           let output = Command::new(&self.op_path)
               .arg("account")
               .arg("list")
               .arg("--format")
               .arg("json")
               .output()
               .map_err(|e| OnePasswordError::CommandFailed(e.to_string()))?;
           
           // If command succeeds and has output, we're authenticated
           Ok(output.status.success() && !output.stdout.is_empty())
       }
       
       /// Get current account info
       pub fn whoami(&self) -> Result<AccountInfo, OnePasswordError> {
           if !self.is_authenticated()? {
               return Err(OnePasswordError::NotAuthenticated);
           }
           
           let output = Command::new(&self.op_path)
               .arg("whoami")
               .arg("--format")
               .arg("json")
               .output()
               .map_err(|e| OnePasswordError::CommandFailed(e.to_string()))?;
           
           if !output.status.success() {
               return Err(OnePasswordError::NotAuthenticated);
           }
           
           // Parse JSON response
           let account: AccountInfo = serde_json::from_slice(&output.stdout)
               .map_err(|e| OnePasswordError::InvalidJson(e.to_string()))?;
           
           Ok(account)
       }
   }
   
   #[derive(Debug, Deserialize)]
   pub struct AccountInfo {
       pub email: String,
       pub name: Option<String>,
       pub account_uuid: String,
   }
   ```

**Testing Your Implementation**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        assert!(OnePasswordCli::check_version_compatibility("2.24.0").is_ok());
        assert!(OnePasswordCli::check_version_compatibility("2.0.0").is_ok());
        assert!(OnePasswordCli::check_version_compatibility("1.12.0").is_err());
    }
    
    #[test]
    fn test_cli_detection() {
        // This test requires 1Password CLI installed
        if let Ok(cli) = OnePasswordCli::new() {
            assert!(cli.version.is_some());
            println!("Detected version: {:?}", cli.version);
        } else {
            println!("1Password CLI not found - skipping test");
        }
    }
}
```

**Debugging Guide**:

**Error**: "1Password CLI not found"
**Solution**: 
1. Install from https://1password.com/downloads/command-line/
2. Ensure `op` is in PATH: `which op` (Unix) or `where op` (Windows)
3. Restart terminal after installation

**Error**: "Not authenticated"
**Solution**: Run `eval $(op signin)` in terminal first

**Error**: Version check fails
**Solution**: Update 1Password CLI: `op update`

**When You're Stuck**:
1. Test op commands manually first
2. Check examples: `examples/1password/credential_flow.rs`
3. Use `--debug` flag with op for verbose output
4. Ask in Slack: #security-integration

---

### 🛑 CHECKPOINT 4.1: CLI Wrapper Foundation Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 32 hours + 8 hours review = 40 hours total

**Pre-Checkpoint Checklist**:
- [ ] CLI detection works on all platforms
- [ ] Version checking implemented
- [ ] Authentication status detection works
- [ ] Error messages helpful
- [ ] No security vulnerabilities

**Review Focus**:
- Security: No credential leaks
- Cross-platform compatibility
- Error handling quality

---

### 4.2 Credential Discovery (24 hours)

#### Task 4.2.1: Vault and Item Listing (8 hours)

💡 **Junior Dev Concept**: Credential Discovery
**What it is**: Finding available credentials without accessing their values
**Why this way**: Users browse what's available, we never see the actual secrets
**Real Example**: Like file browser showing files without opening them

**Prerequisites**:
- [ ] Understand: 1Password's vault concept
- [ ] Create: Test items in 1Password
- [ ] Review: JSON parsing with serde

**Implementation**:

1. **Define Data Types** (2 hours)
   ```rust
   // src/onepassword/types.rs
   
   use serde::{Deserialize, Serialize};
   use chrono::{DateTime, Utc};
   
   /// A 1Password vault
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Vault {
       pub id: String,
       pub name: String,
       #[serde(rename = "type")]
       pub vault_type: VaultType,
       pub created_at: DateTime<Utc>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
   pub enum VaultType {
       Personal,
       Shared,
       Private,
   }
   
   /// A 1Password item (credential)
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Item {
       pub id: String,
       pub title: String,
       pub category: ItemCategory,
       pub vault: VaultReference,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
       #[serde(default)]
       pub tags: Vec<String>,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
   pub enum ItemCategory {
       Login,
       SecureNote,
       Password,
       #[serde(rename = "SSH_KEY")]
       SshKey,
       Document,
       Identity,
       #[serde(other)]
       Other,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct VaultReference {
       pub id: String,
       pub name: Option<String>,
   }
   ```

2. **Implement Vault Listing** (3 hours)
   ```rust
   impl OnePasswordCli {
       /// List all accessible vaults
       pub fn list_vaults(&self) -> Result<Vec<Vault>, OnePasswordError> {
           self.ensure_authenticated()?;
           
           let output = self.run_command(&["vault", "list", "--format", "json"])?;
           
           let vaults: Vec<Vault> = serde_json::from_str(&output)
               .map_err(|e| OnePasswordError::InvalidJson(
                   format!("Failed to parse vaults: {}", e)
               ))?;
           
           Ok(vaults)
       }
       
       /// Run op command with args
       fn run_command(&self, args: &[&str]) -> Result<String, OnePasswordError> {
           let output = Command::new(&self.op_path)
               .args(args)
               .output()
               .map_err(|e| OnePasswordError::CommandFailed(
                   format!("Failed to run op: {}", e)
               ))?;
           
           if !output.status.success() {
               let error = String::from_utf8_lossy(&output.stderr);
               return Err(OnePasswordError::CommandFailed(error.to_string()));
           }
           
           Ok(String::from_utf8_lossy(&output.stdout).to_string())
       }
       
       fn ensure_authenticated(&self) -> Result<(), OnePasswordError> {
           if !self.is_authenticated()? {
               return Err(OnePasswordError::NotAuthenticated);
           }
           Ok(())
       }
   }
   ```

3. **Implement Item Discovery** (3 hours)
   ```rust
   impl OnePasswordCli {
       /// List items, optionally filtered by category
       pub fn list_items(
           &self, 
           category: Option<ItemCategory>,
           vault_id: Option<&str>,
       ) -> Result<Vec<Item>, OnePasswordError> {
           self.ensure_authenticated()?;
           
           let mut args = vec!["item", "list", "--format", "json"];
           
           // Add category filter
           if let Some(cat) = category {
               args.push("--categories");
               args.push(match cat {
                   ItemCategory::SshKey => "SSH Key",
                   ItemCategory::Login => "Login",
                   ItemCategory::Password => "Password",
                   // ... map other categories
               });
           }
           
           // Add vault filter
           if let Some(vault) = vault_id {
               args.push("--vault");
               args.push(vault);
           }
           
           let output = self.run_command(&args)?;
           
           let items: Vec<Item> = serde_json::from_str(&output)
               .map_err(|e| OnePasswordError::InvalidJson(
                   format!("Failed to parse items: {}", e)
               ))?;
           
           Ok(items)
       }
       
       /// List SSH keys specifically
       pub fn list_ssh_keys(&self) -> Result<Vec<SshKeyInfo>, OnePasswordError> {
           let items = self.list_items(Some(ItemCategory::SshKey), None)?;
           
           // Get additional SSH key metadata
           let mut ssh_keys = Vec::new();
           for item in items {
               if let Ok(details) = self.get_ssh_key_info(&item.id) {
                   ssh_keys.push(details);
               }
           }
           
           Ok(ssh_keys)
       }
   }
   
   #[derive(Debug, Clone)]
   pub struct SshKeyInfo {
       pub item_id: String,
       pub title: String,
       pub fingerprint: Option<String>,
       pub key_type: Option<String>,
       pub vault_name: String,
       pub reference: String, // op://vault/item/field
   }
   ```

**Security Considerations**:
```rust
// ❌ BAD: Logging sensitive data
log::debug!("Items: {:?}", items); // Could contain sensitive titles

// ✅ GOOD: Log only non-sensitive info
log::debug!("Found {} items", items.len());

// ❌ BAD: Storing actual key
let private_key = op.read_secret(&reference)?;

// ✅ GOOD: Store only reference
profile.ssh_key_ref = Some(reference); // op://Personal/GitHub/private key
```

---

### 🛑 CHECKPOINT 4.2: Credential Discovery Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 24 hours + 8 hours review = 32 hours total

**Pre-Checkpoint Checklist**:
- [ ] Vault listing works
- [ ] Item discovery functional
- [ ] SSH key metadata retrieved
- [ ] No sensitive data logged
- [ ] Performance acceptable

---

## Week 2: Advanced Integration

### 4.3 Secure Credential Retrieval (24 hours)

#### Task 4.3.1: Biometric Authentication (8 hours)

💡 **Junior Dev Concept**: Biometric Authentication
**What it is**: Using fingerprint/face instead of password
**Why important**: Better user experience, more secure
**How it works**: 1Password handles it, we just trigger it

[Implementation continues with biometric support...]

### 4.4 Testing and Security Hardening (16 hours)

#### Task 4.4.1: Mock Implementation (8 hours)

💡 **Junior Dev Concept**: Mocking External Dependencies
**What it is**: Fake 1Password for testing without the real thing
**Why critical**: Tests must run in CI without 1Password
**How**: Trait-based design allows swapping implementations

[Mock implementation details...]

---

### 🛑 FINAL CHECKPOINT 4: 1Password Integration Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Final Deliverables**:
- Secure 1Password CLI wrapper
- Credential discovery without exposure
- Biometric authentication support
- Comprehensive mock for testing
- Zero security vulnerabilities

---

## Common Issues and Solutions

### Issue: Biometric prompt not appearing
**Symptom**: Command hangs waiting for password
**Solution**: Enable biometric unlock: `op account add --biometric`

### Issue: JSON parsing fails
**Symptom**: InvalidJson errors
**Debug**: Save output to file, validate JSON, check for version changes

## Security Checklist

Before completing Phase 4:
- [ ] No credentials in logs (grep for patterns)
- [ ] All secrets use SensitiveString
- [ ] Command arguments don't contain secrets
- [ ] Error messages don't leak information
- [ ] Mock doesn't include real credentials
- [ ] Integration tests use separate vault

## Summary

Phase 4 provides secure 1Password integration with extensive safeguards against credential exposure. The careful approach ensures security while maintaining usability.