# Comprehensive Troubleshooting Guide for Git Signing

## Overview

This guide provides detailed troubleshooting scenarios for both SSH and GPG signing issues, with step-by-step resolution procedures and preventive measures.

## Quick Diagnosis Tool

```rust
// src/signing/troubleshoot.rs

use std::process::Command;
use colored::*;

pub struct SigningDiagnostics;

impl SigningDiagnostics {
    /// Run comprehensive signing diagnostics
    pub fn run_full_diagnostics() -> DiagnosticReport {
        println!("{}", "🔍 Running Git Signing Diagnostics...".bold());
        
        let mut report = DiagnosticReport::new();
        
        // Check Git version
        report.add_check("Git Version", Self::check_git_version());
        
        // Check signing configuration
        report.add_check("Git Signing Config", Self::check_git_config());
        
        // Check SSH signing
        report.add_check("SSH Signing", Self::check_ssh_signing());
        
        // Check GPG signing
        report.add_check("GPG Signing", Self::check_gpg_signing());
        
        // Check 1Password integration
        report.add_check("1Password Integration", Self::check_1password());
        
        report
    }
    
    fn check_git_version() -> CheckResult {
        match Command::new("git").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                let (major, minor) = Self::parse_git_version(&version);
                
                if major > 2 || (major == 2 && minor >= 34) {
                    CheckResult::Pass(format!("Git {} (SSH signing supported)", version.trim()))
                } else {
                    CheckResult::Warning(format!(
                        "Git {} - SSH signing requires 2.34+", 
                        version.trim()
                    ))
                }
            }
            Err(e) => CheckResult::Fail(format!("Git not found: {}", e)),
        }
    }
}

#[derive(Debug)]
pub struct DiagnosticReport {
    checks: Vec<(String, CheckResult)>,
}

#[derive(Debug)]
pub enum CheckResult {
    Pass(String),
    Warning(String),
    Fail(String),
}
```

## SSH Signing Troubleshooting

### Scenario 1: SSH Key Not Found

**Symptoms:**
```
error: Load key "/path/to/key": No such file or directory
fatal: failed to write commit object
```

**Diagnosis:**
```bash
# Check configured key
git config user.signingkey

# Verify key exists
ls -la ~/.ssh/

# Check SSH agent
ssh-add -l
```

**Solutions:**

1. **Key path is wrong:**
   ```bash
   # Fix the path
   git config user.signingkey ~/.ssh/id_ed25519.pub
   ```

2. **Key not generated:**
   ```bash
   # Generate new SSH key
   ssh-keygen -t ed25519 -C "your_email@example.com"
   
   # Add to SSH agent
   ssh-add ~/.ssh/id_ed25519
   ```

3. **Using 1Password reference incorrectly:**
   ```bash
   # Wrong: Direct reference
   git config user.signingkey "op://Personal/GitHub SSH Key/public key"
   
   # Correct: Use key:: prefix
   git config user.signingkey "key::op://Personal/GitHub SSH Key/public key"
   ```

**Automated Fix:**
```rust
pub fn fix_ssh_key_not_found() -> Result<(), SigningError> {
    // Check current config
    let key_path = GitConfig::get("user.signingkey")?;
    
    if key_path.starts_with("op://") && !key_path.starts_with("key::") {
        // Fix 1Password reference
        GitConfig::set("user.signingkey", &format!("key::{}", key_path))?;
        println!("✓ Fixed 1Password key reference");
    } else if !Path::new(&key_path).exists() {
        // Try to find the key
        let possible_keys = vec![
            "~/.ssh/id_ed25519.pub",
            "~/.ssh/id_rsa.pub",
            "~/.ssh/id_ecdsa.pub",
        ];
        
        for key in possible_keys {
            let expanded = shellexpand::tilde(key);
            if Path::new(expanded.as_ref()).exists() {
                GitConfig::set("user.signingkey", expanded.as_ref())?;
                println!("✓ Updated key path to: {}", expanded);
                return Ok(());
            }
        }
        
        return Err(SigningError::KeyNotFound);
    }
    
    Ok(())
}
```

### Scenario 2: SSH Agent Issues

**Symptoms:**
```
error: agent refused operation
sign_and_send_pubkey: signing failed: agent refused operation
```

