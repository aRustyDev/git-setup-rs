//! Apply command implementation for git-setup-rs.
//!
//! This command applies a git profile, configuring git user settings and signing keys.

use super::{Command, CommandContext};
use crate::{
    cli::Args,
    config::types::{Profile, KeyType, Scope},
    error::{GitSetupError, Result},
    external::git::GitConfigScope,
    detection::RepositoryContext,
};
use async_trait::async_trait;

/// Command implementation for applying git profiles.
pub struct ApplyCommand;

impl ApplyCommand {
    /// Create a new ApplyCommand instance.
    pub fn new() -> Self {
        Self
    }

    /// Determine the git configuration scope based on arguments and profile.
    fn determine_scope(&self, args: &Args, profile: &Profile) -> Result<GitConfigScope> {
        // Check for conflicting flags
        if args.global && args.system {
            return Err(GitSetupError::Git(
                "Cannot use --global and --system flags together".to_string(),
            ));
        }

        // Use command line flags first
        if args.global {
            return Ok(GitConfigScope::Global);
        }
        if args.system {
            return Ok(GitConfigScope::System);
        }

        // Use profile's default scope
        match profile.scope {
            Some(Scope::Global) => Ok(GitConfigScope::Global),
            Some(Scope::System) => Ok(GitConfigScope::System),
            Some(Scope::Local) | None => Ok(GitConfigScope::Local),
        }
    }

    /// Apply git user configuration from profile.
    async fn apply_user_config(
        &self,
        profile: &Profile,
        scope: GitConfigScope,
        context: &CommandContext,
    ) -> Result<()> {
        // Set user.email (required)
        context.git_wrapper.set_config("user.email", &profile.git_user_email, scope)?;

        // Set user.name if provided
        if let Some(name) = &profile.git_user_name {
            context.git_wrapper.set_config("user.name", name, scope)?;
        }

        Ok(())
    }

    /// Configure signing based on profile key type.
    async fn configure_signing(
        &self,
        profile: &Profile,
        scope: GitConfigScope,
        context: &CommandContext,
    ) -> Result<()> {
        match profile.key_type {
            KeyType::Ssh => {
                self.configure_ssh_signing(profile, scope, context).await?;
            }
            KeyType::Gpg => {
                self.configure_gpg_signing(profile, scope, context).await?;
            }
            KeyType::X509 => {
                context.git_wrapper.configure_x509_signing(scope)?;
            }
            KeyType::Gitsign => {
                context.git_wrapper.configure_gitsign(scope)?;
            }
        }

        Ok(())
    }

    /// Configure SSH signing for the profile.
    async fn configure_ssh_signing(
        &self,
        profile: &Profile,
        scope: GitConfigScope,
        context: &CommandContext,
    ) -> Result<()> {
        let signing_key = if profile.one_password {
            self.get_ssh_key_from_1password(profile, context).await?
        } else {
            profile.signing_key.clone()
                .ok_or_else(|| GitSetupError::Git("SSH signing key not configured".to_string()))?
        };

        context.git_wrapper.configure_ssh_signing(
            &signing_key,
            profile.allowed_signers.as_deref(),
            scope,
        )?;

        Ok(())
    }

    /// Configure GPG signing for the profile.
    async fn configure_gpg_signing(
        &self,
        profile: &Profile,
        scope: GitConfigScope,
        context: &CommandContext,
    ) -> Result<()> {
        let signing_key = if profile.one_password {
            self.get_gpg_key_from_1password(profile, context).await?
        } else {
            profile.signing_key.clone()
                .ok_or_else(|| GitSetupError::Git("GPG signing key not configured".to_string()))?
        };

        context.git_wrapper.configure_gpg_signing(&signing_key, scope)?;

        Ok(())
    }

    /// Get SSH key from 1Password.
    async fn get_ssh_key_from_1password(
        &self,
        profile: &Profile,
        context: &CommandContext,
    ) -> Result<String> {
        let ssh_key_title = profile.ssh_key_title.as_ref()
            .ok_or_else(|| GitSetupError::Git("SSH key title not configured for 1Password".to_string()))?;

        let ssh_keys = context.onepassword_wrapper.list_ssh_keys(profile.vault_name.as_deref()).await?;
        let ssh_key = ssh_keys.into_iter()
            .find(|key| key.title == *ssh_key_title)
            .ok_or_else(|| GitSetupError::Git(format!("SSH key '{}' not found in 1Password", ssh_key_title)))?;

        context.onepassword_wrapper.get_ssh_public_key(&ssh_key.id).await
    }

