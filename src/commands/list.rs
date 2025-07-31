//! List command implementation for git-setup-rs.
//!
//! This command lists all profiles with various output formats and filtering options.

use super::{Command, CommandContext};
use crate::{
    cli::{Args, OutputFormat},
    error::Result,
    output::{OutputFormatter, JsonFormatter, YamlFormatter, CsvFormatter},
};
use async_trait::async_trait;

/// Command implementation for listing profiles.
pub struct ListCommand;

impl ListCommand {
    /// Create a new ListCommand instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ListCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Command for ListCommand {
    async fn execute(&self, args: &Args, context: &CommandContext) -> Result<()> {
        // Load all profiles from the profile manager
        let profiles = context.profile_manager.list()?;

        // Apply filtering if needed
        let filtered_profiles = if let Some(pattern) = args.profile.as_ref() {
            // Filter profiles by pattern
            profiles.into_iter()
                .filter(|p| p.name.contains(pattern) || 
                           p.git_user_email.contains(pattern) ||
                           p.git_user_name.as_ref().map_or(false, |n| n.contains(pattern)))
                .collect()
        } else {
            profiles
        };

        // Check if no profiles found
        if filtered_profiles.is_empty() {
            if args.profile.is_some() {
                println!("No profiles found matching pattern: {}", args.profile.as_ref().unwrap());
            } else {
                println!("No profiles found.");
                println!("Use 'git-setup add <name>' to create a profile.");
            }
            return Ok(());
        }

        // Format output based on requested format
        let output = match args.output {
            OutputFormat::Json => {
                let formatter = JsonFormatter::new();
                formatter.format_profiles(&filtered_profiles)?
            }
            OutputFormat::Yaml => {
                let formatter = YamlFormatter::new();
                formatter.format_profiles(&filtered_profiles)?
            }
            OutputFormat::Csv => {
                let formatter = CsvFormatter::new();
                formatter.format_profiles(&filtered_profiles)?
            }
            OutputFormat::Tabular => {
                // Use simple table format for now
                let mut output = String::new();
                output.push_str("NAME\tEMAIL\tKEY_TYPE\tSCOPE\n");
                for profile in &filtered_profiles {
                    output.push_str(&format!(
                        "{}\t{}\t{:?}\t{:?}\n",
                        profile.name,
                        profile.git_user_email,
                        profile.key_type,
                        profile.scope.as_ref().unwrap_or(&crate::config::types::Scope::Local)
                    ));
                }
                output
            }
            OutputFormat::Toml => {
                // Simple TOML-like output
                let mut output = String::new();
                for profile in &filtered_profiles {
                    output.push_str(&format!("[[profiles]]\n"));
                    output.push_str(&format!("name = \"{}\"\n", profile.name));
                    output.push_str(&format!("git_user_email = \"{}\"\n", profile.git_user_email));
                    if let Some(name) = &profile.git_user_name {
                        output.push_str(&format!("git_user_name = \"{}\"\n", name));
                    }
                    output.push_str(&format!("key_type = \"{:?}\"\n", profile.key_type));
                    output.push_str(&format!("one_password = {}\n", profile.one_password));
                    output.push('\n');
                }
                output
            }
        };

        // Print the formatted output
        if !args.quiet {
            print!("{}", output);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "list"
    }

    fn description(&self) -> &'static str {
        "List all profiles with optional filtering and formatting"
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

    /// Test that ListCommand can be created.
    #[test]
    fn test_list_command_creation() {
        let cmd = ListCommand::new();
        assert_eq!(cmd.name(), "list");
        assert_eq!(cmd.description(), "List all profiles with optional filtering and formatting");
    }

    /// Test listing profiles with default (tabular) format.
    #[tokio::test]
    async fn test_list_profiles_tabular_format() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
            create_test_profile("personal", "personal@example.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            output: OutputFormat::Tabular,
            quiet: true, // Suppress output for testing
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test listing profiles with JSON format.
    #[tokio::test]
    async fn test_list_profiles_json_format() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            output: OutputFormat::Json,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test listing profiles with YAML format.
    #[tokio::test]
    async fn test_list_profiles_yaml_format() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            output: OutputFormat::Yaml,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test listing empty profiles.
    #[tokio::test]
    async fn test_list_empty_profiles() {
        let profile_manager = Arc::new(MockProfileManager::new());
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            output: OutputFormat::Tabular,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test filtering profiles by name pattern.
    #[tokio::test]
    async fn test_list_profiles_with_filter() {
        let profiles = vec![
            create_test_profile("work-project", "work@example.com"),
            create_test_profile("personal", "personal@example.com"),
            create_test_profile("work-client", "client@example.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            profile: Some("work".to_string()),
            output: OutputFormat::Tabular,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test filtering profiles by email pattern.
    #[tokio::test]
    async fn test_list_profiles_filter_by_email() {
        let profiles = vec![
            create_test_profile("work", "work@company.com"),
            create_test_profile("personal", "personal@gmail.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            profile: Some("company".to_string()),
            output: OutputFormat::Tabular,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test filtering with no matches.
    #[tokio::test]
    async fn test_list_profiles_no_matches() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            profile: Some("nonexistent".to_string()),
            output: OutputFormat::Tabular,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test CSV format output.
    #[tokio::test]
    async fn test_list_profiles_csv_format() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            output: OutputFormat::Csv,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test TOML format output.
    #[tokio::test]
    async fn test_list_profiles_toml_format() {
        let profiles = vec![
            create_test_profile("work", "work@example.com"),
        ];

        let profile_manager = Arc::new(MockProfileManager::with_profiles(profiles));
        let mut context = create_test_context();
        context.profile_manager = profile_manager;

        let args = Args {
            list: true,
            output: OutputFormat::Toml,
            quiet: true,
            ..Default::default()
        };

        let cmd = ListCommand::new();
        let result = cmd.execute(&args, &context).await;

        assert!(result.is_ok());
    }

    /// Test that ListCommand implements Send + Sync.
    #[test]
    fn test_list_command_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ListCommand>();
    }
}