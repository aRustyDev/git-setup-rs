//! Import command implementation for git-setup-rs.
//!
//! This command imports profiles from 1Password agent.toml configuration.

use super::{Command, CommandContext};
use crate::{
    cli::Args,
    config::types::{Profile, KeyType, Scope},
    error::{GitSetupError, Result},
};
use async_trait::async_trait;

/// Command implementation for importing profiles from 1Password.
pub struct ImportCommand;

impl ImportCommand {
    /// Create a new ImportCommand instance.
    pub fn new() -> Self {
        Self
    }

    /// Import profiles from 1Password vault.
    async fn import_from_1password(&self, context: &CommandContext) -> Result<Vec<Profile>> {
        // Get all SSH keys from 1Password
        let ssh_keys = context.onepassword_wrapper.list_ssh_keys(None).await?;
        
        let mut profiles = Vec::new();
        
        for ssh_key in ssh_keys {
            // Create a profile for each SSH key
            let profile = Profile {
                name: ssh_key.title.clone(),
                git_user_name: Some(ssh_key.title.clone()),
                git_user_email: format!("{}@example.com", ssh_key.title.to_lowercase().replace(" ", ".")),
                key_type: KeyType::Ssh,
                signing_key: None, // Will be retrieved from 1Password when needed
                vault_name: Some("Default".to_string()),
                ssh_key_title: Some(ssh_key.title),
                scope: Some(Scope::Local),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: true,
            };
            
            profiles.push(profile);
        }
        
        Ok(profiles)
    }

    /// Check if a profile with the same name already exists.
    async fn profile_exists(&self, name: &str, context: &CommandContext) -> Result<bool> {
        match context.profile_manager.read(name)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Save imported profiles, handling conflicts.
    async fn save_profiles(&self, profiles: Vec<Profile>, context: &CommandContext) -> Result<(usize, usize)> {
        let mut created = 0;
        let mut skipped = 0;
        
        for profile in profiles {
            if self.profile_exists(&profile.name, context).await? {
                println!("Skipping existing profile: {}", profile.name);
                skipped += 1;
            } else {
                context.profile_manager.write(&profile)?;
                println!("Imported profile: {}", profile.name);
                created += 1;
            }
        }
        
        Ok((created, skipped))
    }

    /// Print import summary.
    fn print_import_summary(&self, created: usize, skipped: usize) {
        println!("\n✓ Import completed");
        println!("  Created: {} profiles", created);
        if skipped > 0 {
            println!("  Skipped: {} profiles (already exist)", skipped);
        }
    }
}

impl Default for ImportCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for ImportCommand {
    async fn execute(&self, args: &Args, context: &CommandContext) -> Result<()> {
        if !args.import {
            return Err(GitSetupError::Git("Import flag is required for import command".to_string()));
        }

        println!("Importing profiles from 1Password...");

        // Import profiles from 1Password
        let profiles = self.import_from_1password(context).await?;
        
        if profiles.is_empty() {
            println!("No SSH keys found in 1Password.");
            return Ok(());
        }

        // Save imported profiles
        let (created, skipped) = self.save_profiles(profiles, context).await?;

        // Print summary
        if !args.quiet {
            self.print_import_summary(created, skipped);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "import"
    }

    fn description(&self) -> &'static str {
        "Import profiles from 1Password agent.toml configuration"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::tests::create_test_context,
        profile::mock::MockProfileManager,
        external::onepassword::{MockOnePasswordWrapper, SshKeyItem},
    };
    use std::sync::Arc;

    /// Helper function to create test args for importing profiles.
    fn create_import_args() -> Args {
        Args {
            import: true,
            quiet: true,
            ..Default::default()
        }
    }

    /// Test that ImportCommand can be created.
    #[test]
    fn test_import_command_creation() {
        let cmd = ImportCommand::new();
        assert_eq!(cmd.name(), "import");
        assert_eq!(cmd.description(), "Import profiles from 1Password agent.toml configuration");
    }

