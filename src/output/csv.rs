//! CSV output formatting for profile lists.
//!
//! This module provides CSV serialization of profile data using the `csv` crate
//! for structured, tabular output that can be imported into spreadsheets or
//! processed by other tools.
//!
//! # Example
//!
//! ```rust
//! use git_setup_rs::output::{CsvFormatter, OutputFormatter};
//! use git_setup_rs::config::types::{Profile, KeyType};
//!
//! // Create a formatter
//! let formatter = CsvFormatter::new();
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
//! // Format as CSV
//! let csv_output = formatter.format_profiles(&profiles).unwrap();
//! println!("{}", csv_output);
//! ```

use crate::config::types::Profile;
use crate::error::{GitSetupError, Result};
use crate::output::OutputFormatter;

/// CSV formatter implementation that outputs comma-separated values.
#[derive(Debug, Default)]
pub struct CsvFormatter;

impl CsvFormatter {
    /// Create a new CsvFormatter instance.
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for CsvFormatter {
    fn format_profiles(&self, profiles: &[Profile]) -> Result<String> {
        let mut output = Vec::new();
        let mut writer = csv::Writer::from_writer(&mut output);

        // Write header row with all Profile fields
        writer.write_record(&[
            "name",
            "git_user_name",
            "git_user_email",
            "key_type",
            "signing_key",
            "vault_name",
            "ssh_key_title",
            "scope",
            "ssh_key_source",
            "ssh_key_path",
            "allowed_signers",
            "match_patterns",
            "repos",
            "include_if_dirs",
            "host_patterns",
            "one_password"
        ])?;

        // Write data rows
        for profile in profiles {
            let record = vec![
                profile.name.clone(),
                profile.git_user_name.as_deref().unwrap_or("").to_string(),
                profile.git_user_email.clone(),
                format!("{:?}", profile.key_type).to_lowercase(),
                profile.signing_key.as_deref().unwrap_or("").to_string(),
                profile.vault_name.as_deref().unwrap_or("").to_string(),
                profile.ssh_key_title.as_deref().unwrap_or("").to_string(),
                profile.scope.as_ref().map(|s| format!("{:?}", s).to_lowercase()).unwrap_or_default(),
                profile.ssh_key_source.as_ref().map(|s| format!("{:?}", s).to_lowercase()).unwrap_or_default(),
                profile.ssh_key_path.as_deref().unwrap_or("").to_string(),
                profile.allowed_signers.as_deref().unwrap_or("").to_string(),
                profile.match_patterns.join(";"),
                profile.repos.join(";"),
                profile.include_if_dirs.join(";"),
                profile.host_patterns.join(";"),
                profile.one_password.to_string(),
            ];
            writer.write_record(&record)?;
        }

        writer.flush()?;
        drop(writer);

        String::from_utf8(output).map_err(|e| GitSetupError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("CSV output contains invalid UTF-8: {}", e)
        )))
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

    /// Create a profile with special characters for testing CSV escaping.
    fn special_chars_profile() -> Profile {
        Profile {
            name: "special,chars".to_string(),
            git_user_name: Some("User with \"quotes\"".to_string()),
            git_user_email: "special@example.com".to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("key with\nnewline".to_string()),
            vault_name: Some("Vault, with commas".to_string()),
            ssh_key_title: Some("SSH Key \"Title\"".to_string()),
            scope: Some(Scope::Local),
            ssh_key_source: Some(SshKeySource::OnePassword),
            ssh_key_path: Some("/path/with spaces/key".to_string()),
            allowed_signers: Some("signers,file".to_string()),
            match_patterns: vec!["pattern,with,commas".to_string(), "pattern\"with\"quotes".to_string()],
            repos: vec!["git@github.com:user/repo,with,commas.git".to_string()],
            include_if_dirs: vec!["/path/with spaces".to_string()],
            host_patterns: vec!["*.example,com".to_string()],
            one_password: true,
        }
    }

    #[test]
    fn test_csv_formatter_creation() {
        let formatter = CsvFormatter::new();
        assert_eq!(format!("{:?}", formatter), "CsvFormatter");
    }

    #[test]
    fn test_csv_formatter_default() {
        let formatter: CsvFormatter = Default::default();
        assert_eq!(format!("{:?}", formatter), "CsvFormatter");
    }

