//! Main fuzzy matcher implementation for profile matching.
//!
//! This module contains the primary FuzzyMatcher implementation that combines
//! multiple matching algorithms and provides multi-field matching capabilities.

use super::{
    MatchingAlgorithm, FuzzyMatcher as FuzzyMatcherTrait, MatchResult, FieldMatch, MatchedField,
    LevenshteinMatcher, SubstringMatcher, FuzzyAlgorithm,
};
use crate::config::types::Profile;
use std::sync::Arc;

/// Configuration for fuzzy matching behavior.
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Minimum score threshold for matches (0.0 to 1.0)
    pub min_score: f64,
    /// Minimum score for "best match" confidence (0.0 to 1.0)
    pub best_match_threshold: f64,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Whether to match against profile names
    pub match_name: bool,
    /// Whether to match against git user emails
    pub match_email: bool,
    /// Whether to match against git user names
    pub match_user_name: bool,
    /// Whether to match against vault names
    pub match_vault_name: bool,
    /// Whether to match against SSH key titles
    pub match_ssh_key_title: bool,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            min_score: 0.4,
            best_match_threshold: 0.8,
            max_results: 10,
            match_name: true,
            match_email: false,
            match_user_name: false,
            match_vault_name: false,
            match_ssh_key_title: false,
        }
    }
}

/// Primary fuzzy matcher implementation.
///
/// Combines multiple matching algorithms and provides configurable
/// multi-field matching for git profiles.
pub struct ProfileFuzzyMatcher {
    config: MatchConfig,
    primary_algorithm: Arc<dyn MatchingAlgorithm>,
    fallback_algorithms: Vec<Arc<dyn MatchingAlgorithm>>,
}

impl ProfileFuzzyMatcher {
    /// Create a new fuzzy matcher with default configuration.
    pub fn new() -> Self {
        Self {
            config: MatchConfig::default(),
            primary_algorithm: Arc::new(FuzzyAlgorithm::default()),
            fallback_algorithms: vec![
                Arc::new(SubstringMatcher::default()),
                Arc::new(LevenshteinMatcher::default()),
            ],
        }
    }

    /// Create a new fuzzy matcher with custom configuration.
    pub fn with_config(config: MatchConfig) -> Self {
        Self {
            config,
            primary_algorithm: Arc::new(FuzzyAlgorithm::default()),
            fallback_algorithms: vec![
                Arc::new(SubstringMatcher::default()),
                Arc::new(LevenshteinMatcher::default()),
            ],
        }
    }

    /// Set the primary matching algorithm.
    pub fn with_primary_algorithm(mut self, algorithm: Arc<dyn MatchingAlgorithm>) -> Self {
        self.primary_algorithm = algorithm;
        self
    }

    /// Add a fallback algorithm.
    pub fn with_fallback_algorithm(mut self, algorithm: Arc<dyn MatchingAlgorithm>) -> Self {
        self.fallback_algorithms.push(algorithm);
        self
    }

    /// Update the configuration.
    pub fn with_updated_config<F>(mut self, update_fn: F) -> Self
    where
        F: FnOnce(&mut MatchConfig),
    {
        update_fn(&mut self.config);
        self
    }

