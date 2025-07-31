//! Main auto-detection implementation.
//!
//! This module implements the main AutoDetector that combines multiple detection
//! rules to automatically select the best matching profile for the current context.

use super::{
    context::{ContextExtractor, RepositoryContext},
    rules::*,
    DetectionConfig, DetectionResult, MatchedRule, ProfileDetector,
};
use crate::{
    config::types::Profile, error::Result, external::git::GitWrapper, profile::ProfileManager,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub struct AutoDetector<P: ProfileManager, G: GitWrapper> {
    profile_manager: Arc<P>,
    git: Arc<G>,
    config: DetectionConfig,
    rules: Vec<Box<dyn DetectionRule>>,
    cache: HashMap<String, DetectionResult>,
}

impl<P: ProfileManager, G: GitWrapper + Clone + Send + Sync> AutoDetector<P, G> {
    pub fn new(profile_manager: Arc<P>, git: Arc<G>) -> Self {
        Self::with_config(profile_manager, git, DetectionConfig::default())
    }

    pub fn with_config(
        profile_manager: Arc<P>,
        git: Arc<G>,
        config: DetectionConfig,
    ) -> Self {
        let rules = Self::build_rules(&config);

        Self {
            profile_manager,
            git,
            config,
            rules,
            cache: HashMap::new(),
        }
    }

    fn build_rules(config: &DetectionConfig) -> Vec<Box<dyn DetectionRule>> {
        let mut rules: Vec<Box<dyn DetectionRule>> = Vec::new();

        if config.check_remote_url {
            rules.push(Box::new(RemoteUrlRule::new()));
        }
        if config.check_directory {
            rules.push(Box::new(DirectoryPathRule::new()));
        }
        if config.check_include_if {
            rules.push(Box::new(IncludeIfDirRule::new()));
        }
        if config.check_hostname {
            rules.push(Box::new(HostnameRule::new()));
        }
        if config.check_git_config {
            rules.push(Box::new(GitConfigRule::new()));
        }

        rules
    }

    fn score_profile(
        &self,
        profile: &Profile,
        context: &RepositoryContext,
    ) -> Option<DetectionResult> {
        let mut matched_rules = Vec::new();
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        // Run all rules
        for rule in &self.rules {
            if let Some(score) = rule.matches(profile, context) {
                let priority = rule.priority();
                let weight = priority as u8 as f64 / 100.0;

                matched_rules.push(MatchedRule {
                    rule_name: rule.name().to_string(),
                    priority,
                    confidence: score,
                });

                total_score += score * weight;
                total_weight += weight;
            }
        }

        if matched_rules.is_empty() {
            return None;
        }

        // Calculate weighted average
        let confidence = total_score / total_weight;

        if confidence < self.config.min_confidence {
            return None;
        }

        // Build reason string
        let reason = self.build_reason(&matched_rules, profile);

        Some(DetectionResult {
            profile: profile.clone(),
            confidence,
            matched_rules: matched_rules.clone(),
            reason: reason.clone(),
            reasons: vec![reason],
        })
    }

    fn build_reason(&self, rules: &[MatchedRule], profile: &Profile) -> String {
        let mut parts = Vec::new();

        // Sort by priority and confidence
        let mut sorted_rules = rules.to_vec();
        sorted_rules.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(b.confidence.partial_cmp(&a.confidence).unwrap())
        });

        for rule in sorted_rules.iter().take(2) {
            match rule.rule_name.as_str() {
                "remote_url" => parts.push("repository URL matches".to_string()),
                "directory_path" => parts.push("directory pattern matches".to_string()),
                "include_if_dir" => parts.push("in configured directory".to_string()),
                "hostname" => parts.push("hostname matches".to_string()),
                "git_config" => parts.push("git config matches".to_string()),
                _ => {}
            }
        }

        if parts.is_empty() {
            format!("Profile '{}' detected", profile.name)
        } else {
            format!("Profile '{}' detected: {}", profile.name, parts.join(", "))
        }
    }
}

impl<P: ProfileManager, G: GitWrapper + Clone + Send + Sync> ProfileDetector for AutoDetector<P, G> {
    fn detect(&self) -> Result<Option<DetectionResult>> {
        let current_dir = std::env::current_dir()?;
        self.detect_in(&current_dir)
    }

    fn detect_in(&self, path: &Path) -> Result<Option<DetectionResult>> {
        // Check cache if enabled
        if self.config.enable_cache {
            let cache_key = path.to_string_lossy().to_string();
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(Some(cached.clone()));
            }
        }

        // Extract repository context
        let extractor = ContextExtractor::new((*self.git).clone());
        let context = extractor.extract_in(path)?;

        // Get all profiles
        let profiles = self.profile_manager.list()?;

        // Score each profile
        let mut results: Vec<DetectionResult> = profiles
            .iter()
            .filter_map(|profile| self.score_profile(profile, &context))
            .collect();

