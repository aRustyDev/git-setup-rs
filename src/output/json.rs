//! JSON output formatting for profile lists.
//!
//! This module provides JSON serialization of profile data using the `serde_json` crate
//! for pretty-printed, human-readable output.
//!
//! # Example
//!
//! ```rust
//! use git_setup_rs::output::{JsonFormatter, OutputFormatter};
//! use git_setup_rs::config::types::{Profile, KeyType};
//!
//! // Create a formatter
//! let formatter = JsonFormatter::new();
//!
//! // Create some sample profiles
//! let profiles = vec![
//!     Profile {
//!         name: "work".to_string(),
//!         git_user_email: "work@example.com".to_string(),
//!         key_type: KeyType::Ssh,
//!         // ... other fields
//!         git_user_name: None,
//!         signing_key: None,
//!         vault_name: None,
//!         ssh_key_title: None,
//!         scope: None,
//!         ssh_key_source: None,
//!         ssh_key_path: None,
//!         allowed_signers: None,
//!         match_patterns: vec![],
//!         repos: vec![],
//!         include_if_dirs: vec![],
//!         host_patterns: vec![],
//!         one_password: false,
//!     }
//! ];
//!
//! // Format as JSON
//! let json_output = formatter.format_profiles(&profiles).unwrap();
//! println!("{}", json_output);
//! ```

use crate::config::types::Profile;
use crate::error::{GitSetupError, Result};
use serde_json;

/// Trait for formatting profile data into different output formats.
pub trait OutputFormatter {
    /// Format a list of profiles into a string representation.
    ///
    /// # Arguments
    /// * `profiles` - A slice of Profile structs to format
    ///
    /// # Returns
    /// * `Ok(String)` - The formatted output
    /// * `Err(GitSetupError)` - If serialization fails
    fn format_profiles(&self, profiles: &[Profile]) -> Result<String>;
}

/// JSON formatter implementation that outputs pretty-printed JSON.
#[derive(Debug, Default)]
pub struct JsonFormatter;

