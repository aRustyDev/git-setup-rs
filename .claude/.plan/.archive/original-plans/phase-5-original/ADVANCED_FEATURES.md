# Advanced Features Implementation Guide

## Overview
This document covers advanced features. These features are REQUIRED for SPEC compliance and MUST be implemented in Phase 5 - no exceptions or delays.

## 1. Profile Auto-Detection by Remote URL

### Purpose
Automatically detect and suggest the appropriate profile based on the current repository's remote URL.

### Implementation Details

#### Pattern Matching Logic
```rust
// src/profile/detector.rs
use glob::Pattern;

pub struct RemoteUrlDetector;

impl RemoteUrlDetector {
    pub fn detect_profiles(
        remote_url: &str,
        profiles: &[Profile]
    ) -> Vec<&Profile> {
        profiles.iter()
            .filter(|profile| {
                profile.match_patterns.iter().any(|pattern| {
                    Self::matches_pattern(remote_url, pattern)
                })
            })
            .collect()
    }

    fn matches_pattern(url: &str, pattern: &str) -> bool {
        // Handle both SSH and HTTPS URLs
        let normalized_url = Self::normalize_url(url);

        // MUST use glob matching for patterns with *
        // NO shortcuts or "simple" alternatives
        Pattern::new(pattern)
            .map(|p| p.matches(&normalized_url))
            .unwrap_or(false)
    }

    fn normalize_url(url: &str) -> String {
        // Convert SSH URLs to a normalized format
        // git@github.com:user/repo.git -> github.com/user/repo
        if url.starts_with("git@") {
            url.replacen("git@", "", 1)
               .replacen(":", "/", 1)
               .trim_end_matches(".git")
               .to_string()
        } else {
            url.trim_end_matches(".git")
               .trim_start_matches("https://")
               .trim_start_matches("http://")
               .to_string()
        }
    }
}
```

#### TDD Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_url_matching() {
        let profile = Profile {
            name: "work".to_string(),
            match_patterns: vec!["github.com:company/*".to_string()],
            ..Default::default()
        };

        let detector = RemoteUrlDetector;
        let url = "git@github.com:company/project.git";
        let matches = detector.detect_profiles(url, &[profile]);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "work");
    }

    #[test]
    fn test_https_url_matching() {
        let profile = Profile {
            name: "personal".to_string(),
            match_patterns: vec!["github.com/myuser/*".to_string()],
            ..Default::default()
        };

        let detector = RemoteUrlDetector;
        let url = "https://github.com/myuser/project.git";
        let matches = detector.detect_profiles(url, &[profile]);

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_glob_pattern_matching() {
        let profile = Profile {
            name: "oss".to_string(),
            match_patterns: vec!["github.com/rust-lang/*".to_string()],
            ..Default::default()
        };

        let detector = RemoteUrlDetector;
        let url = "git@github.com:rust-lang/rust.git";
        let matches = detector.detect_profiles(url, &[profile]);

        assert_eq!(matches.len(), 1);
    }
}
```

### Integration with CLI
MANDATORY: Auto-detection MUST run when no profile specified (SPEC Section 12.2):
```rust
// In main command handler
if args.profile.is_none() {
    let remote_url = git_client.get_remote_url().await?;
    if let Some(url) = remote_url {
        let matches = detector.detect_profiles(&url, &profiles);

        match matches.len() {
            0 => println!("No profile matches current repository"),
            1 => {
                println!("Auto-detected profile: {}", matches[0].name);
                apply_profile(matches[0], scope)?;
            }
            _ => {
                println!("Multiple profiles match:");
                for profile in matches {
                    println!("  - {}", profile.name);
                }
                // Launch profile selector TUI
            }
        }
    }
}
```

## 2. Import from agent.toml

### Purpose
Import existing SSH keys configured in 1Password's agent.toml file to create profiles automatically.

### Implementation Details

#### Agent.toml Parser
```rust
// src/agent/parser.rs
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    #[serde(rename = "ssh-keys")]
    pub ssh_keys: Vec<SshKeyEntry>,
}

#[derive(Debug, Deserialize)]
pub struct SshKeyEntry {
    pub item: String,
    pub vault: String,
    #[serde(default)]
    pub account: Option<String>,
}

