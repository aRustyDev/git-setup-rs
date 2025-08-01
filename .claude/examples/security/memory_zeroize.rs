//! Memory Zeroization Example
//! 
//! This example demonstrates secure handling of sensitive data in memory.
//! When dealing with passwords, keys, or tokens, we must ensure they're
//! completely removed from memory when no longer needed.

use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};
use std::fmt;

/// A password that automatically clears itself from memory
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecurePassword {
    // Private field - never expose directly
    inner: String,
}

impl SecurePassword {
    /// Create a new secure password
    pub fn new(password: String) -> Self {
        SecurePassword { inner: password }
    }
    
    /// Safely access the password value
    /// 
    /// WARNING: The returned reference is only valid while self is borrowed.
    /// Do NOT store this reference or convert to String!
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }
    
    /// Perform an operation with the password
    pub fn use_password<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        f(&self.inner)
    }
}

// Prevent accidental logging of passwords
impl fmt::Debug for SecurePassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecurePassword(***)")
    }
}

impl fmt::Display for SecurePassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

/// An API token that clears on drop
pub struct SecureToken {
    token: Zeroizing<String>,
}

impl SecureToken {
    pub fn new(token: String) -> Self {
        // Zeroizing<T> automatically clears T when dropped
        SecureToken {
            token: Zeroizing::new(token),
        }
    }
    
    pub fn expose_secret(&self) -> &str {
        &self.token
    }
}

/// Example: Secure buffer for binary data (like keys)
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        SecureBuffer { data }
    }
    
    pub fn expose_secret(&self) -> &[u8] {
        &self.data
    }
}

// Manual implementation of Drop for custom zeroization
impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // Zeroize the data
        self.data.zeroize();
    }
}

/// Example: Working with temporary secrets
pub fn process_with_secret<F, R>(secret: String, processor: F) -> R
where
    F: FnOnce(&str) -> R,
{
    // Zeroizing ensures the string is cleared after use
    let secure = Zeroizing::new(secret);
    processor(&secure)
}

/// Demonstration of secure vs insecure patterns
mod examples {
    use super::*;
    
    /// ❌ BAD: Password remains in memory after use
    pub fn insecure_login(username: &str, password: String) -> bool {
        // Password string allocated on heap
        let result = check_credentials(username, &password);
        
        // When password goes out of scope, memory is NOT cleared!
        // The password bytes may remain in memory indefinitely
        result
    }
    
    /// ✅ GOOD: Password is zeroized after use
    pub fn secure_login(username: &str, password: String) -> bool {
        // Convert to secure password immediately
        let secure_pass = SecurePassword::new(password);
        
        // Use the password
        let result = secure_pass.use_password(|pass| {
            check_credentials(username, pass)
        });
        
        // secure_pass is automatically zeroized when dropped
        result
    }
    
    /// ❌ BAD: Cloning sensitive data
    pub fn insecure_share_token(token: &str) -> (String, String) {
        let copy1 = token.to_string();  // Now in memory twice!
        let copy2 = token.to_string();  // Three times!
        (copy1, copy2)
    }
    
    /// ✅ GOOD: Controlled access without cloning
    pub fn secure_use_token<R>(token: &SecureToken, f: impl FnOnce(&str) -> R) -> R {
        f(token.expose_secret())
    }
    
    fn check_credentials(_username: &str, _password: &str) -> bool {
        // Stub implementation
        true
    }
}

/// Tests demonstrating zeroization
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_zeroization() {
        let password_ptr: *const u8;
        let password_len: usize;
        
        {
            let secure = SecurePassword::new("super-secret-password".to_string());
            password_ptr = secure.expose_secret().as_ptr();
            password_len = secure.expose_secret().len();
            
            // Password is accessible while in scope
            assert_eq!(secure.expose_secret(), "super-secret-password");
        } // secure is dropped and zeroized here
        
        // SAFETY: We're checking if memory was cleared
        // In production code, never do this!
        unsafe {
            let bytes = std::slice::from_raw_parts(password_ptr, password_len);
            // All bytes should be zero
            assert!(bytes.iter().all(|&b| b == 0));
        }
    }
    
    #[test]
    fn test_no_accidental_logging() {
        let password = SecurePassword::new("secret123".to_string());
        
        // These should not expose the password
        let debug = format!("{:?}", password);
        let display = format!("{}", password);
        
        assert_eq!(debug, "SecurePassword(***)");
        assert_eq!(display, "***");
    }
}

/// Example program showing secure credential handling
fn main() {
    println!("Secure Memory Handling Examples\n");
    
    // Example 1: Secure password input
    println!("1. Secure password handling:");
    let password = SecurePassword::new("my-secure-password".to_string());
    
    // Use password without exposing it
    password.use_password(|pass| {
        println!("   Password length: {}", pass.len());
        println!("   First char: {}", pass.chars().next().unwrap_or('?'));
    });
    
    // Can't accidentally print it
    println!("   Password value: {}", password);  // Prints: ***
    
    // Example 2: Temporary secret processing
    println!("\n2. Temporary secret processing:");
    let result = process_with_secret("temporary-api-key".to_string(), |key| {
        // Process the key
        key.starts_with("temp")
    });
    println!("   Processing result: {}", result);
    // Key is now zeroized
    
    // Example 3: Binary data (like encryption keys)
    println!("\n3. Secure binary buffer:");
    let key_data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    let secure_key = SecureBuffer::new(key_data);
    
    println!("   Key length: {} bytes", secure_key.expose_secret().len());
    // secure_key will be zeroized when dropped
    
    println!("\n✅ All sensitive data will be securely zeroized!");
}

/// Junior Developer Notes:
/// 
/// 1. **Why Zeroization Matters**:
///    - Dropped Strings leave data in memory
///    - Memory can be read by attackers (core dumps, cold boot)
///    - Sensitive data must be explicitly cleared
/// 
/// 2. **Zeroization Rules**:
///    - Wrap ALL sensitive data (passwords, tokens, keys)
///    - Never clone or copy sensitive data
///    - Use references or closures for access
///    - Clear immediately after use
/// 
/// 3. **Common Pitfalls**:
///    ```rust
///    // ❌ BAD: Creates copies
///    let copy = sensitive.to_string();
///    
///    // ❌ BAD: Logs sensitive data
///    println!("Token: {}", token);
///    
///    // ❌ BAD: Stores in non-zeroizing collection
///    vec.push(password.to_string());
///    ```
/// 
/// 4. **Best Practices**:
///    - Convert to secure types ASAP
///    - Use `expose_secret()` sparingly
///    - Implement Debug/Display to prevent leaks
///    - Test zeroization in debug builds
/// 
/// Exercise: Create a SecureConfig struct that:
/// 1. Holds multiple secrets (password, API key, etc)
/// 2. Zeroizes all fields on drop
/// 3. Provides safe access methods
/// 4. Prevents accidental exposure via Debug/Display