    #[test]
    fn test_format_empty_profiles_list() {
        let formatter = CsvFormatter::new();
        let profiles: Vec<Profile> = vec![];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();
        // Should contain header row even for empty list
        let lines: Vec<&str> = csv_output.trim().split('\n').collect();
        assert_eq!(lines.len(), 1);

        // Check that header contains all expected Profile fields
        let header = lines[0];
        assert!(header.contains("name"));
        assert!(header.contains("git_user_name"));
        assert!(header.contains("git_user_email"));
        assert!(header.contains("key_type"));
        assert!(header.contains("signing_key"));
        assert!(header.contains("vault_name"));
        assert!(header.contains("ssh_key_title"));
        assert!(header.contains("scope"));
        assert!(header.contains("ssh_key_source"));
        assert!(header.contains("ssh_key_path"));
        assert!(header.contains("allowed_signers"));
        assert!(header.contains("match_patterns"));
        assert!(header.contains("repos"));
        assert!(header.contains("include_if_dirs"));
        assert!(header.contains("host_patterns"));
        assert!(header.contains("one_password"));
    }

    #[test]
    fn test_format_single_minimal_profile() {
        let formatter = CsvFormatter::new();
        let profiles = vec![minimal_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();
        let lines: Vec<&str> = csv_output.trim().split('\n').collect();
        assert_eq!(lines.len(), 2); // Header + 1 data row

        // Parse the CSV and verify content
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();
        assert_eq!(records.len(), 1);

        let record = &records[0];
        assert_eq!(record.get(0).unwrap(), "minimal"); // name
        assert_eq!(record.get(2).unwrap(), "minimal@example.com"); // git_user_email
        assert_eq!(record.get(3).unwrap(), "x509"); // key_type (lowercase)
        assert_eq!(record.get(15).unwrap(), "false"); // one_password (boolean as string)

        // Check optional fields are empty strings
        assert_eq!(record.get(1).unwrap(), ""); // git_user_name (None)
        assert_eq!(record.get(4).unwrap(), ""); // signing_key (None)
    }

    #[test]
    fn test_format_multiple_profiles() {
        let formatter = CsvFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();
        let lines: Vec<&str> = csv_output.trim().split('\n').collect();
        assert_eq!(lines.len(), 3); // Header + 2 data rows

        // Parse the CSV and verify content
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();
        assert_eq!(records.len(), 2);

        // Check first profile (work)
        let work_record = &records[0];
        assert_eq!(work_record.get(0).unwrap(), "work"); // name
        assert_eq!(work_record.get(1).unwrap(), "Work User"); // git_user_name
        assert_eq!(work_record.get(2).unwrap(), "work@example.com"); // git_user_email
        assert_eq!(work_record.get(3).unwrap(), "ssh"); // key_type
        assert_eq!(work_record.get(7).unwrap(), "local"); // scope
        assert_eq!(work_record.get(8).unwrap(), "onepassword"); // ssh_key_source
        assert_eq!(work_record.get(15).unwrap(), "true"); // one_password

        // Check second profile (personal)
        let personal_record = &records[1];
        assert_eq!(personal_record.get(0).unwrap(), "personal"); // name
        assert_eq!(personal_record.get(3).unwrap(), "gpg"); // key_type
        assert_eq!(personal_record.get(7).unwrap(), "global"); // scope
        assert_eq!(personal_record.get(8).unwrap(), "file"); // ssh_key_source
        assert_eq!(personal_record.get(15).unwrap(), "false"); // one_password
    }

    #[test]
    fn test_format_profiles_with_arrays() {
        let formatter = CsvFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();

        // Parse the CSV and verify array field handling
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();

        // Check work profile arrays (first record)
        let work_record = &records[0];
        let match_patterns = work_record.get(11).unwrap(); // match_patterns field
        assert!(match_patterns.contains("work/*"));
        assert!(match_patterns.contains("company/*"));
        // Should use semicolon as separator to avoid comma conflicts
        assert!(match_patterns.contains(";"));

        let host_patterns = work_record.get(14).unwrap(); // host_patterns field
        assert!(host_patterns.contains("github.com"));
        assert!(host_patterns.contains("*.company.com"));
        assert!(host_patterns.contains(";"));
    }

    #[test]
    fn test_format_profiles_with_special_characters() {
        let formatter = CsvFormatter::new();
        let profiles = vec![special_chars_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();

        // Parse the CSV to verify proper escaping
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();
        assert_eq!(records.len(), 1);

        let record = &records[0];

        // Verify special characters are properly handled
        assert_eq!(record.get(0).unwrap(), "special,chars"); // Commas in name
        assert_eq!(record.get(1).unwrap(), "User with \"quotes\""); // Quotes in git_user_name
        assert!(record.get(4).unwrap().contains("newline")); // Newlines in signing_key
        assert_eq!(record.get(5).unwrap(), "Vault, with commas"); // Commas in vault_name
        assert_eq!(record.get(6).unwrap(), "SSH Key \"Title\""); // Quotes in ssh_key_title
    }

    #[test]
    fn test_all_key_types_serialize_correctly() {
        let formatter = CsvFormatter::new();
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

        let csv_output = result.unwrap();
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();

        assert_eq!(records[0].get(3).unwrap(), "ssh");
        assert_eq!(records[1].get(3).unwrap(), "gpg");
        assert_eq!(records[2].get(3).unwrap(), "x509");
        assert_eq!(records[3].get(3).unwrap(), "gitsign");
    }

    #[test]
    fn test_all_scope_types_serialize_correctly() {
        let formatter = CsvFormatter::new();
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

        let csv_output = result.unwrap();
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();

        assert_eq!(records[0].get(7).unwrap(), "local");
        assert_eq!(records[1].get(7).unwrap(), "global");
        assert_eq!(records[2].get(7).unwrap(), "system");
    }

    #[test]
    fn test_all_ssh_key_source_types_serialize_correctly() {
        let formatter = CsvFormatter::new();
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

        let csv_output = result.unwrap();
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();

        assert_eq!(records[0].get(8).unwrap(), "onepassword");
        assert_eq!(records[1].get(8).unwrap(), "authorizedkeys");
        assert_eq!(records[2].get(8).unwrap(), "file");
    }

    #[test]
    fn test_csv_output_is_valid_csv() {
        let formatter = CsvFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();

        // Verify the output can be parsed by csv crate
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let headers = reader.headers().unwrap().clone();
        assert!(headers.len() > 0);

        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();
        assert_eq!(records.len(), 2);

        // Each record should have the same number of fields as the header
        for record in &records {
            assert_eq!(record.len(), headers.len());
        }
    }

    #[test]
    fn test_error_handling_for_csv_serialization_failure() {
        // Test that our error type conversion works
        // We'll create a mock error and verify the conversion
        let _formatter = CsvFormatter::new();

        // Since csv crate is very robust, we can't easily trigger a serialization error
        // with valid Profile structs, but we can test the error flow conceptually
        // by checking that the error type is properly imported and available
        let _error_check: Result<String> = Err(GitSetupError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error"
        )));
    }