    /// Get GPG key from 1Password.
    async fn get_gpg_key_from_1password(
        &self,
        profile: &Profile,
        context: &CommandContext,
    ) -> Result<String> {
        let gpg_keys = context.onepassword_wrapper.list_gpg_keys(profile.vault_name.as_deref()).await?;
        
        // Find GPG key by title if specified
        let gpg_key = if let Some(title) = &profile.ssh_key_title {
            gpg_keys.into_iter()
                .find(|key| key.title == *title)
                .ok_or_else(|| GitSetupError::Git(format!("GPG key '{}' not found in 1Password", title)))?
        } else {
            gpg_keys.into_iter()
                .next()
                .ok_or_else(|| GitSetupError::Git("No GPG keys found in 1Password".to_string()))?
        };

        Ok(gpg_key.fingerprint)
    }

    /// Get profile to apply, either from args or auto-detection.
    async fn get_profile_to_apply(&self, args: &Args, context: &CommandContext) -> Result<Profile> {
        if let Some(profile_name) = &args.profile {
            // Explicit profile specified
            return context.profile_manager.read(profile_name)?
                .ok_or_else(|| GitSetupError::ProfileNotFound { name: profile_name.clone() });
        }

        // Try auto-detection
        let repo_context = RepositoryContext::from_current_dir()?;
        let detection_result = context.profile_detector.detect_profile(&repo_context)?;

        match detection_result.confidence {
            confidence if confidence >= 0.8 => {
                // High confidence, use the detected profile
                Ok(detection_result.profile)
            }
            confidence if confidence >= 0.5 => {
                // Medium confidence, ask user for confirmation
                println!("Detected profile '{}' (confidence: {:.1}%)", 
                    detection_result.profile.name, confidence * 100.0);
                
                // For now, just use the detected profile
                // In a real implementation, you'd prompt the user
                Ok(detection_result.profile)
            }
            _ => {
                // Low confidence, require explicit profile
                Err(GitSetupError::Git(
                    "No profile specified and auto-detection failed. Please specify a profile name.".to_string()
                ))
            }
        }
    }

    /// Print success message with applied configuration details.
    fn print_success_message(&self, profile: &Profile, scope: GitConfigScope) {
        let scope_str = match scope {
            GitConfigScope::Local => "local",
            GitConfigScope::Global => "global",
            GitConfigScope::System => "system",
        };

        println!("✓ Applied profile '{}' with {} scope", profile.name, scope_str);
        println!("  Email: {}", profile.git_user_email);
        
        if let Some(name) = &profile.git_user_name {
            println!("  Name: {}", name);
        }
        
        println!("  Key type: {:?}", profile.key_type);
        
        if profile.one_password {
            println!("  Key source: 1Password");
        }
    }
}

impl Default for ApplyCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for ApplyCommand {
    async fn execute(&self, args: &Args, context: &CommandContext) -> Result<()> {
        // Get the profile to apply
        let profile = self.get_profile_to_apply(args, context).await?;

        // Determine configuration scope
        let scope = self.determine_scope(args, &profile)?;

        // Apply user configuration
        self.apply_user_config(&profile, scope, context).await?;

        // Configure signing
        self.configure_signing(&profile, scope, context).await?;

        // Print success message
        if !args.quiet {
            self.print_success_message(&profile, scope);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "apply"
    }

    fn description(&self) -> &'static str {
        "Apply a git profile configuration"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::types::{Profile, KeyType, Scope},
        commands::tests::create_test_context,
        profile::mock::MockProfileManager,
        external::{
            git::MockGitWrapper,
            onepassword::{MockOnePasswordWrapper, SshKeyItem},
        },
        detection::{DetectionResult, MockProfileDetector},
    };
    use std::sync::Arc;

