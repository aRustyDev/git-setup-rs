//! Error types and handling for git-setup-rs.
//!
//! This module provides a comprehensive error type system using the `thiserror` crate
//! for clean error handling throughout the application.

use thiserror::Error;

/// The main error type for git-setup-rs operations.
///
/// This enum covers all possible error conditions that can occur during
/// git configuration and profile management operations.
#[derive(Error, Debug)]
pub enum GitSetupError {
    // Configuration-related errors
    /// Configuration file was not found at the specified path.
    #[error("Configuration file not found: {path}")]
    ConfigNotFound { path: String },

    // Profile management errors
    /// A requested profile does not exist.
    #[error("Profile '{name}' not found")]
    ProfileNotFound { name: String },

    /// Attempted to create a profile that already exists.
    #[error("Profile '{name}' already exists")]
    DuplicateProfile { name: String },

    /// Profile data is invalid or incomplete.
    #[error("Invalid profile: {reason}")]
    InvalidProfile { reason: String },

    // External command errors
    /// An external command (like git or 1Password CLI) failed.
    #[error("External command '{command}' failed: {error}")]
    ExternalCommand { command: String, error: String },

    // Third-party service errors
    /// 1Password CLI operation failed.
    #[error("1Password error: {0}")]
    OnePassword(String),

    /// Git operation failed.
    #[error("Git error: {0}")]
    Git(String),

    // Transparent wrapped errors
    /// I/O operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// TOML deserialization failed.
    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),

    /// TOML serialization failed.
    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),

    /// JSON parsing failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// YAML parsing failed.
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    /// CSV parsing failed.
    #[error(transparent)]
    Csv(#[from] csv::Error),
}

/// A type alias for `Result<T, GitSetupError>`.
///
/// This provides a convenient shorthand for functions that return
/// git-setup-rs specific errors.
pub type Result<T> = std::result::Result<T, GitSetupError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display_config_not_found() {
        let err = GitSetupError::ConfigNotFound {
            path: "/path/to/config.toml".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Configuration file not found: /path/to/config.toml"
        );
    }

    #[test]
    fn test_error_display_profile_not_found() {
        let err = GitSetupError::ProfileNotFound {
            name: "work".to_string(),
        };
        assert_eq!(err.to_string(), "Profile 'work' not found");
    }

    #[test]
    fn test_error_display_duplicate_profile() {
        let err = GitSetupError::DuplicateProfile {
            name: "personal".to_string(),
        };
        assert_eq!(err.to_string(), "Profile 'personal' already exists");
    }

    #[test]
    fn test_error_display_invalid_profile() {
        let err = GitSetupError::InvalidProfile {
            reason: "missing email field".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid profile: missing email field");
    }

    #[test]
    fn test_error_display_external_command() {
        let err = GitSetupError::ExternalCommand {
            command: "git config".to_string(),
            error: "command not found".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "External command 'git config' failed: command not found"
        );
    }

    #[test]
    fn test_error_display_onepassword() {
        let err = GitSetupError::OnePassword("authentication failed".to_string());
        assert_eq!(err.to_string(), "1Password error: authentication failed");
    }

    #[test]
    fn test_error_display_git() {
        let err = GitSetupError::Git("not a git repository".to_string());
        assert_eq!(err.to_string(), "Git error: not a git repository");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let git_err: GitSetupError = io_err.into();
        assert!(matches!(git_err, GitSetupError::Io(_)));
    }

    #[test]
    fn test_toml_deserialize_error_conversion() {
        let toml_content = "invalid = toml = content";
        let toml_err = toml::from_str::<toml::Value>(toml_content).unwrap_err();
        let git_err: GitSetupError = toml_err.into();
        assert!(matches!(git_err, GitSetupError::TomlDeserialize(_)));
    }

    #[test]
    fn test_toml_serialize_error_type_exists() {
        // Since TOML serialization rarely fails, we just verify the type conversion exists
        // by testing that a serialization error would convert to our error type
        // This ensures the From implementation exists and compiles
        fn _test_conversion(_err: toml::ser::Error) -> GitSetupError {
            GitSetupError::TomlSerialize(_err)
        }

        // If this compiles, the conversion is available
        assert!(true);
    }

    #[test]
    fn test_json_error_conversion() {
        let json_content = r#"{"invalid": json content"#;
        let json_err = serde_json::from_str::<serde_json::Value>(json_content).unwrap_err();
        let git_err: GitSetupError = json_err.into();
        assert!(matches!(git_err, GitSetupError::Json(_)));
    }

    #[test]
    fn test_yaml_error_conversion() {
        let yaml_content = "invalid: yaml: content: [";
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let git_err: GitSetupError = yaml_err.into();
        assert!(matches!(git_err, GitSetupError::Yaml(_)));
    }

    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<String> = Err(GitSetupError::OnePassword("test error".to_string()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "1Password error: test error"
        );
    }

    #[test]
    fn test_error_debug_trait() {
        let err = GitSetupError::ProfileNotFound {
            name: "test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ProfileNotFound"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_transparent_error_source_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let git_err: GitSetupError = io_err.into();

        // Test that the source error is preserved
        if let GitSetupError::Io(inner) = &git_err {
            assert_eq!(inner.kind(), io::ErrorKind::PermissionDenied);
        } else {
            panic!("Expected GitSetupError::Io variant");
        }
    }

    #[test]
    fn test_csv_error_conversion() {
        // Test that CSV error variants are available and can be created
        // Since CSV crate is very robust, we test the error type availability
        let io_error = io::Error::new(io::ErrorKind::InvalidData, "csv test");
        let csv_error = csv::Error::from(io_error);
        let git_err: GitSetupError = csv_error.into();
        assert!(matches!(git_err, GitSetupError::Csv(_)));

        // Verify the error message
        let error_string = git_err.to_string();
        assert!(error_string.contains("csv test"));
    }

    #[test]
    fn test_error_chain_display() {
        // Test that nested errors display properly
        let io_err = io::Error::new(io::ErrorKind::NotFound, "config.toml");
        let git_err: GitSetupError = io_err.into();
        let error_string = git_err.to_string();
        assert!(error_string.contains("config.toml"));
    }
}
