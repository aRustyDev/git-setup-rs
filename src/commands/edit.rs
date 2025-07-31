//! Edit command implementation for git-setup-rs.
//!
//! This command provides interactive profile editing with validation.

use super::{Command, CommandContext};
use crate::{
    cli::Args,
    config::types::Profile,
    error::{GitSetupError, Result},
};
use async_trait::async_trait;

/// Command implementation for editing profiles.
pub struct EditCommand;

impl EditCommand {
    /// Create a new EditCommand instance.
    pub fn new() -> Self {
        Self
    }

    /// Load existing profile for editing.
    async fn load_profile(&self, name: &str, context: &CommandContext) -> Result<Profile> {
        context.profile_manager.read(name)?
            .ok_or_else(|| GitSetupError::ProfileNotFound { name: name.to_string() })
    }

    /// Edit profile interactively.
    /// In a real implementation, this would use the TUI for interactive editing.
    async fn edit_profile_interactive(&self, profile: &Profile, _context: &CommandContext) -> Result<Profile> {
        println!("Editing profile: {}", profile.name);
        println!("Current email: {}", profile.git_user_email);
        
        // For now, return the profile unchanged
        // In a full implementation, this would use the TUI to edit fields
        Ok(profile.clone())
    }

    /// Validate profile data before saving.
    fn validate_profile(&self, profile: &Profile) -> Result<()> {
        // Validate required fields
        if profile.name.is_empty() {
            return Err(GitSetupError::Git("Profile name cannot be empty".to_string()));
        }

        if profile.git_user_email.is_empty() {
            return Err(GitSetupError::Git("Email cannot be empty".to_string()));
        }

        // Validate email format (basic check)
        if !profile.git_user_email.contains('@') {
            return Err(GitSetupError::Git("Invalid email format".to_string()));
        }

        // Validate 1Password configuration
        if profile.one_password {
            if profile.vault_name.is_none() {
                return Err(GitSetupError::Git("Vault name is required for 1Password integration".to_string()));
            }
            if profile.ssh_key_title.is_none() {
                return Err(GitSetupError::Git("SSH key title is required for 1Password integration".to_string()));
            }
        }

        Ok(())
    }

    /// Print success message after profile editing.
    fn print_success_message(&self, profile: &Profile) {
        println!("✓ Profile '{}' updated successfully", profile.name);
        println!("  Email: {}", profile.git_user_email);
        
        if let Some(name) = &profile.git_user_name {
            println!("  Name: {}", name);
        }
        
        println!("  Key type: {:?}", profile.key_type);
        
        if profile.one_password {
            println!("  1Password: Enabled");
            if let Some(vault) = &profile.vault_name {
                println!("  Vault: {}", vault);
            }
        }
    }
}

impl Default for EditCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for EditCommand {
    async fn execute(&self, args: &Args, context: &CommandContext) -> Result<()> {
        let profile_name = args.edit.as_ref()
            .ok_or_else(|| GitSetupError::Git("Profile name is required for edit command".to_string()))?;

        // Load the existing profile
        let profile = self.load_profile(profile_name, context).await?;

        // Edit the profile interactively
        let edited_profile = self.edit_profile_interactive(&profile, context).await?;

        // Validate the edited profile
        self.validate_profile(&edited_profile)?;

        // Save the updated profile
        context.profile_manager.write(&edited_profile)?;

        // Print success message
        if !args.quiet {
            self.print_success_message(&edited_profile);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "edit"
    }

    fn description(&self) -> &'static str {
        "Edit an existing profile with interactive configuration"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::types::{Profile, KeyType, Scope},
        commands::tests::create_test_context,
        profile::mock::MockProfileManager,
    };
    use std::sync::Arc;

    /// Helper function to create test args for editing a profile.
    fn create_edit_args(name: &str) -> Args {
        Args {
            edit: Some(name.to_string()),
            quiet: true,
            ..Default::default()
        }
    }

    /// Helper function to create a test profile.
    fn create_test_profile(name: &str, email: &str) -> Profile {
        Profile {
            name: name.to_string(),
            git_user_name: Some(format!("{} User", name)),
            git_user_email: email.to_string(),
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

    /// Test that EditCommand can be created.
    #[test]
    fn test_edit_command_creation() {
        let cmd = EditCommand::new();
        assert_eq!(cmd.name(), "edit");
        assert_eq!(cmd.description(), "Edit an existing profile with interactive configuration");
    }

    /// Test editing an existing profile.
    #[tokio::test]
    async fn test_edit_profile_success() {
        let profile = create_test_profile("test-profile", "test@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = create_edit_args("test-profile");
        let cmd = EditCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test editing a non-existent profile.
    #[tokio::test]
    async fn test_edit_nonexistent_profile() {
        let profile_manager = Arc::new(MockProfileManager::new());
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = create_edit_args("nonexistent");
        let cmd = EditCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::ProfileNotFound { .. }));
    }

    /// Test loading a profile for editing.
    #[tokio::test]
    async fn test_load_profile() {
        let profile = create_test_profile("test-profile", "test@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile.clone()]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = EditCommand::new();
        let loaded_profile = cmd.load_profile("test-profile", &context).await.unwrap();

        assert_eq!(loaded_profile.name, profile.name);
        assert_eq!(loaded_profile.git_user_email, profile.git_user_email);
    }

    /// Test loading a non-existent profile.
    #[tokio::test]
    async fn test_load_nonexistent_profile() {
        let profile_manager = Arc::new(MockProfileManager::new());
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = EditCommand::new();
        let result = cmd.load_profile("nonexistent", &context).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::ProfileNotFound { .. }));
    }

    /// Test profile validation.
    #[test]
    fn test_profile_validation() {
        let cmd = EditCommand::new();

        // Test empty name
        let profile = Profile {
            name: "".to_string(),
            git_user_email: "test@example.com".to_string(),
            ..Default::default()
        };
        let result = cmd.validate_profile(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name cannot be empty"));

        // Test empty email
        let profile = Profile {
            name: "test".to_string(),
            git_user_email: "".to_string(),
            ..Default::default()
        };
        let result = cmd.validate_profile(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Email cannot be empty"));

        // Test invalid email format
        let profile = Profile {
            name: "test".to_string(),
            git_user_email: "invalid-email".to_string(),
            ..Default::default()
        };
        let result = cmd.validate_profile(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid email format"));

        // Test valid profile
        let profile = Profile {
            name: "test".to_string(),
            git_user_email: "test@example.com".to_string(),
            ..Default::default()
        };
        let result = cmd.validate_profile(&profile);
        assert!(result.is_ok());
    }

    /// Test that EditCommand implements Send + Sync.
    #[test]
    fn test_edit_command_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EditCommand>();
    }

    /// Test edit command without profile name.
    #[tokio::test]
    async fn test_edit_command_no_profile_name() {
        let context = create_test_context();
        let args = Args {
            edit: None,
            quiet: true,
            ..Default::default()
        };
        
        let cmd = EditCommand::new();
        let result = cmd.execute(&args, &context).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Profile name is required"));
    }

    /// Test interactive editing (currently just returns the same profile).
    #[tokio::test]
    async fn test_edit_profile_interactive() {
        let profile = create_test_profile("test-profile", "test@example.com");
        let context = create_test_context();
        let cmd = EditCommand::new();
        
        let edited_profile = cmd.edit_profile_interactive(&profile, &context).await.unwrap();
        
        // Currently, interactive editing just returns the same profile
        assert_eq!(edited_profile.name, profile.name);
        assert_eq!(edited_profile.git_user_email, profile.git_user_email);
    }
}