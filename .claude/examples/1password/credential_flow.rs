//! 1Password Credential Retrieval Flow
//! 
//! This example demonstrates the complete flow of retrieving credentials
//! from 1Password, including error handling and caching strategies.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::process::Command;

/// Cache for 1Password credentials to avoid repeated CLI calls
pub struct CredentialCache {
    /// Cached credentials with expiration time
    cache: HashMap<String, (String, Instant)>,
    /// How long to cache credentials (default: 5 minutes)
    ttl: Duration,
}

impl CredentialCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }
    
    /// Get credential from cache if still valid
    pub fn get(&self, reference: &str) -> Option<String> {
        self.cache.get(reference).and_then(|(value, expires_at)| {
            if Instant::now() < *expires_at {
                Some(value.clone())
            } else {
                None
            }
        })
    }
    
    /// Store credential in cache
    pub fn set(&mut self, reference: String, value: String) {
        let expires_at = Instant::now() + self.ttl;
        self.cache.insert(reference, (value, expires_at));
    }
    
    /// Clear expired entries
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, expires_at)| now < *expires_at);
    }
}

/// Complete credential retrieval flow with caching and error handling
pub struct OnePasswordFlow {
    cache: CredentialCache,
}

impl OnePasswordFlow {
    pub fn new() -> Self {
        Self {
            cache: CredentialCache::new(),
        }
    }
    
    /// Main credential retrieval method with all safety checks
    pub fn get_credential(&mut self, reference: &str) -> Result<String, String> {
        // Step 1: Validate reference format
        if !self.validate_reference(reference) {
            return Err("Invalid 1Password reference format".into());
        }
        
        // Step 2: Check cache first
        if let Some(cached) = self.cache.get(reference) {
            println!("✓ Using cached credential for {}", reference);
            return Ok(cached);
        }
        
        // Step 3: Check if 1Password CLI is available
        if !self.is_op_installed()? {
            return Err("1Password CLI (op) is not installed".into());
        }
        
        // Step 4: Check authentication status
        if !self.is_authenticated()? {
            println!("⚠️  Not authenticated to 1Password");
            println!("Please run: eval $(op signin)");
            return Err("Not authenticated to 1Password".into());
        }
        
        // Step 5: Retrieve credential
        println!("🔑 Retrieving credential from 1Password...");
        let credential = self.retrieve_from_op(reference)?;
        
        // Step 6: Cache for future use
        self.cache.set(reference.to_string(), credential.clone());
        
        // Step 7: Cleanup old cache entries
        self.cache.cleanup();
        
        Ok(credential)
    }
    
    /// Validate reference format (op://vault/item/field)
    fn validate_reference(&self, reference: &str) -> bool {
        reference.starts_with("op://") && reference.matches('/').count() >= 3
    }
    
    /// Check if 1Password CLI is installed
    fn is_op_installed(&self) -> Result<bool, String> {
        match Command::new("op").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("✓ 1Password CLI version: {}", version.trim());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }
    
    /// Check if authenticated to 1Password
    fn is_authenticated(&self) -> Result<bool, String> {
        match Command::new("op").arg("account").arg("get").output() {
            Ok(output) => Ok(output.status.success()),
            Err(e) => Err(format!("Failed to check auth status: {}", e)),
        }
    }
    
    /// Actually retrieve credential from 1Password
    fn retrieve_from_op(&self, reference: &str) -> Result<String, String> {
        let output = Command::new("op")
            .arg("read")
            .arg(reference)
            .output()
            .map_err(|e| format!("Failed to run op command: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("1Password error: {}", error))
        }
    }
}

/// Example usage showing the complete flow
fn main() {
    let mut flow = OnePasswordFlow::new();
    
    // Example references
    let ssh_key_ref = "op://Personal/GitHub SSH Key/private key";
    let api_token_ref = "op://Work/API Token/credential";
    
    // Retrieve SSH key
    match flow.get_credential(ssh_key_ref) {
        Ok(key) => {
            println!("✅ Retrieved SSH key ({} bytes)", key.len());
            // In real usage, you'd write this to a temporary file
        }
        Err(e) => {
            eprintln!("❌ Failed to get SSH key: {}", e);
        }
    }
    
    // Retrieve API token (will use cache if called again quickly)
    match flow.get_credential(api_token_ref) {
        Ok(token) => {
            println!("✅ Retrieved API token");
            // Use token for API calls
        }
        Err(e) => {
            eprintln!("❌ Failed to get API token: {}", e);
        }
    }
}

/// Junior Developer Learning Points:
/// 
/// 1. **Caching Strategy**:
///    - Cache credentials to avoid repeated 1Password prompts
///    - Use time-based expiration for security
///    - Clean up expired entries to prevent memory leaks
/// 
/// 2. **Error Handling Flow**:
///    - Validate input before making external calls
///    - Check prerequisites (CLI installed, authenticated)
///    - Provide helpful error messages for users
/// 
/// 3. **Security Considerations**:
///    - Never log credential values
///    - Use short cache TTLs
///    - Clear credentials from memory when done
/// 
/// 4. **User Experience**:
///    - Show progress indicators
///    - Cache to avoid repeated auth prompts
///    - Provide clear instructions for failures
/// 
/// Exercise: Add a method to clear specific cached credentials
/// Hint: Add a `clear_credential(&mut self, reference: &str)` method