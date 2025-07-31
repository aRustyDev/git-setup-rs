//! Command handlers for git-setup-rs.
//!
//! This module provides the main command routing and execution logic.

use super::{
    Command, CommandContext,
    list::ListCommand,
    apply::ApplyCommand,
    add::AddCommand,
    delete::DeleteCommand,
    edit::EditCommand,
    import::ImportCommand,
};
use crate::{
    cli::Args,
    error::{GitSetupError, Result},
};
use std::sync::Arc;

/// Main command handler that routes commands to their implementations.
pub struct CommandHandler {
    context: CommandContext,
}

impl CommandHandler {
    /// Create a new CommandHandler with the provided context.
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }

    /// Execute the appropriate command based on the provided arguments.
    pub async fn execute(&self, args: &Args) -> Result<()> {
        // Handle version flag
        if args.version {
            println!("git-setup-rs {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

        // Route to appropriate command based on args
        let command: Arc<dyn Command> = if args.list {
            Arc::new(ListCommand::new())
        } else if args.add.is_some() {
            Arc::new(AddCommand::new())
        } else if args.delete.is_some() {
            Arc::new(DeleteCommand::new())
        } else if args.edit.is_some() {
            Arc::new(EditCommand::new())
        } else if args.import {
            Arc::new(ImportCommand::new())
        } else if args.profile.is_some() {
            // If a profile name is provided without other flags, apply it
            Arc::new(ApplyCommand::new())
        } else {
            // Default to list command if no specific command is provided
            Arc::new(ListCommand::new())
        };

        // Execute the command
        command.execute(args, &self.context).await
    }

    /// Get the context for testing purposes.
    #[cfg(test)]
    pub fn context(&self) -> &CommandContext {
        &self.context
    }
}

/// Builder for creating CommandHandler instances with all dependencies.
pub struct CommandHandlerBuilder {
    context_builder: super::CommandContextBuilder,
}

impl CommandHandlerBuilder {
    /// Create a new CommandHandlerBuilder.
    pub fn new() -> Self {
        Self {
            context_builder: super::CommandContextBuilder::new(),
        }
    }

    /// Set the profile manager dependency.
    pub fn with_profile_manager(mut self, profile_manager: Arc<dyn crate::profile::ProfileManager>) -> Self {
        self.context_builder = self.context_builder.with_profile_manager(profile_manager);
        self
    }

    /// Set the git wrapper dependency.
    pub fn with_git_wrapper(mut self, git_wrapper: Arc<dyn crate::external::git::GitWrapper>) -> Self {
        self.context_builder = self.context_builder.with_git_wrapper(git_wrapper);
        self
    }

    /// Set the 1Password wrapper dependency.
    pub fn with_onepassword_wrapper(mut self, onepassword_wrapper: Arc<dyn crate::external::onepassword::OnePasswordWrapper>) -> Self {
        self.context_builder = self.context_builder.with_onepassword_wrapper(onepassword_wrapper);
        self
    }

    /// Set the GPG wrapper dependency.
    pub fn with_gpg_wrapper(mut self, gpg_wrapper: Arc<dyn crate::external::gpg::GpgWrapper>) -> Self {
        self.context_builder = self.context_builder.with_gpg_wrapper(gpg_wrapper);
        self
    }

    /// Set the profile detector dependency.
    pub fn with_profile_detector(mut self, profile_detector: Arc<dyn crate::detection::ProfileDetector>) -> Self {
        self.context_builder = self.context_builder.with_profile_detector(profile_detector);
        self
    }

    /// Set the fuzzy matcher dependency.
    pub fn with_fuzzy_matcher(mut self, fuzzy_matcher: Arc<dyn crate::matching::FuzzyMatcher>) -> Self {
        self.context_builder = self.context_builder.with_fuzzy_matcher(fuzzy_matcher);
        self
    }

    /// Build the CommandHandler.
    pub fn build(self) -> Result<CommandHandler> {
        let context = self.context_builder.build()?;
        Ok(CommandHandler::new(context))
    }
}

impl Default for CommandHandlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cli::OutputFormat,
        commands::tests::create_test_context,
        config::types::{Profile, KeyType, Scope},
        profile::mock::MockProfileManager,
    };
    use std::sync::Arc;

    /// Helper function to create a test profile.
    fn create_test_profile(name: &str, email: &str) -> Profile {
        Profile {
            name: name.to_string(),
            git_user_name: Some(format!("{} User", name)),
            git_user_email: email.to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("test-key".to_string()),
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

    /// Test that CommandHandler can be created.
    #[test]
    fn test_command_handler_creation() {
        let context = create_test_context();
        let handler = CommandHandler::new(context);
        assert_eq!(handler.context().profile_manager.list().unwrap().len(), 0);
    }

    /// Test version command.
    #[tokio::test]
    async fn test_version_command() {
        let context = create_test_context();
        let handler = CommandHandler::new(context);
        
        let args = Args {
            version: true,
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test list command routing.
    #[tokio::test]
    async fn test_list_command_routing() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
            create_test_profile("personal", "personal@example.com"),
        ];
        
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        
        let handler = CommandHandler::new(context);
        
        let args = Args {
            list: true,
            output: OutputFormat::Json,
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test add command routing.
    #[tokio::test]
    async fn test_add_command_routing() {
        let context = create_test_context();
        let handler = CommandHandler::new(context);
        
        let args = Args {
            add: Some("new-profile".to_string()),
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test delete command routing.
    #[tokio::test]
    async fn test_delete_command_routing() {
        let profile = create_test_profile("test-profile", "test@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        
        let handler = CommandHandler::new(context);
        
        let args = Args {
            delete: Some("test-profile".to_string()),
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test edit command routing.
    #[tokio::test]
    async fn test_edit_command_routing() {
        let profile = create_test_profile("test-profile", "test@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        
        let handler = CommandHandler::new(context);
        
        let args = Args {
            edit: Some("test-profile".to_string()),
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test import command routing.
    #[tokio::test]
    async fn test_import_command_routing() {
        let context = create_test_context();
        let handler = CommandHandler::new(context);
        
        let args = Args {
            import: true,
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test apply command routing with profile name.
    #[tokio::test]
    async fn test_apply_command_routing() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        
        let handler = CommandHandler::new(context);
        
        let args = Args {
            profile: Some("work".to_string()),
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test default command routing (should list profiles).
    #[tokio::test]
    async fn test_default_command_routing() {
        let context = create_test_context();
        let handler = CommandHandler::new(context);
        
        let args = Args {
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test CommandHandlerBuilder.
    #[test]
    fn test_command_handler_builder() {
        let profile_manager = Arc::new(MockProfileManager::new());
        
        let result = CommandHandlerBuilder::new()
            .with_profile_manager(profile_manager)
            .build();
        
        // Should fail because other dependencies are missing
        assert!(result.is_err());
    }

    /// Test CommandHandlerBuilder with all dependencies.
    #[test]
    fn test_command_handler_builder_complete() {
        let context = create_test_context();
        
        let result = CommandHandlerBuilder::new()
            .with_profile_manager(context.profile_manager)
            .with_git_wrapper(context.git_wrapper)
            .with_onepassword_wrapper(context.onepassword_wrapper)
            .with_gpg_wrapper(context.gpg_wrapper)
            .with_profile_detector(context.profile_detector)
            .with_fuzzy_matcher(context.fuzzy_matcher)
            .build();
        
        assert!(result.is_ok());
    }

    /// Test that CommandHandler can handle multiple command types.
    #[tokio::test]
    async fn test_multiple_command_types() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
            create_test_profile("personal", "personal@example.com"),
        ];
        
        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        
        let handler = CommandHandler::new(context);
        
        // Test list command
        let args = Args {
            list: true,
            quiet: true,
            ..Default::default()
        };
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
        
        // Test add command
        let args = Args {
            add: Some("new-profile".to_string()),
            quiet: true,
            ..Default::default()
        };
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
        
        // Test apply command
        let args = Args {
            profile: Some("work".to_string()),
            quiet: true,
            ..Default::default()
        };
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }

    /// Test command priority (list flag should take precedence over profile name).
    #[tokio::test]
    async fn test_command_priority() {
        let profile = create_test_profile("work", "work@example.com");
        let profile_manager = Arc::new(MockProfileManager::with_profiles(vec![profile]));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;
        
        let handler = CommandHandler::new(context);
        
        // Even with a profile name, list flag should take precedence
        let args = Args {
            profile: Some("work".to_string()),
            list: true,
            quiet: true,
            ..Default::default()
        };
        
        let result = handler.execute(&args).await;
        assert!(result.is_ok());
    }
}