    /// Test importing profiles from 1Password.
    #[tokio::test]
    async fn test_import_from_1password() {
        let ssh_keys = vec![
            SshKeyItem {
                id: "key1".to_string(),
                title: "Work SSH Key".to_string(),
                vault: crate::external::onepassword::Vault {
                    id: "vault-id".to_string(),
                    name: "test-vault".to_string(),
                },
                category: "SSH_KEY".to_string(),
                public_key: Some("ssh-ed25519 AAAAC3...".to_string()),
                private_key: None,
            },
            SshKeyItem {
                id: "key2".to_string(),
                title: "Personal SSH Key".to_string(),
                vault: crate::external::onepassword::Vault {
                    id: "vault-id".to_string(),
                    name: "test-vault".to_string(),
                },
                category: "SSH_KEY".to_string(),
                public_key: Some("ssh-rsa AAAAB3...".to_string()),
                private_key: None,
            },
        ];

        let mut onepassword_wrapper = MockOnePasswordWrapper::new();
        onepassword_wrapper.set_ssh_keys(ssh_keys);

        let profile_manager = Arc::new(MockProfileManager::new());
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        context.onepassword_wrapper = Arc::new(onepassword_wrapper);

        let args = create_import_args();
        let cmd = ImportCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test importing with no SSH keys available.
    #[tokio::test]
    async fn test_import_no_ssh_keys() {
        let onepassword_wrapper = MockOnePasswordWrapper::new(); // No SSH keys
        let profile_manager = Arc::new(MockProfileManager::new());
        
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        context.onepassword_wrapper = Arc::new(onepassword_wrapper);

        let args = create_import_args();
        let cmd = ImportCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test importing with existing profiles (should skip).
    #[tokio::test]
    async fn test_import_with_existing_profiles() {
        let ssh_keys = vec![
            SshKeyItem {
                id: "key1".to_string(),
                title: "Work SSH Key".to_string(),
                vault: crate::external::onepassword::Vault {
                    id: "vault-id".to_string(),
                    name: "test-vault".to_string(),
                },
                category: "SSH_KEY".to_string(),
                public_key: Some("ssh-ed25519 AAAAC3...".to_string()),
                private_key: None,
            },
        ];

        let mut onepassword_wrapper = MockOnePasswordWrapper::new();
        onepassword_wrapper.set_ssh_keys(ssh_keys);

        // Create existing profile with same name
        let existing_profile = Profile {
            name: "Work SSH Key".to_string(),
            git_user_email: "existing@example.com".to_string(),
            ..Default::default()
        };

        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![existing_profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        context.onepassword_wrapper = Arc::new(onepassword_wrapper);

        let args = create_import_args();
        let cmd = ImportCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test that ImportCommand implements Send + Sync.
    #[test]
    fn test_import_command_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ImportCommand>();
    }

    /// Test import command without import flag.
    #[tokio::test]
    async fn test_import_command_no_import_flag() {
        let context = create_test_context();
        let args = Args {
            import: false,
            quiet: true,
            ..Default::default()
        };
        
        let cmd = ImportCommand::new();
        let result = cmd.execute(&args, &context).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Import flag is required"));
    }

    /// Test profile existence check.
    #[tokio::test]
    async fn test_profile_exists() {
        let existing_profile = Profile {
            name: "existing".to_string(),
            git_user_email: "existing@example.com".to_string(),
            ..Default::default()
        };

        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![existing_profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = ImportCommand::new();
        
        // Test existing profile
        let exists = cmd.profile_exists("existing", &context).await.unwrap();
        assert!(exists);

        // Test non-existing profile
        let exists = cmd.profile_exists("nonexistent", &context).await.unwrap();
        assert!(!exists);
    }

    /// Test saving imported profiles.
    #[tokio::test]
    async fn test_save_profiles() {
        let profiles = vec![
            Profile {
                name: "profile1".to_string(),
                git_user_email: "profile1@example.com".to_string(),
                ..Default::default()
            },
            Profile {
                name: "profile2".to_string(),
                git_user_email: "profile2@example.com".to_string(),
                ..Default::default()
            },
        ];

        // Create existing profile with same name as profile1
        let existing_profile = Profile {
            name: "profile1".to_string(),
            git_user_email: "existing@example.com".to_string(),
            ..Default::default()
        };

        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![existing_profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let cmd = ImportCommand::new();
        let (created, skipped) = cmd.save_profiles(profiles, &context).await.unwrap();

        assert_eq!(created, 1); // profile2 created
        assert_eq!(skipped, 1); // profile1 skipped
    }

    /// Test import from 1Password with profile creation.
    #[tokio::test]
    async fn test_import_from_1password_profile_creation() {
        let ssh_keys = vec![
            SshKeyItem {
                id: "key1".to_string(),
                title: "Test SSH Key".to_string(),
                vault: crate::external::onepassword::Vault {
                    id: "vault-id".to_string(),
                    name: "test-vault".to_string(),
                },
                category: "SSH_KEY".to_string(),
                public_key: Some("ssh-ed25519 AAAAC3...".to_string()),
                private_key: None,
            },
        ];

        let mut onepassword_wrapper = MockOnePasswordWrapper::new();
        onepassword_wrapper.set_ssh_keys(ssh_keys);

        let mut context = create_test_context();
        context.onepassword_wrapper = Arc::new(onepassword_wrapper);

        let cmd = ImportCommand::new();
        let profiles = cmd.import_from_1password(&context).await.unwrap();

        assert_eq!(profiles.len(), 1);
        let profile = &profiles[0];
        assert_eq!(profile.name, "Test SSH Key");
        assert_eq!(profile.git_user_email, "test.ssh.key@example.com");
        assert_eq!(profile.key_type, KeyType::Ssh);
        assert!(profile.one_password);
        assert_eq!(profile.ssh_key_title, Some("Test SSH Key".to_string()));
        assert_eq!(profile.vault_name, Some("Default".to_string()));
    }
}