        // Sort by confidence
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // TODO: Cache result if enabled - requires mutable access to cache
        // For now, we don't cache to maintain the immutable interface

        Ok(results.into_iter().next())
    }

    fn detect_all(&self) -> Result<Vec<DetectionResult>> {
        let current_dir = std::env::current_dir()?;

        // Extract repository context
        let extractor = ContextExtractor::new((*self.git).clone());
        let context = extractor.extract_in(&current_dir)?;

        // Get all profiles
        let profiles = self.profile_manager.list()?;

        // Score each profile
        let mut results: Vec<DetectionResult> = profiles
            .iter()
            .filter_map(|profile| self.score_profile(profile, &context))
            .collect();

        // Sort by confidence
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::types::{KeyType, Scope},
        external::git::MockGitWrapper,
        profile::mock::MockProfileManager,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn test_profiles() -> Vec<Profile> {
        vec![
            Profile {
                name: "work".to_string(),
                git_user_name: Some("Work User".to_string()),
                git_user_email: "work@company.com".to_string(),
                repos: vec!["git@github.com:company/*".to_string()],
                match_patterns: vec!["*/work/*".to_string()],
                include_if_dirs: vec![],
                host_patterns: vec![],
                key_type: KeyType::Ssh,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: Some(Scope::Local),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                one_password: false,
            },
            Profile {
                name: "personal".to_string(),
                git_user_name: Some("Personal User".to_string()),
                git_user_email: "me@personal.com".to_string(),
                repos: vec!["git@github.com:myuser/*".to_string()],
                match_patterns: vec![],
                include_if_dirs: vec!["/home/user/personal".to_string()],
                host_patterns: vec![],
                key_type: KeyType::Gpg,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: Some(Scope::Global),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                one_password: false,
            },
            Profile {
                name: "hostname-test".to_string(),
                git_user_name: Some("Hostname User".to_string()),
                git_user_email: "test@hostname.com".to_string(),
                repos: vec![],
                match_patterns: vec![],
                include_if_dirs: vec![],
                host_patterns: vec!["test-*".to_string()],
                key_type: KeyType::Ssh,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: Some(Scope::Local),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                one_password: false,
            },
        ]
    }

    #[test]
    fn test_detection_by_remote_url() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let mut config = HashMap::new();
        config.insert(
            "remote.origin.url".to_string(),
            "git@github.com:company/project.git".to_string(),
        );
        let git = Arc::new(MockGitWrapper::new().with_config(config));

        let detector = AutoDetector::new(profile_manager, git);

        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let result = detector.detect_in(temp_dir.path()).unwrap();
        assert!(result.is_some());

        let detection = result.unwrap();
        assert_eq!(detection.profile.name, "work");
        assert!(detection.confidence > 0.7);
        assert!(detection.reason.contains("repository URL"));
    }

    #[test]
    fn test_detection_by_directory() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let git = Arc::new(MockGitWrapper::new());
        let detector = AutoDetector::new(profile_manager, git);

        let work_dir = PathBuf::from("/home/user/work/project");
        let result = detector.detect_in(&work_dir).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().profile.name, "work");
    }

    #[test]
    fn test_detection_by_include_if_dir() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let git = Arc::new(MockGitWrapper::new());
        let detector = AutoDetector::new(profile_manager, git);

        let personal_dir = PathBuf::from("/home/user/personal/project");
        let result = detector.detect_in(&personal_dir).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().profile.name, "personal");
    }

    #[test]
    fn test_no_detection() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let git = Arc::new(MockGitWrapper::new());
        let detector = AutoDetector::new(profile_manager, git);

        let unrelated_dir = PathBuf::from("/tmp/random");
        let result = detector.detect_in(&unrelated_dir).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_confidence_threshold() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let git = Arc::new(MockGitWrapper::new());
        let config = DetectionConfig {
            min_confidence: 0.9, // Very high threshold
            ..Default::default()
        };

        let detector = AutoDetector::with_config(profile_manager, git, config);

        let work_dir = PathBuf::from("/home/user/work/project");
        let result = detector.detect_in(&work_dir).unwrap();

        // Should not match with such high threshold
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_all() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let mut config = HashMap::new();
        config.insert(
            "remote.origin.url".to_string(),
            "git@github.com:company/project.git".to_string(),
        );
        let git = Arc::new(MockGitWrapper::new().with_config(config));

        let detector = AutoDetector::new(profile_manager, git);

        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // Change to temp directory for detect_all
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let results = detector.detect_all().unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].profile.name, "work");
        // Results should be sorted by confidence
        if results.len() > 1 {
            for i in 1..results.len() {
                assert!(results[i - 1].confidence >= results[i].confidence);
            }
        }
    }

    #[test]
    fn test_multiple_rule_matches() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let mut config = HashMap::new();
        config.insert(
            "remote.origin.url".to_string(),
            "git@github.com:company/project.git".to_string(),
        );
        config.insert("user.email".to_string(), "work@company.com".to_string());
        let git = Arc::new(MockGitWrapper::new().with_config(config));

        let detector = AutoDetector::new(profile_manager, git);

        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let work_subdir = temp_dir.path().join("work").join("project");
        std::fs::create_dir_all(&work_subdir).unwrap();

        let result = detector.detect_in(&work_subdir).unwrap();
        assert!(result.is_some());

        let detection = result.unwrap();
        assert_eq!(detection.profile.name, "work");
        // Should have multiple matched rules
        assert!(detection.matched_rules.len() > 1);
        // Higher confidence due to multiple matches
        assert!(detection.confidence > 0.8);
    }

    #[test]
    fn test_build_rules_with_config() {
        let config = DetectionConfig {
            check_remote_url: true,
            check_directory: false,
            check_include_if: true,
            check_hostname: false,
            check_git_config: true,
            ..Default::default()
        };

        let rules = AutoDetector::<MockProfileManager, MockGitWrapper>::build_rules(&config);

        assert_eq!(rules.len(), 3); // remote_url, include_if, git_config

        let rule_names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
        assert!(rule_names.contains(&"remote_url"));
        assert!(rule_names.contains(&"include_if_dir"));
        assert!(rule_names.contains(&"git_config"));
        assert!(!rule_names.contains(&"directory_path"));
        assert!(!rule_names.contains(&"hostname"));
    }

    #[test]
    fn test_detector_with_custom_config() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let git = Arc::new(MockGitWrapper::new());

        let config = DetectionConfig {
            min_confidence: 0.3,
            check_remote_url: false,
            check_directory: true,
            check_include_if: false,
            check_hostname: false,
            check_git_config: false,
            enable_cache: false,
        };

        let detector = AutoDetector::with_config(profile_manager, git, config);

        // Should only use directory matching
        let work_dir = PathBuf::from("/home/user/work/project");
        let result = detector.detect_in(&work_dir).unwrap();
        assert!(result.is_some());

        let detection = result.unwrap();
        assert_eq!(detection.profile.name, "work");
        assert_eq!(detection.matched_rules.len(), 1);
        assert_eq!(detection.matched_rules[0].rule_name, "directory_path");
    }

    #[test]
    fn test_reason_building() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));

        let mut config = HashMap::new();
        config.insert(
            "remote.origin.url".to_string(),
            "git@github.com:company/project.git".to_string(),
        );
        config.insert("user.email".to_string(), "work@company.com".to_string());
        let git = Arc::new(MockGitWrapper::new().with_config(config));

        let detector = AutoDetector::new(profile_manager, git);

        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let result = detector.detect_in(temp_dir.path()).unwrap();
        assert!(result.is_some());

        let detection = result.unwrap();
        assert!(detection.reason.contains("Profile 'work' detected"));
        // Should mention the highest priority matches
        assert!(
            detection.reason.contains("repository URL") ||
            detection.reason.contains("git config")
        );
    }

    #[test]
    fn test_empty_profiles_list() {
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![]));
        let git = Arc::new(MockGitWrapper::new());

        let detector = AutoDetector::new(profile_manager, git);

        let temp_dir = TempDir::new().unwrap();
        let result = detector.detect_in(temp_dir.path()).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_profile_manager_error_handling() {
        use crate::error::GitSetupError;

        struct FailingProfileManager;

        impl ProfileManager for FailingProfileManager {
            fn create(&self, _profile: Profile) -> Result<()> {
                Err(GitSetupError::InvalidProfile { reason: "test".to_string() })
            }
            fn read(&self, _name: &str) -> Result<Option<Profile>> {
                Err(GitSetupError::InvalidProfile { reason: "test".to_string() })
            }
            fn update(&self, _name: &str, _profile: Profile) -> Result<()> {
                Err(GitSetupError::InvalidProfile { reason: "test".to_string() })
            }
            fn delete(&self, _name: &str) -> Result<()> {
                Err(GitSetupError::InvalidProfile { reason: "test".to_string() })
            }
            fn list(&self) -> Result<Vec<Profile>> {
                Err(GitSetupError::InvalidProfile { reason: "test".to_string() })
            }
            fn exists(&self, _name: &str) -> Result<bool> {
                Err(GitSetupError::InvalidProfile { reason: "test".to_string() })
            }
        }

        let profile_manager = Arc::new(FailingProfileManager);
        let git = Arc::new(MockGitWrapper::new());

        let detector = AutoDetector::new(profile_manager, git);

        let temp_dir = TempDir::new().unwrap();
        let result = detector.detect_in(temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_git_error_handling() {
        let profiles = test_profiles();
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let git = Arc::new(MockGitWrapper::new().with_failure());

        let detector = AutoDetector::new(profile_manager, git);

        let temp_dir = TempDir::new().unwrap();
        let result = detector.detect_in(temp_dir.path());

        // Should handle git errors gracefully and still return results
        // based on non-git rules (like directory patterns)
        assert!(result.is_ok());
    }
}
