# Phase 7: Signing Methods - Basics - Work Plan

## Prerequisites

Before starting Phase 7, ensure you understand Git's signing mechanisms and have test keys ready.

**Required from Previous Phases**:
- ✅ Profile system with signing configuration (Phase 2)
- ✅ 1Password integration for key references (Phase 4)
- ✅ Git configuration management (Phase 2)
- ✅ Secure credential handling (Phase 1)

**Required Knowledge**:
- **Git Signing**: How Git signs commits and tags (*critical*)
- **SSH Keys**: Public/private key basics (*required*)
- **GPG Basics**: Key management concepts (*helpful*)
- **Security**: Key handling best practices (*critical*)

💡 **Junior Dev Resources**:
- 📚 [Git Signing Explained](https://docs.github.com/en/authentication/managing-commit-signature-verification) - GitHub's guide (30 min)
- 📖 [SSH vs GPG Signing](https://docs.github.com/en/authentication/managing-commit-signature-verification/about-commit-signature-verification) - Comparison guide
- 📖 [SSH Key Basics](https://www.ssh.com/academy/ssh/keygen) - Understanding keys
- 🔧 Practice: Generate test keys first
- 📝 Examples: `examples/signing/` directory
- 🔑 [GPG Implementation Guide](GPG_IMPLEMENTATION.md) - **Complete GPG implementation**
- 🔧 [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) - **Essential for debugging signing issues**
- 🔐 [Security Implementation Examples](../SECURITY_IMPLEMENTATION_EXAMPLES.md) - Secure key handling

## Quick Reference - Essential Resources

### Git Signing Commands
```bash
# SSH signing (Git 2.34+)
git config gpg.format ssh
git config user.signingkey ~/.ssh/id_ed25519.pub
git commit -S -m "Signed with SSH"

# GPG signing (traditional)
git config gpg.format openpgp
git config user.signingkey ABCD1234
git commit -S -m "Signed with GPG"

# Verify signatures
git log --show-signature
```

### Key Concepts
1. **SSH Signing**: Newer, simpler, integrates with existing SSH keys
2. **GPG Signing**: Traditional, more complex, widely supported
3. **Verification**: GitHub/GitLab show "Verified" badge
4. **Key Storage**: Never store private keys in git-setup!

## Overview

Phase 7 implements basic signing methods (SSH and GPG) for Git commits and tags. This phase focuses on making signing accessible to developers who haven't used it before.

**What You'll Build**:
1. SSH key discovery and configuration
2. GPG key detection and setup
3. Git signing configuration
4. Signature verification helpers
5. Clear error messages for troubleshooting

**Success Looks Like**:
- User's commits show "Verified" on GitHub
- SSH signing "just works" with existing keys
- GPG setup is as painless as possible
- Clear guidance when things go wrong

**Time Estimate**: 2 weeks (80 hours)
- Week 1: SSH signing implementation (40h)
- Week 2: GPG signing basics (40h)

## Done Criteria Checklist

Phase 7 is complete when:
- [ ] SSH signing works with local keys
- [ ] SSH signing works with 1Password keys
- [ ] GPG key detection functional
- [ ] Git configured correctly for signing
- [ ] Verification helper shows status
- [ ] Error messages guide fixes
- [ ] Tests cover all scenarios
- [ ] Documentation complete

## Week 1: SSH Signing Implementation

### 5C.1 SSH Key Discovery and Management (24 hours)

#### Task 5C.1.1: SSH Key Discovery (8 hours)

💡 **Junior Dev Concept**: SSH Key Discovery
**What it is**: Finding SSH keys on the user's system or in 1Password
**Why careful**: Private keys are sensitive, we only work with public keys
**Real Example**: GitHub needs your public key to verify your commits

**Prerequisites**:
- [ ] Generate test SSH key pair
- [ ] Understand: Public vs private keys
- [ ] Read: SSH key formats

**Visual SSH Signing Flow**:
```
Your Computer                    Git                        GitHub
─────────────                   ───                        ──────
SSH Private Key                  │                           │
(~/.ssh/id_ed25519)              │                           │
        │                        │                           │
        ├─ Sign commit ────────> │                           │
        │                        ├─ Push with signature ───> │
        │                        │                           ├─ Verify with
        │                        │                           │  your public key
        │                        │                           │
Never leaves your computer!      │                           └─ ✓ Verified
```

**Step-by-Step Implementation**:

1. **Create SSH Module Structure** (1 hour)
   ```rust
   // src/signing/ssh/mod.rs
   
   pub mod discovery;
   pub mod config;
   pub mod verification;
   
   use std::path::PathBuf;
   use crate::security::SensitiveString;
   
   /// Information about an SSH key
   #[derive(Debug, Clone)]
   pub struct SshKeyInfo {
       /// Path to public key file
       pub public_key_path: PathBuf,
       
       /// Path to private key (if found)
       pub private_key_path: Option<PathBuf>,
       
       /// Key fingerprint
       pub fingerprint: String,
       
       /// Key type (e.g., "ed25519", "rsa")
       pub key_type: String,
       
       /// Key comment (usually email)
       pub comment: Option<String>,
       
       /// 1Password reference if applicable
       pub op_reference: Option<String>,
   }
   ```

2. **Implement Local Key Discovery** (4 hours)
   ```rust
   // src/signing/ssh/discovery.rs
   
   use std::fs;
   use std::path::{Path, PathBuf};
   use regex::Regex;
   
   /// Discover SSH keys in standard locations
   pub struct SshKeyDiscovery;
   
   impl SshKeyDiscovery {
       /// Find all SSH keys in user's home directory
       pub fn discover_local_keys() -> Result<Vec<SshKeyInfo>, SigningError> {
           let mut keys = Vec::new();
           
           // Get SSH directory
           let ssh_dir = Self::get_ssh_directory()?;
           if !ssh_dir.exists() {
               return Ok(keys); // No SSH directory, no keys
           }
           
           // Read directory
           let entries = fs::read_dir(&ssh_dir)
               .map_err(|e| SigningError::Discovery(format!(
                   "Cannot read SSH directory: {}", e
               )))?;
           
           // Look for public keys
           for entry in entries.flatten() {
               let path = entry.path();
               
               // Check if it's a public key
               if Self::is_public_key(&path) {
                   if let Ok(info) = Self::analyze_key_pair(&path) {
                       keys.push(info);
                   }
               }
           }
           
           // Sort by modification time (newest first)
           keys.sort_by(|a, b| {
               let time_a = fs::metadata(&a.public_key_path)
                   .and_then(|m| m.modified())
                   .ok();
               let time_b = fs::metadata(&b.public_key_path)
                   .and_then(|m| m.modified())
                   .ok();
               time_b.cmp(&time_a)
           });
           
           Ok(keys)
       }
       
       /// Get user's SSH directory
       fn get_ssh_directory() -> Result<PathBuf, SigningError> {
           let home = dirs::home_dir()
               .ok_or_else(|| SigningError::Discovery(
                   "Cannot find home directory".to_string()
               ))?;
           
           Ok(home.join(".ssh"))
       }
       
       /// Check if file is a public key
       fn is_public_key(path: &Path) -> bool {
           if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
               // Common public key patterns
               name.ends_with(".pub") || 
               name == "id_rsa.pub" ||
               name == "id_ed25519.pub" ||
               name == "id_ecdsa.pub"
           } else {
               false
           }
       }
       
       /// Analyze a public key and find its private key
       fn analyze_key_pair(public_path: &Path) -> Result<SshKeyInfo, SigningError> {
           // Read public key
           let public_key_content = fs::read_to_string(public_path)
               .map_err(|e| SigningError::Discovery(format!(
                   "Cannot read public key: {}", e
               )))?;
           
           // Parse key type and content
           let parts: Vec<&str> = public_key_content.trim().split_whitespace().collect();
           if parts.len() < 2 {
               return Err(SigningError::Discovery(
                   "Invalid public key format".to_string()
               ));
           }
           
           let key_type = parts[0].to_string();
           let comment = parts.get(2).map(|s| s.to_string());
           
           // Generate fingerprint
           let fingerprint = Self::calculate_fingerprint(public_path)?;
           
           // Look for private key
           let private_key_path = Self::find_private_key(public_path);
           
           Ok(SshKeyInfo {
               public_key_path: public_path.to_path_buf(),
               private_key_path,
               fingerprint,
               key_type: key_type.replace("ssh-", ""),
               comment,
               op_reference: None,
           })
       }
       
       /// Find corresponding private key
       fn find_private_key(public_path: &Path) -> Option<PathBuf> {
           // Remove .pub extension to get private key path
           if let Some(stem) = public_path.file_stem() {
               let private_path = public_path.with_file_name(stem);
               if private_path.exists() && private_path.is_file() {
                   return Some(private_path);
               }
           }
           None
       }
       
       /// Calculate SSH key fingerprint
       fn calculate_fingerprint(public_path: &Path) -> Result<String, SigningError> {
           use std::process::Command;
           
           let output = Command::new("ssh-keygen")
               .arg("-l")
               .arg("-f")
               .arg(public_path)
               .output()
               .map_err(|e| SigningError::Discovery(format!(
                   "Cannot calculate fingerprint: {}", e
               )))?;
           
           if output.status.success() {
               let fingerprint = String::from_utf8_lossy(&output.stdout);
               // Extract just the fingerprint part
               if let Some(fp) = fingerprint.split_whitespace().nth(1) {
                   Ok(fp.to_string())
               } else {
                   Ok(fingerprint.trim().to_string())
               }
           } else {
               Err(SigningError::Discovery(
                   "Failed to calculate fingerprint".to_string()
               ))
           }
       }
   }
   ```
   
   ⚠️ **Common Mistake**: Trying to read private keys
   ✅ **Solution**: Only work with public keys for discovery

3. **Integrate with 1Password** (3 hours)
   ```rust
   impl SshKeyDiscovery {
       /// Discover SSH keys from 1Password
       pub fn discover_1password_keys(
           op_client: &OnePasswordCli
       ) -> Result<Vec<SshKeyInfo>, SigningError> {
           let mut keys = Vec::new();
           
           // Get SSH keys from 1Password
           let op_keys = op_client.list_ssh_keys()
               .map_err(|e| SigningError::Discovery(format!(
                   "1Password error: {}", e
               )))?;
           
           for op_key in op_keys {
               keys.push(SshKeyInfo {
                   public_key_path: PathBuf::new(), // No local file
                   private_key_path: None,
                   fingerprint: op_key.fingerprint.unwrap_or_default(),
                   key_type: op_key.key_type.unwrap_or_else(|| "unknown".to_string()),
                   comment: Some(op_key.title.clone()),
                   op_reference: Some(op_key.reference),
               });
           }
           
           Ok(keys)
       }
       
       /// Discover all available SSH keys
       pub fn discover_all(
           op_client: Option<&OnePasswordCli>
       ) -> Result<Vec<SshKeyInfo>, SigningError> {
           let mut all_keys = Vec::new();
           
           // Local keys first
           all_keys.extend(Self::discover_local_keys()?);
           
           // Then 1Password keys if available
           if let Some(op) = op_client {
               if let Ok(op_keys) = Self::discover_1password_keys(op) {
                   all_keys.extend(op_keys);
               }
           }
           
           Ok(all_keys)
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
    fn test_key_discovery_empty_dir() {
        let temp = TempDir::new().unwrap();
        std::env::set_var("HOME", temp.path());
        
        let keys = SshKeyDiscovery::discover_local_keys().unwrap();
        assert!(keys.is_empty());
    }
    
    #[test]
    fn test_public_key_detection() {
        assert!(SshKeyDiscovery::is_public_key(Path::new("id_rsa.pub")));
        assert!(SshKeyDiscovery::is_public_key(Path::new("custom.pub")));
        assert!(!SshKeyDiscovery::is_public_key(Path::new("id_rsa")));
    }
}
```

**Debugging Guide**:

**Issue**: No keys found
**Debug**: Check `~/.ssh/` directory exists and has `.pub` files

**Issue**: Fingerprint calculation fails
**Solution**: Ensure `ssh-keygen` is in PATH

**Issue**: 1Password keys not showing
**Solution**: Check authentication and vault permissions

---

### 🛑 CHECKPOINT 7.1: SSH Key Discovery Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 24 hours + 4 hours review = 28 hours total

**Pre-Checkpoint Checklist**:
- [ ] Local SSH keys discovered
- [ ] 1Password keys integrated
- [ ] Fingerprints calculated
- [ ] No private key exposure
- [ ] Tests comprehensive

---

### 5C.2 SSH Signing Configuration (16 hours)

#### Task 5C.2.1: Git SSH Configuration (8 hours)

💡 **Junior Dev Concept**: Configuring Git for SSH Signing
**What it is**: Telling Git to use SSH keys instead of GPG for signing
**Why SSH**: Simpler than GPG, reuses existing SSH keys
**Requirements**: Git 2.34+ for SSH signing support

**Implementation**:

1. **Create Git Configuration Helper** (4 hours)
   ```rust
   // src/signing/ssh/config.rs
   
   use crate::git::GitConfig;
   use std::path::Path;
   
   /// Configure Git for SSH signing
   pub struct SshSigningConfig;
   
   impl SshSigningConfig {
       /// Check if Git supports SSH signing
       pub fn check_git_version() -> Result<bool, SigningError> {
           let output = Command::new("git")
               .arg("--version")
               .output()
               .map_err(|e| SigningError::Configuration(format!(
                   "Cannot check Git version: {}", e
               )))?;
           
           if output.status.success() {
               let version = String::from_utf8_lossy(&output.stdout);
               // Parse version (e.g., "git version 2.34.0")
               let supports_ssh = Self::parse_version(&version)
                   .map(|(major, minor, _)| major > 2 || (major == 2 && minor >= 34))
                   .unwrap_or(false);
               
               Ok(supports_ssh)
           } else {
               Err(SigningError::Configuration(
                   "Failed to get Git version".to_string()
               ))
           }
       }
       
       /// Configure Git to use SSH signing
       pub fn configure_ssh_signing(
           key_info: &SshKeyInfo,
           scope: ConfigScope,
       ) -> Result<(), SigningError> {
           // Check Git version first
           if !Self::check_git_version()? {
               return Err(SigningError::Configuration(
                   "Git 2.34+ required for SSH signing".to_string()
               ));
           }
           
           let git_config = GitConfig::new(scope);
           
           // Set signing format to SSH
           git_config.set("gpg.format", "ssh")?;
           
           // Set signing key
           if let Some(op_ref) = &key_info.op_reference {
               // For 1Password keys, use the reference
               git_config.set("user.signingkey", &format!("key::{}", op_ref))?;
           } else {
               // For local keys, use the public key path
               let key_path = key_info.public_key_path.to_string_lossy();
               git_config.set("user.signingkey", &key_path)?;
           }
           
           // Enable commit signing by default (optional)
           git_config.set("commit.gpgsign", "true")?;
           
           Ok(())
       }
       
       /// Set up allowed signers file for verification
       pub fn setup_allowed_signers(
           keys: &[SshKeyInfo],
       ) -> Result<PathBuf, SigningError> {
           let home = dirs::home_dir()
               .ok_or_else(|| SigningError::Configuration(
                   "Cannot find home directory".to_string()
               ))?;
           
           let allowed_signers_path = home.join(".ssh").join("allowed_signers");
           let mut content = String::new();
           
           for key in keys {
               if let Ok(pub_key) = fs::read_to_string(&key.public_key_path) {
                   let email = key.comment.as_deref().unwrap_or("user@example.com");
                   content.push_str(&format!("{} {}\n", email, pub_key.trim()));
               }
           }
           
           fs::write(&allowed_signers_path, content)
               .map_err(|e| SigningError::Configuration(format!(
                   "Cannot write allowed_signers: {}", e
               )))?;
           
           // Configure Git to use this file
           let git_config = GitConfig::new(ConfigScope::Global);
           git_config.set(
               "gpg.ssh.allowedSignersFile",
               &allowed_signers_path.to_string_lossy()
           )?;
           
           Ok(allowed_signers_path)
       }
   }
   ```

2. **Create Signing Test Helper** (4 hours)
   ```rust
   /// Test SSH signing configuration
   pub struct SshSigningTest;
   
   impl SshSigningTest {
       /// Create a test commit to verify signing works
       pub fn test_signing(repo_path: &Path) -> Result<bool, SigningError> {
           // Create test file
           let test_file = repo_path.join("signing_test.txt");
           fs::write(&test_file, "SSH signing test")?;
           
           // Stage file
           Command::new("git")
               .current_dir(repo_path)
               .args(&["add", "signing_test.txt"])
               .output()?;
           
           // Create signed commit
           let output = Command::new("git")
               .current_dir(repo_path)
               .args(&["commit", "-S", "-m", "Test SSH signing"])
               .output()?;
           
           if output.status.success() {
               // Verify signature
               let verify = Command::new("git")
                   .current_dir(repo_path)
                   .args(&["log", "--show-signature", "-1"])
                   .output()?;
               
               let output_str = String::from_utf8_lossy(&verify.stdout);
               Ok(output_str.contains("Good signature") || 
                  output_str.contains("signature verified"))
           } else {
               let error = String::from_utf8_lossy(&output.stderr);
               Err(SigningError::Configuration(format!(
                   "Signing failed: {}", error
               )))
           }
       }
   }
   ```

---

## Week 2: GPG Signing Basics

### 5C.3 GPG Key Discovery and Setup (40 hours)

#### Task 5C.3.1: GPG Key Discovery (12 hours)

💡 **Junior Dev Concept**: GPG Key Management
**What it is**: Finding and using GPG keys for traditional Git signing
**Why still needed**: Many organizations require GPG, wider tool support
**Complexity**: GPG is more complex than SSH but still important

📖 **Complete Implementation**: See [GPG Implementation Guide](GPG_IMPLEMENTATION.md) for full GPG key discovery, configuration, generation, and integration code with examples.

---

### 🛑 FINAL CHECKPOINT 7: Signing Basics Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Final Deliverables**:
- SSH signing fully functional
- GPG signing basics working
- Git configuration automated
- Verification helpers included
- Comprehensive error handling

---

## Common Issues and Solutions

### SSH Signing Issues

**Issue**: "Git version too old"
**Solution**: Update Git to 2.34+: `brew upgrade git` or equivalent

**Issue**: "No matching key found"
**Solution**: Ensure public key path is correct in Git config

**Issue**: "Signing failed"
**Debug**: Check SSH agent is running: `ssh-add -l`

### GPG Signing Issues

📖 **Complete Troubleshooting Guide**: See [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) for comprehensive SSH and GPG signing troubleshooting with automated fixes and diagnostic tools.

**Issue**: "No secret key"
**Solution**: Import key or generate new one: `gpg --gen-key`

**Issue**: "Signing failed: Inappropriate ioctl"
**Solution**: Set `export GPG_TTY=$(tty)` in shell

## Summary

Phase 7 provides comprehensive support for both SSH and GPG signing, with SSH as the recommended modern approach. The implementation prioritizes ease of use while maintaining security.