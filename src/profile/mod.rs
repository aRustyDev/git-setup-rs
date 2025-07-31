//! Profile management module for git-setup-rs.
//!
//! This module provides functionality for managing git configuration profiles,
//! including CRUD operations, validation, and persistence.

pub mod manager;
pub mod mock;

use crate::{config::types::Profile, error::Result, matching::{MatchResult, ProfileFuzzyMatcher, FuzzyMatcher}};

/// Trait defining all profile management operations.
///
/// This trait provides a unified interface for profile management, allowing
/// different implementations (in-memory, file-based, etc.) while maintaining
/// consistency across the application.
pub trait ProfileManager: Send + Sync {
    /// Create a new profile.
    ///
    /// # Arguments
    /// * `profile` - The profile to create
    ///
    /// # Errors
    /// Returns an error if:
    /// - A profile with the same name already exists
    /// - The profile data is invalid
    fn create(&self, profile: Profile) -> Result<()>;

    /// Read a profile by name.
    ///
    /// # Arguments
    /// * `name` - The name of the profile to retrieve
    ///
    /// # Returns
    /// - `Ok(Some(profile))` if the profile exists
    /// - `Ok(None)` if the profile does not exist
    /// - `Err` if an error occurs during retrieval
    fn read(&self, name: &str) -> Result<Option<Profile>>;

    /// Update an existing profile.
    ///
    /// # Arguments
    /// * `name` - The name of the profile to update
    /// * `profile` - The new profile data
    ///
    /// # Errors
    /// Returns an error if:
    /// - The profile does not exist
    /// - The new profile data is invalid
    /// - The profile is being renamed to a name that already exists
    fn update(&self, name: &str, profile: Profile) -> Result<()>;

    /// Delete a profile by name.
    ///
    /// # Arguments
    /// * `name` - The name of the profile to delete
    ///
    /// # Errors
    /// Returns an error if the profile does not exist
    fn delete(&self, name: &str) -> Result<()>;

    /// List all profiles.
    ///
    /// # Returns
    /// A vector of all profiles, sorted by name
    fn list(&self) -> Result<Vec<Profile>>;

    /// Check if a profile exists.
    ///
    /// # Arguments
    /// * `name` - The name of the profile to check
    ///
    /// # Returns
    /// `true` if the profile exists, `false` otherwise
    fn exists(&self, name: &str) -> Result<bool>;

    /// Find profiles using fuzzy matching.
    ///
    /// # Arguments
    /// * `query` - The search query string
    ///
    /// # Returns
    /// A vector of match results sorted by score (highest first)
    fn fuzzy_find(&self, query: &str) -> Result<Vec<MatchResult>> {
        let profiles = self.list()?;
        let matcher = ProfileFuzzyMatcher::new();
        Ok(matcher.find_matches(query, &profiles))
    }

