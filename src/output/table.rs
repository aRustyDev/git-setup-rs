//! Tabular output formatting for profile lists.
//!
//! This module provides human-readable table formatting of profile data using the `comfy-table` crate
//! for professional terminal display with borders, alignment, and proper column sizing.
//!
//! # Example
//!
//! ```rust
//! use git_setup_rs::output::{TableFormatter, OutputFormatter};
//! use git_setup_rs::config::types::{Profile, KeyType, Scope};
//!
//! // Create a formatter
//! let formatter = TableFormatter::new();
//!
//! // Create some sample profiles
//! let profiles = vec![
//!     Profile {
//!         name: "work".to_string(),
//!         git_user_email: "work@example.com".to_string(),
//!         key_type: KeyType::Ssh,
//!         scope: Some(Scope::Local),
//!         // ... other fields
//!         git_user_name: None,
//!         signing_key: Some("~/.ssh/work_ed25519.pub".to_string()),
//!         vault_name: Some("Work Vault".to_string()),
//!         ssh_key_title: None,
//!         ssh_key_source: None,
//!         ssh_key_path: None,
//!         allowed_signers: None,
//!         match_patterns: vec![],
//!         repos: vec![],
//!         include_if_dirs: vec![],
//!         host_patterns: vec![],
//!         one_password: true,
//!     }
//! ];
//!
//! // Format as table
//! let table_output = formatter.format_profiles(&profiles).unwrap();
//! println!("{}", table_output);
//! ```

use crate::config::types::Profile;
use crate::error::Result;
use crate::output::OutputFormatter;
use comfy_table::{presets::UTF8_FULL, Attribute, Cell, ContentArrangement, Table};

/// Maximum width for truncating long field values to maintain readable table layout.
const MAX_FIELD_WIDTH: usize = 30;

/// Message displayed when there are no profiles to show.
const NO_PROFILES_MESSAGE: &str = "No profiles found.";

/// Tabular formatter implementation that outputs professional tables with borders and alignment.
#[derive(Debug, Default)]
pub struct TableFormatter;

impl TableFormatter {
    /// Create a new TableFormatter instance.
    pub fn new() -> Self {
        Self
    }

    /// Truncate a string to the maximum field width with ellipsis if needed.
    fn truncate_field(value: &str) -> String {
        if value.len() <= MAX_FIELD_WIDTH {
            value.to_string()
        } else {
            format!("{}...", &value[..MAX_FIELD_WIDTH.saturating_sub(3)])
        }
    }

    /// Format an optional string field, showing "none" for None values.
    fn format_optional(value: &Option<String>) -> String {
        match value {
            Some(v) => Self::truncate_field(v),
            None => "none".to_string(),
        }
    }

    /// Format the scope field, showing "none" for None values.
    fn format_scope(scope: &Option<crate::config::types::Scope>) -> String {
        match scope {
            Some(scope) => match scope {
                crate::config::types::Scope::Local => "local".to_string(),
                crate::config::types::Scope::Global => "global".to_string(),
                crate::config::types::Scope::System => "system".to_string(),
            },
            None => "none".to_string(),
        }
    }

    /// Format the key type field.
    fn format_key_type(key_type: &crate::config::types::KeyType) -> String {
        match key_type {
            crate::config::types::KeyType::Ssh => "ssh".to_string(),
            crate::config::types::KeyType::Gpg => "gpg".to_string(),
            crate::config::types::KeyType::X509 => "x509".to_string(),
            crate::config::types::KeyType::Gitsign => "gitsign".to_string(),
        }
    }
}