impl AgentConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read agent.toml from {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| "Failed to parse agent.toml")
    }
}
```

#### Import Logic
```rust
// src/profile/importer.rs
pub struct AgentImporter<'a> {
    op_client: &'a dyn OnePasswordOperations,
    profile_manager: &'a mut ProfileManager,
}

impl<'a> AgentImporter<'a> {
    pub async fn import_from_agent_toml(&mut self, path: &Path) -> Result<ImportResult> {
        let agent_config = AgentConfig::load(path)?;
        let mut imported = 0;
        let mut skipped = 0;
        let mut errors = Vec::new();

        for key_entry in agent_config.ssh_keys {
            match self.import_key_entry(&key_entry).await {
                Ok(ImportStatus::Created) => imported += 1,
                Ok(ImportStatus::AlreadyExists) => skipped += 1,
                Err(e) => errors.push((key_entry.item.clone(), e)),
            }
        }

        Ok(ImportResult {
            imported,
            skipped,
            errors,
        })
    }

    async fn import_key_entry(&mut self, entry: &SshKeyEntry) -> Result<ImportStatus> {
        // Check if profile already exists
        let profile_name = Self::generate_profile_name(&entry.item);
        if self.profile_manager.profile_exists(&profile_name) {
            return Ok(ImportStatus::AlreadyExists);
        }

        // Fetch public key from 1Password
        let public_key = self.op_client
            .get_public_key(&entry.vault, &entry.item)
            .await?;

        // MUST extract email from SSH key comment
        // Format MUST be: profile@imported if no email found
        let email = Self::extract_email_from_key(&public_key)
            .unwrap_or_else(|| format!("{}@imported", profile_name));

        // Create profile
        let profile = Profile {
            name: profile_name,
            git_user_email: email,
            ssh_key_title: entry.item.clone(),
            vault_name: entry.vault.clone(),
            signing_key: public_key,
            key_type: KeyType::Ssh,
            ssh_key_source: SshKeySource::OnePassword,
            one_password: true,
            ..Default::default()
        };

        self.profile_manager.add_profile(profile)?;
        Ok(ImportStatus::Created)
    }

    fn generate_profile_name(item_name: &str) -> String {
        // Convert "Work SSH Key" -> "work-ssh-key"
        item_name
            .to_lowercase()
            .replace(' ', "-")
            .replace("ssh-key", "")
            .replace("ssh", "")
            .trim_matches('-')
            .to_string()
    }

    fn extract_email_from_key(public_key: &str) -> Option<String> {
        // SSH keys often have email in comment: ssh-ed25519 AAAA... user@example.com
        public_key.split_whitespace()
            .nth(2)
            .filter(|s| s.contains('@'))
            .map(|s| s.to_string())
    }
}
```

#### TDD Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_import_creates_profiles() {
        let mut mock_op = MockOnePasswordOperations::new();
        mock_op
            .expect_get_public_key()
            .with(eq("Personal"), eq("GitHub SSH"))
            .returning(|_, _| Ok("ssh-ed25519 AAAA... user@example.com".to_string()));

        let temp_dir = TempDir::new().unwrap();
        let agent_path = temp_dir.path().join("agent.toml");
        std::fs::write(&agent_path, r#"
            [[ssh-keys]]
            item = "GitHub SSH"
            vault = "Personal"
        "#).unwrap();

        let mut profile_manager = ProfileManager::new(Config::default());
        let mut importer = AgentImporter {
            op_client: &mock_op,
            profile_manager: &mut profile_manager,
        };

        let result = importer.import_from_agent_toml(&agent_path).await.unwrap();

        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 0);
        assert!(profile_manager.profile_exists("github"));
    }
}
```

## 3. Git IncludeIf Management

### Purpose
Configure Git to use different profiles based on the repository's directory location using Git's includeIf feature.

### Implementation Details