    /// Get the best fuzzy match for a query.
    ///
    /// # Arguments
    /// * `query` - The search query string
    ///
    /// # Returns
    /// The best match if confidence is high enough, None otherwise
    fn fuzzy_best(&self, query: &str) -> Result<Option<Profile>> {
        let profiles = self.list()?;
        let matcher = ProfileFuzzyMatcher::new();
        Ok(matcher.find_best_match(query, &profiles)
            .map(|result| result.profile))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::config::types::{KeyType, Scope};

    /// Test that the ProfileManager trait is object-safe
    #[test]
    fn test_trait_is_object_safe() {
        // This test ensures the trait can be used as a trait object
        fn _assert_object_safe(_: &dyn ProfileManager) {}
    }

    /// Test that ProfileManager trait has Send + Sync bounds
    #[test]
    fn test_trait_has_send_sync_bounds() {
        fn _assert_send_sync<T: Send + Sync>() {}
        fn _assert_profile_manager_bounds<T: ProfileManager>() {
            _assert_send_sync::<T>();
        }
    }

    /// Test that Arc<dyn ProfileManager> can be used
    #[test]
    fn test_arc_dyn_profile_manager() {
        // This pattern is common for sharing the manager across threads
        fn _assert_arc_usage(_: Arc<dyn ProfileManager>) {}
    }

    /// Create a test profile for use in tests
    fn create_test_profile(name: &str) -> Profile {
        Profile {
            name: name.to_string(),
            git_user_name: Some(format!("{} User", name)),
            git_user_email: format!("{}@example.com", name.to_lowercase()),
            key_type: KeyType::Ssh,
            signing_key: Some("ssh-ed25519 AAAAC3...".to_string()),
            vault_name: Some(format!("{} Vault", name)),
            ssh_key_title: Some(format!("{} SSH Key", name)),
            scope: Some(Scope::Local),
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password: false,
        }
    }

    /// Mock implementation of ProfileManager for testing
    struct MockProfileManager {
        profiles: Vec<Profile>,
    }

    impl MockProfileManager {
        fn new() -> Self {
            Self {
                profiles: vec![
                    create_test_profile("work-project"),
                    create_test_profile("personal"),
                    create_test_profile("opensource"),
                ],
            }
        }
    }

    impl ProfileManager for MockProfileManager {
        fn create(&self, _profile: Profile) -> Result<()> {
            Ok(())
        }

        fn read(&self, name: &str) -> Result<Option<Profile>> {
            Ok(self.profiles.iter().find(|p| p.name == name).cloned())
        }

        fn update(&self, _name: &str, _profile: Profile) -> Result<()> {
            Ok(())
        }

        fn delete(&self, _name: &str) -> Result<()> {
            Ok(())
        }

        fn list(&self) -> Result<Vec<Profile>> {
            Ok(self.profiles.clone())
        }

        fn exists(&self, name: &str) -> Result<bool> {
            Ok(self.profiles.iter().any(|p| p.name == name))
        }
    }

    #[test]
    fn test_fuzzy_find_integration() {
        let manager = MockProfileManager::new();

        // Test exact match
        let results = manager.fuzzy_find("personal").unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].profile.name, "personal");
        assert!(results[0].score > 0.9);

        // Test fuzzy match
        let results = manager.fuzzy_find("wrk").unwrap();
        assert!(!results.is_empty());
        let work_match = results.iter().find(|r| r.profile.name == "work-project");
        assert!(work_match.is_some());

        // Test no match
        let results = manager.fuzzy_find("xyzzyx").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_best_integration() {
        let manager = MockProfileManager::new();

        // Test exact match returns best
        let best = manager.fuzzy_best("personal").unwrap();
        assert!(best.is_some());
        assert_eq!(best.unwrap().name, "personal");

        // Test low confidence returns None
        let best = manager.fuzzy_best("xyz").unwrap();
        assert!(best.is_none());

        // Test empty query returns None
        let best = manager.fuzzy_best("").unwrap();
        assert!(best.is_none());
    }

    #[test]
    fn test_fuzzy_methods_use_profile_list() {
        let manager = MockProfileManager::new();

        // Verify the fuzzy methods use the same profiles as list()
        let all_profiles = manager.list().unwrap();
        let fuzzy_results = manager.fuzzy_find("o").unwrap(); // Should match multiple profiles

        // All fuzzy results should be from the profile list
        for result in fuzzy_results {
            assert!(all_profiles.iter().any(|p| p.name == result.profile.name));
        }
    }

    #[test]
    fn test_fuzzy_find_sorted_by_score() {
        let manager = MockProfileManager::new();

        let results = manager.fuzzy_find("o").unwrap();

        // Results should be sorted by score (highest first)
        if results.len() > 1 {
            for i in 1..results.len() {
                assert!(
                    results[i - 1].score >= results[i].score,
                    "Results should be sorted by score descending"
                );
            }
        }
    }

    #[test]
    fn test_fuzzy_methods_handle_empty_profile_list() {
        struct EmptyProfileManager;

        impl ProfileManager for EmptyProfileManager {
            fn create(&self, _profile: Profile) -> Result<()> { Ok(()) }
            fn read(&self, _name: &str) -> Result<Option<Profile>> { Ok(None) }
            fn update(&self, _name: &str, _profile: Profile) -> Result<()> { Ok(()) }
            fn delete(&self, _name: &str) -> Result<()> { Ok(()) }
            fn list(&self) -> Result<Vec<Profile>> { Ok(vec![]) }
            fn exists(&self, _name: &str) -> Result<bool> { Ok(false) }
        }

        let manager = EmptyProfileManager;

        let results = manager.fuzzy_find("anything").unwrap();
        assert!(results.is_empty());

        let best = manager.fuzzy_best("anything").unwrap();
        assert!(best.is_none());
    }
}
