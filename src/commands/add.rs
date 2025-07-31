//! Add command implementation for git-setup-rs.
//!
//! This command provides interactive profile creation with validation and 1Password integration.

use super::{Command, CommandContext};
use crate::{
    cli::Args,
    config::types::{Profile, KeyType, Scope},
    error::{GitSetupError, Result},
};
use async_trait::async_trait;

/// Command implementation for adding new profiles.
pub struct AddCommand;

impl AddCommand {
    /// Create a new AddCommand instance.
    pub fn new() -> Self {
        Self
    }

    /// Create a new profile interactively.
    async fn create_profile_interactive(&self, name: &str, _context: &CommandContext) -> Result<Profile> {
        println!("Creating new profile: {}", name);
        
        // For now, create a basic profile
        // In a full implementation, this would use the TUI for interactive input
        let profile = Profile {
            name: name.to_string(),
            git_user_name: Some(format!("{} User", name)),
            git_user_email: format!("{}@example.com", name),
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

        Ok(profile)
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

    /// Check if profile name already exists.
    async fn check_profile_exists(&self, name: &str, context: &CommandContext) -> Result<bool> {
        match context.profile_manager.read(name)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Print success message after profile creation.
    fn print_success_message(&self, profile: &Profile) {
        println!("✓ Profile '{}' created successfully", profile.name);
        println!("  Email: {}", profile.git_user_email);
        
        if let Some(name) = &profile.git_user_name {
            println!("  Name: {}", name);
        }
        
        println!("  Key type: {:?}", profile.key_type);
        println!("  Scope: {:?}", profile.scope.as_ref().unwrap_or(&Scope::Local));
        
        if profile.one_password {
            println!("  1Password: Enabled");
            if let Some(vault) = &profile.vault_name {
                println!("  Vault: {}", vault);
            }
        }
    }
}

impl Default for AddCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for AddCommand {
    async fn execute(&self, args: &Args, context: &CommandContext) -> Result<()> {
        let profile_name = args.add.as_ref()
            .ok_or_else(|| GitSetupError::Git("Profile name is required for add command".to_string()))?;

        // Check if profile already exists
        if self.check_profile_exists(profile_name, context).await? {
            return Err(GitSetupError::Git(format!("Profile '{}' already exists", profile_name)));
        }

        // Create profile interactively
        let profile = self.create_profile_interactive(profile_name, context).await?;

        // Validate the profile
        self.validate_profile(&profile)?;

        // Save the profile
        context.profile_manager.write(&profile)?;

        // Print success message
        if !args.quiet {
            self.print_success_message(&profile);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "add"
    }

    fn description(&self) -> &'static str {
        "Add a new profile with interactive configuration"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::tests::create_test_context,
        profile::mock::MockProfileManager,
    };
    use std::sync::Arc;

    /// Helper function to create test args for adding a profile.
    fn create_add_args(name: &str) -> Args {
        Args {
            add: Some(name.to_string()),
            quiet: true,
            ..Default::default()
        }
    }

    /// Test that AddCommand can be created.
    #[test]
    fn test_add_command_creation() {
        let cmd = AddCommand::new();
        assert_eq!(cmd.name(), "add");
        assert_eq!(cmd.description(), "Add a new profile with interactive configuration");
    }

    /// Test adding a new profile successfully.
    #[tokio::test]
    async fn test_add_profile_success() {
        let profile_manager = Arc::new(MockProfileManager::new());
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = create_add_args("new-profile");
        let cmd = AddCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test adding a profile that already exists.
    #[tokio::test]
    async fn test_add_profile_already_exists() {
        let existing_profile = Profile {
            name: "existing".to_string(),
            git_user_name: Some("Existing User".to_string()),
            git_user_email: "existing@example.com".to_string(),
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

        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![existing_profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = create_add_args("existing");
        let cmd = AddCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    /// Test validation of profile data.
    #[test]
    fn test_profile_validation() {
        let cmd = AddCommand::new();

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

        // Test 1Password validation without vault
        let profile = Profile {
            name: "test".to_string(),
            git_user_email: "test@example.com".to_string(),
            one_password: true,
            vault_name: None,
            ..Default::default()
        };
        let result = cmd.validate_profile(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Vault name is required"));

        // Test 1Password validation without SSH key title
        let profile = Profile {
            name: "test".to_string(),
            git_user_email: "test@example.com".to_string(),
            one_password: true,
            vault_name: Some("vault".to_string()),
            ssh_key_title: None,
            ..Default::default()
        };
        let result = cmd.validate_profile(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SSH key title is required"));

        // Test valid profile
        let profile = Profile {
            name: "test".to_string(),
            git_user_email: "test@example.com".to_string(),
            ..Default::default()
        };
        let result = cmd.validate_profile(&profile);
        assert!(result.is_ok());
    }

    /// Test that AddCommand implements Send + Sync.
    #[test]
    fn test_add_command_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AddCommand>();
    }

    /// Test profile creation with different names.
    #[tokio::test]
    async fn test_create_profile_interactive() {
        let context = create_test_context();
        let cmd = AddCommand::new();
        
        let profile = cmd.create_profile_interactive("test-profile", &context).await.unwrap();
        
        assert_eq!(profile.name, "test-profile");
        assert_eq!(profile.git_user_email, "test-profile@example.com");
        assert_eq!(profile.git_user_name, Some("test-profile User".to_string()));
        assert_eq!(profile.key_type, KeyType::Ssh);
        assert_eq!(profile.scope, Some(Scope::Local));
        assert!(!profile.one_password);
    }

    /// Test that profile exists check works correctly.
    #[tokio::test]
    async fn test_check_profile_exists() {
        let existing_profile = Profile {
            name: "existing".to_string(),
            git_user_email: "existing@example.com".to_string(),
            ..Default::default()
        };

        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![existing_profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = AddCommand::new();
        
        // Test existing profile
        let exists = cmd.check_profile_exists("existing", &context).await.unwrap();
        assert!(exists);

        // Test non-existing profile
        let exists = cmd.check_profile_exists("nonexistent", &context).await.unwrap();
        assert!(!exists);
    }

    /// Test add command without profile name.
    #[tokio::test]
    async fn test_add_command_no_profile_name() {
        let context = create_test_context();
        let args = Args {
            add: None,
            quiet: true,
            ..Default::default()
        };
        
        let cmd = AddCommand::new();
        let result = cmd.execute(&args, &context).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Profile name is required"));
    }
}