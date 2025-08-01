# Comprehensive Mock Testing Guide for 1Password Integration

## Overview

This guide provides detailed patterns and examples for mocking the 1Password CLI integration, enabling thorough testing without requiring actual 1Password installation or credentials.

## Why Mock Testing is Critical

1. **CI/CD Integration**: Tests must run without 1Password CLI
2. **Security**: No real credentials in test environments
3. **Reproducibility**: Consistent test results
4. **Edge Cases**: Test scenarios impossible with real CLI
5. **Performance**: Mock tests run faster
6. **Development**: Work without 1Password license

## Mock Architecture

### Trait-Based Design

```rust
// src/onepassword/traits.rs

use async_trait::async_trait;
use crate::onepassword::{
    Vault, Item, ItemCategory, SshKeyInfo, AccountInfo,
    OnePasswordError, SecretReference
};

/// Trait for 1Password operations - enables mocking
#[async_trait]
pub trait OnePasswordProvider: Send + Sync {
    /// Check if authenticated
    async fn is_authenticated(&self) -> Result<bool, OnePasswordError>;
    
    /// Get account information
    async fn whoami(&self) -> Result<AccountInfo, OnePasswordError>;
    
    /// List vaults
    async fn list_vaults(&self) -> Result<Vec<Vault>, OnePasswordError>;
    
    /// List items
    async fn list_items(
        &self,
        category: Option<ItemCategory>,
        vault_id: Option<&str>,
    ) -> Result<Vec<Item>, OnePasswordError>;
    
    /// Get SSH key information
    async fn get_ssh_key_info(&self, item_id: &str) -> Result<SshKeyInfo, OnePasswordError>;
    
    /// Read a secret (returns SensitiveString)
    async fn read_secret(&self, reference: &SecretReference) -> Result<SensitiveString, OnePasswordError>;
    
    /// Create a secret reference
    fn create_reference(&self, vault: &str, item: &str, field: &str) -> SecretReference;
}

/// Reference to a secret in 1Password
#[derive(Debug, Clone, PartialEq)]
pub struct SecretReference {
    /// The op:// URL
    pub url: String,
    /// Parsed components for validation
    pub vault: String,
    pub item: String,
    pub field: String,
}

impl SecretReference {
    pub fn parse(url: &str) -> Result<Self, OnePasswordError> {
        // Parse op://vault/item/field format
        let parts: Vec<&str> = url
            .strip_prefix("op://")
            .ok_or_else(|| OnePasswordError::InvalidReference(url.to_string()))?
            .split('/')
            .collect();
        
        if parts.len() != 3 {
            return Err(OnePasswordError::InvalidReference(url.to_string()));
        }
        
        Ok(Self {
            url: url.to_string(),
            vault: parts[0].to_string(),
            item: parts[1].to_string(),
            field: parts[2].to_string(),
        })
    }
}
```

### Mock Implementation

