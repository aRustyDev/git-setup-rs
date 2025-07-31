//! Auto detection module for git-setup-rs.
//!
//! This module provides automatic profile detection based on repository context,
//! directory location, remote URLs, hostname, and other heuristics to intelligently
//! select the appropriate Git profile.

pub mod context;
pub mod detector;
pub mod rules;

pub use context::{ContextExtractor, RemoteInfo, RepositoryContext};
pub use detector::AutoDetector;
pub use rules::{DetectionRule, RulePriority};

use crate::{config::types::Profile, error::Result};
use std::path::Path;

/// Result of auto-detection
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// The detected profile
    pub profile: Profile,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Which rule(s) matched
    pub matched_rules: Vec<MatchedRule>,
    /// Why this profile was selected
    pub reason: String,
    /// Detailed reasons for the detection
    pub reasons: Vec<String>,
}

/// Information about a matched rule
#[derive(Debug, Clone)]
pub struct MatchedRule {
    pub rule_name: String,
    pub priority: RulePriority,
    pub confidence: f64,
}

/// Trait for auto-detection strategies
pub trait ProfileDetector: Send + Sync {
    /// Detect the best profile for current context
    fn detect(&self) -> Result<Option<DetectionResult>>;

    /// Detect with a specific working directory
    fn detect_in(&self, path: &Path) -> Result<Option<DetectionResult>>;

    /// Get all possible matches with scores
    fn detect_all(&self) -> Result<Vec<DetectionResult>>;

    /// Detect profile for a given repository context
    fn detect_profile(&self, context: &RepositoryContext) -> Result<DetectionResult>;
}

/// Configuration for auto-detection
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Minimum confidence threshold (0.0 - 1.0)
    pub min_confidence: f64,

    /// Enable remote URL matching
    pub check_remote_url: bool,

    /// Enable directory path matching
    pub check_directory: bool,

    /// Enable include-if directory matching
    pub check_include_if: bool,

    /// Enable hostname matching
    pub check_hostname: bool,

    /// Enable existing git config matching
    pub check_git_config: bool,

    /// Cache detection results
    pub enable_cache: bool,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            check_remote_url: true,
            check_directory: true,
            check_include_if: true,
            check_hostname: true,
            check_git_config: true,
            enable_cache: true,
        }
    }
}

/// Mock implementation of ProfileDetector for testing
#[cfg(test)]
pub struct MockProfileDetector {
    detection_result: Option<DetectionResult>,
}

#[cfg(test)]
impl MockProfileDetector {
    pub fn new() -> Self {
        Self {
            detection_result: None,
        }
    }

    pub fn set_detection_result(&mut self, result: DetectionResult) {
        self.detection_result = Some(result);
    }
}

#[cfg(test)]
impl ProfileDetector for MockProfileDetector {
    fn detect(&self) -> Result<Option<DetectionResult>> {
        Ok(self.detection_result.clone())
    }

    fn detect_in(&self, _path: &Path) -> Result<Option<DetectionResult>> {
        Ok(self.detection_result.clone())
    }

    fn detect_all(&self) -> Result<Vec<DetectionResult>> {
        if let Some(result) = &self.detection_result {
            Ok(vec![result.clone()])
        } else {
            Ok(vec![])
        }
    }

    fn detect_profile(&self, _context: &RepositoryContext) -> Result<DetectionResult> {
        self.detection_result.clone().ok_or_else(|| {
            crate::error::GitSetupError::Git("No detection result set".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_config_default() {
        let config = DetectionConfig::default();
        assert_eq!(config.min_confidence, 0.6);
        assert!(config.check_remote_url);
        assert!(config.check_directory);
        assert!(config.check_include_if);
        assert!(config.check_hostname);
        assert!(config.check_git_config);
        assert!(config.enable_cache);
    }

    #[test]
    fn test_detection_result_debug() {
        use crate::config::types::{KeyType, Scope};

        let profile = Profile {
            name: "test".to_string(),
            git_user_name: Some("Test User".to_string()),
            git_user_email: "test@example.com".to_string(),
            key_type: KeyType::Ssh,
            signing_key: None,
            vault_name: None,
            ssh_key_title: None,
            scope: Some(Scope::Local),
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password: false,
        };

        let result = DetectionResult {
            profile,
            confidence: 0.8,
            matched_rules: vec![],
            reason: "Test detection".to_string(),
            reasons: vec!["Test detection".to_string()],
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("DetectionResult"));
        assert!(debug_str.contains("confidence: 0.8"));
    }

    #[test]
    fn test_matched_rule_debug() {
        let rule = MatchedRule {
            rule_name: "test_rule".to_string(),
            priority: RulePriority::High,
            confidence: 0.9,
        };

        let debug_str = format!("{:?}", rule);
        assert!(debug_str.contains("MatchedRule"));
        assert!(debug_str.contains("test_rule"));
    }

    #[test]
    fn test_profile_detector_trait_bounds() {
        // Verify the trait has the required bounds
        fn _assert_send_sync<T: Send + Sync>() {}
        fn _assert_profile_detector_bounds<T: ProfileDetector>() {
            _assert_send_sync::<T>();
        }
    }
}
