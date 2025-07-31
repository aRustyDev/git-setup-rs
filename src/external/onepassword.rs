//! 1Password CLI wrapper for git-setup-rs.
//!
//! This module provides a trait-based abstraction for 1Password CLI operations,
//! allowing for easy testing through mock implementations while providing a real
//! implementation that uses std::process::Command to execute op commands.

use crate::error::{GitSetupError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// Represents a 1Password vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: String,
    pub name: String,
}

/// Represents a 1Password SSH key item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyItem {
    pub id: String,
    pub title: String,
    pub vault: Vault,
    pub category: String,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
}

/// Represents a 1Password GPG key item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeyItem {
    pub id: String,
    pub title: String,
    pub vault: Vault,
    pub category: String,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub passphrase: Option<String>,
}

/// Template for creating GPG items in 1Password.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgItemTemplate {
    pub title: String,
    pub vault: String,
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
}

/// Trait for 1Password CLI operations.
///
/// This trait allows for easy testing by providing a mock implementation
/// while keeping the real implementation using system op commands.
pub trait OnePasswordWrapper {
    /// Check if the user is authenticated with 1Password.
    /// Uses `op whoami` to verify authentication status.
    fn is_authenticated(&self) -> Result<bool>;

    /// Get the current authenticated user information.
    fn whoami(&self) -> Result<String>;

    /// List all available vaults.
    fn list_vaults(&self) -> Result<Vec<Vault>>;

    /// List SSH key items in a specific vault.
    fn list_ssh_keys(&self, vault_name: Option<&str>) -> Result<Vec<SshKeyItem>>;

    /// Get a specific SSH key by title from a vault.
    fn get_ssh_key(&self, title: &str, vault_name: &str) -> Result<Option<SshKeyItem>>;

    /// Get the public key content for an SSH key item.
    fn get_ssh_public_key(&self, item_id: &str) -> Result<String>;

    /// List GPG key items with the "gpg" tag.
    fn list_gpg_keys(&self, vault_name: Option<&str>) -> Result<Vec<GpgKeyItem>>;

    /// Get a specific GPG key by title from a vault.
    fn get_gpg_key(&self, title: &str, vault_name: &str) -> Result<Option<GpgKeyItem>>;

    /// Create a new GPG item in 1Password using the custom JSON structure.
    fn create_gpg_item(&self, template: &GpgItemTemplate) -> Result<String>;

    /// Create a new SSH key in 1Password.
    fn create_ssh_key(&self, title: &str, vault_name: &str) -> Result<String>;

    /// Read a specific field from an item using op:// syntax.
    fn read_field(&self, reference: &str) -> Result<String>;

    /// Update a GPG item with new key data.
    fn update_gpg_item(&self, item_id: &str, template: &GpgItemTemplate) -> Result<()>;
}

/// Real implementation of OnePasswordWrapper using std::process::Command.
pub struct SystemOnePasswordWrapper;

impl SystemOnePasswordWrapper {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemOnePasswordWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl OnePasswordWrapper for SystemOnePasswordWrapper {
    fn is_authenticated(&self) -> Result<bool> {
        match self.whoami() {
            Ok(_) => Ok(true),
            Err(GitSetupError::OnePassword(msg)) if msg.contains("not authenticated") => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn whoami(&self) -> Result<String> {
        let mut cmd = Command::new("op");
        cmd.arg("whoami");

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if user.is_empty() {
                        Err(GitSetupError::OnePassword("empty response from op whoami".to_string()))
                    } else {
                        Ok(user)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op whoami failed: {}", stderr)))
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(GitSetupError::ExternalCommand {
                        command: "op".to_string(),
                        error: "1Password CLI not found. Please install the op CLI tool.".to_string(),
                    })
                } else {
                    Err(GitSetupError::ExternalCommand {
                        command: "op whoami".to_string(),
                        error: e.to_string(),
                    })
                }
            }
        }
    }

