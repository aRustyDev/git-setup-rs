//! Fuzzy matching module for profile selection.
//!
//! This module provides fuzzy matching functionality to help users find profiles
//! with partial or misspelled names. It supports multiple matching algorithms
//! and multi-field matching for enhanced usability.

pub mod algorithms;
pub mod matcher;

pub use algorithms::*;
pub use matcher::*;

use crate::config::types::Profile;

/// Trait for scoring algorithms that measure similarity between strings.
pub trait MatchingAlgorithm: Send + Sync {
    /// Calculate similarity score between query and target strings.
    ///
    /// # Arguments
    /// * `query` - The search query string
    /// * `target` - The target string to match against
    ///
    /// # Returns
    /// A score between 0.0 (no match) and 1.0 (perfect match)
    fn score(&self, query: &str, target: &str) -> f64;

    /// Get the name of this matching algorithm.
    fn name(&self) -> &'static str;
}

/// Trait for fuzzy matching implementations.
pub trait FuzzyMatcher: Send + Sync {
    /// Find all profiles matching the query above the minimum threshold.
    ///
    /// # Arguments
    /// * `query` - The search query string
    /// * `profiles` - Slice of profiles to search through
    ///
    /// # Returns
    /// Vector of match results sorted by score (highest first)
    fn find_matches(&self, query: &str, profiles: &[Profile]) -> Vec<MatchResult>;

    /// Find the best single match if confidence is high enough.
    ///
    /// # Arguments
    /// * `query` - The search query string
    /// * `profiles` - Slice of profiles to search through
    ///
    /// # Returns
    /// The best match if score is above high-confidence threshold, None otherwise
    fn find_best_match(&self, query: &str, profiles: &[Profile]) -> Option<MatchResult>;
}

/// Mock implementation of FuzzyMatcher for testing
#[cfg(test)]
pub struct MockFuzzyMatcher {
    match_results: Vec<MatchResult>,
}

#[cfg(test)]
impl MockFuzzyMatcher {
    pub fn new() -> Self {
        Self {
            match_results: Vec::new(),
        }
    }

    pub fn with_results(match_results: Vec<MatchResult>) -> Self {
        Self { match_results }
    }
}

#[cfg(test)]
impl FuzzyMatcher for MockFuzzyMatcher {
    fn find_matches(&self, _query: &str, _profiles: &[Profile]) -> Vec<MatchResult> {
        self.match_results.clone()
    }

    fn find_best_match(&self, _query: &str, _profiles: &[Profile]) -> Option<MatchResult> {
        self.match_results.first().cloned()
    }
}

/// Result of a fuzzy match operation.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched profile
    pub profile: Profile,
    /// The match score (0.0 to 1.0)
    pub score: f64,
    /// The algorithm that produced this score
    pub algorithm: String,
    /// Details about which fields matched
    pub field_matches: Vec<FieldMatch>,
}

/// Information about a match in a specific profile field.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldMatch {
    /// The field that was matched
    pub field: MatchedField,
    /// The score for this specific field
    pub score: f64,
    /// The matched substring if applicable
    pub matched_text: Option<String>,
}

/// Enum representing which profile field was matched.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchedField {
    /// Profile name field
    Name,
    /// Git user email field
    Email,
    /// Git user name field (if present)
    UserName,
    /// Vault name field (if present)
    VaultName,
    /// SSH key title field (if present)
    SshKeyTitle,
}

impl MatchedField {
    /// Get the display name for this field.
    pub fn display_name(&self) -> &'static str {
        match self {
            MatchedField::Name => "name",
            MatchedField::Email => "email",
            MatchedField::UserName => "user name",
            MatchedField::VaultName => "vault name",
            MatchedField::SshKeyTitle => "SSH key title",
        }
    }

    /// Get the field weight for scoring (higher weight = more important).
    pub fn weight(&self) -> f64 {
        match self {
            MatchedField::Name => 1.0,           // Highest priority
            MatchedField::UserName => 0.8,       // High priority
            MatchedField::Email => 0.6,          // Medium priority
            MatchedField::VaultName => 0.4,      // Lower priority
            MatchedField::SshKeyTitle => 0.3,    // Lowest priority
        }
    }
}