    /// Helper function to create a test profile.
    fn create_test_profile(name: &str, email: &str) -> Profile {
        Profile {
            name: name.to_string(),
            git_user_name: Some(format!("{} User", name)),
            git_user_email: email.to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("ssh-ed25519 AAAAC3...".to_string()),
            vault_name: Some("test-vault".to_string()),
            ssh_key_title: Some("test-ssh-key".to_string()),
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

    /// Test that ApplyCommand can be created.
    #[test]
    fn test_apply_command_creation() {
        let cmd = ApplyCommand::new();
        assert_eq!(cmd.name(), "apply");
        assert_eq!(cmd.description(), "Apply a git profile configuration");
    }

    /// Test applying a profile with explicit name.
    #[tokio::test]
    async fn test_apply_explicit_profile() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            profile: Some("work".to_string()),
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test applying a profile with global scope.
    #[tokio::test]
    async fn test_apply_profile_global_scope() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            profile: Some("work".to_string()),
            global: true,
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test applying a profile with system scope.
    #[tokio::test]
    async fn test_apply_profile_system_scope() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            profile: Some("work".to_string()),
            system: true,
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test applying nonexistent profile returns error.
    #[tokio::test]
    async fn test_apply_nonexistent_profile() {
        let profile_manager = Arc::new(MockProfileManager::new());
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            profile: Some("nonexistent".to_string()),
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::ProfileNotFound { .. }));
    }

    /// Test conflicting global and system flags.
    #[tokio::test]
    async fn test_apply_conflicting_scope_flags() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            profile: Some("work".to_string()),
            global: true,
            system: true,
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot use --global and --system"));
    }

    /// Test scope determination from profile.
    #[test]
    fn test_scope_determination() {
        let cmd = ApplyCommand::new();
        
        // Test global scope from profile
        let mut profile = create_test_profile("work", "work@example.com");
        profile.scope = Some(Scope::Global);
        
        let args = Args::default();
        let scope = cmd.determine_scope(&args, &profile).unwrap();
        assert_eq!(scope, GitConfigScope::Global);

        // Test local scope override from args
        let args = Args {
            global: false,
            system: false,
            ..Default::default()
        };
        let scope = cmd.determine_scope(&args, &profile).unwrap();
        assert_eq!(scope, GitConfigScope::Global); // Profile still wins when no flags

        // Test global flag override
        let args = Args {
            global: true,
            ..Default::default()
        };
        let scope = cmd.determine_scope(&args, &profile).unwrap();
        assert_eq!(scope, GitConfigScope::Global);
    }

    /// Test auto-detection with high confidence.
    #[tokio::test]
    async fn test_auto_detection_high_confidence() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile.clone()]));
        
        let mut detector = MockProfileDetector::new();
        detector.set_detection_result(DetectionResult {
            profile: profile.clone(),
            confidence: 0.9,
            matched_rules: vec![],
            reason: "Repository matches pattern".to_string(),
            reasons: vec!["Repository matches pattern".to_string()],
        });

        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        context.profile_detector = Arc::new(detector);

        let args = Args {
            profile: None, // No explicit profile
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test auto-detection with low confidence.
    #[tokio::test]
    async fn test_auto_detection_low_confidence() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile.clone()]));
        
        let mut detector = MockProfileDetector::new();
        detector.set_detection_result(DetectionResult {
            profile: profile.clone(),
            confidence: 0.3,
            matched_rules: vec![],
            reason: "Low confidence match".to_string(),
            reasons: vec!["Low confidence match".to_string()],
        });

        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        context.profile_detector = Arc::new(detector);

        let args = Args {
            profile: None, // No explicit profile
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("auto-detection failed"));
    }

    /// Test applying profile with 1Password SSH key.
    #[tokio::test]
    async fn test_apply_profile_with_1password_ssh() {
        let mut profile = create_test_profile("work", "work@example.com");
        profile.one_password = true;
        profile.signing_key = None; // Will be retrieved from 1Password
        
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        
        let mut onepassword_wrapper = MockOnePasswordWrapper::new();
        onepassword_wrapper.set_ssh_keys(vec![SshKeyItem {
            id: "ssh-key-id".to_string(),
            title: "test-ssh-key".to_string(),
            vault: crate::external::onepassword::Vault {
                id: "vault-id".to_string(),
                name: "test-vault".to_string(),
            },
            category: "SSH_KEY".to_string(),
            public_key: Some("ssh-ed25519 AAAAC3...".to_string()),
            private_key: None,
        }]);

        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        context.onepassword_wrapper = Arc::new(onepassword_wrapper);

        let args = Args {
            profile: Some("work".to_string()),
            quiet: true,
            ..Default::default()
        };

        let cmd = ApplyCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test that ApplyCommand implements Send + Sync.
    #[test]
    fn test_apply_command_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ApplyCommand>();
    }
}