**Diagnosis:**
```bash
# Check if agent is running
eval $(ssh-agent -s)

# List loaded keys
ssh-add -l

# Check agent socket
echo $SSH_AUTH_SOCK
```

**Solutions:**

1. **Agent not running:**
   ```bash
   # Start agent
   eval $(ssh-agent -s)
   
   # Add to shell profile
   echo 'eval $(ssh-agent -s)' >> ~/.bashrc
   ```

2. **Key not loaded:**
   ```bash
   # Add key to agent
   ssh-add ~/.ssh/id_ed25519
   
   # Add with timeout (more secure)
   ssh-add -t 3600 ~/.ssh/id_ed25519
   ```

3. **Agent forwarding issues:**
   ```bash
   # Enable agent forwarding
   ssh -A user@host
   
   # Check forwarding
   ssh-add -l
   ```

**Automated Fix:**
```rust
pub fn fix_ssh_agent_issues() -> Result<(), SigningError> {
    // Check if agent is running
    if std::env::var("SSH_AUTH_SOCK").is_err() {
        // Start agent
        let output = Command::new("ssh-agent")
            .arg("-s")
            .output()?;
        
        // Parse and set environment variables
        let agent_vars = String::from_utf8_lossy(&output.stdout);
        for line in agent_vars.lines() {
            if line.starts_with("SSH_AUTH_SOCK=") {
                let sock = line.split('=').nth(1).unwrap().trim_end_matches(';');
                std::env::set_var("SSH_AUTH_SOCK", sock);
            }
        }
    }
    
    // Load keys
    let keys = SshKeyDiscovery::discover_local_keys()?;
    for key in keys {
        if let Some(private_key) = key.private_key_path {
            Command::new("ssh-add")
                .arg(private_key)
                .status()?;
        }
    }
    
    Ok(())
}
```

### Scenario 3: Git Version Too Old

**Symptoms:**
```
error: unsupported value for gpg.format: ssh
fatal: bad config variable 'gpg.format' in file '.git/config'
```

**Diagnosis:**
```bash
git --version
# If < 2.34, SSH signing not supported
```

**Solutions:**

1. **Update Git (macOS):**
   ```bash
   brew update && brew upgrade git
   ```

2. **Update Git (Ubuntu/Debian):**
   ```bash
   sudo add-apt-repository ppa:git-core/ppa
   sudo apt update
   sudo apt upgrade git
   ```

3. **Update Git (Windows):**
   ```powershell
   winget upgrade Git.Git
   ```

4. **Fallback to GPG:**
   ```bash
   git config gpg.format openpgp
   git config user.signingkey YOUR_GPG_KEY_ID
   ```

## GPG Signing Troubleshooting

### Scenario 4: GPG Key Not Found

**Symptoms:**
```
error: gpg failed to sign the data
fatal: failed to write commit object
```

**Diagnosis:**
```bash
# List secret keys
gpg --list-secret-keys

# Check configured key
git config user.signingkey

# Verify key can sign
gpg --list-secret-keys --keyid-format LONG | grep "\[S\]"
```

**Solutions:**

1. **No GPG keys:**
   ```bash
   # Generate new key
   gpg --full-generate-key
   
   # Or use our wizard
   git-setup signing generate-gpg
   ```

2. **Wrong key ID configured:**
   ```bash
   # List keys with IDs
   gpg --list-secret-keys --keyid-format LONG
   
   # Update Git config
   git config user.signingkey CORRECT_KEY_ID
   ```

3. **Key expired:**
   ```bash
   # Edit key to extend expiration
   gpg --edit-key KEY_ID
   gpg> expire
   # Follow prompts
   gpg> save
   ```

### Scenario 5: GPG Agent/TTY Issues

**Symptoms:**
```
error: gpg failed to sign the data
gpg: signing failed: Inappropriate ioctl for device
```

**Diagnosis:**
```bash
# Check GPG_TTY
echo $GPG_TTY

# Check agent
gpg-connect-agent --dirmngr 'keyinfo --list' /bye
```

**Solutions:**

1. **Set GPG_TTY:**
   ```bash
   export GPG_TTY=$(tty)
   
   # Add to shell profile
   echo 'export GPG_TTY=$(tty)' >> ~/.bashrc
   ```