impl MatchResult {
    /// Create a new match result.
    pub fn new(
        profile: Profile,
        score: f64,
        algorithm: String,
        field_matches: Vec<FieldMatch>,
    ) -> Self {
        Self {
            profile,
            score,
            algorithm,
            field_matches,
        }
    }

    /// Get the primary field that was matched (highest scoring field).
    pub fn primary_field(&self) -> Option<&FieldMatch> {
        self.field_matches
            .iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    /// Check if this is a high-confidence match.
    pub fn is_high_confidence(&self) -> bool {
        self.score >= 0.8
    }

    /// Check if this is an exact match.
    pub fn is_exact(&self) -> bool {
        self.score >= 0.99
    }
}

impl FieldMatch {
    /// Create a new field match.
    pub fn new(field: MatchedField, score: f64, matched_text: Option<String>) -> Self {
        Self {
            field,
            score,
            matched_text,
        }
    }

    /// Apply field weight to the score.
    pub fn weighted_score(&self) -> f64 {
        self.score * self.field.weight()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{KeyType, Scope};

    /// Create a test profile for use in tests.
    pub fn create_test_profile(name: &str) -> Profile {
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

    #[test]
    fn test_matched_field_weights() {
        assert_eq!(MatchedField::Name.weight(), 1.0);
        assert_eq!(MatchedField::UserName.weight(), 0.8);
        assert_eq!(MatchedField::Email.weight(), 0.6);
        assert_eq!(MatchedField::VaultName.weight(), 0.4);
        assert_eq!(MatchedField::SshKeyTitle.weight(), 0.3);
    }

    #[test]
    fn test_matched_field_display_names() {
        assert_eq!(MatchedField::Name.display_name(), "name");
        assert_eq!(MatchedField::Email.display_name(), "email");
        assert_eq!(MatchedField::UserName.display_name(), "user name");
        assert_eq!(MatchedField::VaultName.display_name(), "vault name");
        assert_eq!(MatchedField::SshKeyTitle.display_name(), "SSH key title");
    }

    #[test]
    fn test_field_match_weighted_score() {
        let field_match = FieldMatch::new(MatchedField::Name, 0.8, None);
        assert_eq!(field_match.weighted_score(), 0.8);

        let field_match = FieldMatch::new(MatchedField::Email, 0.8, None);
        assert_eq!(field_match.weighted_score(), 0.48); // 0.8 * 0.6
    }

    #[test]
    fn test_match_result_confidence_levels() {
        let profile = create_test_profile("test");
        let field_matches = vec![FieldMatch::new(MatchedField::Name, 0.9, None)];

        let exact_match = MatchResult::new(
            profile.clone(),
            1.0,
            "test".to_string(),
            field_matches.clone(),
        );
        assert!(exact_match.is_exact());
        assert!(exact_match.is_high_confidence());

        let high_confidence = MatchResult::new(
            profile.clone(),
            0.85,
            "test".to_string(),
            field_matches.clone(),
        );
        assert!(!high_confidence.is_exact());
        assert!(high_confidence.is_high_confidence());

        let low_confidence = MatchResult::new(
            profile,
            0.5,
            "test".to_string(),
            field_matches,
        );
        assert!(!low_confidence.is_exact());
        assert!(!low_confidence.is_high_confidence());
    }

    #[test]
    fn test_match_result_primary_field() {
        let profile = create_test_profile("test");
        let field_matches = vec![
            FieldMatch::new(MatchedField::Email, 0.6, None),
            FieldMatch::new(MatchedField::Name, 0.9, None),
            FieldMatch::new(MatchedField::VaultName, 0.4, None),
        ];

        let match_result = MatchResult::new(
            profile,
            0.9,
            "test".to_string(),
            field_matches,
        );

        let primary = match_result.primary_field().unwrap();
        assert_eq!(primary.field, MatchedField::Name);
        assert_eq!(primary.score, 0.9);
    }

    #[test]
    fn test_create_test_profile() {
        let profile = create_test_profile("work");
        assert_eq!(profile.name, "work");
        assert_eq!(profile.git_user_email, "work@example.com");
        assert_eq!(profile.git_user_name, Some("work User".to_string()));
        assert_eq!(profile.vault_name, Some("work Vault".to_string()));
        assert_eq!(profile.ssh_key_title, Some("work SSH Key".to_string()));
        assert!(matches!(profile.key_type, KeyType::Ssh));
        assert!(matches!(profile.scope, Some(Scope::Local)));
    }
}