```rust
// src/onepassword/mock.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;

/// Mock 1Password provider for testing
pub struct MockOnePasswordProvider {
    /// Whether user is authenticated
    authenticated: Arc<RwLock<bool>>,
    
    /// Mock vaults
    vaults: Arc<RwLock<Vec<Vault>>>,
    
    /// Mock items by vault ID
    items: Arc<RwLock<HashMap<String, Vec<Item>>>>,
    
    /// Mock secrets by reference
    secrets: Arc<RwLock<HashMap<String, SensitiveString>>>,
    
    /// Simulated errors
    error_triggers: Arc<RwLock<HashMap<String, OnePasswordError>>>,
    
    /// Call history for verification
    call_history: Arc<RwLock<Vec<String>>>,
}

impl MockOnePasswordProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            authenticated: Arc::new(RwLock::new(true)),
            vaults: Arc::new(RwLock::new(Vec::new())),
            items: Arc::new(RwLock::new(HashMap::new())),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            error_triggers: Arc::new(RwLock::new(HashMap::new())),
            call_history: Arc::new(RwLock::new(Vec::new())),
        };
        
        // Initialize with default test data
        provider.setup_default_data();
        provider
    }
    
    fn setup_default_data(&mut self) {
        // Add default vaults
        let personal_vault = Vault {
            id: "personal-vault-id".to_string(),
            name: "Personal".to_string(),
            vault_type: VaultType::Personal,
            created_at: Utc::now(),
        };
        
        let work_vault = Vault {
            id: "work-vault-id".to_string(),
            name: "Work".to_string(),
            vault_type: VaultType::Shared,
            created_at: Utc::now(),
        };
        
        // Add SSH keys
        let github_key = Item {
            id: "github-ssh-key".to_string(),
            title: "GitHub SSH Key".to_string(),
            category: ItemCategory::SshKey,
            vault: VaultReference {
                id: "personal-vault-id".to_string(),
                name: Some("Personal".to_string()),
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["development".to_string()],
        };
        
        // Store test data
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            self.vaults.write().await.push(personal_vault);
            self.vaults.write().await.push(work_vault);
            
            self.items.write().await
                .entry("personal-vault-id".to_string())
                .or_insert_with(Vec::new)
                .push(github_key);
            
            // Add test secret
            self.secrets.write().await.insert(
                "op://Personal/GitHub SSH Key/private key".to_string(),
                SensitiveString::from("-----BEGIN OPENSSH PRIVATE KEY-----\ntest-key-content\n-----END OPENSSH PRIVATE KEY-----")
            );
        });
    }
    
    /// Set authentication status
    pub async fn set_authenticated(&self, authenticated: bool) {
        *self.authenticated.write().await = authenticated;
    }
    
    /// Add a mock vault
    pub async fn add_vault(&self, vault: Vault) {
        self.vaults.write().await.push(vault);
    }
    
    /// Add a mock item
    pub async fn add_item(&self, vault_id: String, item: Item) {
        self.items.write().await
            .entry(vault_id)
            .or_insert_with(Vec::new)
            .push(item);
    }
    
    /// Set a mock secret
    pub async fn set_secret(&self, reference: &str, value: SensitiveString) {
        self.secrets.write().await.insert(reference.to_string(), value);
    }
    
    /// Trigger an error for a specific operation
    pub async fn trigger_error_on(&self, operation: &str, error: OnePasswordError) {
        self.error_triggers.write().await
            .insert(operation.to_string(), error);
    }
    
    /// Get call history
    pub async fn get_call_history(&self) -> Vec<String> {
        self.call_history.read().await.clone()
    }
    
    /// Clear call history
    pub async fn clear_history(&self) {
        self.call_history.write().await.clear();
    }
    
    async fn record_call(&self, method: &str) {
        self.call_history.write().await.push(method.to_string());
    }
    
    async fn check_error(&self, operation: &str) -> Result<(), OnePasswordError> {
        if let Some(error) = self.error_triggers.read().await.get(operation) {
            return Err(error.clone());
        }
        Ok(())
    }
}

#[async_trait]
impl OnePasswordProvider for MockOnePasswordProvider {
    async fn is_authenticated(&self) -> Result<bool, OnePasswordError> {
        self.record_call("is_authenticated").await;
        self.check_error("is_authenticated").await?;
        Ok(*self.authenticated.read().await)
    }
    
    async fn whoami(&self) -> Result<AccountInfo, OnePasswordError> {
        self.record_call("whoami").await;
        self.check_error("whoami").await?;
        
        if !*self.authenticated.read().await {
            return Err(OnePasswordError::NotAuthenticated);
        }
        
        Ok(AccountInfo {
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            account_uuid: "test-account-uuid".to_string(),
        })
    }
    
    async fn list_vaults(&self) -> Result<Vec<Vault>, OnePasswordError> {
        self.record_call("list_vaults").await;
        self.check_error("list_vaults").await?;
        
        if !*self.authenticated.read().await {
            return Err(OnePasswordError::NotAuthenticated);
        }
        
        Ok(self.vaults.read().await.clone())
    }
    
    async fn list_items(
        &self,
        category: Option<ItemCategory>,
        vault_id: Option<&str>,
    ) -> Result<Vec<Item>, OnePasswordError> {
        self.record_call(&format!("list_items({:?}, {:?})", category, vault_id)).await;
        self.check_error("list_items").await?;
        
        if !*self.authenticated.read().await {
            return Err(OnePasswordError::NotAuthenticated);
        }
        
        let items = self.items.read().await;
        let mut result = Vec::new();
        
        if let Some(vault) = vault_id {
            if let Some(vault_items) = items.get(vault) {
                result.extend(vault_items.clone());
            }
        } else {
            for vault_items in items.values() {
                result.extend(vault_items.clone());
            }
        }
        
        // Filter by category if specified
        if let Some(cat) = category {
            result.retain(|item| item.category == cat);
        }
        
        Ok(result)
    }
    
    async fn get_ssh_key_info(&self, item_id: &str) -> Result<SshKeyInfo, OnePasswordError> {
        self.record_call(&format!("get_ssh_key_info({})", item_id)).await;
        self.check_error("get_ssh_key_info").await?;
        
        if !*self.authenticated.read().await {
            return Err(OnePasswordError::NotAuthenticated);
        }
        
        // Find the item
        let items = self.items.read().await;
        for vault_items in items.values() {
            if let Some(item) = vault_items.iter().find(|i| i.id == item_id) {
                return Ok(SshKeyInfo {
                    item_id: item.id.clone(),
                    title: item.title.clone(),
                    fingerprint: Some("SHA256:mock-fingerprint".to_string()),
                    key_type: Some("ed25519".to_string()),
                    vault_name: item.vault.name.clone().unwrap_or_default(),
                    reference: format!("op://{}/{}/private key", 
                        item.vault.name.as_deref().unwrap_or("vault"),
                        item.title
                    ),
                });
            }
        }
        
        Err(OnePasswordError::ItemNotFound(item_id.to_string()))
    }
    
    async fn read_secret(&self, reference: &SecretReference) -> Result<SensitiveString, OnePasswordError> {
        self.record_call(&format!("read_secret({})", reference.url)).await;
        self.check_error("read_secret").await?;
        
        if !*self.authenticated.read().await {
            return Err(OnePasswordError::NotAuthenticated);
        }
        
        self.secrets.read().await
            .get(&reference.url)
            .cloned()
            .ok_or_else(|| OnePasswordError::ItemNotFound(reference.url.clone()))
    }
    
    fn create_reference(&self, vault: &str, item: &str, field: &str) -> SecretReference {
        SecretReference {
            url: format!("op://{}/{}/{}", vault, item, field),
            vault: vault.to_string(),
            item: item.to_string(),
            field: field.to_string(),
        }
    }
}
```