    /// Score a single profile against a query.
    fn score_profile(&self, query: &str, profile: &Profile) -> Option<MatchResult> {
        let mut field_matches = Vec::new();

        // Try matching against each enabled field
        if self.config.match_name {
            if let Some(field_match) = self.score_field(query, &profile.name, MatchedField::Name) {
                field_matches.push(field_match);
            }
        }

        if self.config.match_email {
            if let Some(field_match) = self.score_field(query, &profile.git_user_email, MatchedField::Email) {
                field_matches.push(field_match);
            }
        }

        if self.config.match_user_name {
            if let Some(user_name) = &profile.git_user_name {
                if let Some(field_match) = self.score_field(query, user_name, MatchedField::UserName) {
                    field_matches.push(field_match);
                }
            }
        }

        if self.config.match_vault_name {
            if let Some(vault_name) = &profile.vault_name {
                if let Some(field_match) = self.score_field(query, vault_name, MatchedField::VaultName) {
                    field_matches.push(field_match);
                }
            }
        }

        if self.config.match_ssh_key_title {
            if let Some(ssh_key_title) = &profile.ssh_key_title {
                if let Some(field_match) = self.score_field(query, ssh_key_title, MatchedField::SshKeyTitle) {
                    field_matches.push(field_match);
                }
            }
        }

        // Calculate weighted overall score
        if !field_matches.is_empty() {
            let weighted_score = self.calculate_weighted_score(&field_matches);

            if weighted_score >= self.config.min_score {
                return Some(MatchResult::new(
                    profile.clone(),
                    weighted_score,
                    self.primary_algorithm.name().to_string(),
                    field_matches,
                ));
            }
        }

        None
    }

    /// Score a single field against the query using all algorithms.
    fn score_field(&self, query: &str, field_value: &str, field: MatchedField) -> Option<FieldMatch> {
        let mut best_score = 0.0;

        // Try primary algorithm first
        let primary_score = self.primary_algorithm.score(query, field_value);
        if primary_score > best_score {
            best_score = primary_score;
        }

        // Try fallback algorithms
        for algorithm in &self.fallback_algorithms {
            let score = algorithm.score(query, field_value);
            if score > best_score {
                best_score = score;
            }
        }

        if best_score > 0.0 {
            let matched_text = if field_value.to_lowercase().contains(&query.to_lowercase()) {
                Some(query.to_string())
            } else {
                None
            };

            Some(FieldMatch::new(field, best_score, matched_text))
        } else {
            None
        }
    }

    /// Calculate weighted overall score from field matches.
    fn calculate_weighted_score(&self, field_matches: &[FieldMatch]) -> f64 {
        if field_matches.is_empty() {
            return 0.0;
        }

        let total_weighted_score: f64 = field_matches
            .iter()
            .map(|fm| fm.weighted_score())
            .sum();

        let total_weight: f64 = field_matches
            .iter()
            .map(|fm| fm.field.weight())
            .sum();

        if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        }
    }
}

impl Default for ProfileFuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcherTrait for ProfileFuzzyMatcher {
    fn find_matches(&self, query: &str, profiles: &[Profile]) -> Vec<MatchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results: Vec<MatchResult> = profiles
            .iter()
            .filter_map(|profile| self.score_profile(query, profile))
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(self.config.max_results);