    fn list_vaults(&self) -> Result<Vec<Vault>> {
        let mut cmd = Command::new("op");
        cmd.args(["vault", "list", "--format=json"]);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let vault_json = String::from_utf8_lossy(&output.stdout);
                    let vaults: Vec<Vault> = serde_json::from_str(&vault_json)
                        .map_err(GitSetupError::Json)?;
                    Ok(vaults)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op vault list failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: "op vault list --format=json".to_string(),
                error: e.to_string(),
            }),
        }
    }

    fn list_ssh_keys(&self, vault_name: Option<&str>) -> Result<Vec<SshKeyItem>> {
        let mut cmd = Command::new("op");
        cmd.args(["item", "list", "--categories", "SSH Key", "--format=json"]);

        if let Some(vault) = vault_name {
            cmd.args(["--vault", vault]);
        }

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let items_json = String::from_utf8_lossy(&output.stdout);
                    let items: Vec<SshKeyItem> = serde_json::from_str(&items_json)
                        .map_err(GitSetupError::Json)?;
                    Ok(items)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op item list failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: format!("op item list --categories \"SSH Key\" --format=json{}",
                    vault_name.map(|v| format!(" --vault {}", v)).unwrap_or_default()),
                error: e.to_string(),
            }),
        }
    }

    fn get_ssh_key(&self, title: &str, vault_name: &str) -> Result<Option<SshKeyItem>> {
        let ssh_keys = self.list_ssh_keys(Some(vault_name))?;
        Ok(ssh_keys.into_iter().find(|key| key.title == title))
    }

    fn get_ssh_public_key(&self, item_id: &str) -> Result<String> {
        let mut cmd = Command::new("op");
        cmd.args(["item", "get", item_id, "--fields", "public key"]);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let public_key = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if public_key.is_empty() {
                        Err(GitSetupError::OnePassword("no public key found".to_string()))
                    } else {
                        Ok(public_key)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else if stderr.contains("not found") {
                        Err(GitSetupError::OnePassword("SSH key not found".to_string()))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op item get failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: format!("op item get {} --fields \"public key\"", item_id),
                error: e.to_string(),
            }),
        }
    }

    fn list_gpg_keys(&self, vault_name: Option<&str>) -> Result<Vec<GpgKeyItem>> {
        let mut cmd = Command::new("op");
        cmd.args(["item", "list", "--tags", "gpg", "--format=json"]);

        if let Some(vault) = vault_name {
            cmd.args(["--vault", vault]);
        }

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let items_json = String::from_utf8_lossy(&output.stdout);
                    let items: Vec<GpgKeyItem> = serde_json::from_str(&items_json)
                        .map_err(GitSetupError::Json)?;
                    Ok(items)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op item list failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: format!("op item list --tags gpg --format=json{}",
                    vault_name.map(|v| format!(" --vault {}", v)).unwrap_or_default()),
                error: e.to_string(),
            }),
        }
    }

    fn get_gpg_key(&self, title: &str, vault_name: &str) -> Result<Option<GpgKeyItem>> {
        let gpg_keys = self.list_gpg_keys(Some(vault_name))?;
        Ok(gpg_keys.into_iter().find(|key| key.title == title))
    }

    fn create_gpg_item(&self, template: &GpgItemTemplate) -> Result<String> {
        // Create the GPG item JSON structure as defined in CONTEXT.md
        let gpg_item = serde_json::json!({
            "title": template.title,
            "category": "Password",
            "tags": ["gpg"],
            "vault": template.vault,
            "sections": [
                {
                    "id": "pub",
                    "label": "Public"
                },
                {
                    "id": "priv",
                    "label": "Private"
                }
            ],
            "fields": [
                {
                    "id": "key",
                    "section": {"id": "pub"},
                    "type": "text",
                    "label": "key",
                    "value": template.public_key
                },
                {
                    "id": "pw",
                    "section": {"id": "priv"},
                    "type": "password",
                    "label": "password",
                    "value": template.passphrase
                },
                {
                    "id": "key",
                    "section": {"id": "priv"},
                    "type": "password",
                    "label": "key",
                    "value": template.private_key
                }
            ]
        });

        // Write the template to a temporary file
        let template_json = serde_json::to_string_pretty(&gpg_item)
            .map_err(GitSetupError::Json)?;

        let temp_file = std::env::temp_dir().join(format!("gpg-template-{}.json",
            std::process::id()));
        std::fs::write(&temp_file, template_json)
            .map_err(GitSetupError::Io)?;

        let mut cmd = Command::new("op");
        cmd.args([
            "item", "create",
            "--template", temp_file.to_str().unwrap(),
            "--vault", &template.vault,
            "--title", &template.title,
            "--category", "Password",
            "--tags", "gpg"
        ]);

        let result = cmd.output();

        // Clean up the temporary file
        let _ = std::fs::remove_file(&temp_file);

        match result {
            Ok(output) => {
                if output.status.success() {
                    let item_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if item_id.is_empty() {
                        Err(GitSetupError::OnePassword("empty item ID from op item create".to_string()))
                    } else {
                        Ok(item_id)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op item create failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: "op item create --template <file> --vault <vault> --title <title> --category Password --tags gpg".to_string(),
                error: e.to_string(),
            }),
        }
    }

    fn create_ssh_key(&self, title: &str, vault_name: &str) -> Result<String> {
        let mut cmd = Command::new("op");
        cmd.args([
            "item", "create",
            "--ssh-generate-key",
            "--vault", vault_name,
            "--title", title,
            "--category", "SSH Key"
        ]);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let item_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if item_id.is_empty() {
                        Err(GitSetupError::OnePassword("empty item ID from op item create".to_string()))
                    } else {
                        Ok(item_id)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else if stderr.contains("not found") && stderr.contains("vault") {
                        Err(GitSetupError::OnePassword(format!("vault '{}' not found", vault_name)))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op item create failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: format!("op item create --ssh-generate-key --vault {} --title {}", vault_name, title),
                error: e.to_string(),
            }),
        }
    }

    fn read_field(&self, reference: &str) -> Result<String> {
        let mut cmd = Command::new("op");
        cmd.args(["read", reference]);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if value.is_empty() {
                        Err(GitSetupError::OnePassword("empty value from op read".to_string()))
                    } else {
                        Ok(value)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else if stderr.contains("not found") {
                        Err(GitSetupError::OnePassword(format!("field '{}' not found", reference)))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op read failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: format!("op read {}", reference),
                error: e.to_string(),
            }),
        }
    }

    fn update_gpg_item(&self, item_id: &str, template: &GpgItemTemplate) -> Result<()> {
        // For updates, we use op item edit to modify specific fields
        let mut cmd = Command::new("op");
        cmd.args([
            "item", "edit", item_id,
            "--title", &template.title,
            &format!("pub.key={}", template.public_key),
            &format!("priv.password={}", template.passphrase),
            &format!("priv.key={}", template.private_key),
        ]);

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if stderr.contains("not currently signed in") || stderr.contains("not authenticated") {
                        Err(GitSetupError::OnePassword("not authenticated".to_string()))
                    } else if stderr.contains("not found") {
                        Err(GitSetupError::OnePassword(format!("item '{}' not found", item_id)))
                    } else {
                        Err(GitSetupError::OnePassword(format!("op item edit failed: {}", stderr)))
                    }
                }
            }
            Err(e) => Err(GitSetupError::ExternalCommand {
                command: format!("op item edit {} --title <title> <field updates>", item_id),
                error: e.to_string(),
            }),
        }
    }
}

