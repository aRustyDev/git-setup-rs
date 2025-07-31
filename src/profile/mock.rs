//! Mock implementation of ProfileManager for testing.

use crate::{config::types::Profile, error::{GitSetupError, Result}};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock implementation of ProfileManager for testing.
///
/// This implementation stores profiles in memory and provides
/// configurable behavior for testing error conditions.
#[derive(Debug, Clone)]
pub struct MockProfileManager {
    profiles: Arc<Mutex<HashMap<String, Profile>>>,
    create_should_fail: Arc<Mutex<bool>>,
    read_should_fail: Arc<Mutex<bool>>,
    update_should_fail: Arc<Mutex<bool>>,
    delete_should_fail: Arc<Mutex<bool>>,
}

impl MockProfileManager {
    /// Create a new mock profile manager.
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            create_should_fail: Arc::new(Mutex::new(false)),
            read_should_fail: Arc::new(Mutex::new(false)),
            update_should_fail: Arc::new(Mutex::new(false)),
            delete_should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a mock profile manager with pre-populated profiles.
    pub fn with_profiles(profiles: Vec<Profile>) -> Self {
        let mut profile_map = HashMap::new();
        for profile in profiles {
            profile_map.insert(profile.name.clone(), profile);
        }
        Self {
            profiles: Arc::new(Mutex::new(profile_map)),
            create_should_fail: Arc::new(Mutex::new(false)),
            read_should_fail: Arc::new(Mutex::new(false)),
            update_should_fail: Arc::new(Mutex::new(false)),
            delete_should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Configure create operation to fail.
    pub fn fail_on_create(&self, should_fail: bool) {
        *self.create_should_fail.lock().unwrap() = should_fail;
    }

    /// Configure read operation to fail.
    pub fn fail_on_read(&self, should_fail: bool) {
        *self.read_should_fail.lock().unwrap() = should_fail;
    }

    /// Configure update operation to fail.
    pub fn fail_on_update(&self, should_fail: bool) {
        *self.update_should_fail.lock().unwrap() = should_fail;
    }

    /// Configure delete operation to fail.
    pub fn fail_on_delete(&self, should_fail: bool) {
        *self.delete_should_fail.lock().unwrap() = should_fail;
    }
}

impl Default for MockProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl super::ProfileManager for MockProfileManager {
    fn create(&self, profile: Profile) -> Result<()> {
        if *self.create_should_fail.lock().unwrap() {
            return Err(GitSetupError::DuplicateProfile {
                name: profile.name.clone(),
            });
        }

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
        if *self.read_should_fail.lock().unwrap() {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        let profiles = self.profiles.lock().unwrap();
        Ok(profiles.get(name).cloned())
    }

    fn update(&self, name: &str, profile: Profile) -> Result<()> {
        if *self.update_should_fail.lock().unwrap() {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        let mut profiles = self.profiles.lock().unwrap();
        if !profiles.contains_key(name) {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        // Handle rename case
        if profile.name != name {
            if profiles.contains_key(&profile.name) {
                return Err(GitSetupError::DuplicateProfile {
                    name: profile.name.clone(),
                });
            }
            profiles.remove(name);
        }

        profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    fn delete(&self, name: &str) -> Result<()> {
        if *self.delete_should_fail.lock().unwrap() {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        let mut profiles = self.profiles.lock().unwrap();
        if !profiles.contains_key(name) {
            return Err(GitSetupError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        profiles.remove(name);
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
    fn test_mock_create_profile() {
        let manager = MockProfileManager::new();
        let profile = test_profile();

        assert!(manager.create(profile.clone()).is_ok());
        assert!(manager.exists("test").unwrap());
    }

    #[test]
    fn test_mock_create_duplicate_profile() {
        let manager = MockProfileManager::new();
        let profile = test_profile();

        assert!(manager.create(profile.clone()).is_ok());
        let result = manager.create(profile);
        assert!(matches!(result, Err(GitSetupError::DuplicateProfile { .. })));
    }

    #[test]
    fn test_mock_read_profile() {
        let manager = MockProfileManager::new();
        let profile = test_profile();
        manager.create(profile.clone()).unwrap();

        let retrieved = manager.read("test").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
    }

    #[test]
    fn test_mock_read_nonexistent_profile() {
        let manager = MockProfileManager::new();
        let result = manager.read("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_mock_update_profile() {
        let manager = MockProfileManager::new();
        let mut profile = test_profile();
        manager.create(profile.clone()).unwrap();

        profile.git_user_email = "updated@example.com".to_string();
        assert!(manager.update("test", profile).is_ok());

        let updated = manager.read("test").unwrap().unwrap();
        assert_eq!(updated.git_user_email, "updated@example.com");
    }

    #[test]
    fn test_mock_update_nonexistent_profile() {
        let manager = MockProfileManager::new();
        let profile = test_profile();

        let result = manager.update("nonexistent", profile);
        assert!(matches!(result, Err(GitSetupError::ProfileNotFound { .. })));
    }

    #[test]
    fn test_mock_delete_profile() {
        let manager = MockProfileManager::new();
        let profile = test_profile();
        manager.create(profile).unwrap();

        assert!(manager.delete("test").is_ok());
        assert!(!manager.exists("test").unwrap());
    }

    #[test]
    fn test_mock_delete_nonexistent_profile() {
        let manager = MockProfileManager::new();

        let result = manager.delete("nonexistent");
        assert!(matches!(result, Err(GitSetupError::ProfileNotFound { .. })));
    }

    #[test]
    fn test_mock_list_profiles() {
        let manager = MockProfileManager::new();
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
    fn test_mock_with_profiles() {
        let profile1 = test_profile();
        let mut profile2 = test_profile();
        profile2.name = "test2".to_string();

        let manager = MockProfileManager::with_profiles(vec![profile1, profile2]);

        assert!(manager.exists("test").unwrap());
        assert!(manager.exists("test2").unwrap());
        assert_eq!(manager.list().unwrap().len(), 2);
    }

    #[test]
    fn test_mock_fail_on_create() {
        let manager = MockProfileManager::new();
        manager.fail_on_create(true);

        let profile = test_profile();
        let result = manager.create(profile);
        assert!(matches!(result, Err(GitSetupError::DuplicateProfile { .. })));
    }

    #[test]
    fn test_mock_fail_on_read() {
        let manager = MockProfileManager::new();
        let profile = test_profile();
        manager.create(profile).unwrap();

        manager.fail_on_read(true);
        let result = manager.read("test");
        assert!(matches!(result, Err(GitSetupError::ProfileNotFound { .. })));
    }

    #[test]
    fn test_mock_rename_profile() {
        let manager = MockProfileManager::new();
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
    fn test_mock_rename_to_existing_profile() {
        let manager = MockProfileManager::new();
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
    fn test_mock_thread_safety() {
        use std::thread;

        let manager = Arc::new(MockProfileManager::new());
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
