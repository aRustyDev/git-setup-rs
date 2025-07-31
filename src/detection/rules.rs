//! Detection rules for automatic profile selection.
//!
//! This module implements various rules for detecting which profile should be
//! used based on repository context, directory patterns, hostname, and other factors.

use super::context::RepositoryContext;
use crate::config::types::Profile;
use regex::Regex;
use std::path::Path;

/// Priority levels for detection rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RulePriority {
    /// Highest priority - exact matches
    Exact = 100,
    /// High priority - strong indicators
    High = 75,
    /// Medium priority - good indicators
    Medium = 50,
    /// Low priority - weak indicators
    Low = 25,
}

/// Trait for detection rules
pub trait DetectionRule: Send + Sync {
    /// Name of this rule
    fn name(&self) -> &str;

    /// Priority of this rule
    fn priority(&self) -> RulePriority;

    /// Check if this rule matches
    fn matches(&self, profile: &Profile, context: &RepositoryContext) -> Option<f64>;
}

/// Rule: Match by remote URL
pub struct RemoteUrlRule {
    name: String,
}

impl RemoteUrlRule {
    pub fn new() -> Self {
        Self {
            name: "remote_url".to_string(),
        }
    }

    fn score_url_match(&self, profile_pattern: &str, remote_url: &str) -> f64 {
        // Exact match gets highest score
        if profile_pattern == remote_url {
            return 1.0;
        }

        // Convert pattern to regex
        let pattern = profile_pattern
            .replace("*", ".*")
            .replace("?", ".");

        if let Ok(regex) = Regex::new(&format!("^{}$", pattern)) {
            if regex.is_match(remote_url) {
                return 0.95;
            }
        }

        // Partial match scoring
        if remote_url.contains(profile_pattern) {
            0.7
        } else {
            0.0
        }
    }
}

impl Default for RemoteUrlRule {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionRule for RemoteUrlRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> RulePriority {
        RulePriority::High
    }

    fn matches(&self, profile: &Profile, context: &RepositoryContext) -> Option<f64> {
        // Check if profile has repository patterns
        if profile.repos.is_empty() {
            return None;
        }

        // Check against all remotes
        let mut best_score: f64 = 0.0;

        for remote in &context.remotes {
            for repo_pattern in &profile.repos {
                let score = self.score_url_match(repo_pattern, &remote.url);
                best_score = best_score.max(score);

                if let Some(push_url) = &remote.push_url {
                    let push_score = self.score_url_match(repo_pattern, push_url);
                    best_score = best_score.max(push_score);
                }
            }
        }

        if best_score > 0.0 {
            Some(best_score)
        } else {
            None
        }
    }
}

/// Rule: Match by directory path
pub struct DirectoryPathRule {
    name: String,
}

impl DirectoryPathRule {
    pub fn new() -> Self {
        Self {
            name: "directory_path".to_string(),
        }
    }

    fn matches_pattern(&self, pattern: &str, path: &Path) -> bool {
        // Check if pattern is contained in any path component
        for component in path.components() {
            if let Some(comp_str) = component.as_os_str().to_str() {
                if self.glob_match(pattern, comp_str) {
                    return true;
                }
            }
        }

        // Check full path match
        if let Some(path_str) = path.to_str() {
            self.glob_match(pattern, path_str)
        } else {
            false
        }
    }

    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        let regex_pattern = pattern
            .replace("*", ".*")
            .replace("?", ".");

        Regex::new(&format!("^{}$", regex_pattern))
            .ok()
            .map(|re| re.is_match(text))
            .unwrap_or(false)
    }
}

impl Default for DirectoryPathRule {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionRule for DirectoryPathRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> RulePriority {
        RulePriority::Medium
    }

    fn matches(&self, profile: &Profile, context: &RepositoryContext) -> Option<f64> {
        if profile.match_patterns.is_empty() {
            return None;
        }

        // Check working directory against patterns
        for pattern in &profile.match_patterns {
            if self.matches_pattern(pattern, &context.working_dir) {
                return Some(0.8);
            }

            // Check parent directories with decreasing score
            for (i, parent) in context.parent_dirs.iter().enumerate() {
                if self.matches_pattern(pattern, parent) {
                    let depth_penalty = 0.1 * i as f64;
                    return Some((0.7 - depth_penalty).max(0.4));
                }
            }
        }

        None
    }
}

/// Rule: Match by include-if directories
pub struct IncludeIfDirRule {
    name: String,
}

impl IncludeIfDirRule {
    pub fn new() -> Self {
        Self {
            name: "include_if_dir".to_string(),
        }
    }
}

impl Default for IncludeIfDirRule {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionRule for IncludeIfDirRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> RulePriority {
        RulePriority::High
    }