## Testing Patterns

### 1. Basic Functionality Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_list_vaults() {
        let provider = MockOnePasswordProvider::new();
        
        let vaults = provider.list_vaults().await.unwrap();
        assert_eq!(vaults.len(), 2);
        assert_eq!(vaults[0].name, "Personal");
        assert_eq!(vaults[1].name, "Work");
    }
    
    #[tokio::test]
    async fn test_authentication_required() {
        let provider = MockOnePasswordProvider::new();
        provider.set_authenticated(false).await;
        
        let result = provider.list_vaults().await;
        assert!(matches!(result, Err(OnePasswordError::NotAuthenticated)));
    }
    
    #[tokio::test]
    async fn test_ssh_key_discovery() {
        let provider = MockOnePasswordProvider::new();
        
        let ssh_keys = provider.list_items(
            Some(ItemCategory::SshKey),
            None
        ).await.unwrap();
        
        assert_eq!(ssh_keys.len(), 1);
        assert_eq!(ssh_keys[0].title, "GitHub SSH Key");
    }
}
```

### 2. Error Scenario Tests

```rust
#[tokio::test]
async fn test_network_error_simulation() {
    let provider = MockOnePasswordProvider::new();
    
    // Simulate network error
    provider.trigger_error_on(
        "list_vaults",
        OnePasswordError::CommandFailed("Network timeout".to_string())
    ).await;
    
    let result = provider.list_vaults().await;
    assert!(matches!(result, Err(OnePasswordError::CommandFailed(_))));
}

#[tokio::test]
async fn test_item_not_found() {
    let provider = MockOnePasswordProvider::new();
    
    let result = provider.get_ssh_key_info("non-existent").await;
    assert!(matches!(result, Err(OnePasswordError::ItemNotFound(_))));
}

