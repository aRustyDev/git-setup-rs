//! Mock 1Password Integration for Testing
//! 
//! This example shows how to test 1Password integration without requiring
//! actual 1Password CLI installation. Perfect for development and CI/CD.
//! 
//! Key Concepts for Junior Developers:
//! - Trait-based abstractions allow swapping implementations
//! - Mock objects simulate external dependencies
//! - Tests remain fast and deterministic

use std::collections::HashMap;
use mockall::predicate::*;
use mockall::*;

/// Represents an SSH key from 1Password
#[derive(Debug, Clone, PartialEq)]
pub struct SshKey {
    pub name: String,
    pub reference: String,
    pub fingerprint: String,
}

/// Represents a GPG key from 1Password
#[derive(Debug, Clone)]
pub struct GpgKey {
    pub name: String,
    pub reference: String,
    pub key_id: String,
}

/// Errors that can occur with 1Password operations
#[derive(Debug, thiserror::Error)]
pub enum OnePasswordError {
    #[error("1Password CLI not found")]
    CliNotFound,
    
    #[error("Not authenticated to 1Password")]
    NotAuthenticated,
    
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    
    #[error("Command failed: {0}")]
    CommandFailed(String),
}

/// Trait defining all 1Password operations we need
/// This is what we'll mock in tests
#[automock]
pub trait OnePasswordClient {
    /// List all SSH keys available in 1Password
    fn list_ssh_keys(&self) -> Result<Vec<SshKey>, OnePasswordError>;
    
    /// Get a specific credential by its reference
    fn get_credential(&self, reference: &str) -> Result<String, OnePasswordError>;
    
    /// Check if 1Password CLI is available and authenticated
    fn is_authenticated(&self) -> bool;
    
    /// Get user's email from 1Password account
    fn get_account_email(&self) -> Result<String, OnePasswordError>;
}

/// Real implementation that calls actual 1Password CLI
pub struct RealOnePasswordClient;

impl OnePasswordClient for RealOnePasswordClient {
    fn list_ssh_keys(&self) -> Result<Vec<SshKey>, OnePasswordError> {
        // In real implementation, this would run:
        // op item list --categories "SSH Key" --format json
        unimplemented!("Use mock in tests")
    }
    
    fn get_credential(&self, reference: &str) -> Result<String, OnePasswordError> {
        // In real implementation, this would run:
        // op read "{reference}"
        unimplemented!("Use mock in tests")
    }
    
    fn is_authenticated(&self) -> bool {
        // Would check: op account get
        unimplemented!("Use mock in tests")
    }
    
    fn get_account_email(&self) -> Result<String, OnePasswordError> {
        unimplemented!("Use mock in tests")
    }
}

/// Example: Using mocks in tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_ssh_keys_with_mock() {
        // Step 1: Create mock
        let mut mock = MockOnePasswordClient::new();
        
        // Step 2: Set expectations
        mock.expect_list_ssh_keys()
            .times(1)  // Expect exactly one call
            .returning(|| {
                // Return test data
                Ok(vec![
                    SshKey {
                        name: "GitHub Personal".into(),
                        reference: "op://Personal/GitHub SSH Key/private key".into(),
                        fingerprint: "SHA256:abcd1234...".into(),
                    },
                    SshKey {
                        name: "Work GitLab".into(),
                        reference: "op://Work/GitLab SSH Key/private key".into(),
                        fingerprint: "SHA256:efgh5678...".into(),
                    },
                ])
            });
        
        // Step 3: Use mock in code under test
        let keys = mock.list_ssh_keys().unwrap();
        
        // Step 4: Verify behavior
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].name, "GitHub Personal");
    }
    
    #[test]
    fn test_authentication_failure() {
        let mut mock = MockOnePasswordClient::new();
        
        // Simulate not authenticated
        mock.expect_is_authenticated()
            .returning(|| false);
        
        mock.expect_list_ssh_keys()
            .returning(|| Err(OnePasswordError::NotAuthenticated));
        
        // Test error handling
        assert!(!mock.is_authenticated());
        
        match mock.list_ssh_keys() {
            Err(OnePasswordError::NotAuthenticated) => {
                // Expected error
            }
            other => panic!("Expected NotAuthenticated, got {:?}", other),
        }
    }
    
    #[test]
    fn test_credential_retrieval() {
        let mut mock = MockOnePasswordClient::new();
        
        // Set up specific expectation
        mock.expect_get_credential()
            .with(eq("op://Personal/GitHub SSH Key/private key"))
            .returning(|_| {
                Ok("-----BEGIN OPENSSH PRIVATE KEY-----\ntest-key-content\n-----END OPENSSH PRIVATE KEY-----".into())
            });
        
        let key = mock.get_credential("op://Personal/GitHub SSH Key/private key").unwrap();
        assert!(key.contains("BEGIN OPENSSH PRIVATE KEY"));
    }
}

/// Example: How to use in application code
pub struct Profile {
    pub name: String,
    pub ssh_key_reference: Option<String>,
    client: Box<dyn OnePasswordClient>,
}

impl Profile {
    /// Create profile with real 1Password client
    pub fn new(name: String) -> Self {
        Self {
            name,
            ssh_key_reference: None,
            client: Box::new(RealOnePasswordClient),
        }
    }
    
    /// Create profile with custom client (for testing)
    pub fn new_with_client(name: String, client: Box<dyn OnePasswordClient>) -> Self {
        Self {
            name,
            ssh_key_reference: None,
            client,
        }
    }
    
    /// Get SSH key from 1Password
    pub fn get_ssh_key(&self) -> Result<String, OnePasswordError> {
        match &self.ssh_key_reference {
            Some(reference) => self.client.get_credential(reference),
            None => Err(OnePasswordError::ItemNotFound("No SSH key configured".into())),
        }
    }
}

/// Example test using mock with Profile
#[test]
fn test_profile_with_mock_client() {
    // Create mock client
    let mut mock = MockOnePasswordClient::new();
    
    mock.expect_get_credential()
        .with(eq("op://Personal/test-key"))
        .returning(|_| Ok("mock-ssh-key-content".into()));
    
    // Create profile with mock
    let mut profile = Profile::new_with_client(
        "test".into(),
        Box::new(mock),
    );
    profile.ssh_key_reference = Some("op://Personal/test-key".into());
    
    // Test behavior
    let key = profile.get_ssh_key().unwrap();
    assert_eq!(key, "mock-ssh-key-content");
}

/// Junior Developer Exercise:
/// 
/// 1. Add a new method to OnePasswordClient trait:
///    fn list_gpg_keys(&self) -> Result<Vec<GpgKey>, OnePasswordError>
/// 
/// 2. Add mock expectation in a test
/// 
/// 3. Use the mock to test error handling
/// 
/// Hints:
/// - Copy the pattern from list_ssh_keys
/// - Use expect_list_gpg_keys() on the mock
/// - Return test data or errors as needed