    fn matches(&self, profile: &Profile, context: &RepositoryContext) -> Option<f64> {
        if profile.include_if_dirs.is_empty() {
            return None;
        }

        // Check if we're inside any of the include directories
        for include_dir in &profile.include_if_dirs {
            let include_path = Path::new(include_dir);

            if context.working_dir.starts_with(include_path) {
                return Some(0.9);
            }

            // Check if any parent is the include directory
            for parent in &context.parent_dirs {
                if parent == include_path {
                    return Some(0.85);
                }
            }
        }

        None
    }
}

/// Rule: Match by hostname
pub struct HostnameRule {
    name: String,
}

impl HostnameRule {
    pub fn new() -> Self {
        Self {
            name: "hostname".to_string(),
        }
    }
}

impl Default for HostnameRule {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionRule for HostnameRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> RulePriority {
        RulePriority::Medium
    }

    fn matches(&self, profile: &Profile, context: &RepositoryContext) -> Option<f64> {
        if profile.host_patterns.is_empty() {
            return None;
        }

        for pattern in &profile.host_patterns {
            // Exact match
            if pattern == &context.hostname {
                return Some(0.9);
            }

            // Pattern match
            let regex_pattern = pattern
                .replace("*", ".*")
                .replace("?", ".");

            if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
                if regex.is_match(&context.hostname) {
                    return Some(0.7);
                }
            }
        }

        None
    }
}

/// Rule: Match by existing git config
pub struct GitConfigRule {
    name: String,
}

impl GitConfigRule {
    pub fn new() -> Self {
        Self {
            name: "git_config".to_string(),
        }
    }
}

impl Default for GitConfigRule {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectionRule for GitConfigRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> RulePriority {
        RulePriority::Low
    }

