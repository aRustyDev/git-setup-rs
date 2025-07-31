//! Command module for git-setup-rs.
//!
//! This module provides the command handler architecture for processing CLI operations.
//! It implements a dependency injection pattern to allow for easy testing and modular design.

pub mod handlers;
pub mod list;
pub mod apply;
pub mod add;
pub mod delete;
pub mod edit;
pub mod import;

use crate::{
    cli::Args,
    config::types::Profile,
    error::Result,
    profile::ProfileManager,
    external::{
        git::GitWrapper,
        onepassword::OnePasswordWrapper,
        gpg::GpgWrapper,
    },
    detection::ProfileDetector,
    matching::FuzzyMatcher,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Command context containing all dependencies needed by commands.
///
/// This structure uses dependency injection to provide all external dependencies
/// to command implementations, allowing for easy testing with mocks.
#[derive(Clone)]
pub struct CommandContext {
    pub profile_manager: Arc<dyn ProfileManager>,
    pub git_wrapper: Arc<dyn GitWrapper>,
    pub onepassword_wrapper: Arc<dyn OnePasswordWrapper>,
    pub gpg_wrapper: Arc<dyn GpgWrapper>,
    pub profile_detector: Arc<dyn ProfileDetector>,
    pub fuzzy_matcher: Arc<dyn FuzzyMatcher>,
}

/// Builder for creating CommandContext instances.
///
/// This builder pattern ensures all required dependencies are provided
/// and provides a clean API for dependency injection.
pub struct CommandContextBuilder {
    profile_manager: Option<Arc<dyn ProfileManager>>,
    git_wrapper: Option<Arc<dyn GitWrapper>>,
    onepassword_wrapper: Option<Arc<dyn OnePasswordWrapper>>,
    gpg_wrapper: Option<Arc<dyn GpgWrapper>>,
    profile_detector: Option<Arc<dyn ProfileDetector>>,
    fuzzy_matcher: Option<Arc<dyn FuzzyMatcher>>,
}

impl CommandContextBuilder {
    /// Create a new CommandContextBuilder.
    pub fn new() -> Self {
        Self {
            profile_manager: None,
            git_wrapper: None,
            onepassword_wrapper: None,
            gpg_wrapper: None,
            profile_detector: None,
            fuzzy_matcher: None,
        }
    }

    /// Set the profile manager dependency.
    pub fn with_profile_manager(mut self, profile_manager: Arc<dyn ProfileManager>) -> Self {
        self.profile_manager = Some(profile_manager);
        self
    }

    /// Set the git wrapper dependency.
    pub fn with_git_wrapper(mut self, git_wrapper: Arc<dyn GitWrapper>) -> Self {
        self.git_wrapper = Some(git_wrapper);
        self
    }

    /// Set the 1Password wrapper dependency.
    pub fn with_onepassword_wrapper(mut self, onepassword_wrapper: Arc<dyn OnePasswordWrapper>) -> Self {
        self.onepassword_wrapper = Some(onepassword_wrapper);
        self
    }

    /// Set the GPG wrapper dependency.
    pub fn with_gpg_wrapper(mut self, gpg_wrapper: Arc<dyn GpgWrapper>) -> Self {
        self.gpg_wrapper = Some(gpg_wrapper);
        self
    }

    /// Set the profile detector dependency.
    pub fn with_profile_detector(mut self, profile_detector: Arc<dyn ProfileDetector>) -> Self {
        self.profile_detector = Some(profile_detector);
        self
    }

    /// Set the fuzzy matcher dependency.
    pub fn with_fuzzy_matcher(mut self, fuzzy_matcher: Arc<dyn FuzzyMatcher>) -> Self {
        self.fuzzy_matcher = Some(fuzzy_matcher);
        self
    }

    /// Build the CommandContext.
    ///
    /// # Errors
    /// Returns an error if any required dependency is missing.
    pub fn build(self) -> Result<CommandContext> {
        Ok(CommandContext {
            profile_manager: self.profile_manager
                .ok_or_else(|| crate::error::GitSetupError::Git("ProfileManager not provided".to_string()))?,
            git_wrapper: self.git_wrapper
                .ok_or_else(|| crate::error::GitSetupError::Git("GitWrapper not provided".to_string()))?,
            onepassword_wrapper: self.onepassword_wrapper
                .ok_or_else(|| crate::error::GitSetupError::Git("OnePasswordWrapper not provided".to_string()))?,
            gpg_wrapper: self.gpg_wrapper
                .ok_or_else(|| crate::error::GitSetupError::Git("GpgWrapper not provided".to_string()))?,
            profile_detector: self.profile_detector
                .ok_or_else(|| crate::error::GitSetupError::Git("ProfileDetector not provided".to_string()))?,
            fuzzy_matcher: self.fuzzy_matcher
                .ok_or_else(|| crate::error::GitSetupError::Git("FuzzyMatcher not provided".to_string()))?,
        })
    }
}

impl Default for CommandContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for individual command implementations.
///
/// Each command implements this trait to provide a standardized interface
/// for execution. Commands are async to support external operations.
#[async_trait]
pub trait Command: Send + Sync {
    /// Execute the command with the given arguments and context.
    ///
    /// # Arguments
    /// * `args` - The parsed command line arguments
    /// * `context` - The command context containing all dependencies
    ///
    /// # Returns
    /// Returns Ok(()) on success, or an error on failure.
    async fn execute(&self, args: &Args, context: &CommandContext) -> Result<()>;

    /// Get the name of this command.
    fn name(&self) -> &'static str;

    /// Get a description of this command.
    fn description(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        profile::mock::MockProfileManager,
        external::{
            git::MockGitWrapper,
            onepassword::MockOnePasswordWrapper,
            gpg::MockGpgWrapper,
        },
        detection::{DetectionResult, MockProfileDetector},
        matching::{FuzzyMatcher, MockFuzzyMatcher},
    };

    /// Test that the Command trait is object-safe and has proper bounds.
    #[test]
    fn test_command_trait_object_safety() {
        fn _assert_object_safe(_: &dyn Command) {}
        fn _assert_send_sync<T: Send + Sync>() {}
        fn _assert_command_bounds<T: Command>() {
            _assert_send_sync::<T>();
        }
    }

    /// Test that CommandContext can be created with the builder pattern.
    #[test]
    fn test_command_context_builder() {
        let context = CommandContextBuilder::new()
            .with_profile_manager(Arc::new(MockProfileManager::new()))
            .with_git_wrapper(Arc::new(MockGitWrapper::new()))
            .with_onepassword_wrapper(Arc::new(MockOnePasswordWrapper::new()))
            .with_gpg_wrapper(Arc::new(MockGpgWrapper::new()))
            .with_profile_detector(Arc::new(MockProfileDetector::new()))
            .with_fuzzy_matcher(Arc::new(MockFuzzyMatcher::new()))
            .build();

        assert!(context.is_ok());
    }

    /// Test that CommandContext builder fails when dependencies are missing.
    #[test]
    fn test_command_context_builder_missing_dependencies() {
        let result = CommandContextBuilder::new()
            .with_profile_manager(Arc::new(MockProfileManager::new()))
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("GitWrapper not provided"));
    }

    /// Test that CommandContext can be cloned.
    #[test]
    fn test_command_context_clone() {
        let context = CommandContextBuilder::new()
            .with_profile_manager(Arc::new(MockProfileManager::new()))
            .with_git_wrapper(Arc::new(MockGitWrapper::new()))
            .with_onepassword_wrapper(Arc::new(MockOnePasswordWrapper::new()))
            .with_gpg_wrapper(Arc::new(MockGpgWrapper::new()))
            .with_profile_detector(Arc::new(MockProfileDetector::new()))
            .with_fuzzy_matcher(Arc::new(MockFuzzyMatcher::new()))
            .build()
            .unwrap();

        let _cloned = context.clone();
    }

    /// Helper function to create a test CommandContext.
    pub fn create_test_context() -> CommandContext {
        CommandContextBuilder::new()
            .with_profile_manager(Arc::new(MockProfileManager::new()))
            .with_git_wrapper(Arc::new(MockGitWrapper::new()))
            .with_onepassword_wrapper(Arc::new(MockOnePasswordWrapper::new()))
            .with_gpg_wrapper(Arc::new(MockGpgWrapper::new()))
            .with_profile_detector(Arc::new(MockProfileDetector::new()))
            .with_fuzzy_matcher(Arc::new(MockFuzzyMatcher::new()))
            .build()
            .unwrap()
    }
}