/// Mock implementation of OnePasswordWrapper for testing.
pub struct MockOnePasswordWrapper {
    authenticated: bool,
    should_fail: bool,
    vaults: Vec<Vault>,
    ssh_keys: Vec<SshKeyItem>,
    gpg_keys: Vec<GpgKeyItem>,
    field_values: HashMap<String, String>,
}

impl MockOnePasswordWrapper {
    pub fn new() -> Self {
        Self {
            authenticated: true,
            should_fail: false,
            vaults: vec![
                Vault {
                    id: "vault1".to_string(),
                    name: "Personal".to_string(),
                },
                Vault {
                    id: "vault2".to_string(),
                    name: "Work".to_string(),
                },
            ],
            ssh_keys: Vec::new(),
            gpg_keys: Vec::new(),
            field_values: HashMap::new(),
        }
    }

    pub fn set_ssh_keys(&mut self, ssh_keys: Vec<SshKeyItem>) {
        self.ssh_keys = ssh_keys;
    }

    /// Configure the mock to fail operations.
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    /// Configure authentication status.
    pub fn with_authenticated(mut self, authenticated: bool) -> Self {
        self.authenticated = authenticated;
        self
    }

    /// Pre-populate with SSH keys.
    pub fn with_ssh_keys(mut self, ssh_keys: Vec<SshKeyItem>) -> Self {
        self.ssh_keys = ssh_keys;
        self
    }