2. **Configure pinentry:**
   ```bash
   # macOS
   brew install pinentry-mac
   echo "pinentry-program $(which pinentry-mac)" >> ~/.gnupg/gpg-agent.conf
   
   # Linux with GUI
   sudo apt install pinentry-gtk2
   echo "pinentry-program $(which pinentry-gtk-2)" >> ~/.gnupg/gpg-agent.conf
   ```

3. **Restart agent:**
   ```bash
   gpgconf --kill gpg-agent
   gpg-connect-agent reloadagent /bye
   ```

**Automated Fix:**
```rust
pub fn fix_gpg_tty_issues() -> Result<(), SigningError> {
    // Set GPG_TTY
    if let Ok(tty_output) = Command::new("tty").output() {
        let tty = String::from_utf8_lossy(&tty_output.stdout).trim().to_string();
        std::env::set_var("GPG_TTY", &tty);
        
        // Also update shell profile
        let shell_rc = dirs::home_dir()
            .map(|h| h.join(".bashrc"))
            .unwrap();
        
        let mut content = fs::read_to_string(&shell_rc)?;
        if !content.contains("GPG_TTY") {
            content.push_str("\n# Added by git-setup\nexport GPG_TTY=$(tty)\n");
            fs::write(&shell_rc, content)?;
        }
    }
    
    // Restart GPG agent
    Command::new("gpgconf")
        .args(&["--kill", "gpg-agent"])
        .status()?;
    
    Ok(())
}
```

### Scenario 6: Passphrase Entry Problems

**Symptoms:**
- No passphrase prompt appears
- Passphrase prompt appears in wrong place
- "No secret key" after entering passphrase

**Solutions:**

1. **GUI environment without pinentry-gui:**
   ```bash
   # Install GUI pinentry
   # macOS
   brew install pinentry-mac
   
   # Linux
   sudo apt install pinentry-gnome3  # or pinentry-qt
   ```

2. **SSH session issues:**
   ```bash
   # Use curses pinentry for SSH
   echo "pinentry-program /usr/bin/pinentry-curses" >> ~/.gnupg/gpg-agent.conf
   
   # Or enable loopback
   echo "allow-loopback-pinentry" >> ~/.gnupg/gpg-agent.conf
   git config gpg.program "gpg --pinentry-mode loopback"
   ```

## 1Password Integration Troubleshooting

### Scenario 7: 1Password CLI Not Found

**Symptoms:**
```
error: cannot run op: No such file or directory
```

**Solutions:**

1. **Install 1Password CLI:**
   ```bash
   # macOS
   brew install --cask 1password-cli
   
   # Linux
   curl -sS https://downloads.1password.com/linux/keys/1password.asc | \
     sudo gpg --dearmor --output /usr/share/keyrings/1password-archive-keyring.gpg
   ```

2. **Add to PATH:**
   ```bash
   # Find op location
   which op || find / -name "op" 2>/dev/null
   
   # Add to PATH
   export PATH="$PATH:/path/to/1password-cli"
   ```

### Scenario 8: 1Password Authentication Issues

**Symptoms:**
```
error: You are not currently signed in
```

**Solutions:**

1. **Sign in to 1Password:**
   ```bash
   op signin
   ```

2. **Enable biometric unlock:**
   ```bash
   op account add --address my.1password.com --email you@example.com
   # Follow prompts to enable biometric
   ```

3. **Use service account (CI/CD):**
   ```bash
   export OP_SERVICE_ACCOUNT_TOKEN="your-token"
   ```

## Verification Issues

### Scenario 9: Signature Verification Fails

**Symptoms:**
```
gpg: Can't check signature: No public key
```

**Solutions:**

1. **Configure allowed signers (SSH):**
   ```bash
   # Create allowed_signers file
   echo "your@email.com $(cat ~/.ssh/id_ed25519.pub)" >> ~/.ssh/allowed_signers
   
   # Configure Git
   git config gpg.ssh.allowedSignersFile ~/.ssh/allowed_signers
   ```

2. **Import public keys (GPG):**
   ```bash
   # Export your public key
   gpg --armor --export your@email.com > pubkey.asc
   
   # Others import it
   gpg --import pubkey.asc
   ```

## Platform-Specific Issues

### Windows Issues

