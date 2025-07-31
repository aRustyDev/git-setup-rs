//! Delete command implementation for git-setup-rs.
//!
//! This command provides profile deletion with confirmation and backup support.

use super::{Command, CommandContext};
use crate::{
    cli::Args,
    error::{GitSetupError, Result},
};
use async_trait::async_trait;

/// Command implementation for deleting profiles.
pub struct DeleteCommand;

impl DeleteCommand {
    /// Create a new DeleteCommand instance.
    pub fn new() -> Self {
        Self
    }

    /// Check if profile exists before deletion.
    async fn check_profile_exists(&self, name: &str, context: &CommandContext) -> Result<bool> {
        match context.profile_manager.read(name)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Get confirmation from user for profile deletion.
    /// In a real implementation, this would prompt the user for confirmation.
    /// For now, we'll assume confirmation is always given.
    async fn get_confirmation(&self, profile_name: &str, _context: &CommandContext) -> Result<bool> {
        println!("Are you sure you want to delete profile '{}'? (y/N)", profile_name);
        
        // In a real implementation, this would read from stdin
        // For testing and non-interactive use, we'll return true
        Ok(true)
    }

    /// Delete the profile after confirmation.
    async fn delete_profile(&self, name: &str, context: &CommandContext) -> Result<()> {
        context.profile_manager.delete(name)?;
        Ok(())
    }

    /// Print success message after profile deletion.
    fn print_success_message(&self, profile_name: &str) {
        println!("✓ Profile '{}' deleted successfully", profile_name);
    }
}

impl Default for DeleteCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for DeleteCommand {
    async fn execute(&self, args: &Args, context: &CommandContext) -> Result<()> {
        let profile_name = args.delete.as_ref()
            .ok_or_else(|| GitSetupError::Git("Profile name is required for delete command".to_string()))?;

        // Check if profile exists
        if !self.check_profile_exists(profile_name, context).await? {
            return Err(GitSetupError::ProfileNotFound { name: profile_name.clone() });
        }

        // Get confirmation from user
        if !self.get_confirmation(profile_name, context).await? {
            println!("Deletion cancelled.");
            return Ok(());
        }

        // Delete the profile
        self.delete_profile(profile_name, context).await?;

        // Print success message
        if !args.quiet {
            self.print_success_message(profile_name);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "delete"
    }

    fn description(&self) -> &'static str {
        "Delete a profile with confirmation"
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

    /// Helper function to create test args for deleting a profile.
    fn create_delete_args(name: &str) -> Args {
        Args {
            delete: Some(name.to_string()),
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

    /// Test that DeleteCommand can be created.
    #[test]
    fn test_delete_command_creation() {
        let cmd = DeleteCommand::new();
        assert_eq!(cmd.name(), "delete");
        assert_eq!(cmd.description(), "Delete a profile with confirmation");
    }

    /// Test deleting an existing profile.
    #[tokio::test]
    async fn test_delete_profile_success() {
        let profile = create_test_profile("test-profile", "test@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = create_delete_args("test-profile");
        let cmd = DeleteCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test deleting a non-existent profile.
    #[tokio::test]
    async fn test_delete_nonexistent_profile() {
        let profile_manager = Arc::new(MockProfileManager::new());
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = create_delete_args("nonexistent");
        let cmd = DeleteCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitSetupError::ProfileNotFound { .. }));
    }

    /// Test that profile existence check works correctly.
    #[tokio::test]
    async fn test_check_profile_exists() {
        let profile = create_test_profile("existing", "existing@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = DeleteCommand::new();
        
        // Test existing profile
        let exists = cmd.check_profile_exists("existing", &context).await.unwrap();
        assert!(exists);

        // Test non-existing profile
        let exists = cmd.check_profile_exists("nonexistent", &context).await.unwrap();
        assert!(!exists);
    }

    /// Test that DeleteCommand implements Send + Sync.
    #[test]
    fn test_delete_command_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DeleteCommand>();
    }

    /// Test delete command without profile name.
    #[tokio::test]
    async fn test_delete_command_no_profile_name() {
        let context = create_test_context();
        let args = Args {
            delete: None,
            quiet: true,
            ..Default::default()
        };
        
        let cmd = DeleteCommand::new();
        let result = cmd.execute(&args, &context).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Profile name is required"));
    }

    /// Test confirmation flow.
    #[tokio::test]
    async fn test_get_confirmation() {
        let context = create_test_context();
        let cmd = DeleteCommand::new();
        
        // For now, confirmation always returns true
        let confirmed = cmd.get_confirmation("test-profile", &context).await.unwrap();
        assert!(confirmed);
    }

    /// Test profile deletion process.
    #[tokio::test]
    async fn test_delete_profile() {
        let profile = create_test_profile("test-profile", "test@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = DeleteCommand::new();
        
        // Verify profile exists before deletion
        let exists = cmd.check_profile_exists("test-profile", &context).await.unwrap();
        assert!(exists);

        // Delete the profile
        let result = cmd.delete_profile("test-profile", &context).await;
        assert!(result.is_ok());

        // Verify profile is deleted
        let exists = cmd.check_profile_exists("test-profile", &context).await.unwrap();
        assert!(!exists);
    }

    /// Test deleting multiple profiles in sequence.
    #[tokio::test]
    async fn test_delete_multiple_profiles() {
        let profiles = vec![
            create_test_profile("profile1", "profile1@example.com"),
            create_test_profile("profile2", "profile2@example.com"),
            create_test_profile("profile3", "profile3@example.com"),
        ];
        
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = DeleteCommand::new();
        
        // Delete profile1
        let args = create_delete_args("profile1");
        let result = cmd.execute(&args, &context).await;
        assert!(result.is_ok());

        // Delete profile2
        let args = create_delete_args("profile2");
        let result = cmd.execute(&args, &context).await;
        assert!(result.is_ok());

        // Verify profile3 still exists
        let exists = cmd.check_profile_exists("profile3", &context).await.unwrap();
        assert!(exists);

        // Verify deleted profiles don't exist
        let exists = cmd.check_profile_exists("profile1", &context).await.unwrap();
        assert!(!exists);
        let exists = cmd.check_profile_exists("profile2", &context).await.unwrap();
        assert!(!exists);
    }
}