impl OutputFormatter for TableFormatter {
    fn format_profiles(&self, profiles: &[Profile]) -> Result<String> {
        // Handle empty profile list
        if profiles.is_empty() {
            return Ok(NO_PROFILES_MESSAGE.to_string());
        }

        // Create table with UTF8 borders
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        // Set header
        table.set_header(vec![
            Cell::new("Name").add_attribute(Attribute::Bold),
            Cell::new("Email").add_attribute(Attribute::Bold),
            Cell::new("Key Type").add_attribute(Attribute::Bold),
            Cell::new("Signing Key").add_attribute(Attribute::Bold),
            Cell::new("Vault").add_attribute(Attribute::Bold),
            Cell::new("Scope").add_attribute(Attribute::Bold),
            Cell::new("1Password").add_attribute(Attribute::Bold),
        ]);

        // Add rows for each profile
        for profile in profiles {
            table.add_row(vec![
                Cell::new(&Self::truncate_field(&profile.name)),
                Cell::new(&Self::truncate_field(&profile.git_user_email)),
                Cell::new(&Self::format_key_type(&profile.key_type)),
                Cell::new(&Self::format_optional(&profile.signing_key)),
                Cell::new(&Self::format_optional(&profile.vault_name)),
                Cell::new(&Self::format_scope(&profile.scope)),
                Cell::new(&profile.one_password.to_string()),
            ]);
        }

        Ok(table.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{KeyType, Scope, SshKeySource};

    /// Create test profiles with different configurations for comprehensive testing.
    fn test_profiles() -> Vec<Profile> {
        vec![
            Profile {
                name: "work".to_string(),
                git_user_name: Some("Work User".to_string()),
                git_user_email: "work@example.com".to_string(),
                key_type: KeyType::Ssh,
                signing_key: Some("~/.ssh/work_ed25519.pub".to_string()),
                vault_name: Some("Work Vault".to_string()),
                ssh_key_title: Some("Work SSH Key".to_string()),
                scope: Some(Scope::Local),
                ssh_key_source: Some(SshKeySource::OnePassword),
                ssh_key_path: Some("~/.ssh/work_ed25519".to_string()),
                allowed_signers: Some("~/.ssh/allowed_signers_work".to_string()),
                match_patterns: vec!["work/*".to_string(), "company/*".to_string()],
                repos: vec!["git@github.com:company/repo1.git".to_string()],
                include_if_dirs: vec!["/work/projects".to_string()],
                host_patterns: vec!["github.com".to_string(), "*.company.com".to_string()],
                one_password: true,
            },
            Profile {
                name: "personal".to_string(),
                git_user_name: Some("Personal User".to_string()),
                git_user_email: "personal@example.com".to_string(),
                key_type: KeyType::Gpg,
                signing_key: Some("0x1234567890ABCDEF".to_string()),
                vault_name: None,
                ssh_key_title: None,
                scope: Some(Scope::Global),
                ssh_key_source: Some(SshKeySource::File),
                ssh_key_path: Some("~/.ssh/personal_rsa".to_string()),
                allowed_signers: None,
                match_patterns: vec!["personal/*".to_string()],
                repos: vec!["git@github.com:personal/repo1.git".to_string()],
                include_if_dirs: vec!["/home/user/personal".to_string()],
                host_patterns: vec!["github.com".to_string()],
                one_password: false,
            },
        ]
    }

    /// Create a minimal profile for testing edge cases.
    fn minimal_profile() -> Profile {
        Profile {
            name: "minimal".to_string(),
            git_user_name: None,
            git_user_email: "minimal@example.com".to_string(),
            key_type: KeyType::X509,
            signing_key: None,
            vault_name: None,
            ssh_key_title: None,
            scope: None,
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

    /// Create a profile with long field values for testing truncation.
    fn long_field_profile() -> Profile {
        Profile {
            name: "very_long_profile_name_that_exceeds_max_width".to_string(),
            git_user_name: Some("Very Long User Name That Should Be Truncated".to_string()),
            git_user_email: "very_long_email_address_that_should_be_truncated@example.com".to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("~/.ssh/very_long_key_path_that_should_be_truncated_ed25519.pub".to_string()),
            vault_name: Some("Very Long Vault Name That Should Be Truncated".to_string()),
            ssh_key_title: None,
            scope: Some(Scope::System),
            ssh_key_source: None,
            ssh_key_path: None,
            allowed_signers: None,
            match_patterns: vec![],
            repos: vec![],
            include_if_dirs: vec![],
            host_patterns: vec![],
            one_password: true,
        }
    }

    #[test]
    fn test_table_formatter_creation() {
        let formatter = TableFormatter::new();
        assert_eq!(format!("{:?}", formatter), "TableFormatter");
    }

    #[test]
    fn test_table_formatter_default() {
        let formatter: TableFormatter = Default::default();
        assert_eq!(format!("{:?}", formatter), "TableFormatter");
    }

    #[test]
    fn test_format_empty_profiles_list() {
        let formatter = TableFormatter::new();
        let profiles: Vec<Profile> = vec![];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();
        assert_eq!(table_output, NO_PROFILES_MESSAGE);
    }

    #[test]
    fn test_format_single_profile() {
        let formatter = TableFormatter::new();
        let profiles = vec![minimal_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        // Verify it contains the expected profile data
        assert!(table_output.contains("minimal"));
        assert!(table_output.contains("minimal@example.com"));
        assert!(table_output.contains("x509"));
        assert!(table_output.contains("none")); // For optional fields
        assert!(table_output.contains("false")); // For one_password

        // Verify it has table structure
        assert!(table_output.contains("┌")); // Top border
        assert!(table_output.contains("└")); // Bottom border
        assert!(table_output.contains("│")); // Side borders
        assert!(table_output.contains("Name")); // Header
    }

    #[test]
    fn test_format_multiple_profiles() {
        let formatter = TableFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        // Verify it contains data from both profiles
        assert!(table_output.contains("work"));
        assert!(table_output.contains("work@example.com"));
        assert!(table_output.contains("personal"));
        assert!(table_output.contains("personal@example.com"));
        assert!(table_output.contains("ssh"));
        assert!(table_output.contains("gpg"));
        assert!(table_output.contains("local"));
        assert!(table_output.contains("global"));
        assert!(table_output.contains("Work Vault"));
        assert!(table_output.contains("true"));
        assert!(table_output.contains("false"));

        // Verify table structure
        assert!(table_output.contains("┌")); // Top border
        assert!(table_output.contains("└")); // Bottom border
        assert!(table_output.contains("├")); // Row separators
        assert!(table_output.contains("Name")); // Header
        assert!(table_output.contains("Email"));
        assert!(table_output.contains("Key Type"));
        assert!(table_output.contains("Signing Key"));
        assert!(table_output.contains("Vault"));
        assert!(table_output.contains("Scope"));
        assert!(table_output.contains("1Password"));
    }

    #[test]
    fn test_truncate_field() {
        // Short field should remain unchanged
        let short = "short";
        assert_eq!(TableFormatter::truncate_field(short), "short");

        // Field at max width should remain unchanged
        let max_width = "a".repeat(MAX_FIELD_WIDTH);
        assert_eq!(TableFormatter::truncate_field(&max_width), max_width);

        // Long field should be truncated with ellipsis
        let long = "a".repeat(MAX_FIELD_WIDTH + 10);
        let truncated = TableFormatter::truncate_field(&long);
        assert!(truncated.len() <= MAX_FIELD_WIDTH);
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.len(), MAX_FIELD_WIDTH);
    }

    #[test]
    fn test_format_optional() {
        // Some value should be truncated if needed
        let some_value = Some("test value".to_string());
        assert_eq!(TableFormatter::format_optional(&some_value), "test value");

        // None value should show "none"
        let none_value: Option<String> = None;
        assert_eq!(TableFormatter::format_optional(&none_value), "none");

        // Long Some value should be truncated
        let long_value = Some("a".repeat(MAX_FIELD_WIDTH + 10));
        let formatted = TableFormatter::format_optional(&long_value);
        assert!(formatted.len() <= MAX_FIELD_WIDTH);
        assert!(formatted.ends_with("..."));
    }

    #[test]
    fn test_format_scope() {
        assert_eq!(TableFormatter::format_scope(&Some(Scope::Local)), "local");
        assert_eq!(TableFormatter::format_scope(&Some(Scope::Global)), "global");
        assert_eq!(TableFormatter::format_scope(&Some(Scope::System)), "system");
        assert_eq!(TableFormatter::format_scope(&None), "none");
    }

    #[test]
    fn test_format_key_type() {
        assert_eq!(TableFormatter::format_key_type(&KeyType::Ssh), "ssh");
        assert_eq!(TableFormatter::format_key_type(&KeyType::Gpg), "gpg");
        assert_eq!(TableFormatter::format_key_type(&KeyType::X509), "x509");
        assert_eq!(TableFormatter::format_key_type(&KeyType::Gitsign), "gitsign");
    }

    #[test]
    fn test_format_profiles_with_long_fields() {
        let formatter = TableFormatter::new();
        let profiles = vec![long_field_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        // Verify long fields are present but truncated (based on actual output)
        assert!(table_output.contains("very_long_profile_name_that..."));
        assert!(table_output.contains("very_long_email_address_tha..."));
        assert!(table_output.contains("~/.ssh/very_long_key_path_t..."));
        assert!(table_output.contains("Very Long Vault Name That S..."));

        // Verify structure is maintained
        assert!(table_output.contains("system")); // Scope should not be truncated
        assert!(table_output.contains("ssh")); // Key type should not be truncated
        assert!(table_output.contains("true")); // Boolean should not be truncated
    }

    #[test]
    fn test_all_key_types_format_correctly() {
        let formatter = TableFormatter::new();
        let profiles = vec![
            Profile {
                name: "ssh_profile".to_string(),
                git_user_email: "ssh@example.com".to_string(),
                key_type: KeyType::Ssh,
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "gpg_profile".to_string(),
                git_user_email: "gpg@example.com".to_string(),
                key_type: KeyType::Gpg,
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "x509_profile".to_string(),
                git_user_email: "x509@example.com".to_string(),
                key_type: KeyType::X509,
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "gitsign_profile".to_string(),
                git_user_email: "gitsign@example.com".to_string(),
                key_type: KeyType::Gitsign,
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
        ];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        assert!(table_output.contains("ssh"));
        assert!(table_output.contains("gpg"));
        assert!(table_output.contains("x509"));
        assert!(table_output.contains("gitsign"));
    }

    #[test]
    fn test_all_scope_types_format_correctly() {
        let formatter = TableFormatter::new();
        let profiles = vec![
            Profile {
                name: "local_scope".to_string(),
                git_user_email: "local@example.com".to_string(),
                key_type: KeyType::Ssh,
                scope: Some(Scope::Local),
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "global_scope".to_string(),
                git_user_email: "global@example.com".to_string(),
                key_type: KeyType::Ssh,
                scope: Some(Scope::Global),
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "system_scope".to_string(),
                git_user_email: "system@example.com".to_string(),
                key_type: KeyType::Ssh,
                scope: Some(Scope::System),
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
        ];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        assert!(table_output.contains("local"));
        assert!(table_output.contains("global"));
        assert!(table_output.contains("system"));
    }

    #[test]
    fn test_boolean_formatting() {
        let formatter = TableFormatter::new();
        let profiles = vec![
            Profile {
                name: "onepassword_true".to_string(),
                git_user_email: "true@example.com".to_string(),
                key_type: KeyType::Ssh,
                scope: None,
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: true,
            },
            Profile {
                name: "onepassword_false".to_string(),
                git_user_email: "false@example.com".to_string(),
                key_type: KeyType::Ssh,
                scope: None,
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
        ];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        assert!(table_output.contains("true"));
        assert!(table_output.contains("false"));
    }

    #[test]
    fn test_table_has_proper_structure() {
        let formatter = TableFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        // Verify UTF8 table borders are present
        assert!(table_output.contains("┌")); // Top-left corner
        assert!(table_output.contains("┐")); // Top-right corner
        assert!(table_output.contains("└")); // Bottom-left corner
        assert!(table_output.contains("┘")); // Bottom-right corner
        assert!(table_output.contains("├")); // Left T-junction
        assert!(table_output.contains("┤")); // Right T-junction
        assert!(table_output.contains("┬")); // Top T-junction
        assert!(table_output.contains("┴")); // Bottom T-junction
        assert!(table_output.contains("┼")); // Cross
        assert!(table_output.contains("─")); // Horizontal line
        assert!(table_output.contains("│")); // Vertical line

        // Verify headers are present
        assert!(table_output.contains("Name"));
        assert!(table_output.contains("Email"));
        assert!(table_output.contains("Key Type"));
        assert!(table_output.contains("Signing Key"));
        assert!(table_output.contains("Vault"));
        assert!(table_output.contains("Scope"));
        assert!(table_output.contains("1Password"));
    }

    #[test]
    fn test_no_profiles_message() {
        let formatter = TableFormatter::new();
        let profiles: Vec<Profile> = vec![];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output, NO_PROFILES_MESSAGE);
    }

    #[test]
    fn test_special_characters_in_profile_data() {
        let formatter = TableFormatter::new();
        let profiles = vec![
            Profile {
                name: "test-profile_with.special@chars".to_string(),
                git_user_email: "test+special@sub-domain.example.com".to_string(),
                key_type: KeyType::Ssh,
                signing_key: Some("~/.ssh/key-with-dashes_and.dots".to_string()),
                vault_name: Some("Vault (with parentheses) & symbols".to_string()),
                ssh_key_title: None,
                scope: Some(Scope::Local),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                git_user_name: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: true,
            },
        ];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        // Verify special characters are handled properly (truncated as expected)
        assert!(table_output.contains("test-profile_with.special@c..."));
        assert!(table_output.contains("test+special@sub-domain.exa..."));
        assert!(table_output.contains("~/.ssh/key-with-dashes_and...."));
        assert!(table_output.contains("Vault (with parentheses) & ..."));
    }

    #[test]
    fn test_edge_case_empty_strings() {
        let formatter = TableFormatter::new();
        let profiles = vec![
            Profile {
                name: "".to_string(), // Empty name
                git_user_email: "empty-name@example.com".to_string(),
                key_type: KeyType::Ssh,
                signing_key: Some("".to_string()), // Empty signing key
                vault_name: Some("".to_string()), // Empty vault name
                ssh_key_title: None,
                scope: None,
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                git_user_name: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
        ];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let table_output = result.unwrap();

        // Verify empty strings are handled (they should appear as empty cells)
        assert!(table_output.contains("empty-name@example.com"));
        assert!(table_output.contains("ssh"));
        assert!(table_output.contains("none")); // For scope
        assert!(table_output.contains("false")); // For one_password
    }
}
