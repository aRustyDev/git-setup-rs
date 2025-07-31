//! GPG operations wrapper for git-setup-rs.
//!
//! This module provides a trait-based abstraction for GPG operations,
//! allowing for easy testing through mock implementations while providing a real
//! implementation that uses std::process::Command to execute gpg commands.

use crate::error::{GitSetupError, Result};
use std::collections::HashMap;
use std::process::Command;

/// Information about a GPG key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpgKeyInfo {
    /// The key ID (short or long format)
    pub key_id: String,
    /// The key fingerprint
    pub fingerprint: String,
    /// The user ID (name and email)
    pub user_id: String,
    /// Key type (RSA, DSA, etc.)
    pub key_type: String,
    /// Key size in bits
    pub key_size: Option<u32>,
    /// Creation date (ISO format)
    pub creation_date: String,
    /// Expiration date (ISO format, None if no expiration)
    pub expiration_date: Option<String>,
    /// Trust level (ultimate, full, marginal, etc.)
    pub trust_level: String,
}

/// Parameters for generating a new GPG key.
#[derive(Debug, Clone)]
pub struct GpgKeyGenParams {
    /// Full name for the key
    pub name: String,
    /// Email address for the key
    pub email: String,
    /// Comment for the key (optional)
    pub comment: Option<String>,
    /// Key type (RSA, DSA, etc.)
    pub key_type: String,
    /// Key size in bits
    pub key_size: u32,
    /// Expiration (0 for no expiration)
    pub expiration_days: u32,
    /// Passphrase for the key (optional)
    pub passphrase: Option<String>,
}

/// Trait for GPG operations.
///
/// This trait allows for easy testing by providing a mock implementation
/// while keeping the real implementation using system gpg commands.
pub trait GpgWrapper {
    /// List all GPG keys in the keyring.
    fn list_keys(&self) -> Result<Vec<GpgKeyInfo>>;

    /// List only secret (private) keys in the keyring.
    fn list_secret_keys(&self) -> Result<Vec<GpgKeyInfo>>;

    /// Get information about a specific key by ID or fingerprint.
    fn get_key_info(&self, key_id: &str) -> Result<Option<GpgKeyInfo>>;

    /// Import a GPG key from a file.
    fn import_key_from_file(&self, key_file: &str) -> Result<String>;

    /// Import a GPG key from raw data.
    fn import_key_from_data(&self, key_data: &str) -> Result<String>;

    /// Import a GPG key with a passphrase.
    fn import_key_with_passphrase(&self, key_data: &str, passphrase: &str) -> Result<String>;

    /// Export a public key by ID.
    fn export_public_key(&self, key_id: &str) -> Result<String>;

    /// Export a private key by ID (requires passphrase).
    fn export_private_key(&self, key_id: &str, passphrase: Option<&str>) -> Result<String>;

    /// Generate a new GPG key pair.
    fn generate_key(&self, params: GpgKeyGenParams) -> Result<String>;

    /// Validate a GPG key format.
    fn validate_key(&self, key_data: &str) -> Result<bool>;

    /// Extract fingerprint from key data.
    fn extract_fingerprint(&self, key_data: &str) -> Result<String>;

    /// Delete a key from the keyring.
    fn delete_key(&self, key_id: &str, delete_secret: bool) -> Result<()>;

    /// Sign data with a GPG key.
    fn sign_data(&self, data: &str, key_id: &str, passphrase: Option<&str>) -> Result<String>;

    /// Verify a GPG signature.
    fn verify_signature(&self, data: &str, signature: &str) -> Result<bool>;
}

/// Real GPG wrapper implementation using std::process::Command.
pub struct SystemGpgWrapper {
    /// Path to the GPG binary
    gpg_path: String,
}

impl SystemGpgWrapper {
    /// Create a new SystemGpgWrapper with the default GPG path.
    pub fn new() -> Self {
        Self {
            gpg_path: "gpg".to_string(),
        }
    }

    /// Create a new SystemGpgWrapper with a custom GPG path.
    pub fn with_path(gpg_path: String) -> Self {
        Self { gpg_path }
    }
}