#[tokio::test]
async fn test_invalid_json_simulation() {
    let provider = MockOnePasswordProvider::new();
    
    provider.trigger_error_on(
        "list_items",
        OnePasswordError::InvalidJson("Unexpected token".to_string())
    ).await;
    
    let result = provider.list_items(None, None).await;
    assert!(matches!(result, Err(OnePasswordError::InvalidJson(_))));
}
```

### 3. Integration Tests

```rust
// tests/integration/onepassword_integration.rs

use git_setup_rs::{
    ProfileManager,
    onepassword::{MockOnePasswordProvider, OnePasswordProvider},
};

#[tokio::test]
async fn test_profile_with_onepassword_reference() {
    // Setup mock provider
    let op_provider = Arc::new(MockOnePasswordProvider::new());
    
    // Add test SSH key
    op_provider.set_secret(
        "op://Personal/GitHub SSH Key/private key",
        SensitiveString::from("test-ssh-private-key")
    ).await;
    
    // Create profile manager with mock
    let profile_manager = ProfileManager::builder()
        .with_onepassword_provider(op_provider.clone())
        .build()
        .unwrap();
    
    // Create profile with 1Password reference
    let profile = Profile {
        name: "test".to_string(),
        git: GitConfig {
            user_name: "Test User".to_string(),
            user_email: "test@example.com".to_string(),
            ..Default::default()
        },
        signing: Some(SigningConfig {
            method: SigningMethod::Ssh,
            ssh_key_ref: Some("op://Personal/GitHub SSH Key/private key".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    
    profile_manager.create_profile(profile).await.unwrap();
    
    // Apply profile and verify SSH key is fetched
    profile_manager.apply_profile("test").await.unwrap();
    
    // Verify the mock was called
    let history = op_provider.get_call_history().await;
    assert!(history.contains(&"read_secret(op://Personal/GitHub SSH Key/private key)".to_string()));
}
```

### 4. Behavior Verification Tests

```rust
#[tokio::test]
async fn test_call_sequence_verification() {
    let provider = MockOnePasswordProvider::new();
    
    // Perform operations
    provider.is_authenticated().await.unwrap();
    provider.list_vaults().await.unwrap();
    provider.list_items(Some(ItemCategory::SshKey), None).await.unwrap();
    
    // Verify call sequence
    let history = provider.get_call_history().await;
    assert_eq!(history, vec![
        "is_authenticated",
        "list_vaults",
        "list_items(Some(SshKey), None)",
    ]);
}

#[tokio::test]
async fn test_concurrent_access() {
    let provider = Arc::new(MockOnePasswordProvider::new());
    
    // Spawn multiple concurrent operations
    let mut handles = vec![];
    
    for i in 0..10 {
        let provider = provider.clone();
        let handle = tokio::spawn(async move {
            provider.list_vaults().await.unwrap();
            provider.list_items(None, None).await.unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all operations succeeded
    let history = provider.get_call_history().await;
    assert_eq!(history.len(), 20); // 10 * 2 operations
}
```

### 5. State-Based Tests

```rust
#[tokio::test]
async fn test_stateful_mock() {
    let provider = MockOnePasswordProvider::new();
    
    // Add item dynamically
    let new_item = Item {
        id: "new-ssh-key".to_string(),
        title: "New SSH Key".to_string(),
        category: ItemCategory::SshKey,
        vault: VaultReference {
            id: "personal-vault-id".to_string(),
            name: Some("Personal".to_string()),
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: vec![],
    };
    
    provider.add_item("personal-vault-id".to_string(), new_item).await;
    
    // Verify it appears in listing
    let items = provider.list_items(
        Some(ItemCategory::SshKey),
        Some("personal-vault-id")
    ).await.unwrap();
    
    assert_eq!(items.len(), 2); // Original + new
    assert!(items.iter().any(|i| i.title == "New SSH Key"));
}
```

## Advanced Mocking Scenarios

### 1. Biometric Authentication Simulation

```rust
impl MockOnePasswordProvider {
    /// Simulate biometric prompt
    pub async fn simulate_biometric_prompt(&self, duration: Duration) -> Result<(), OnePasswordError> {
        self.record_call("biometric_prompt").await;
        
        // Simulate prompt delay
        tokio::time::sleep(duration).await;
        
        // Check if user "approved" (for testing)
        if self.biometric_approved.load(Ordering::Relaxed) {
            Ok(())
        } else {
            Err(OnePasswordError::BiometricCancelled)
        }
    }
}
```

### 2. Rate Limiting Simulation

```rust
pub struct RateLimitedMock {
    inner: MockOnePasswordProvider,
    call_count: Arc<AtomicUsize>,
    rate_limit: usize,
    window: Duration,
    last_reset: Arc<RwLock<Instant>>,
}

impl RateLimitedMock {
    async fn check_rate_limit(&self) -> Result<(), OnePasswordError> {
        let mut last_reset = self.last_reset.write().await;
        let now = Instant::now();
        
        if now.duration_since(*last_reset) > self.window {
            self.call_count.store(0, Ordering::Relaxed);
            *last_reset = now;
        }
        
        let count = self.call_count.fetch_add(1, Ordering::Relaxed);
        if count >= self.rate_limit {
            Err(OnePasswordError::RateLimited)
        } else {
            Ok(())
        }
    }
}
```

### 3. Vault Permission Simulation

```rust
impl MockOnePasswordProvider {
    pub async fn set_vault_permission(&self, vault_id: &str, permission: VaultPermission) {
        self.vault_permissions.write().await
            .insert(vault_id.to_string(), permission);
    }
    
    async fn check_vault_access(&self, vault_id: &str) -> Result<(), OnePasswordError> {
        let permissions = self.vault_permissions.read().await;
        
        match permissions.get(vault_id) {
            Some(VaultPermission::ReadWrite) => Ok(()),
            Some(VaultPermission::ReadOnly) => Ok(()),
            Some(VaultPermission::NoAccess) => {
                Err(OnePasswordError::AccessDenied(vault_id.to_string()))
            }
            None => Ok(()), // Default allow
        }
    }
}

#[derive(Clone, Copy)]
enum VaultPermission {
    ReadWrite,
    ReadOnly,
    NoAccess,
}
```

## Testing Best Practices

### 1. Test Data Builders

```rust
pub struct TestDataBuilder {
    provider: MockOnePasswordProvider,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self {
            provider: MockOnePasswordProvider::new(),
        }
    }
    
    pub fn with_ssh_key(mut self, name: &str) -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let item = Item {
                id: format!("{}-id", name),
                title: name.to_string(),
                category: ItemCategory::SshKey,
                // ... other fields
            };
            
            self.provider.add_item("personal-vault-id".to_string(), item).await;
        });
        
        self
    }
    
    pub fn with_error_on(mut self, operation: &str, error: OnePasswordError) -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            self.provider.trigger_error_on(operation, error).await;
        });
        
        self
    }
    
    pub fn build(self) -> MockOnePasswordProvider {
        self.provider
    }
}

// Usage
#[tokio::test]
async fn test_with_builder() {
    let provider = TestDataBuilder::new()
        .with_ssh_key("GitHub")
        .with_ssh_key("GitLab")
        .with_error_on("read_secret", OnePasswordError::BiometricCancelled)
        .build();
    
    // Test with configured provider
}
```

### 2. Snapshot Testing

```rust
#[tokio::test]
async fn test_vault_listing_snapshot() {
    let provider = MockOnePasswordProvider::new();
    let vaults = provider.list_vaults().await.unwrap();
    
    // Use insta for snapshot testing
    insta::assert_json_snapshot!(vaults, {
        "[].created_at" => "[TIMESTAMP]",
    });
}
```

### 3. Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_reference_parsing(s in "op://[a-zA-Z0-9]+/[a-zA-Z0-9 ]+/[a-zA-Z0-9]+") {
        let reference = SecretReference::parse(&s);
        prop_assert!(reference.is_ok());
        
        let ref_parsed = reference.unwrap();
        prop_assert_eq!(ref_parsed.url, s);
    }
    
    #[test]
    fn test_invalid_reference_format(s in "[^o][^p].*") {
        let reference = SecretReference::parse(&s);
        prop_assert!(reference.is_err());
    }
}
```

## Mock Configuration

### Environment-Based Configuration

```rust
pub struct MockConfig {
    pub default_authenticated: bool,
    pub latency: Option<Duration>,
    pub failure_rate: f32,
}

impl MockConfig {
    pub fn from_env() -> Self {
        Self {
            default_authenticated: env::var("MOCK_OP_AUTHENTICATED")
                .map(|v| v == "true")
                .unwrap_or(true),
            latency: env::var("MOCK_OP_LATENCY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_millis),
            failure_rate: env::var("MOCK_OP_FAILURE_RATE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0),
        }
    }
}
```

### Test Categories

```rust
#[cfg(test)]
mod unit_tests {
    // Fast, isolated tests
}

#[cfg(test)]
mod integration_tests {
    // Tests with ProfileManager integration
}

#[cfg(test)]
mod stress_tests {
    // High load, concurrent access
}

#[cfg(test)]
mod security_tests {
    // Credential handling, no leaks
}
```

## Debugging Mock Tests

### Debug Output

```rust
impl MockOnePasswordProvider {
    pub fn enable_debug(&mut self) {
        self.debug_enabled = true;
    }
    
    async fn debug_log(&self, message: &str) {
        if self.debug_enabled {
            eprintln!("[MOCK] {}", message);
        }
    }
}
```

### Test Helpers

```rust
/// Assert that a secret was never accessed
pub async fn assert_secret_not_accessed(provider: &MockOnePasswordProvider, reference: &str) {
    let history = provider.get_call_history().await;
    assert!(
        !history.iter().any(|call| call.contains(reference)),
        "Secret {} was unexpectedly accessed", 
        reference
    );
}

/// Assert specific call sequence
pub async fn assert_call_sequence(provider: &MockOnePasswordProvider, expected: &[&str]) {
    let history = provider.get_call_history().await;
    assert_eq!(
        history.len(), 
        expected.len(), 
        "Expected {} calls, got {}", 
        expected.len(), 
        history.len()
    );
    
    for (actual, expected) in history.iter().zip(expected.iter()) {
        assert_eq!(actual, expected);
    }
}
```

## Common Testing Patterns

### 1. Happy Path Test
```rust
#[tokio::test]
async fn test_complete_workflow() {
    let provider = MockOnePasswordProvider::new();
    
    // List vaults
    let vaults = provider.list_vaults().await.unwrap();
    assert!(!vaults.is_empty());
    
    // Find SSH keys
    let ssh_keys = provider.list_items(
        Some(ItemCategory::SshKey),
        Some(&vaults[0].id)
    ).await.unwrap();
    assert!(!ssh_keys.is_empty());
    
    // Get key info
    let key_info = provider.get_ssh_key_info(&ssh_keys[0].id).await.unwrap();
    
    // Read secret
    let reference = SecretReference::parse(&key_info.reference).unwrap();
    let secret = provider.read_secret(&reference).await.unwrap();
    assert!(!secret.expose_secret().is_empty());
}
```

### 2. Error Recovery Test
```rust
#[tokio::test]
async fn test_retry_on_failure() {
    let provider = Arc::new(Mutex::new(MockOnePasswordProvider::new()));
    let retry_count = Arc::new(AtomicUsize::new(0));
    
    // Fail first two attempts
    provider.lock().await.trigger_error_on(
        "list_vaults",
        OnePasswordError::CommandFailed("Temporary failure".to_string())
    ).await;
    
    // Retry logic
    let mut result = Err(OnePasswordError::NotAuthenticated);
    for _ in 0..3 {
        retry_count.fetch_add(1, Ordering::Relaxed);
        
        result = provider.lock().await.list_vaults().await;
        
        if result.is_ok() {
            break;
        }
        
        // Clear error after first attempt
        if retry_count.load(Ordering::Relaxed) == 1 {
            provider.lock().await.error_triggers.write().await.clear();
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    assert!(result.is_ok());
    assert_eq!(retry_count.load(Ordering::Relaxed), 2);
}
```

## Summary

This comprehensive mock testing guide enables thorough testing of 1Password integration without external dependencies. The mock implementation supports all testing scenarios from unit tests to complex integration tests, ensuring the security and reliability of the credential management system.