1. **Path separators:**
   ```powershell
   # Use forward slashes even on Windows
   git config user.signingkey C:/Users/You/.ssh/id_ed25519.pub
   ```

2. **SSH agent not available:**
   ```powershell
   # Use Windows OpenSSH
   Get-Service ssh-agent | Set-Service -StartupType Automatic
   Start-Service ssh-agent
   ```

### macOS Issues

1. **Keychain integration:**
   ```bash
   # Add SSH key to keychain
   ssh-add --apple-use-keychain ~/.ssh/id_ed25519
   ```

2. **GPG suite conflicts:**
   ```bash
   # Prefer homebrew GPG
   brew install gnupg
   export PATH="/usr/local/bin:$PATH"
   ```

### Linux Issues

1. **SELinux blocking GPG:**
   ```bash
   # Check SELinux logs
   sudo ausearch -m avc -ts recent | grep gpg
   
   # Create policy if needed
   sudo setsebool -P gpg_read_user_content on
   ```

## Diagnostic Commands Reference

```bash
# Complete diagnostic script
#!/bin/bash

echo "=== Git Signing Diagnostics ==="

echo -e "\n--- Git Configuration ---"
git --version
git config --get gpg.format
git config --get user.signingkey
git config --get commit.gpgsign

echo -e "\n--- SSH Configuration ---"
ls -la ~/.ssh/*.pub 2>/dev/null || echo "No SSH public keys found"
ssh-add -l 2>/dev/null || echo "SSH agent not running or no keys loaded"

echo -e "\n--- GPG Configuration ---"
gpg --version | head -1
gpg --list-secret-keys --keyid-format SHORT
echo "GPG_TTY=$GPG_TTY"

echo -e "\n--- 1Password Status ---"
op --version 2>/dev/null || echo "1Password CLI not found"
op whoami 2>/dev/null || echo "Not signed in to 1Password"

echo -e "\n--- Test Signing ---"
echo "test" | git hash-object -w --stdin > /tmp/test-blob
if git config --get gpg.format | grep -q ssh; then
    ssh-keygen -Y sign -n git -f $(git config --get user.signingkey | sed 's/key:://') /tmp/test-blob
else
    echo "test" | gpg --clearsign
fi
```

## Preventive Measures

1. **Regular key maintenance:**
   ```bash
   # Check key expiration monthly
   gpg --list-secret-keys | grep expires
   ```

2. **Backup signing keys:**
   ```bash
   # Backup GPG keys
   gpg --export-secret-keys --armor > gpg-backup.asc
   
   # Store SSH keys in 1Password
   op item create --category "SSH Key" --title "Git Signing Key" \
     private_key="$(cat ~/.ssh/id_ed25519)"
   ```

3. **Test signing regularly:**
   ```bash
   # Add to git hooks
   cat > .git/hooks/pre-commit << 'EOF'
   #!/bin/bash
   if ! git commit -S -m "test" --dry-run 2>/dev/null; then
       echo "Warning: Commit signing may be broken"
   fi
   EOF
   chmod +x .git/hooks/pre-commit
   ```

## Quick Fix Script

```rust
// src/signing/quickfix.rs

pub struct SigningQuickFix;

impl SigningQuickFix {
    pub async fn run() -> Result<(), SigningError> {
        println!("🔧 Running Git Signing Quick Fix...\n");
        
        // Detect signing method
        let format = GitConfig::get("gpg.format").unwrap_or_else(|_| "openpgp".to_string());
        
        match format.as_str() {
            "ssh" => {
                println!("📝 Detected SSH signing");
                Self::fix_ssh_signing().await?;
            }
            "openpgp" | "gpg" => {
                println!("🔐 Detected GPG signing");
                Self::fix_gpg_signing().await?;
            }
            _ => {
                println!("❓ Unknown signing format: {}", format);
            }
        }
        
        // Test signing
        println!("\n🧪 Testing signing configuration...");
        if Self::test_signing().await? {
            println!("✅ Signing is working correctly!");
        } else {
            println!("❌ Signing test failed. Run 'git-setup signing troubleshoot' for details.");
        }
        
        Ok(())
    }
}
```

This comprehensive troubleshooting guide covers all common signing issues with step-by-step solutions and automated fixes where possible.