impl Default for SystemGpgWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemGpgWrapper {
    /// Parse GPG key listing output in colon format.
    fn parse_key_listing(&self, output: &str) -> Result<Vec<GpgKeyInfo>> {
        let mut keys = Vec::new();
        let mut current_key: Option<GpgKeyInfo> = None;

        for line in output.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() < 2 {
                continue;
            }

            match fields[0] {
                "pub" | "sec" => {
                    // Save previous key if exists
                    if let Some(key) = current_key.take() {
                        keys.push(key);
                    }

                    // Start new key
                    if fields.len() >= 7 {
                        let key_size = fields[2].parse().ok();
                        let key_type = match fields[3] {
                            "1" => "RSA",
                            "16" => "ElGamal",
                            "17" => "DSA",
                            "18" => "ECDH",
                            "19" => "ECDSA",
                            "22" => "EdDSA",
                            _ => "Unknown",
                        }.to_string();

                        current_key = Some(GpgKeyInfo {
                            key_id: fields[4].to_string(),
                            fingerprint: String::new(), // Will be filled by fpr line
                            user_id: String::new(), // Will be filled by uid line
                            key_type,
                            key_size,
                            creation_date: fields[5].to_string(),
                            expiration_date: if fields[6].is_empty() { None } else { Some(fields[6].to_string()) },
                            trust_level: match fields[1] {
                                "u" => "ultimate",
                                "f" => "full",
                                "m" => "marginal",
                                "n" => "never",
                                "e" => "expired",
                                "r" => "revoked",
                                "-" => "unknown",
                                _ => "unknown",
                            }.to_string(),
                        });
                    }
                }
                "fpr" => {
                    // Fingerprint line - only take fingerprint for main key (when fingerprint is empty)
                    if let Some(ref mut key) = current_key {
                        if fields.len() >= 10 && key.fingerprint.is_empty() {
                            key.fingerprint = fields[9].to_string();
                        }
                    }
                }
                "uid" => {
                    // User ID line
                    if let Some(ref mut key) = current_key {
                        if fields.len() >= 10 && key.user_id.is_empty() {
                            key.user_id = fields[9].to_string();
                        }
                    }
                }
                _ => {}
            }
        }

        // Don't forget the last key
        if let Some(key) = current_key {
            keys.push(key);
        }

        Ok(keys)
    }
}

