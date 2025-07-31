//! Profile manager implementation for git-setup-rs.

use crate::{
    config::types::Profile,
    error::{GitSetupError, Result},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory implementation of ProfileManager.
///
/// This implementation stores profiles in memory and is suitable for
/// temporary storage during application runtime. For persistent storage,
/// use FileProfileManager.
#[derive(Debug, Clone)]
pub struct ProfileManagerImpl {
    profiles: Arc<Mutex<HashMap<String, Profile>>>,
    default_profile: Arc<Mutex<Option<String>>>,
}

impl ProfileManagerImpl {
    /// Create a new ProfileManagerImpl instance.
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            default_profile: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the default profile name.
    pub fn get_default(&self) -> Result<Option<String>> {
        Ok(self.default_profile.lock().unwrap().clone())
    }

    /// Set the default profile.
    pub fn set_default(&self, name: &str) -> Result<()> {
        let profiles = self.profiles.lock().unwrap();
        if !profiles.contains_key(name) {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }
        drop(profiles); // Release the lock before acquiring the next one

        *self.default_profile.lock().unwrap() = Some(name.to_string());
        Ok(())
    }

    /// Find profiles matching a pattern.
    pub fn find(&self, pattern: &str) -> Result<Vec<Profile>> {
        let pattern_lower = pattern.to_lowercase();
        let profiles = self.profiles.lock().unwrap();

        let mut matches: Vec<Profile> = profiles
            .values()
            .filter(|p| p.name.to_lowercase().contains(&pattern_lower))
            .cloned()
            .collect();

        matches.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(matches)
    }

    /// Validate a profile.
    fn validate(&self, profile: &Profile) -> Result<()> {
        // Validate name
        if profile.name.is_empty() {
            return Err(GitSetupError::InvalidProfile {
                reason: "Profile name cannot be empty".to_string(),
            });
        }

        // Validate name doesn't contain invalid characters
        if profile.name.contains('/') || profile.name.contains('\\') {
            return Err(GitSetupError::InvalidProfile {
                reason: "Profile name cannot contain path separators".to_string(),
            });
        }

        // Validate profile name length
        if profile.name.len() > 100 {
            return Err(GitSetupError::InvalidProfile {
                reason: "Profile name cannot exceed 100 characters".to_string(),
            });
        }

        // Validate profile name format (alphanumeric + dash/underscore)
        if !profile.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(GitSetupError::InvalidProfile {
                reason: "Profile name can only contain alphanumeric characters, dashes, and underscores".to_string(),
            });
        }

        // Validate email
        if profile.git_user_email.is_empty() {
            return Err(GitSetupError::InvalidProfile {
                reason: "Email address cannot be empty".to_string(),
            });
        }

        if !profile.git_user_email.contains('@') {
            return Err(GitSetupError::InvalidProfile {
                reason: "Invalid email address format".to_string(),
            });
        }

        // Basic email validation
        let parts: Vec<&str> = profile.git_user_email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(GitSetupError::InvalidProfile {
                reason: "Invalid email address format".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for ProfileManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl super::ProfileManager for ProfileManagerImpl {
    fn create(&self, profile: Profile) -> Result<()> {
        self.validate(&profile)?;

        let mut profiles = self.profiles.lock().unwrap();
        if profiles.contains_key(&profile.name) {
            return Err(GitSetupError::DuplicateProfile {
                name: profile.name.clone(),
            });
        }

        profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    fn read(&self, name: &str) -> Result<Option<Profile>> {
        let profiles = self.profiles.lock().unwrap();
        Ok(profiles.get(name).cloned())
    }

    fn update(&self, name: &str, profile: Profile) -> Result<()> {
        self.validate(&profile)?;

        let mut profiles = self.profiles.lock().unwrap();
        if !profiles.contains_key(name) {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        // If renaming, check new name doesn't exist
        if profile.name != name && profiles.contains_key(&profile.name) {
            return Err(GitSetupError::DuplicateProfile {
                name: profile.name.clone(),
            });
        }

        // Handle rename
        if profile.name != name {
            profiles.remove(name);

            // Update default profile reference if needed
            let mut default = self.default_profile.lock().unwrap();
            if default.as_ref() == Some(&name.to_string()) {
                *default = Some(profile.name.clone());
            }
        }

        profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    fn delete(&self, name: &str) -> Result<()> {
        let mut profiles = self.profiles.lock().unwrap();
        if !profiles.contains_key(name) {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        profiles.remove(name);

        // Clear default if it was deleted
        let mut default = self.default_profile.lock().unwrap();
        if default.as_ref() == Some(&name.to_string()) {
            *default = None;
        }

        Ok(())
    }

    fn list(&self) -> Result<Vec<Profile>> {
        let profiles = self.profiles.lock().unwrap();
        let mut profile_list: Vec<Profile> = profiles.values().cloned().collect();
        profile_list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profile_list)
    }

    fn exists(&self, name: &str) -> Result<bool> {
        let profiles = self.profiles.lock().unwrap();
        Ok(profiles.contains_key(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{KeyType, Scope};
    use crate::profile::ProfileManager;

    fn test_profile() -> Profile {
        Profile {
            name: "test".to_string(),
            git_user_name: Some("Test User".to_string()),
            git_user_email: "test@example.com".to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("~/.ssh/id_ed25519.pub".to_string()),
            vault_name: Some("Test Vault".to_string()),
            ssh_key_title: Some("Test SSH Key".to_string()),
            scope: Some(Scope::Local),
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password: true,
        }
    }

    #[test]
    fn test_create_profile() {
        let manager = ProfileManagerImpl::new();
        let profile = test_profile();

        assert!(manager.create(profile.clone()).is_ok());
        assert!(manager.exists("test").unwrap());
    }

    #[test]
    fn test_create_duplicate_profile() {
        let manager = ProfileManagerImpl::new();
        let profile = test_profile();

        assert!(manager.create(profile.clone()).is_ok());
        let result = manager.create(profile);
        assert!(matches!(result, Err(GitSetupError::DuplicateProfile { .. })));
    }

    #[test]
    fn test_get_profile() {
        let manager = ProfileManagerImpl::new();
        let profile = test_profile();
        manager.create(profile.clone()).unwrap();

        let retrieved = manager.read("test").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
    }

    #[test]
    fn test_get_nonexistent_profile() {
        let manager = ProfileManagerImpl::new();
        let result = manager.read("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_profile() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();
        manager.create(profile.clone()).unwrap();

        profile.git_user_email = "updated@example.com".to_string();
        assert!(manager.update("test", profile).is_ok());

        let updated = manager.read("test").unwrap().unwrap();
        assert_eq!(updated.git_user_email, "updated@example.com");
    }

    #[test]
    fn test_update_nonexistent_profile() {
        let manager = ProfileManagerImpl::new();
        let profile = test_profile();

        let result = manager.update("nonexistent", profile);
        assert!(matches!(result, Err(GitSetupError::ProfileNotFound { .. })));
    }

    #[test]
    fn test_delete_profile() {
        let manager = ProfileManagerImpl::new();
        let profile = test_profile();
        manager.create(profile).unwrap();

        assert!(manager.delete("test").is_ok());
        assert!(!manager.exists("test").unwrap());
    }

    #[test]
    fn test_delete_nonexistent_profile() {
        let manager = ProfileManagerImpl::new();

        let result = manager.delete("nonexistent");
        assert!(matches!(result, Err(GitSetupError::ProfileNotFound { .. })));
    }

    #[test]
    fn test_list_profiles() {
        let manager = ProfileManagerImpl::new();
        let profile1 = test_profile();
        let mut profile2 = test_profile();
        profile2.name = "test2".to_string();

        manager.create(profile1).unwrap();
        manager.create(profile2).unwrap();

        let profiles = manager.list().unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].name, "test");
        assert_eq!(profiles[1].name, "test2");
    }

    #[test]
    fn test_profile_validation_empty_name() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();
        profile.name = "".to_string();

        let result = manager.validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_profile_validation_invalid_email() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();

        // Test invalid email - no @
        profile.git_user_email = "invalid-email".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());

        // Test empty email
        profile.git_user_email = "".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());

        // Test @ at the beginning
        profile.git_user_email = "@example.com".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());

        // Test @ at the end
        profile.git_user_email = "test@".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_validation_invalid_name_characters() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();

        // Test with forward slash
        profile.name = "test/profile".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path separators"));

        // Test with backslash
        profile.name = "test\\profile".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_validation_name_too_long() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();
        profile.name = "a".repeat(101);

        let result = manager.validate(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("100 characters"));
    }

    #[test]
    fn test_profile_validation_invalid_name_format() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();

        // Test with spaces
        profile.name = "test profile".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());

        // Test with special characters
        profile.name = "test!profile".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_err());

        // Valid names should pass
        profile.name = "test-profile_123".to_string();
        let result = manager.validate(&profile);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_profiles() {
        let manager = ProfileManagerImpl::new();
        let mut profile1 = test_profile();
        profile1.name = "work-project".to_string();
        let mut profile2 = test_profile();
        profile2.name = "work-client".to_string();
        let mut profile3 = test_profile();
        profile3.name = "personal".to_string();

        manager.create(profile1).unwrap();
        manager.create(profile2).unwrap();
        manager.create(profile3).unwrap();

        let results = manager.find("work").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "work-client");
        assert_eq!(results[1].name, "work-project");
    }

    #[test]
    fn test_find_profiles_case_insensitive() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();
        profile.name = "WorkProfile".to_string();
        manager.create(profile).unwrap();

        let results = manager.find("work").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "WorkProfile");
    }

    #[test]
    fn test_default_profile() {
        let manager = ProfileManagerImpl::new();
        let profile = test_profile();
        manager.create(profile).unwrap();

        // No default initially
        assert!(manager.get_default().unwrap().is_none());

        // Set default
        assert!(manager.set_default("test").is_ok());
        let default = manager.get_default().unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap(), "test");

        // Set non-existent as default
        let result = manager.set_default("nonexistent");
        assert!(matches!(result, Err(GitSetupError::ProfileNotFound { .. })));
    }

    #[test]
    fn test_rename_profile() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();
        manager.create(profile.clone()).unwrap();

        // Rename the profile
        profile.name = "renamed".to_string();
        assert!(manager.update("test", profile).is_ok());

        // Old name should not exist
        assert!(!manager.exists("test").unwrap());
        // New name should exist
        assert!(manager.exists("renamed").unwrap());
    }

    #[test]
    fn test_rename_to_existing_profile() {
        let manager = ProfileManagerImpl::new();
        let mut profile1 = test_profile();
        let mut profile2 = test_profile();
        profile2.name = "test2".to_string();

        manager.create(profile1.clone()).unwrap();
        manager.create(profile2).unwrap();

        // Try to rename profile1 to profile2's name
        profile1.name = "test2".to_string();
        let result = manager.update("test", profile1);
        assert!(matches!(result, Err(GitSetupError::DuplicateProfile { .. })));
    }

    #[test]
    fn test_rename_updates_default() {
        let manager = ProfileManagerImpl::new();
        let mut profile = test_profile();
        manager.create(profile.clone()).unwrap();
        manager.set_default("test").unwrap();

        // Rename the profile
        profile.name = "renamed".to_string();
        manager.update("test", profile).unwrap();

        // Default should be updated
        let default = manager.get_default().unwrap();
        assert_eq!(default.unwrap(), "renamed");
    }

    #[test]
    fn test_delete_clears_default() {
        let manager = ProfileManagerImpl::new();
        let profile = test_profile();
        manager.create(profile).unwrap();
        manager.set_default("test").unwrap();

        // Delete the default profile
        manager.delete("test").unwrap();

        // Default should be cleared
        assert!(manager.get_default().unwrap().is_none());
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let manager = Arc::new(ProfileManagerImpl::new());
        let mut handles = vec![];

        // Create profiles from multiple threads
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let mut profile = test_profile();
                profile.name = format!("test{}", i);
                manager_clone.create(profile).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all profiles were created
        let profiles = manager.list().unwrap();
        assert_eq!(profiles.len(), 10);
    }
}