impl JsonFormatter {
    /// Create a new JsonFormatter instance.
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_profiles(&self, profiles: &[Profile]) -> Result<String> {
        // Use serde_json's pretty printing to format the profiles
        serde_json::to_string_pretty(profiles).map_err(GitSetupError::Json)
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

    #[test]
    fn test_json_formatter_creation() {
        let formatter = JsonFormatter::new();
        assert_eq!(format!("{:?}", formatter), "JsonFormatter");
    }

    #[test]
    fn test_json_formatter_default() {
        let formatter: JsonFormatter = Default::default();
        assert_eq!(format!("{:?}", formatter), "JsonFormatter");
    }

    #[test]
    fn test_format_empty_profiles_list() {
        let formatter = JsonFormatter::new();
        let profiles: Vec<Profile> = vec![];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let json_output = result.unwrap();
        assert_eq!(json_output.trim(), "[]");
    }

    #[test]
    fn test_format_single_profile() {
        let formatter = JsonFormatter::new();
        let profiles = vec![minimal_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let json_output = result.unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert!(parsed.is_array());

        let profiles_array = parsed.as_array().unwrap();
        assert_eq!(profiles_array.len(), 1);

        let profile_obj = &profiles_array[0];
        assert_eq!(profile_obj["name"], "minimal");
        assert_eq!(profile_obj["git_user_email"], "minimal@example.com");
        assert_eq!(profile_obj["key_type"], "x509");
        assert_eq!(profile_obj["one_password"], false);
    }

    #[test]
    fn test_format_multiple_profiles() {
        let formatter = JsonFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let json_output = result.unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert!(parsed.is_array());

        let profiles_array = parsed.as_array().unwrap();
        assert_eq!(profiles_array.len(), 2);

        // Check first profile (work)
        let work_profile = &profiles_array[0];
        assert_eq!(work_profile["name"], "work");
        assert_eq!(work_profile["git_user_email"], "work@example.com");
        assert_eq!(work_profile["git_user_name"], "Work User");
        assert_eq!(work_profile["key_type"], "ssh");
        assert_eq!(work_profile["scope"], "local");
        assert_eq!(work_profile["ssh_key_source"], "onepassword");
        assert_eq!(work_profile["one_password"], true);

        // Check second profile (personal)
        let personal_profile = &profiles_array[1];
        assert_eq!(personal_profile["name"], "personal");
        assert_eq!(personal_profile["git_user_email"], "personal@example.com");
        assert_eq!(personal_profile["key_type"], "gpg");
        assert_eq!(personal_profile["scope"], "global");
        assert_eq!(personal_profile["ssh_key_source"], "file");
        assert_eq!(personal_profile["one_password"], false);
    }

    #[test]
    fn test_format_profiles_with_arrays() {
        let formatter = JsonFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let json_output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        let profiles_array = parsed.as_array().unwrap();

        // Check work profile arrays
        let work_profile = &profiles_array[0];
        let match_patterns = work_profile["match_patterns"].as_array().unwrap();
        assert_eq!(match_patterns.len(), 2);
        assert_eq!(match_patterns[0], "work/*");
        assert_eq!(match_patterns[1], "company/*");

        let host_patterns = work_profile["host_patterns"].as_array().unwrap();
        assert_eq!(host_patterns.len(), 2);
        assert_eq!(host_patterns[0], "github.com");
        assert_eq!(host_patterns[1], "*.company.com");
    }

    #[test]
    fn test_format_profiles_with_optional_fields() {
        let formatter = JsonFormatter::new();
        let profiles = vec![minimal_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let json_output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        let profiles_array = parsed.as_array().unwrap();
        let profile = &profiles_array[0];

        // Check that optional fields are null when None
        assert!(profile["git_user_name"].is_null());
        assert!(profile["signing_key"].is_null());
        assert!(profile["vault_name"].is_null());
        assert!(profile["ssh_key_title"].is_null());
        assert!(profile["scope"].is_null());
        assert!(profile["ssh_key_source"].is_null());
        assert!(profile["ssh_key_path"].is_null());
        assert!(profile["allowed_signers"].is_null());

        // Check that array fields are empty arrays
        assert_eq!(profile["match_patterns"].as_array().unwrap().len(), 0);
        assert_eq!(profile["repos"].as_array().unwrap().len(), 0);
        assert_eq!(profile["include_if_dirs"].as_array().unwrap().len(), 0);
        assert_eq!(profile["host_patterns"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_json_output_is_pretty_printed() {
        let formatter = JsonFormatter::new();
        let profiles = vec![minimal_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let json_output = result.unwrap();

        // Pretty-printed JSON should contain newlines and proper indentation
        assert!(json_output.contains('\n'));
        assert!(json_output.contains("  ")); // Should have indentation

        // Should not be single-line JSON
        assert_ne!(json_output.lines().count(), 1);
    }

    #[test]
    fn test_all_key_types_serialize_correctly() {
        let formatter = JsonFormatter::new();
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

        let json_output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        let profiles_array = parsed.as_array().unwrap();

        assert_eq!(profiles_array[0]["key_type"], "ssh");
        assert_eq!(profiles_array[1]["key_type"], "gpg");
        assert_eq!(profiles_array[2]["key_type"], "x509");
        assert_eq!(profiles_array[3]["key_type"], "gitsign");
    }

    #[test]
    fn test_all_scope_types_serialize_correctly() {
        let formatter = JsonFormatter::new();
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

        let json_output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        let profiles_array = parsed.as_array().unwrap();

        assert_eq!(profiles_array[0]["scope"], "local");
        assert_eq!(profiles_array[1]["scope"], "global");
        assert_eq!(profiles_array[2]["scope"], "system");
    }

    #[test]
    fn test_all_ssh_key_source_types_serialize_correctly() {
        let formatter = JsonFormatter::new();
        let profiles = vec![
            Profile {
                name: "onepassword_source".to_string(),
                git_user_email: "onepassword@example.com".to_string(),
                key_type: KeyType::Ssh,
                ssh_key_source: Some(SshKeySource::OnePassword),
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "authorizedkeys_source".to_string(),
                git_user_email: "authorizedkeys@example.com".to_string(),
                key_type: KeyType::Ssh,
                ssh_key_source: Some(SshKeySource::AuthorizedKeys),
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            },
            Profile {
                name: "file_source".to_string(),
                git_user_email: "file@example.com".to_string(),
                key_type: KeyType::Ssh,
                ssh_key_source: Some(SshKeySource::File),
                git_user_name: None,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
                scope: None,
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

        let json_output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        let profiles_array = parsed.as_array().unwrap();

        assert_eq!(profiles_array[0]["ssh_key_source"], "onepassword");
        assert_eq!(profiles_array[1]["ssh_key_source"], "authorizedkeys");
        assert_eq!(profiles_array[2]["ssh_key_source"], "file");
    }

    // This test will fail until we implement error handling
    #[test]
    fn test_error_handling_for_serialization_failure() {
        // We can't easily trigger a JSON serialization error with valid Profile structs
        // since serde_json is very robust, but we can test that our error type conversion works
        let json_err = serde_json::from_str::<serde_json::Value>(r#"{"invalid": json"#).unwrap_err();
        let git_err: GitSetupError = json_err.into();

        // Verify the error conversion works
        assert!(matches!(git_err, GitSetupError::Json(_)));
        // The error message contains "EOF while parsing" not "JSON", so check for that
        assert!(git_err.to_string().contains("EOF") || git_err.to_string().contains("expected"));
    }

}