        results
    }

    fn find_best_match(&self, query: &str, profiles: &[Profile]) -> Option<MatchResult> {
        if query.is_empty() {
            return None;
        }

        self.find_matches(query, profiles)
            .into_iter()
            .next()
            .filter(|result| result.score >= self.config.best_match_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{KeyType, Scope};

    fn create_test_profiles() -> Vec<Profile> {
        vec![
            Profile {
                name: "work-project".to_string(),
                git_user_name: Some("John Doe".to_string()),
                git_user_email: "john.doe@company.com".to_string(),
                key_type: KeyType::Ssh,
                vault_name: Some("Work Vault".to_string()),
                ssh_key_title: Some("Work Laptop SSH Key".to_string()),
                scope: Some(Scope::Local),
                signing_key: Some("ssh-ed25519 AAAAC3...".to_string()),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "personal".to_string(),
                git_user_name: Some("John Personal".to_string()),
                git_user_email: "john@personal.com".to_string(),
                key_type: KeyType::Gpg,
                vault_name: Some("Personal Vault".to_string()),
                ssh_key_title: Some("Personal SSH Key".to_string()),
                scope: Some(Scope::Global),
                signing_key: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: true,
            },
            Profile {
                name: "opensource".to_string(),
                git_user_name: Some("OSS Contributor".to_string()),
                git_user_email: "contributor@github.com".to_string(),
                key_type: KeyType::Ssh,
                vault_name: None,
                ssh_key_title: None,
                scope: Some(Scope::Global),
                signing_key: Some("ssh-ed25519 BBBBBB...".to_string()),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
        ]
    }

    #[test]
    fn test_new_matcher_has_default_config() {
        let matcher = ProfileFuzzyMatcher::new();
        assert_eq!(matcher.config.min_score, 0.4);
        assert_eq!(matcher.config.best_match_threshold, 0.8);
        assert_eq!(matcher.config.max_results, 10);
        assert!(matcher.config.match_name);
        assert!(!matcher.config.match_email);
    }

    #[test]
    fn test_exact_name_match() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        let results = matcher.find_matches("personal", &profiles);

        // There should be at least one result for exact name match
        assert!(results.len() >= 1, "Should find at least one match");

        // The first (highest scoring) result should be the exact match
        assert_eq!(results[0].profile.name, "personal");
        assert!(results[0].score > 0.9, "Expected high score for exact match, got {}", results[0].score);

        // Check that we matched on the name field
        let has_name_match = results[0].field_matches.iter()
            .any(|fm| fm.field == MatchedField::Name);
        assert!(has_name_match, "Should match on name field");
    }

    #[test]
    fn test_fuzzy_name_match() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        let results = matcher.find_matches("wrk", &profiles);
        assert!(!results.is_empty(), "Should find fuzzy matches for 'wrk'");

        // Should match "work-project"
        let work_match = results.iter().find(|r| r.profile.name == "work-project");
        assert!(work_match.is_some(), "Should match 'work-project' with fuzzy search");

        let work_match = work_match.unwrap();
        assert!(work_match.score >= 0.4, "Fuzzy match should score above threshold");
    }

    #[test]
    fn test_no_matches_below_threshold() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        let results = matcher.find_matches("xyzzyx", &profiles);
        assert!(results.is_empty(), "Should not match completely unrelated strings");
    }

    #[test]
    fn test_empty_query_returns_no_matches() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        let results = matcher.find_matches("", &profiles);
        assert!(results.is_empty(), "Empty query should return no matches");
    }

    #[test]
    fn test_results_sorted_by_score() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        let results = matcher.find_matches("o", &profiles); // Should match multiple profiles

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
    fn test_max_results_limit() {
        let config = MatchConfig {
            max_results: 2,
            min_score: 0.1, // Very low threshold to get more matches
            ..Default::default()
        };
        let matcher = ProfileFuzzyMatcher::with_config(config);
        let profiles = create_test_profiles();

        let results = matcher.find_matches("o", &profiles);
        assert!(results.len() <= 2, "Should respect max_results limit");
    }

    #[test]
    fn test_best_match_high_confidence() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        // Exact match should be high confidence
        let best = matcher.find_best_match("personal", &profiles);
        assert!(best.is_some(), "Exact match should be high confidence");
        assert_eq!(best.unwrap().profile.name, "personal");
    }

    #[test]
    fn test_best_match_low_confidence() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        // Very fuzzy match should not qualify as "best"
        let best = matcher.find_best_match("xyz", &profiles);
        assert!(best.is_none(), "Low confidence match should not be 'best'");
    }

    #[test]
    fn test_multi_field_matching() {
        let config = MatchConfig {
            match_name: true,
            match_email: true,
            match_user_name: true,
            match_vault_name: true,
            match_ssh_key_title: true,
            ..Default::default()
        };
        let matcher = ProfileFuzzyMatcher::with_config(config);
        let profiles = create_test_profiles();

        // Should match "work" in both name and vault name
        let results = matcher.find_matches("work", &profiles);
        assert!(!results.is_empty(), "Should find multi-field matches");

        let work_result = results.iter().find(|r| r.profile.name == "work-project");
        assert!(work_result.is_some(), "Should match work-project");

        let work_result = work_result.unwrap();
        assert!(work_result.field_matches.len() >= 1, "Should have field matches");

        // Check that we match the expected fields
        let has_name_match = work_result.field_matches.iter()
            .any(|fm| fm.field == MatchedField::Name);
        let has_vault_match = work_result.field_matches.iter()
            .any(|fm| fm.field == MatchedField::VaultName);

        assert!(has_name_match || has_vault_match, "Should match name or vault field");
    }

    #[test]
    fn test_field_weight_scoring() {
        let config = MatchConfig {
            match_name: true,
            match_email: true,
            ..Default::default()
        };
        let matcher = ProfileFuzzyMatcher::with_config(config);

        // Create a profile where email matches better than name
        let profile = Profile {
            name: "different".to_string(),
            git_user_email: "test@example.com".to_string(),
            git_user_name: None,
            key_type: KeyType::Ssh,
            vault_name: None,
            ssh_key_title: None,
            scope: Some(Scope::Local),
            signing_key: None,
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password: false,
        };

        let results = matcher.find_matches("test", &[profile]);

        if !results.is_empty() {
            // Name field should have higher weight than email, so even if email matches perfectly,
            // the overall score should be weighted appropriately
            assert!(results[0].score > 0.0, "Should get some score for matching");
        }
    }

    #[test]
    fn test_match_config_fields() {
        let config = MatchConfig {
            match_name: false,
            match_email: true,
            ..Default::default()
        };
        let matcher = ProfileFuzzyMatcher::with_config(config);
        let profiles = create_test_profiles();

        // Should not match on name when disabled
        let results = matcher.find_matches("personal", &profiles);
        for result in &results {
            assert!(
                !result.field_matches.iter().any(|fm| fm.field == MatchedField::Name),
                "Should not match name field when disabled"
            );
        }
    }

    #[test]
    fn test_custom_algorithms() {
        let matcher = ProfileFuzzyMatcher::new()
            .with_primary_algorithm(Arc::new(LevenshteinMatcher::default()));

        let profiles = create_test_profiles();
        let results = matcher.find_matches("personal", &profiles);

        assert!(!results.is_empty(), "Should work with custom algorithm");
        assert_eq!(results[0].algorithm, "levenshtein");
    }

    #[test]
    fn test_config_builder_pattern() {
        let matcher = ProfileFuzzyMatcher::new()
            .with_updated_config(|config| {
                config.min_score = 0.6;
                config.max_results = 5;
                config.match_email = true;
            });

        assert_eq!(matcher.config.min_score, 0.6);
        assert_eq!(matcher.config.max_results, 5);
        assert!(matcher.config.match_email);
    }

    #[test]
    fn test_weighted_score_calculation() {
        let matcher = ProfileFuzzyMatcher::new();

        let field_matches = vec![
            FieldMatch::new(MatchedField::Name, 0.8, None),      // weight 1.0
            FieldMatch::new(MatchedField::Email, 0.6, None),     // weight 0.6
        ];

        let weighted_score = matcher.calculate_weighted_score(&field_matches);

        // Expected: (0.8 * 1.0 + 0.6 * 0.6) / (1.0 + 0.6) = (0.8 + 0.36) / 1.6 = 0.725
        let expected = (0.8 * 1.0 + 0.6 * 0.6) / (1.0 + 0.6);
        assert!((weighted_score - expected).abs() < 0.001,
               "Expected weighted score {}, got {}", expected, weighted_score);
    }

    #[test]
    fn test_case_insensitive_matching() {
        let matcher = ProfileFuzzyMatcher::new();
        let profiles = create_test_profiles();

        let results_lower = matcher.find_matches("personal", &profiles);
        let results_upper = matcher.find_matches("PERSONAL", &profiles);
        let results_mixed = matcher.find_matches("Personal", &profiles);

        assert_eq!(results_lower.len(), results_upper.len());
        assert_eq!(results_lower.len(), results_mixed.len());

        if !results_lower.is_empty() {
            assert_eq!(results_lower[0].profile.name, results_upper[0].profile.name);
            assert_eq!(results_lower[0].profile.name, results_mixed[0].profile.name);
        }
    }
}