impl GpgWrapper for SystemGpgWrapper {
    fn list_keys(&self) -> Result<Vec<GpgKeyInfo>> {
        let output = Command::new(&self.gpg_path)
            .args(&["--list-keys", "--with-colons", "--fingerprint"])
            .output()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --list-keys --with-colons --fingerprint", self.gpg_path),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --list-keys --with-colons --fingerprint", self.gpg_path),
                error: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_key_listing(&stdout)
    }

    fn list_secret_keys(&self) -> Result<Vec<GpgKeyInfo>> {
        let output = Command::new(&self.gpg_path)
            .args(&["--list-secret-keys", "--with-colons", "--fingerprint"])
            .output()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --list-secret-keys --with-colons --fingerprint", self.gpg_path),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --list-secret-keys --with-colons --fingerprint", self.gpg_path),
                error: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_key_listing(&stdout)
    }

    fn get_key_info(&self, key_id: &str) -> Result<Option<GpgKeyInfo>> {
        let output = Command::new(&self.gpg_path)
            .args(&["--list-keys", "--with-colons", "--fingerprint", key_id])
            .output()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --list-keys --with-colons --fingerprint {}", self.gpg_path, key_id),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            // Key not found is not an error, just return None
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let keys = self.parse_key_listing(&stdout)?;
        Ok(keys.into_iter().next())
    }

    fn import_key_from_file(&self, key_file: &str) -> Result<String> {
        let output = Command::new(&self.gpg_path)
            .args(&["--import", key_file])
            .output()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --import {}", self.gpg_path, key_file),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --import {}", self.gpg_path, key_file),
                error: stderr.to_string(),
            });
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(stderr.to_string())
    }

    fn import_key_from_data(&self, key_data: &str) -> Result<String> {
        let mut child = Command::new(&self.gpg_path)
            .args(&["--import"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --import", self.gpg_path),
                error: e.to_string(),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(key_data.as_bytes()).map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --import", self.gpg_path),
                error: format!("Failed to write to stdin: {}", e),
            })?;
        }

        let output = child.wait_with_output().map_err(|e| GitSetupError::ExternalCommand {
            command: format!("{} --import", self.gpg_path),
            error: e.to_string(),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --import", self.gpg_path),
                error: stderr.to_string(),
            });
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(stderr.to_string())
    }

    fn import_key_with_passphrase(&self, key_data: &str, passphrase: &str) -> Result<String> {
        let mut child = Command::new(&self.gpg_path)
            .args(&["--import", "--batch", "--yes", "--passphrase-fd", "0"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --import --batch --yes --passphrase-fd 0", self.gpg_path),
                error: e.to_string(),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            // First write the passphrase, then the key data
            writeln!(stdin, "{}", passphrase).map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --import", self.gpg_path),
                error: format!("Failed to write passphrase to stdin: {}", e),
            })?;
            stdin.write_all(key_data.as_bytes()).map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --import", self.gpg_path),
                error: format!("Failed to write key data to stdin: {}", e),
            })?;
        }

        let output = child.wait_with_output().map_err(|e| GitSetupError::ExternalCommand {
            command: format!("{} --import", self.gpg_path),
            error: e.to_string(),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --import", self.gpg_path),
                error: stderr.to_string(),
            });
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(stderr.to_string())
    }

    fn export_public_key(&self, key_id: &str) -> Result<String> {
        let output = Command::new(&self.gpg_path)
            .args(&["--export", "--armor", key_id])
            .output()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --export --armor {}", self.gpg_path, key_id),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --export --armor {}", self.gpg_path, key_id),
                error: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    fn export_private_key(&self, key_id: &str, passphrase: Option<&str>) -> Result<String> {
        let mut cmd = Command::new(&self.gpg_path);
        cmd.args(&["--export-secret-keys", "--armor"]);

        if passphrase.is_some() {
            cmd.args(&["--batch", "--yes", "--passphrase-fd", "0"]);
        }

        cmd.arg(key_id);

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --export-secret-keys --armor {}", self.gpg_path, key_id),
                error: e.to_string(),
            })?;

        if let Some(passphrase) = passphrase {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                writeln!(stdin, "{}", passphrase).map_err(|e| GitSetupError::ExternalCommand {
                    command: format!("{} --export-secret-keys", self.gpg_path),
                    error: format!("Failed to write passphrase to stdin: {}", e),
                })?;
            }
        }

        let output = child.wait_with_output().map_err(|e| GitSetupError::ExternalCommand {
            command: format!("{} --export-secret-keys --armor {}", self.gpg_path, key_id),
            error: e.to_string(),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --export-secret-keys --armor {}", self.gpg_path, key_id),
                error: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    fn generate_key(&self, params: GpgKeyGenParams) -> Result<String> {
        // Create batch parameter file content
        let batch_params = format!(
            "Key-Type: {}\n\
             Key-Length: {}\n\
             Name-Real: {}\n\
             Name-Email: {}\n\
             Expire-Date: {}\n\
             {}\
             %commit\n",
            params.key_type,
            params.key_size,
            params.name,
            params.email,
            if params.expiration_days == 0 { "0".to_string() } else { format!("{}d", params.expiration_days) },
            params.comment.as_ref().map(|c| format!("Name-Comment: {}", c)).unwrap_or_default()
        );

        let mut cmd = Command::new(&self.gpg_path);
        cmd.args(&["--batch", "--gen-key"]);

        if params.passphrase.is_some() {
            cmd.args(&["--passphrase-fd", "0"]);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --batch --gen-key", self.gpg_path),
                error: e.to_string(),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;

            if let Some(passphrase) = &params.passphrase {
                writeln!(stdin, "{}", passphrase).map_err(|e| GitSetupError::ExternalCommand {
                    command: format!("{} --gen-key", self.gpg_path),
                    error: format!("Failed to write passphrase to stdin: {}", e),
                })?;
            }

            stdin.write_all(batch_params.as_bytes()).map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --gen-key", self.gpg_path),
                error: format!("Failed to write batch params to stdin: {}", e),
            })?;
        }

        let output = child.wait_with_output().map_err(|e| GitSetupError::ExternalCommand {
            command: format!("{} --batch --gen-key", self.gpg_path),
            error: e.to_string(),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --batch --gen-key", self.gpg_path),
                error: stderr.to_string(),
            });
        }

        // Parse output to extract key ID
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Look for key ID in the output (GPG outputs key creation info to stderr)
        for line in stderr.lines() {
            if line.contains("key") && line.contains("marked as ultimately trusted") {
                // Extract key ID from line like "gpg: key ABCD1234 marked as ultimately trusted"
                if let Some(key_part) = line.split_whitespace().find(|s| s.len() >= 8 && s.chars().all(|c| c.is_ascii_hexdigit())) {
                    return Ok(key_part.to_string());
                }
            }
        }

        // Fallback: return stderr content
        Ok(stderr.to_string())
    }

    fn validate_key(&self, key_data: &str) -> Result<bool> {
        let mut child = Command::new(&self.gpg_path)
            .args(&["--show-keys", "--with-colons"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --show-keys --with-colons", self.gpg_path),
                error: e.to_string(),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(key_data.as_bytes()).map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --show-keys", self.gpg_path),
                error: format!("Failed to write key data to stdin: {}", e),
            })?;
        }

        let output = child.wait_with_output().map_err(|e| GitSetupError::ExternalCommand {
            command: format!("{} --show-keys --with-colons", self.gpg_path),
            error: e.to_string(),
        })?;

        // If command succeeds and has output, the key is valid
        Ok(output.status.success() && !output.stdout.is_empty())
    }

    fn extract_fingerprint(&self, key_data: &str) -> Result<String> {
        let mut child = Command::new(&self.gpg_path)
            .args(&["--show-keys", "--with-colons", "--fingerprint"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --show-keys --with-colons --fingerprint", self.gpg_path),
                error: e.to_string(),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(key_data.as_bytes()).map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --show-keys", self.gpg_path),
                error: format!("Failed to write key data to stdin: {}", e),
            })?;
        }

        let output = child.wait_with_output().map_err(|e| GitSetupError::ExternalCommand {
            command: format!("{} --show-keys --with-colons --fingerprint", self.gpg_path),
            error: e.to_string(),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --show-keys --fingerprint", self.gpg_path),
                error: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse output for fingerprint line
        for line in stdout.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 10 && fields[0] == "fpr" {
                return Ok(fields[9].to_string());
            }
        }

        Err(GitSetupError::ExternalCommand {
            command: format!("{} --show-keys --fingerprint", self.gpg_path),
            error: "No fingerprint found in key data".to_string(),
        })
    }

    fn delete_key(&self, key_id: &str, delete_secret: bool) -> Result<()> {
        if delete_secret {
            // Delete secret key first
            let output = Command::new(&self.gpg_path)
                .args(&["--batch", "--yes", "--delete-secret-keys", key_id])
                .output()
                .map_err(|e| GitSetupError::ExternalCommand {
                    command: format!("{} --batch --yes --delete-secret-keys {}", self.gpg_path, key_id),
                    error: e.to_string(),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(GitSetupError::ExternalCommand {
                    command: format!("{} --delete-secret-keys {}", self.gpg_path, key_id),
                    error: stderr.to_string(),
                });
            }
        }

        // Delete public key
        let output = Command::new(&self.gpg_path)
            .args(&["--batch", "--yes", "--delete-keys", key_id])
            .output()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --batch --yes --delete-keys {}", self.gpg_path, key_id),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --delete-keys {}", self.gpg_path, key_id),
                error: stderr.to_string(),
            });
        }

        Ok(())
    }

    fn sign_data(&self, data: &str, key_id: &str, passphrase: Option<&str>) -> Result<String> {
        let mut cmd = Command::new(&self.gpg_path);
        cmd.args(&["--armor", "--detach-sign", "--local-user", key_id]);

        if let Some(_) = passphrase {
            cmd.args(&["--batch", "--yes", "--passphrase-fd", "0"]);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --armor --detach-sign --local-user {}", self.gpg_path, key_id),
                error: e.to_string(),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;

            if let Some(passphrase) = passphrase {
                writeln!(stdin, "{}", passphrase).map_err(|e| GitSetupError::ExternalCommand {
                    command: format!("{} --sign", self.gpg_path),
                    error: format!("Failed to write passphrase to stdin: {}", e),
                })?;
            }

            stdin.write_all(data.as_bytes()).map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --sign", self.gpg_path),
                error: format!("Failed to write data to stdin: {}", e),
            })?;
        }

        let output = child.wait_with_output().map_err(|e| GitSetupError::ExternalCommand {
            command: format!("{} --armor --detach-sign --local-user {}", self.gpg_path, key_id),
            error: e.to_string(),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSetupError::ExternalCommand {
                command: format!("{} --sign", self.gpg_path),
                error: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    fn verify_signature(&self, data: &str, signature: &str) -> Result<bool> {
        // Create temporary files for data and signature
        use std::io::Write;
        let mut data_file = tempfile::NamedTempFile::new().map_err(|e| GitSetupError::Io(e))?;
        let mut sig_file = tempfile::NamedTempFile::new().map_err(|e| GitSetupError::Io(e))?;

        data_file.write_all(data.as_bytes()).map_err(|e| GitSetupError::Io(e))?;
        sig_file.write_all(signature.as_bytes()).map_err(|e| GitSetupError::Io(e))?;

        let output = Command::new(&self.gpg_path)
            .args(&["--verify", sig_file.path().to_str().unwrap(), data_file.path().to_str().unwrap()])
            .output()
            .map_err(|e| GitSetupError::ExternalCommand {
                command: format!("{} --verify", self.gpg_path),
                error: e.to_string(),
            })?;

        Ok(output.status.success())
    }
}

/// Mock GPG wrapper for testing.
pub struct MockGpgWrapper {
    /// Mock key data
    keys: Vec<GpgKeyInfo>,
    /// Mock behavior flags
    should_fail: HashMap<String, bool>,
    /// Mock return values
    return_values: HashMap<String, String>,
}

impl MockGpgWrapper {
    /// Create a new MockGpgWrapper.
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            should_fail: HashMap::new(),
            return_values: HashMap::new(),
        }
    }

    /// Add a mock key to the wrapper.
    pub fn add_key(&mut self, key: GpgKeyInfo) {
        self.keys.push(key);
    }

    /// Set whether a specific operation should fail.
    pub fn set_should_fail(&mut self, operation: &str, should_fail: bool) {
        self.should_fail.insert(operation.to_string(), should_fail);
    }

    /// Set a return value for a specific operation.
    pub fn set_return_value(&mut self, operation: &str, value: &str) {
        self.return_values.insert(operation.to_string(), value.to_string());
    }

    /// Helper to check if an operation should fail.
    fn check_should_fail(&self, operation: &str) -> bool {
        self.should_fail.get(operation).copied().unwrap_or(false)
    }

    /// Helper to get a return value for an operation.
    fn get_return_value(&self, operation: &str) -> Option<String> {
        self.return_values.get(operation).cloned()
    }
}