    /// Pre-populate with GPG keys.
    pub fn with_gpg_keys(mut self, gpg_keys: Vec<GpgKeyItem>) -> Self {
        self.gpg_keys = gpg_keys;
        self
    }

    /// Pre-populate with field values for op read operations.
    pub fn with_field_values(mut self, field_values: HashMap<String, String>) -> Self {
        self.field_values = field_values;
        self
    }

    /// Pre-populate with vaults.
    pub fn with_vaults(mut self, vaults: Vec<Vault>) -> Self {
        self.vaults = vaults;
        self
    }
}

impl Default for MockOnePasswordWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl OnePasswordWrapper for MockOnePasswordWrapper {
    fn is_authenticated(&self) -> Result<bool> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        Ok(self.authenticated)
    }

    fn whoami(&self) -> Result<String> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }
        Ok("test-user@example.com".to_string())
    }

    fn list_vaults(&self) -> Result<Vec<Vault>> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }
        Ok(self.vaults.clone())
    }

    fn list_ssh_keys(&self, vault_name: Option<&str>) -> Result<Vec<SshKeyItem>> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        if let Some(vault) = vault_name {
            Ok(self
                .ssh_keys
                .iter()
                .filter(|key| key.vault.name == vault)
                .cloned()
                .collect())
        } else {
            Ok(self.ssh_keys.clone())
        }
    }

    fn get_ssh_key(&self, title: &str, vault_name: &str) -> Result<Option<SshKeyItem>> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        Ok(self
            .ssh_keys
            .iter()
            .find(|key| key.title == title && key.vault.name == vault_name)
            .cloned())
    }

    fn get_ssh_public_key(&self, item_id: &str) -> Result<String> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        if let Some(key) = self.ssh_keys.iter().find(|k| k.id == item_id) {
            if let Some(ref public_key) = key.public_key {
                Ok(public_key.clone())
            } else {
                Err(GitSetupError::OnePassword("no public key found".to_string()))
            }
        } else {
            Err(GitSetupError::OnePassword("SSH key not found".to_string()))
        }
    }

    fn list_gpg_keys(&self, vault_name: Option<&str>) -> Result<Vec<GpgKeyItem>> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        if let Some(vault) = vault_name {
            Ok(self
                .gpg_keys
                .iter()
                .filter(|key| key.vault.name == vault)
                .cloned()
                .collect())
        } else {
            Ok(self.gpg_keys.clone())
        }
    }

    fn get_gpg_key(&self, title: &str, vault_name: &str) -> Result<Option<GpgKeyItem>> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        Ok(self
            .gpg_keys
            .iter()
            .find(|key| key.title == title && key.vault.name == vault_name)
            .cloned())
    }

    fn create_gpg_item(&self, template: &GpgItemTemplate) -> Result<String> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        // Return a mock item ID
        Ok(format!("gpg-item-{}", template.title.replace(' ', "-").to_lowercase()))
    }

    fn create_ssh_key(&self, title: &str, vault_name: &str) -> Result<String> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        // Check if vault exists
        if !self.vaults.iter().any(|v| v.name == vault_name) {
            return Err(GitSetupError::OnePassword(format!("vault '{}' not found", vault_name)));
        }

        // Return a mock item ID
        Ok(format!("ssh-key-{}", title.replace(' ', "-").to_lowercase()))
    }

    fn read_field(&self, reference: &str) -> Result<String> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        self.field_values
            .get(reference)
            .cloned()
            .ok_or_else(|| GitSetupError::OnePassword(format!("field '{}' not found", reference)))
    }

    fn update_gpg_item(&self, _item_id: &str, _template: &GpgItemTemplate) -> Result<()> {
        if self.should_fail {
            return Err(GitSetupError::OnePassword("Mock 1Password failure".to_string()));
        }
        if !self.authenticated {
            return Err(GitSetupError::OnePassword("not authenticated".to_string()));
        }

        // Mock successful update
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test authentication functionality
    #[test]
    fn test_mock_is_authenticated_success() {
        let wrapper = MockOnePasswordWrapper::new();
        let result = wrapper.is_authenticated().unwrap();
        assert!(result);
    }

    #[test]
    fn test_mock_is_authenticated_not_authenticated() {
        let wrapper = MockOnePasswordWrapper::new().with_authenticated(false);
        let result = wrapper.is_authenticated().unwrap();
        assert!(!result);
    }

    #[test]
    fn test_mock_is_authenticated_failure() {
        let wrapper = MockOnePasswordWrapper::new().with_failure();
        let result = wrapper.is_authenticated();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    #[test]
    fn test_mock_whoami_success() {
        let wrapper = MockOnePasswordWrapper::new();
        let result = wrapper.whoami().unwrap();
        assert_eq!(result, "test-user@example.com");
    }

    #[test]
    fn test_mock_whoami_not_authenticated() {
        let wrapper = MockOnePasswordWrapper::new().with_authenticated(false);
        let result = wrapper.whoami();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    #[test]
    fn test_mock_whoami_failure() {
        let wrapper = MockOnePasswordWrapper::new().with_failure();
        let result = wrapper.whoami();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    // Test vault operations
    #[test]
    fn test_mock_list_vaults_success() {
        let wrapper = MockOnePasswordWrapper::new();
        let vaults = wrapper.list_vaults().unwrap();
        assert_eq!(vaults.len(), 2);
        assert_eq!(vaults[0].name, "Personal");
        assert_eq!(vaults[1].name, "Work");
    }

    #[test]
    fn test_mock_list_vaults_not_authenticated() {
        let wrapper = MockOnePasswordWrapper::new().with_authenticated(false);
        let result = wrapper.list_vaults();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    #[test]
    fn test_mock_list_vaults_failure() {
        let wrapper = MockOnePasswordWrapper::new().with_failure();
        let result = wrapper.list_vaults();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    // Test SSH key operations
    #[test]
    fn test_mock_list_ssh_keys_empty() {
        let wrapper = MockOnePasswordWrapper::new();
        let keys = wrapper.list_ssh_keys(None).unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_mock_list_ssh_keys_with_data() {
        let ssh_keys = vec![
            SshKeyItem {
                id: "ssh1".to_string(),
                title: "Work SSH Key".to_string(),
                vault: Vault {
                    id: "vault2".to_string(),
                    name: "Work".to_string(),
                },
                category: "SSH Key".to_string(),
                public_key: Some("ssh-ed25519 AAAAC3...work".to_string()),
                private_key: None,
            },
            SshKeyItem {
                id: "ssh2".to_string(),
                title: "Personal SSH Key".to_string(),
                vault: Vault {
                    id: "vault1".to_string(),
                    name: "Personal".to_string(),
                },
                category: "SSH Key".to_string(),
                public_key: Some("ssh-ed25519 AAAAC3...personal".to_string()),
                private_key: None,
            },
        ];

        let wrapper = MockOnePasswordWrapper::new().with_ssh_keys(ssh_keys);

        // Test listing all SSH keys
        let all_keys = wrapper.list_ssh_keys(None).unwrap();
        assert_eq!(all_keys.len(), 2);

        // Test filtering by vault
        let work_keys = wrapper.list_ssh_keys(Some("Work")).unwrap();
        assert_eq!(work_keys.len(), 1);
        assert_eq!(work_keys[0].title, "Work SSH Key");

        let personal_keys = wrapper.list_ssh_keys(Some("Personal")).unwrap();
        assert_eq!(personal_keys.len(), 1);
        assert_eq!(personal_keys[0].title, "Personal SSH Key");

        // Test vault that doesn't exist
        let empty_keys = wrapper.list_ssh_keys(Some("NonExistent")).unwrap();
        assert!(empty_keys.is_empty());
    }

    #[test]
    fn test_mock_get_ssh_key_found() {
        let ssh_keys = vec![SshKeyItem {
            id: "ssh1".to_string(),
            title: "Work SSH Key".to_string(),
            vault: Vault {
                id: "vault2".to_string(),
                name: "Work".to_string(),
            },
            category: "SSH Key".to_string(),
            public_key: Some("ssh-ed25519 AAAAC3...work".to_string()),
            private_key: None,
        }];

        let wrapper = MockOnePasswordWrapper::new().with_ssh_keys(ssh_keys);
        let key = wrapper.get_ssh_key("Work SSH Key", "Work").unwrap();
        assert!(key.is_some());
        assert_eq!(key.unwrap().title, "Work SSH Key");
    }

    #[test]
    fn test_mock_get_ssh_key_not_found() {
        let wrapper = MockOnePasswordWrapper::new();
        let key = wrapper.get_ssh_key("NonExistent", "Work").unwrap();
        assert!(key.is_none());
    }

    #[test]
    fn test_mock_get_ssh_public_key_success() {
        let ssh_keys = vec![SshKeyItem {
            id: "ssh1".to_string(),
            title: "Work SSH Key".to_string(),
            vault: Vault {
                id: "vault2".to_string(),
                name: "Work".to_string(),
            },
            category: "SSH Key".to_string(),
            public_key: Some("ssh-ed25519 AAAAC3...work".to_string()),
            private_key: None,
        }];

        let wrapper = MockOnePasswordWrapper::new().with_ssh_keys(ssh_keys);
        let public_key = wrapper.get_ssh_public_key("ssh1").unwrap();
        assert_eq!(public_key, "ssh-ed25519 AAAAC3...work");
    }

    #[test]
    fn test_mock_get_ssh_public_key_not_found() {
        let wrapper = MockOnePasswordWrapper::new();
        let result = wrapper.get_ssh_public_key("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    #[test]
    fn test_mock_get_ssh_public_key_no_public_key() {
        let ssh_keys = vec![SshKeyItem {
            id: "ssh1".to_string(),
            title: "Work SSH Key".to_string(),
            vault: Vault {
                id: "vault2".to_string(),
                name: "Work".to_string(),
            },
            category: "SSH Key".to_string(),
            public_key: None, // No public key
            private_key: None,
        }];

        let wrapper = MockOnePasswordWrapper::new().with_ssh_keys(ssh_keys);
        let result = wrapper.get_ssh_public_key("ssh1");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    // Test GPG key operations
    #[test]
    fn test_mock_list_gpg_keys_empty() {
        let wrapper = MockOnePasswordWrapper::new();
        let keys = wrapper.list_gpg_keys(None).unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_mock_list_gpg_keys_with_data() {
        let gpg_keys = vec![
            GpgKeyItem {
                id: "gpg1".to_string(),
                title: "Work GPG Key".to_string(),
                vault: Vault {
                    id: "vault2".to_string(),
                    name: "Work".to_string(),
                },
                category: "Password".to_string(),
                public_key: Some("-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string()),
                private_key: Some("-----BEGIN PGP MOCK KEY BLOCK-----...".to_string()),
                passphrase: Some("secret-passphrase".to_string()),
            },
        ];

        let wrapper = MockOnePasswordWrapper::new().with_gpg_keys(gpg_keys);

        // Test listing all GPG keys
        let all_keys = wrapper.list_gpg_keys(None).unwrap();
        assert_eq!(all_keys.len(), 1);

        // Test filtering by vault
        let work_keys = wrapper.list_gpg_keys(Some("Work")).unwrap();
        assert_eq!(work_keys.len(), 1);
        assert_eq!(work_keys[0].title, "Work GPG Key");

        // Test vault that doesn't exist
        let empty_keys = wrapper.list_gpg_keys(Some("NonExistent")).unwrap();
        assert!(empty_keys.is_empty());
    }

    #[test]
    fn test_mock_get_gpg_key_found() {
        let gpg_keys = vec![GpgKeyItem {
            id: "gpg1".to_string(),
            title: "Work GPG Key".to_string(),
            vault: Vault {
                id: "vault2".to_string(),
                name: "Work".to_string(),
            },
            category: "Password".to_string(),
            public_key: Some("-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string()),
            private_key: Some("-----BEGIN PGP MOCK KEY BLOCK-----...".to_string()),
            passphrase: Some("secret-passphrase".to_string()),
        }];

        let wrapper = MockOnePasswordWrapper::new().with_gpg_keys(gpg_keys);
        let key = wrapper.get_gpg_key("Work GPG Key", "Work").unwrap();
        assert!(key.is_some());
        assert_eq!(key.unwrap().title, "Work GPG Key");
    }

    #[test]
    fn test_mock_get_gpg_key_not_found() {
        let wrapper = MockOnePasswordWrapper::new();
        let key = wrapper.get_gpg_key("NonExistent", "Work").unwrap();
        assert!(key.is_none());
    }

    // Test GPG item creation
    #[test]
    fn test_mock_create_gpg_item_success() {
        let wrapper = MockOnePasswordWrapper::new();
        let template = GpgItemTemplate {
            title: "Test GPG Key".to_string(),
            vault: "Personal".to_string(),
            public_key: "-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string(),
            private_key: "-----BEGIN PGP MOCK KEY BLOCK-----...".to_string(),
            passphrase: "secret-passphrase".to_string(),
        };

        let item_id = wrapper.create_gpg_item(&template).unwrap();
        assert_eq!(item_id, "gpg-item-test-gpg-key");
    }

    #[test]
    fn test_mock_create_gpg_item_not_authenticated() {
        let wrapper = MockOnePasswordWrapper::new().with_authenticated(false);
        let template = GpgItemTemplate {
            title: "Test GPG Key".to_string(),
            vault: "Personal".to_string(),
            public_key: "-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string(),
            private_key: "-----BEGIN PGP MOCK KEY BLOCK-----...".to_string(),
            passphrase: "secret-passphrase".to_string(),
        };

        let result = wrapper.create_gpg_item(&template);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    // Test SSH key creation
    #[test]
    fn test_mock_create_ssh_key_success() {
        let wrapper = MockOnePasswordWrapper::new();
        let item_id = wrapper.create_ssh_key("Test SSH Key", "Personal").unwrap();
        assert_eq!(item_id, "ssh-key-test-ssh-key");
    }

    #[test]
    fn test_mock_create_ssh_key_vault_not_found() {
        let wrapper = MockOnePasswordWrapper::new();
        let result = wrapper.create_ssh_key("Test SSH Key", "NonExistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    // Test field reading
    #[test]
    fn test_mock_read_field_success() {
        let mut field_values = HashMap::new();
        field_values.insert("op://Personal/Test/password".to_string(), "secret-value".to_string());

        let wrapper = MockOnePasswordWrapper::new().with_field_values(field_values);
        let value = wrapper.read_field("op://Personal/Test/password").unwrap();
        assert_eq!(value, "secret-value");
    }

    #[test]
    fn test_mock_read_field_not_found() {
        let wrapper = MockOnePasswordWrapper::new();
        let result = wrapper.read_field("op://Personal/NonExistent/password");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    // Test update operations
    #[test]
    fn test_mock_update_gpg_item_success() {
        let wrapper = MockOnePasswordWrapper::new();
        let template = GpgItemTemplate {
            title: "Updated GPG Key".to_string(),
            vault: "Personal".to_string(),
            public_key: "-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string(),
            private_key: "-----BEGIN PGP MOCK KEY BLOCK-----...".to_string(),
            passphrase: "new-passphrase".to_string(),
        };

        let result = wrapper.update_gpg_item("gpg-item-1", &template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_update_gpg_item_not_authenticated() {
        let wrapper = MockOnePasswordWrapper::new().with_authenticated(false);
        let template = GpgItemTemplate {
            title: "Updated GPG Key".to_string(),
            vault: "Personal".to_string(),
            public_key: "-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string(),
            private_key: "-----BEGIN PGP MOCK KEY BLOCK-----...".to_string(),
            passphrase: "new-passphrase".to_string(),
        };

        let result = wrapper.update_gpg_item("gpg-item-1", &template);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::OnePassword(_)));
    }

    // Test mock configuration
    #[test]
    fn test_mock_with_custom_vaults() {
        let custom_vaults = vec![
            Vault {
                id: "custom1".to_string(),
                name: "Custom Vault".to_string(),
            },
        ];

        let wrapper = MockOnePasswordWrapper::new().with_vaults(custom_vaults);
        let vaults = wrapper.list_vaults().unwrap();
        assert_eq!(vaults.len(), 1);
        assert_eq!(vaults[0].name, "Custom Vault");
    }

    // Note: SystemOnePasswordWrapper integration tests are omitted from unit tests
    // because they require a real 1Password CLI installation and authentication.
    // These should be tested separately in integration tests or with environment setup.
    // The wrapper will return appropriate errors if op CLI is not available or not authenticated.

    #[test]
    fn test_system_wrapper_new_and_default() {
        let wrapper1 = SystemOnePasswordWrapper::new();
        let wrapper2 = SystemOnePasswordWrapper::default();
        // Both should be created successfully
        drop(wrapper1);
        drop(wrapper2);
    }

    // Test data structure serialization/deserialization
    #[test]
    fn test_vault_serde() {
        let vault = Vault {
            id: "vault1".to_string(),
            name: "Test Vault".to_string(),
        };

        let json = serde_json::to_string(&vault).unwrap();
        let parsed: Vault = serde_json::from_str(&json).unwrap();
        assert_eq!(vault.id, parsed.id);
        assert_eq!(vault.name, parsed.name);
    }

    #[test]
    fn test_ssh_key_item_serde() {
        let ssh_key = SshKeyItem {
            id: "ssh1".to_string(),
            title: "Test SSH Key".to_string(),
            vault: Vault {
                id: "vault1".to_string(),
                name: "Test Vault".to_string(),
            },
            category: "SSH Key".to_string(),
            public_key: Some("ssh-ed25519 AAAAC3...".to_string()),
            private_key: None,
        };

        let json = serde_json::to_string(&ssh_key).unwrap();
        let parsed: SshKeyItem = serde_json::from_str(&json).unwrap();
        assert_eq!(ssh_key.id, parsed.id);
        assert_eq!(ssh_key.title, parsed.title);
        assert_eq!(ssh_key.public_key, parsed.public_key);
    }

    #[test]
    fn test_gpg_key_item_serde() {
        let gpg_key = GpgKeyItem {
            id: "gpg1".to_string(),
            title: "Test GPG Key".to_string(),
            vault: Vault {
                id: "vault1".to_string(),
                name: "Test Vault".to_string(),
            },
            category: "Password".to_string(),
            public_key: Some("-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string()),
            private_key: Some("-----BEGIN PGP MOCK KEY BLOCK-----...".to_string()),
            passphrase: Some("secret".to_string()),
        };

        let json = serde_json::to_string(&gpg_key).unwrap();
        let parsed: GpgKeyItem = serde_json::from_str(&json).unwrap();
        assert_eq!(gpg_key.id, parsed.id);
        assert_eq!(gpg_key.title, parsed.title);
        assert_eq!(gpg_key.passphrase, parsed.passphrase);
    }

    #[test]
    fn test_gpg_item_template_serde() {
        let template = GpgItemTemplate {
            title: "Test GPG Key".to_string(),
            vault: "Personal".to_string(),
            public_key: "-----BEGIN PGP PUBLIC KEY BLOCK-----...".to_string(),
            private_key: "-----BEGIN PGP MOCK KEY BLOCK-----...".to_string(),
            passphrase: "secret-passphrase".to_string(),
        };

        let json = serde_json::to_string(&template).unwrap();
        let parsed: GpgItemTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(template.title, parsed.title);
        assert_eq!(template.vault, parsed.vault);
        assert_eq!(template.passphrase, parsed.passphrase);
    }
}