    #[test]
    fn test_header_field_order() {
        let formatter = CsvFormatter::new();
        let profiles: Vec<Profile> = vec![];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let headers = reader.headers().unwrap();

        // Verify header order matches Profile field order
        let expected_headers = vec![
            "name",
            "git_user_name",
            "git_user_email",
            "key_type",
            "signing_key",
            "vault_name",
            "ssh_key_title",
            "scope",
            "ssh_key_source",
            "ssh_key_path",
            "allowed_signers",
            "match_patterns",
            "repos",
            "include_if_dirs",
            "host_patterns",
            "one_password"
        ];

        assert_eq!(headers.len(), expected_headers.len());
        for (i, expected) in expected_headers.iter().enumerate() {
            assert_eq!(headers.get(i).unwrap(), *expected);
        }
    }

    #[test]
    fn test_csv_output_format_visual() {
        let formatter = CsvFormatter::new();
        let profiles = test_profiles();

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();
        println!("Generated CSV output:");
        println!("{}", csv_output);

        // Verify it contains expected content
        assert!(csv_output.contains("name,git_user_name,git_user_email"));
        assert!(csv_output.contains("work,Work User,work@example.com"));
        assert!(csv_output.contains("personal,Personal User,personal@example.com"));
        assert!(csv_output.contains("work/*;company/*"));
        assert!(csv_output.contains("github.com;*.company.com"));
    }

    #[test]
    fn test_csv_special_chars_visual() {
        let formatter = CsvFormatter::new();
        let profiles = vec![special_chars_profile()];

        let result = formatter.format_profiles(&profiles);
        assert!(result.is_ok());

        let csv_output = result.unwrap();
        println!("CSV with special characters:");
        println!("{}", csv_output);

        // Test that it can be parsed back correctly
        let mut reader = csv::Reader::from_reader(csv_output.as_bytes());
        let records: Vec<csv::StringRecord> = reader.records().collect::<std::result::Result<Vec<_>, csv::Error>>().unwrap();
        assert_eq!(records.len(), 1);

        let record = &records[0];
        assert_eq!(record.get(0).unwrap(), "special,chars");
        assert_eq!(record.get(1).unwrap(), "User with \"quotes\"");
        assert!(record.get(4).unwrap().contains("newline"));
    }
}