impl Default for MockGpgWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl GpgWrapper for MockGpgWrapper {
    fn list_keys(&self) -> Result<Vec<GpgKeyInfo>> {
        if self.check_should_fail("list_keys") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --list-keys".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.keys.clone())
    }

    fn list_secret_keys(&self) -> Result<Vec<GpgKeyInfo>> {
        if self.check_should_fail("list_secret_keys") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --list-secret-keys".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        // Filter to only secret keys (for now, return all)
        Ok(self.keys.clone())
    }

    fn get_key_info(&self, key_id: &str) -> Result<Option<GpgKeyInfo>> {
        if self.check_should_fail("get_key_info") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --list-keys".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        let key = self.keys.iter()
            .find(|k| k.key_id == key_id || k.fingerprint == key_id)
            .cloned();
        Ok(key)
    }

    fn import_key_from_file(&self, _key_file: &str) -> Result<String> {
        if self.check_should_fail("import_key_from_file") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --import".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("import_key_from_file")
            .unwrap_or_else(|| "Mock key imported successfully".to_string()))
    }

    fn import_key_from_data(&self, _key_data: &str) -> Result<String> {
        if self.check_should_fail("import_key_from_data") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --import".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("import_key_from_data")
            .unwrap_or_else(|| "Mock key imported successfully".to_string()))
    }

    fn import_key_with_passphrase(&self, _key_data: &str, _passphrase: &str) -> Result<String> {
        if self.check_should_fail("import_key_with_passphrase") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --import".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("import_key_with_passphrase")
            .unwrap_or_else(|| "Mock key imported successfully".to_string()))
    }

    fn export_public_key(&self, _key_id: &str) -> Result<String> {
        if self.check_should_fail("export_public_key") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --export".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("export_public_key")
            .unwrap_or_else(|| "-----BEGIN PGP PUBLIC KEY BLOCK-----\nMock public key\n-----END PGP PUBLIC KEY BLOCK-----".to_string()))
    }

    fn export_private_key(&self, _key_id: &str, _passphrase: Option<&str>) -> Result<String> {
        if self.check_should_fail("export_private_key") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --export-secret-keys".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("export_private_key")
            .unwrap_or_else(|| "-----BEGIN PGP MOCK KEY BLOCK-----\nMock test key\n-----END PGP MOCK KEY BLOCK-----".to_string()))
    }

    fn generate_key(&self, _params: GpgKeyGenParams) -> Result<String> {
        if self.check_should_fail("generate_key") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --gen-key".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("generate_key")
            .unwrap_or_else(|| "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234".to_string()))
    }

    fn validate_key(&self, _key_data: &str) -> Result<bool> {
        if self.check_should_fail("validate_key") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --show-keys".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(true)
    }

    fn extract_fingerprint(&self, _key_data: &str) -> Result<String> {
        if self.check_should_fail("extract_fingerprint") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --show-keys".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("extract_fingerprint")
            .unwrap_or_else(|| "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234".to_string()))
    }

    fn delete_key(&self, _key_id: &str, _delete_secret: bool) -> Result<()> {
        if self.check_should_fail("delete_key") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --delete-keys".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(())
    }

    fn sign_data(&self, _data: &str, _key_id: &str, _passphrase: Option<&str>) -> Result<String> {
        if self.check_should_fail("sign_data") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --sign".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(self.get_return_value("sign_data")
            .unwrap_or_else(|| "-----BEGIN PGP SIGNATURE-----\nMock signature\n-----END PGP SIGNATURE-----".to_string()))
    }

    fn verify_signature(&self, _data: &str, _signature: &str) -> Result<bool> {
        if self.check_should_fail("verify_signature") {
            return Err(GitSetupError::ExternalCommand {
                command: "gpg --verify".to_string(),
                error: "Mock failure".to_string(),
            });
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_key() -> GpgKeyInfo {
        GpgKeyInfo {
            key_id: "ABCD1234".to_string(),
            fingerprint: "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234".to_string(),
            user_id: "Test User <test@example.com>".to_string(),
            key_type: "RSA".to_string(),
            key_size: Some(2048),
            creation_date: "2023-01-01".to_string(),
            expiration_date: None,
            trust_level: "ultimate".to_string(),
        }
    }

    fn create_test_gen_params() -> GpgKeyGenParams {
        GpgKeyGenParams {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            comment: Some("Test key".to_string()),
            key_type: "RSA".to_string(),
            key_size: 2048,
            expiration_days: 0,
            passphrase: Some("test_passphrase".to_string()),
        }
    }

    #[test]
    fn test_gpg_key_info_creation() {
        let key = create_test_key();
        assert_eq!(key.key_id, "ABCD1234");
        assert_eq!(key.fingerprint, "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234");
        assert_eq!(key.user_id, "Test User <test@example.com>");
        assert_eq!(key.key_type, "RSA");
        assert_eq!(key.key_size, Some(2048));
        assert_eq!(key.creation_date, "2023-01-01");
        assert_eq!(key.expiration_date, None);
        assert_eq!(key.trust_level, "ultimate");
    }

    #[test]
    fn test_gpg_key_gen_params_creation() {
        let params = create_test_gen_params();
        assert_eq!(params.name, "Test User");
        assert_eq!(params.email, "test@example.com");
        assert_eq!(params.comment, Some("Test key".to_string()));
        assert_eq!(params.key_type, "RSA");
        assert_eq!(params.key_size, 2048);
        assert_eq!(params.expiration_days, 0);
        assert_eq!(params.passphrase, Some("test_passphrase".to_string()));
    }

    #[test]
    fn test_system_gpg_wrapper_creation() {
        let wrapper = SystemGpgWrapper::new();
        assert_eq!(wrapper.gpg_path, "gpg");
    }

    #[test]
    fn test_system_gpg_wrapper_with_path() {
        let wrapper = SystemGpgWrapper::with_path("/usr/local/bin/gpg".to_string());
        assert_eq!(wrapper.gpg_path, "/usr/local/bin/gpg");
    }

    #[test]
    fn test_system_gpg_wrapper_default() {
        let wrapper = SystemGpgWrapper::default();
        assert_eq!(wrapper.gpg_path, "gpg");
    }

    // Tests with non-existent GPG binary
    #[test]
    fn test_system_gpg_wrapper_list_keys_fails_with_bad_path() {
        let wrapper = SystemGpgWrapper::with_path("/nonexistent/gpg".to_string());
        let result = wrapper.list_keys();
        assert!(result.is_err());
        if let Err(GitSetupError::ExternalCommand { command, .. }) = result {
            assert!(command.contains("/nonexistent/gpg --list-keys"));
        } else {
            panic!("Expected ExternalCommand error");
        }
    }

    #[test]
    fn test_system_gpg_wrapper_import_key_from_file_fails_with_nonexistent_file() {
        let wrapper = SystemGpgWrapper::new();
        let result = wrapper.import_key_from_file("/nonexistent/file.asc");
        assert!(result.is_err());
    }

    #[test]
    fn test_system_gpg_wrapper_generate_key_fails_with_bad_path() {
        let wrapper = SystemGpgWrapper::with_path("/nonexistent/gpg".to_string());
        let params = create_test_gen_params();
        let result = wrapper.generate_key(params);
        assert!(result.is_err());
    }

    // Mock wrapper tests
    #[test]
    fn test_mock_gpg_wrapper_creation() {
        let wrapper = MockGpgWrapper::new();
        assert!(wrapper.keys.is_empty());
        assert!(wrapper.should_fail.is_empty());
        assert!(wrapper.return_values.is_empty());
    }

    #[test]
    fn test_mock_gpg_wrapper_add_key() {
        let mut wrapper = MockGpgWrapper::new();
        let key = create_test_key();
        wrapper.add_key(key.clone());
        assert_eq!(wrapper.keys.len(), 1);
        assert_eq!(wrapper.keys[0], key);
    }

    #[test]
    fn test_mock_gpg_wrapper_list_keys_success() {
        let mut wrapper = MockGpgWrapper::new();
        let key = create_test_key();
        wrapper.add_key(key.clone());

        let result = wrapper.list_keys();
        assert!(result.is_ok());
        let keys = result.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], key);
    }

    #[test]
    fn test_mock_gpg_wrapper_list_keys_failure() {
        let mut wrapper = MockGpgWrapper::new();
        wrapper.set_should_fail("list_keys", true);

        let result = wrapper.list_keys();
        assert!(result.is_err());
        if let Err(GitSetupError::ExternalCommand { command, error }) = result {
            assert_eq!(command, "gpg --list-keys");
            assert_eq!(error, "Mock failure");
        } else {
            panic!("Expected ExternalCommand error");
        }
    }

    #[test]
    fn test_mock_gpg_wrapper_get_key_info_by_id() {
        let mut wrapper = MockGpgWrapper::new();
        let key = create_test_key();
        wrapper.add_key(key.clone());

        let result = wrapper.get_key_info("ABCD1234");
        assert!(result.is_ok());
        let found_key = result.unwrap();
        assert!(found_key.is_some());
        assert_eq!(found_key.unwrap(), key);
    }

    #[test]
    fn test_mock_gpg_wrapper_get_key_info_by_fingerprint() {
        let mut wrapper = MockGpgWrapper::new();
        let key = create_test_key();
        wrapper.add_key(key.clone());

        let result = wrapper.get_key_info("ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234");
        assert!(result.is_ok());
        let found_key = result.unwrap();
        assert!(found_key.is_some());
        assert_eq!(found_key.unwrap(), key);
    }

    #[test]
    fn test_mock_gpg_wrapper_get_key_info_not_found() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.get_key_info("NOTFOUND");
        assert!(result.is_ok());
        let found_key = result.unwrap();
        assert!(found_key.is_none());
    }

    #[test]
    fn test_mock_gpg_wrapper_import_key_from_data_success() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.import_key_from_data("mock key data");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock key imported successfully");
    }

    #[test]
    fn test_mock_gpg_wrapper_import_key_from_data_custom_return() {
        let mut wrapper = MockGpgWrapper::new();
        wrapper.set_return_value("import_key_from_data", "Custom import message");

        let result = wrapper.import_key_from_data("mock key data");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Custom import message");
    }

    #[test]
    fn test_mock_gpg_wrapper_export_public_key() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.export_public_key("ABCD1234");
        assert!(result.is_ok());
        let exported = result.unwrap();
        assert!(exported.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----"));
        assert!(exported.contains("-----END PGP PUBLIC KEY BLOCK-----"));
    }

    #[test]
    fn test_mock_gpg_wrapper_generate_key_success() {
        let wrapper = MockGpgWrapper::new();
        let params = create_test_gen_params();

        let result = wrapper.generate_key(params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234");
    }

    #[test]
    fn test_mock_gpg_wrapper_validate_key() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.validate_key("mock key data");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_mock_gpg_wrapper_extract_fingerprint() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.extract_fingerprint("mock key data");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234");
    }

    #[test]
    fn test_mock_gpg_wrapper_delete_key() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.delete_key("ABCD1234", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_gpg_wrapper_sign_data() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.sign_data("test data", "ABCD1234", Some("passphrase"));
        assert!(result.is_ok());
        let signature = result.unwrap();
        assert!(signature.contains("-----BEGIN PGP SIGNATURE-----"));
        assert!(signature.contains("-----END PGP SIGNATURE-----"));
    }

    #[test]
    fn test_mock_gpg_wrapper_verify_signature() {
        let wrapper = MockGpgWrapper::new();

        let result = wrapper.verify_signature("test data", "mock signature");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_parse_gpg_key_listing() {
        let wrapper = SystemGpgWrapper::new();

        // Mock GPG output in colon format
        let gpg_output = "\
pub:u:2048:1:ABCD1234EFGH5678:1577836800:::-:::scESC::::::23::0:
fpr:::::::::ABCD1234EFGH5678ABCD1234EFGH5678ABCD1234:
uid:u::::1577836800::A1B2C3D4E5F6::Test User <test@example.com>::::::::::0:
sub:u:2048:1:1234ABCD5678EFGH:1577836800::::::e::::::23:
fpr:::::::::1234ABCD5678EFGH1234ABCD5678EFGH1234ABCD:
";

        let result = wrapper.parse_key_listing(gpg_output);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.len(), 1);

        let key = &keys[0];
        assert_eq!(key.key_id, "ABCD1234EFGH5678");
        assert_eq!(key.fingerprint, "ABCD1234EFGH5678ABCD1234EFGH5678ABCD1234");
        assert_eq!(key.user_id, "Test User <test@example.com>");
        assert_eq!(key.key_type, "RSA");
        assert_eq!(key.key_size, Some(2048));
        assert_eq!(key.trust_level, "ultimate");
    }

    #[test]
    fn test_parse_empty_gpg_key_listing() {
        let wrapper = SystemGpgWrapper::new();

        let result = wrapper.parse_key_listing("");
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[test]
    fn test_parse_multiple_gpg_keys() {
        let wrapper = SystemGpgWrapper::new();

        // Mock GPG output with multiple keys
        let gpg_output = "\
pub:u:2048:1:AAAA1111BBBB2222:1577836800:::-:::scESC::::::23::0:
fpr:::::::::AAAA1111BBBB2222AAAA1111BBBB2222AAAA1111:
uid:u::::1577836800::A1B2C3D4E5F6::First User <first@example.com>::::::::::0:
pub:f:4096:1:CCCC3333DDDD4444:1577836800:::-:::scESC::::::23::0:
fpr:::::::::CCCC3333DDDD4444CCCC3333DDDD4444CCCC3333:
uid:f::::1577836800::E5F6A1B2C3D4::Second User <second@example.com>::::::::::0:
";

        let result = wrapper.parse_key_listing(gpg_output);
        assert!(result.is_ok());

        let keys = result.unwrap();
        assert_eq!(keys.len(), 2);

        assert_eq!(keys[0].key_id, "AAAA1111BBBB2222");
        assert_eq!(keys[0].user_id, "First User <first@example.com>");
        assert_eq!(keys[0].trust_level, "ultimate");

        assert_eq!(keys[1].key_id, "CCCC3333DDDD4444");
        assert_eq!(keys[1].user_id, "Second User <second@example.com>");
        assert_eq!(keys[1].trust_level, "full");
    }
}