    fn matches(&self, profile: &Profile, context: &RepositoryContext) -> Option<f64> {
        let mut score: f64 = 0.0;

        // If current email matches profile email
        if let Some(current_email) = &context.current_email {
            if current_email == &profile.git_user_email {
                score += 0.6;
            }
        }

        // If current name matches profile name
        if let (Some(current_name), Some(profile_name)) =
            (&context.current_name, &profile.git_user_name)
        {
            if current_name == profile_name {
                score += 0.5;
            }
        }

        if score > 0.0 {
            Some(score.min(1.0))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{KeyType, Scope};
    use std::path::PathBuf;

    fn test_profile() -> Profile {
        Profile {
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
        }
    }

    fn test_context() -> RepositoryContext {
        use super::super::context::RemoteInfo;

        RepositoryContext {
            working_dir: PathBuf::from("/home/user/projects/work"),
            repo_root: Some(PathBuf::from("/home/user/projects/work")),
            remotes: vec![RemoteInfo {
                name: "origin".to_string(),
                url: "git@github.com:company/project.git".to_string(),
                push_url: None,
            }],
            current_email: Some("test@example.com".to_string()),
            current_name: Some("Test User".to_string()),
            hostname: "work-laptop".to_string(),
            parent_dirs: vec![
                PathBuf::from("/home/user/projects"),
                PathBuf::from("/home/user"),
            ],
        }
    }

    #[test]
    fn test_rule_priority_ordering() {
        assert!(RulePriority::Exact > RulePriority::High);
        assert!(RulePriority::High > RulePriority::Medium);
        assert!(RulePriority::Medium > RulePriority::Low);
    }

    #[test]
    fn test_remote_url_rule_exact_match() {
        let rule = RemoteUrlRule::new();
        let context = test_context();

        // Test exact match
        let mut profile = test_profile();
        profile.repos = vec!["git@github.com:company/project.git".to_string()];
        assert_eq!(rule.matches(&profile, &context), Some(1.0));
    }

    #[test]
    fn test_remote_url_rule_pattern_match() {
        let rule = RemoteUrlRule::new();
        let context = test_context();

        // Test pattern match
        let mut profile = test_profile();
        profile.repos = vec!["git@github.com:company/*".to_string()];
        assert_eq!(rule.matches(&profile, &context), Some(0.95));
    }

    #[test]
    fn test_remote_url_rule_partial_match() {
        let rule = RemoteUrlRule::new();
        let context = test_context();

        // Test partial match
        let mut profile = test_profile();
        profile.repos = vec!["company".to_string()];
        assert_eq!(rule.matches(&profile, &context), Some(0.7));
    }

    #[test]
    fn test_remote_url_rule_no_match() {
        let rule = RemoteUrlRule::new();
        let context = test_context();

        // Test no match
        let mut profile = test_profile();
        profile.repos = vec!["git@gitlab.com:other/repo.git".to_string()];
        assert_eq!(rule.matches(&profile, &context), None);
    }

    #[test]
    fn test_remote_url_rule_empty_repos() {
        let rule = RemoteUrlRule::new();
        let context = test_context();

        let profile = test_profile();
        assert_eq!(rule.matches(&profile, &context), None);
    }

    #[test]
    fn test_directory_path_rule_direct_match() {
        let rule = DirectoryPathRule::new();
        let context = test_context();

        // Test direct match
        let mut profile = test_profile();
        profile.match_patterns = vec!["*/work".to_string()];
        let result = rule.matches(&profile, &context);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 0.8);
    }

    #[test]
    fn test_directory_path_rule_parent_match() {
        let rule = DirectoryPathRule::new();
        let context = test_context();

        // Test parent match
        let mut profile = test_profile();
        profile.match_patterns = vec!["*/projects".to_string()];
        let result = rule.matches(&profile, &context);
        assert!(result.is_some());
        assert!(result.unwrap() < 0.8); // Should be penalized for depth
    }

    #[test]
    fn test_directory_path_rule_no_match() {
        let rule = DirectoryPathRule::new();
        let context = test_context();

        let mut profile = test_profile();
        profile.match_patterns = vec!["*/nomatch".to_string()];
        assert_eq!(rule.matches(&profile, &context), None);
    }

    #[test]
    fn test_include_if_dir_rule_inside_directory() {
        let rule = IncludeIfDirRule::new();
        let context = test_context();

        let mut profile = test_profile();
        profile.include_if_dirs = vec!["/home/user/projects".to_string()];
        assert_eq!(rule.matches(&profile, &context), Some(0.9));
    }

    #[test]
    fn test_include_if_dir_rule_parent_directory() {
        let rule = IncludeIfDirRule::new();
        let mut context = test_context();
        context.working_dir = PathBuf::from("/home/user/projects/subdir");

        let mut profile = test_profile();
        profile.include_if_dirs = vec!["/home/user/projects".to_string()];
        let result = rule.matches(&profile, &context);
        assert!(result.is_some());
        assert!(result.unwrap() >= 0.85);
    }

    #[test]
    fn test_hostname_rule_exact_match() {
        let rule = HostnameRule::new();
        let context = test_context();

        let mut profile = test_profile();
        profile.host_patterns = vec!["work-laptop".to_string()];
        assert_eq!(rule.matches(&profile, &context), Some(0.9));
    }

    #[test]
    fn test_hostname_rule_pattern_match() {
        let rule = HostnameRule::new();
        let context = test_context();

        let mut profile = test_profile();
        profile.host_patterns = vec!["work-*".to_string()];
        assert_eq!(rule.matches(&profile, &context), Some(0.7));
    }

    #[test]
    fn test_hostname_rule_no_match() {
        let rule = HostnameRule::new();
        let context = test_context();

        let mut profile = test_profile();
        profile.host_patterns = vec!["home-*".to_string()];
        assert_eq!(rule.matches(&profile, &context), None);
    }

    #[test]
    fn test_git_config_rule_email_match() {
        let rule = GitConfigRule::new();
        let mut context = test_context();
        // Only match email, not name
        context.current_name = Some("Different User".to_string());

        let profile = test_profile();
        assert_eq!(rule.matches(&profile, &context), Some(0.6));
    }

    #[test]
    fn test_git_config_rule_name_and_email_match() {
        let rule = GitConfigRule::new();
        let context = test_context();

        let profile = test_profile();
        let result = rule.matches(&profile, &context);
        assert!(result.is_some());
        assert!(result.unwrap() > 0.6); // Should be sum of email + name scores
    }

    #[test]
    fn test_git_config_rule_no_match() {
        let rule = GitConfigRule::new();
        let mut context = test_context();
        context.current_email = Some("different@example.com".to_string());
        context.current_name = Some("Different User".to_string());

        let profile = test_profile();
        assert_eq!(rule.matches(&profile, &context), None);
    }

    #[test]
    fn test_rule_trait_implementations() {
        let remote_rule = RemoteUrlRule::new();
        let dir_rule = DirectoryPathRule::new();
        let include_rule = IncludeIfDirRule::new();
        let hostname_rule = HostnameRule::new();
        let config_rule = GitConfigRule::new();

        // Test that all rules implement the trait correctly
        assert_eq!(remote_rule.name(), "remote_url");
        assert_eq!(remote_rule.priority(), RulePriority::High);

        assert_eq!(dir_rule.name(), "directory_path");
        assert_eq!(dir_rule.priority(), RulePriority::Medium);

        assert_eq!(include_rule.name(), "include_if_dir");
        assert_eq!(include_rule.priority(), RulePriority::High);

        assert_eq!(hostname_rule.name(), "hostname");
        assert_eq!(hostname_rule.priority(), RulePriority::Medium);

        assert_eq!(config_rule.name(), "git_config");
        assert_eq!(config_rule.priority(), RulePriority::Low);
    }

    #[test]
    fn test_rule_default_implementations() {
        let _remote_rule = RemoteUrlRule::default();
        let _dir_rule = DirectoryPathRule::default();
        let _include_rule = IncludeIfDirRule::default();
        let _hostname_rule = HostnameRule::default();
        let _config_rule = GitConfigRule::default();
    }
}