#### IncludeIf Configuration
```rust
// src/git/includeif.rs
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct IncludeIf {
    pub gitdir: String,
    pub path: PathBuf,
}

pub struct IncludeIfManager {
    global_config_path: PathBuf,
}

impl IncludeIfManager {
    pub fn new(global_config_path: impl Into<PathBuf>) -> Self {
        Self {
            global_config_path: global_config_path.into(),
        }
    }

    pub fn add_include_if(&self, profile: &Profile) -> Result<()> {
        if profile.include_if_dirs.is_empty() {
            return Ok(());
        }

        // Create profile-specific config file
        let profile_config_path = self.create_profile_config(profile)?;

        // Add includeIf directives to global config
        for dir in &profile.include_if_dirs {
            self.add_include_directive(dir, &profile_config_path)?;
        }

        Ok(())
    }

    fn create_profile_config(&self, profile: &Profile) -> Result<PathBuf> {
        let config_dir = self.global_config_path
            .parent()
            .ok_or(GitSetupError::InvalidPath)?;
        let profiles_dir = config_dir.join("profiles");
        std::fs::create_dir_all(&profiles_dir)?;

        let profile_path = profiles_dir.join(format!("{}.gitconfig", profile.name));

        let mut config = String::new();
        config.push_str("[user]\n");
        config.push_str(&format!("\tname = {}\n", profile.git_user_name));
        config.push_str(&format!("\temail = {}\n", profile.git_user_email));

        if !profile.signing_key.is_empty() {
            config.push_str("[commit]\n");
            config.push_str("\tgpgsign = true\n");
            config.push_str("[gpg]\n");
            config.push_str(&format!("\tformat = {}\n", profile.key_type));
            config.push_str("[user]\n");
            config.push_str(&format!("\tsigningkey = {}\n", profile.signing_key));
        }

        std::fs::write(&profile_path, config)?;
        Ok(profile_path)
    }

    fn add_include_directive(&self, dir: &str, config_path: &Path) -> Result<()> {
        let expanded_dir = expand_tilde(dir);
        let gitdir_pattern = format!("{}/**", expanded_dir.trim_end_matches('/'));

        // Read existing global config
        let content = std::fs::read_to_string(&self.global_config_path)
            .unwrap_or_default();

        // Check if already exists
        let include_line = format!("[includeIf \"gitdir:{}\"]\n\tpath = {}\n",
            gitdir_pattern,
            config_path.display()
        );

        if content.contains(&include_line) {
            return Ok(());
        }

        // Append to config
        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(&include_line);

        std::fs::write(&self.global_config_path, new_content)?;
        Ok(())
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen('~', &home, 1);
        }
    }
    path.to_string()
}
```

#### TDD Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_include_if_config() {
        let temp_dir = TempDir::new().unwrap();
        let global_config = temp_dir.path().join("gitconfig");

        let profile = Profile {
            name: "work".to_string(),
            git_user_name: "Work User".to_string(),
            git_user_email: "work@company.com".to_string(),
            include_if_dirs: vec!["~/work/".to_string()],
            signing_key: "ssh-ed25519 AAAA...".to_string(),
            key_type: KeyType::Ssh,
            ..Default::default()
        };

        let manager = IncludeIfManager::new(&global_config);
        manager.add_include_if(&profile).unwrap();

        // Verify global config contains includeIf
        let content = std::fs::read_to_string(&global_config).unwrap();
        assert!(content.contains("[includeIf \"gitdir:"));
        assert!(content.contains("/work/**\""));

        // Verify profile config was created
        let profile_config = temp_dir.path()
            .join("profiles")
            .join("work.gitconfig");
        assert!(profile_config.exists());

        let profile_content = std::fs::read_to_string(&profile_config).unwrap();
        assert!(profile_content.contains("name = Work User"));
        assert!(profile_content.contains("email = work@company.com"));
    }
}
```

## 4. Fuzzy Profile Matching

### Purpose
Allow users to type partial profile names and match the most likely profile.

### Implementation Details

#### Fuzzy Matching Algorithm
```rust
// src/profile/fuzzy.rs
pub struct FuzzyMatcher;

impl FuzzyMatcher {
    pub fn score(input: &str, target: &str) -> f64 {
        if input.is_empty() || target.is_empty() {
            return 0.0;
        }

        let input_lower = input.to_lowercase();
        let target_lower = target.to_lowercase();

        // Exact match
        if input_lower == target_lower {
            return 100.0;
        }

        // Prefix match
        if target_lower.starts_with(&input_lower) {
            return 90.0 - (target.len() - input.len()) as f64 * 0.5;
        }

        // Subsequence match
        if Self::is_subsequence(&input_lower, &target_lower) {
            let ratio = input.len() as f64 / target.len() as f64;
            return 50.0 + (ratio * 30.0);
        }

        // Contains match
        if target_lower.contains(&input_lower) {
            return 30.0;
        }

        0.0
    }

    fn is_subsequence(needle: &str, haystack: &str) -> bool {
        let mut haystack_chars = haystack.chars();
        for needle_char in needle.chars() {
            if !haystack_chars.any(|c| c == needle_char) {
                return false;
            }
        }
        true
    }

    pub fn find_best_match<'a>(
        input: &str,
        profiles: &'a [Profile]
    ) -> Option<&'a Profile> {
        profiles.iter()
            .map(|p| (p, Self::score(input, &p.name)))
            .filter(|(_, score)| *score > 0.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(profile, _)| profile)
    }
}
```

#### Property-Based Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_fuzzy_score_bounds(
            input in "[a-z]{1,10}",
            target in "[a-z]{1,20}"
        ) {
            let score = FuzzyMatcher::score(&input, &target);
            assert!(score >= 0.0 && score <= 100.0);
        }

        #[test]
        fn test_exact_match_scores_highest(name in "[a-z]{3,10}") {
            let profiles = vec![
                Profile { name: format!("{}x", name), ..Default::default() },
                Profile { name: name.clone(), ..Default::default() },
                Profile { name: format!("x{}", name), ..Default::default() },
            ];

            let best = FuzzyMatcher::find_best_match(&name, &profiles).unwrap();
            assert_eq!(best.name, name);
        }
    }

    #[test]
    fn test_prefix_matching() {
        let profiles = vec![
            Profile { name: "work".to_string(), ..Default::default() },
            Profile { name: "workspace".to_string(), ..Default::default() },
            Profile { name: "personal".to_string(), ..Default::default() },
        ];

        let match1 = FuzzyMatcher::find_best_match("wor", &profiles).unwrap();
        assert_eq!(match1.name, "work");

        let match2 = FuzzyMatcher::find_best_match("work", &profiles).unwrap();
        assert_eq!(match2.name, "work");
    }
}
```

## 5. File-Based Profile Import/Export

### Purpose
Allow profiles to be imported from or exported to external files for sharing or backup.

### Implementation Details

#### File Operations
```rust
// src/profile/file_ops.rs
use std::path::Path;

pub struct ProfileFileOps;

impl ProfileFileOps {
    pub fn import_profile(path: &Path, format: ExportFormat) -> Result<Profile> {
        let content = std::fs::read_to_string(path)?;

        let profile = match format {
            ExportFormat::Json => serde_json::from_str(&content)?,
            ExportFormat::Yaml => serde_yaml::from_str(&content)?,
            ExportFormat::Toml => toml::from_str(&content)?,
        };

        Self::validate_imported_profile(&profile)?;
        Ok(profile)
    }

    pub fn export_profile(
        profile: &Profile,
        path: &Path,
        format: ExportFormat
    ) -> Result<()> {
        let content = match format {
            ExportFormat::Json => serde_json::to_string_pretty(profile)?,
            ExportFormat::Yaml => serde_yaml::to_string(profile)?,
            ExportFormat::Toml => toml::to_string_pretty(profile)?,
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    fn validate_imported_profile(profile: &Profile) -> Result<()> {
        profile.validate()?;

        // Additional validation for imported profiles
        if profile.name.contains('/') || profile.name.contains('\\') {
            return Err(GitSetupError::Validation(
                "Profile name cannot contain path separators".to_string()
            ));
        }

        Ok(())
    }

    pub fn detect_format(path: &Path) -> Option<ExportFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "json" => Some(ExportFormat::Json),
                "yaml" | "yml" => Some(ExportFormat::Yaml),
                "toml" => Some(ExportFormat::Toml),
                _ => None,
            })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
}
```

## Implementation Priority

1. **Fuzzy Matching** - Essential for user experience
2. **Auto-detection** - Very useful for daily workflow
3. **IncludeIf Management** - Important for directory-based switching
4. **Agent.toml Import** - Useful for initial setup
5. **File Import/Export** - Nice to have for sharing

## Testing Strategy

For each advanced feature:
1. Write comprehensive unit tests first
2. Add integration tests that verify interaction with external tools
3. Include property-based tests where applicable
4. Test error conditions and edge cases
5. Verify compatibility with Go implementation

Remember: These features enhance usability but the core tool can function without them. Implement incrementally and test thoroughly!
