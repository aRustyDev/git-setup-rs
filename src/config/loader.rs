//! Configuration loading and saving implementation.

use crate::{
    config::types::{Config, ConfigPaths, SshDefaults},
    error::{GitSetupError, Result},
    platform::{PlatformPaths, SystemPlatform},
};
use std::fs;
use std::path::{Path, PathBuf};

/// Trait for configuration loading and saving operations.
pub trait ConfigLoaderTrait: Send + Sync {
    /// Load configuration from default location.
    fn load(&self) -> Result<Config>;

    /// Load configuration from specific path.
    fn load_from(&self, path: &Path) -> Result<Config>;

    /// Save configuration to default location.
    fn save(&self, config: &Config) -> Result<()>;

    /// Save configuration to specific path.
    fn save_to(&self, config: &Config, path: &Path) -> Result<()>;

    /// Get the default config path.
    fn default_path(&self) -> Result<PathBuf>;

    /// Check if config exists at default location.
    fn exists(&self) -> bool;

    /// Create default configuration.
    fn create_default(&self) -> Config;

    /// Validate configuration.
    fn validate(&self, config: &Config) -> Result<()>;

    /// Migrate configuration if needed.
    fn migrate_if_needed(&self, config: &mut Config) -> Result<bool>;
}

/// Configuration loader implementation.
#[derive(Debug)]
pub struct ConfigLoader {
    config_path: PathBuf,
    platform: Box<dyn PlatformPaths>,
}

impl ConfigLoader {
    /// Create a new ConfigLoader with the specified path.
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            platform: Box::new(SystemPlatform),
        }
    }

    /// Create a ConfigLoader with custom platform implementation.
    pub fn with_platform(config_path: PathBuf, platform: Box<dyn PlatformPaths>) -> Self {
        Self { config_path, platform }
    }

    /// Create a ConfigLoader using platform default paths.
    pub fn from_platform_default() -> Result<Self> {
        let platform = SystemPlatform;
        let config_dir = platform.config_dir()?;
        let config_path = config_dir.join("config.toml");
        Ok(Self::new(config_path))
    }
}

impl ConfigLoaderTrait for ConfigLoader {
    fn load(&self) -> Result<Config> {
        if self.exists() {
            self.load_from(&self.config_path)
        } else {
            Ok(self.create_default())
        }
    }

    fn load_from(&self, path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        self.validate(&config)?;
        self.expand_paths(&mut config);
        self.migrate_if_needed(&mut config)?;
        Ok(config)
    }

    fn save(&self, config: &Config) -> Result<()> {
        self.save_to(config, &self.config_path)
    }

    fn save_to(&self, config: &Config, path: &Path) -> Result<()> {
        self.validate(config)?;

        let content = toml::to_string_pretty(config)?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write atomically using a temporary file
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    fn default_path(&self) -> Result<PathBuf> {
        Ok(self.config_path.clone())
    }

    fn exists(&self) -> bool {
        self.config_path.exists()
    }

    fn create_default(&self) -> Config {
        Config {
            version: 1,
            defaults: None,
            profiles: Vec::new(),
            config_paths: ConfigPaths {
                global: crate::config::types::ConfigPath {
                    path: self.platform.config_dir().ok()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "~/.config/git/setup".to_string()),
                },
                default: crate::config::types::ConfigPath {
                    path: self.platform.config_dir().ok()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "~/.config/git/setup".to_string()),
                },
                system: crate::config::types::ConfigPath {
                    path: "/etc/git/setup".to_string(),
                },
            },
            ssh_defaults: Some(SshDefaults {
                key_type: "ed25519".to_string(),
                key_size: Some(256),
            }),
        }
    }

    fn validate(&self, config: &Config) -> Result<()> {
        // Validate each profile
        for profile in &config.profiles {
            if profile.name.is_empty() {
                return Err(GitSetupError::InvalidProfile {
                    reason: "Profile name cannot be empty".to_string(),
                });
            }

            if !profile.git_user_email.contains('@') {
                return Err(GitSetupError::InvalidProfile {
                    reason: format!("Invalid email for profile '{}': {}",
                        profile.name, profile.git_user_email),
                });
            }
        }

        // Check for duplicate profile names
        let mut names = std::collections::HashSet::new();
        for profile in &config.profiles {
            if !names.insert(&profile.name) {
                return Err(GitSetupError::InvalidProfile {
                    reason: format!("Duplicate profile name: {}", profile.name),
                });
            }
        }

        Ok(())
    }

    fn migrate_if_needed(&self, config: &mut Config) -> Result<bool> {
        const CURRENT_VERSION: u32 = 1;

        if config.version >= CURRENT_VERSION {
            return Ok(false);
        }

        let original_version = config.version;

        // Migrate from v0 to v1 (example)
        if config.version == 0 {
            // No specific migrations needed yet, just bump version
            config.version = 1;
        }

        // Future migrations would go here:
        // if config.version == 1 {
        //     // migrate to v2
        //     config.version = 2;
        // }

        config.version = CURRENT_VERSION;
        Ok(original_version != config.version)
    }
}

impl ConfigLoader {
    /// Expand tilde and environment variables in configuration paths.
    fn expand_paths(&self, config: &mut Config) {
        // Expand paths in profiles
        for profile in &mut config.profiles {
            if let Some(key) = &profile.signing_key {
                profile.signing_key = Some(self.platform.expand_path(key));
            }
            if let Some(path) = &profile.ssh_key_path {
                profile.ssh_key_path = Some(self.platform.expand_path(path));
            }
            if let Some(signers) = &profile.allowed_signers {
                profile.allowed_signers = Some(self.platform.expand_path(signers));
            }
        }

        // Expand default profile paths if present
        if let Some(default) = &mut config.defaults {
            if let Some(key) = &default.signing_key {
                default.signing_key = Some(self.platform.expand_path(key));
            }
            if let Some(path) = &default.ssh_key_path {
                default.ssh_key_path = Some(self.platform.expand_path(path));
            }
            if let Some(signers) = &default.allowed_signers {
                default.allowed_signers = Some(self.platform.expand_path(signers));
            }
        }

        // Expand config paths
        config.config_paths.global.path = self.platform.expand_path(&config.config_paths.global.path);
        config.config_paths.default.path = self.platform.expand_path(&config.config_paths.default.path);
        config.config_paths.system.path = self.platform.expand_path(&config.config_paths.system.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Profile, KeyType, Scope};
    use tempfile::TempDir;

    fn test_config() -> Config {
        Config {
            version: 1,
            defaults: Some(Profile {
                name: "default".to_string(),
                git_user_name: Some("Default User".to_string()),
                git_user_email: "default@example.com".to_string(),
                key_type: KeyType::Ssh,
                signing_key: Some("~/.ssh/id_ed25519.pub".to_string()),
                vault_name: Some("Default Vault".to_string()),
                ssh_key_title: Some("Test SSH Key".to_string()),
                scope: Some(Scope::Global),
                ssh_key_source: None,
                ssh_key_path: None,
                allowed_signers: None,
                match_patterns: vec![],
                repos: vec![],
                include_if_dirs: vec![],
                host_patterns: vec![],
                one_password: false,
            }),
            profiles: vec![
                Profile {
                    name: "test".to_string(),
                    git_user_name: Some("Test User".to_string()),
                    git_user_email: "test@example.com".to_string(),
                    key_type: KeyType::Ssh,
                    signing_key: Some("~/.ssh/id_ed25519.pub".to_string()),
                    vault_name: Some("Test Vault".to_string()),
                    ssh_key_title: Some("Test SSH Key".to_string()),
                    scope: Some(Scope::Local),
                    ssh_key_source: None,
                    ssh_key_path: None,
                    allowed_signers: None,
                    match_patterns: vec!["work/*".to_string()],
                    repos: vec!["git@github.com:test/repo.git".to_string()],
                    include_if_dirs: vec![],
                    host_patterns: vec![],
                    one_password: true,
                }
            ],
            config_paths: ConfigPaths {
                global: crate::config::types::ConfigPath { path: "/global/config".to_string() },
                default: crate::config::types::ConfigPath { path: "/default/config".to_string() },
                system: crate::config::types::ConfigPath { path: "/system/config".to_string() },
            },
            ssh_defaults: Some(SshDefaults {
                key_type: "ed25519".to_string(),
                key_size: Some(256),
            }),
        }
    }

    #[test]
    fn test_config_toml_serialization() {
        let config = test_config();

        // Test serialization
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("name = \"test\""));
        assert!(toml_str.contains("git_user_email = \"test@example.com\""));
        assert!(toml_str.contains("key_type = \"ssh\""));

        // Test deserialization
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.profiles.len(), 1);
        assert_eq!(parsed.profiles[0].name, "test");
        assert_eq!(parsed.profiles[0].git_user_email, "test@example.com");
        assert!(parsed.defaults.is_some());
        assert!(parsed.ssh_defaults.is_some());
    }

    #[test]
    fn test_config_roundtrip_preserves_data() {
        let original = test_config();

        let toml_str = toml::to_string_pretty(&original).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        // Verify key fields are preserved
        assert_eq!(parsed.profiles.len(), original.profiles.len());
        if let (Some(orig_default), Some(parsed_default)) = (&original.defaults, &parsed.defaults) {
            assert_eq!(orig_default.name, parsed_default.name);
            assert_eq!(orig_default.git_user_email, parsed_default.git_user_email);
        }

        // Verify profile data
        let orig_profile = &original.profiles[0];
        let parsed_profile = &parsed.profiles[0];
        assert_eq!(orig_profile.name, parsed_profile.name);
        assert_eq!(orig_profile.match_patterns, parsed_profile.match_patterns);
        assert_eq!(orig_profile.repos, parsed_profile.repos);
        assert_eq!(orig_profile.one_password, parsed_profile.one_password);
    }

    #[test]
    fn test_config_validation_minimal() {
        let minimal_toml = r#"
            profiles = []

            [config_paths]
            [config_paths.global]
            path = "/global"
            [config_paths.default]
            path = "/default"
            [config_paths.system]
            path = "/system"
        "#;

        let config: Config = toml::from_str(minimal_toml).unwrap();
        assert!(config.profiles.is_empty());
        assert!(config.defaults.is_none());
        assert_eq!(config.config_paths.global.path, "/global");
    }

    #[test]
    fn test_new_config_loader() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let loader = ConfigLoader::new(config_path.clone());

        assert_eq!(loader.config_path, config_path);
        assert!(!loader.exists());
    }

    #[test]
    fn test_from_platform_default() {
        let loader = ConfigLoader::from_platform_default().unwrap();
        let default_path = loader.default_path().unwrap();

        // Should end with config.toml
        assert!(default_path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_load_nonexistent_creates_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let loader = ConfigLoader::new(config_path);

        assert!(!loader.exists());
        let config = loader.load().unwrap();
        assert!(config.profiles.is_empty());
        assert!(config.defaults.is_none());
        assert!(config.ssh_defaults.is_some());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let loader = ConfigLoader::new(config_path);
        let config = test_config();

        // Save
        loader.save(&config).unwrap();
        assert!(loader.exists());

        // Load
        let loaded = loader.load().unwrap();
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].name, "test");
        assert_eq!(loaded.profiles[0].git_user_email, "test@example.com");
    }

    #[test]
    fn test_load_corrupted_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write invalid TOML
        std::fs::write(&config_path, "invalid toml content [incomplete").unwrap();

        let loader = ConfigLoader::new(config_path);
        let result = loader.load();
        assert!(result.is_err());

        // Should be a TOML deserialization error
        match result.unwrap_err() {
            GitSetupError::TomlDeserialize(_) => {}, // Expected
            other => panic!("Expected TomlDeserialize error, got: {:?}", other),
        }
    }

    #[test]
    fn test_load_missing_file_creates_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let loader = ConfigLoader::new(config_path);

        let config = loader.load().unwrap();

        // Should create default config
        assert!(config.profiles.is_empty());
        assert!(config.defaults.is_none());
        assert!(config.ssh_defaults.is_some());
    }

    #[test]
    fn test_atomic_save() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let loader = ConfigLoader::new(config_path.clone());

        // Save initial config
        let config1 = test_config();
        loader.save(&config1).unwrap();

        // Start reading in another "thread"
        let content1 = std::fs::read_to_string(&config_path).unwrap();

        // Save updated config
        let mut config2 = test_config();
        config2.profiles[0].git_user_email = "updated@example.com".to_string();
        loader.save(&config2).unwrap();

        // Original read should still be valid TOML
        let parsed: Config = toml::from_str(&content1).unwrap();
        assert_eq!(parsed.profiles[0].git_user_email, "test@example.com");

        // New read should have updated content
        let content2 = std::fs::read_to_string(&config_path).unwrap();
        let parsed2: Config = toml::from_str(&content2).unwrap();
        assert_eq!(parsed2.profiles[0].git_user_email, "updated@example.com");
    }

    #[test]
    fn test_validate_invalid_config() {
        let loader = ConfigLoader::new(PathBuf::from("test.toml"));

        // Test empty profile name
        let mut config = test_config();
        config.profiles[0].name = "".to_string();
        let result = loader.validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));

        // Test invalid email
        config = test_config();
        config.profiles[0].git_user_email = "invalid-email".to_string();
        let result = loader.validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid email"));

        // Test duplicate names
        config = test_config();
        config.profiles.push(config.profiles[0].clone());
        let result = loader.validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("config.toml");
        let loader = ConfigLoader::new(nested_path.clone());
        let config = test_config();

        // Directory doesn't exist yet
        assert!(!nested_path.parent().unwrap().exists());

        // Save should create the directory
        loader.save(&config).unwrap();
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_save_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        std::fs::create_dir(&readonly_dir).unwrap();

        // Make directory read-only
        let mut perms = std::fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444);
        std::fs::set_permissions(&readonly_dir, perms).unwrap();

        let loader = ConfigLoader::new(readonly_dir.join("config.toml"));
        let config = test_config();
        let result = loader.save(&config);

        // Should fail with IO error
        assert!(result.is_err());
        match result.unwrap_err() {
            GitSetupError::Io(_) => {}, // Expected
            other => panic!("Expected IO error, got: {:?}", other),
        }
    }

    #[test]
    fn test_real_world_config_roundtrip() {
        let config_content = r#"
[[profiles]]
name = "work"
git_user_name = "John Doe"
git_user_email = "john@company.com"
key_type = "ssh"
signing_key = "~/.ssh/work_ed25519.pub"
vault_name = "Work"
ssh_key_title = "Work Laptop"
scope = "local"
match_patterns = ["work/*", "company/*"]
repos = ["git@github.com:company/app.git"]
one_password = true

[[profiles]]
name = "personal"
git_user_email = "john@personal.com"
key_type = "gpg"
signing_key = "0x1234567890ABCDEF"
scope = "global"
one_password = false

[config_paths.global]
path = "/global/config"

[config_paths.default]
path = "/default/config"

[config_paths.system]
path = "/system/config"

[ssh_defaults]
key_type = "ed25519"
key_size = 256
"#;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let loader = ConfigLoader::new(config_path);
        let config = loader.load().unwrap();

        assert_eq!(config.profiles.len(), 2);
        assert_eq!(config.profiles[0].name, "work");
        assert_eq!(config.profiles[1].name, "personal");
        assert!(config.ssh_defaults.is_some());

        // Test that we can save it back
        loader.save(&config).unwrap();

        // Load again and verify
        let reloaded = loader.load().unwrap();
        assert_eq!(reloaded.profiles.len(), 2);
    }

    #[test]
    fn test_path_expansion() {
        let config_content = r#"
[[profiles]]
name = "test"
git_user_email = "test@example.com"
key_type = "ssh"
signing_key = "~/test_key.pub"
ssh_key_path = "~/test_key"
allowed_signers = "~/.ssh/allowed_signers"

[config_paths.global]
path = "~/global/config"

[config_paths.default]
path = "~/default/config"

[config_paths.system]
path = "/system/config"
"#;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let loader = ConfigLoader::new(config_path);
        let config = loader.load().unwrap();

        // Paths should be expanded (~ replaced with home directory)
        let profile = &config.profiles[0];
        assert!(profile.signing_key.as_ref().unwrap().starts_with('/'));
        assert!(profile.ssh_key_path.as_ref().unwrap().starts_with('/'));
        assert!(profile.allowed_signers.as_ref().unwrap().starts_with('/'));

        // Config paths should also be expanded
        assert!(config.config_paths.global.path.starts_with('/'));
        assert!(config.config_paths.default.path.starts_with('/'));

        // System path should remain unchanged (no ~)
        assert_eq!(config.config_paths.system.path, "/system/config");
    }

    #[test]
    fn test_migration() {
        let config_content = r#"
[[profiles]]
name = "test"
git_user_email = "test@example.com"
key_type = "ssh"

[config_paths.global]
path = "/global"

[config_paths.default]
path = "/default"

[config_paths.system]
path = "/system"
"#;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content).unwrap();

        let loader = ConfigLoader::new(config_path);
        let config = loader.load().unwrap();

        // Should have version 1 after migration (defaulted from missing version)
        assert_eq!(config.version, 1);
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "test");
    }

    #[test]
    fn test_migration_no_change_needed() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let loader = ConfigLoader::new(config_path);

        let mut config = test_config();
        config.version = 1; // Already current version

        let migrated = loader.migrate_if_needed(&mut config).unwrap();
        assert!(!migrated); // No migration should occur
        assert_eq!(config.version, 1);
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock implementation of ConfigLoaderTrait for testing.
    #[derive(Debug)]
    pub struct MockConfigLoader {
        pub config: Arc<Mutex<Config>>,
        pub should_fail_load: bool,
        pub should_fail_save: bool,
        pub exists_returns: bool,
    }

    impl MockConfigLoader {
        pub fn new() -> Self {
            Self {
                config: Arc::new(Mutex::new(Config {
                    version: 1,
                    defaults: None,
                    profiles: Vec::new(),
                    config_paths: ConfigPaths {
                        global: crate::config::types::ConfigPath {
                            path: "/mock/global".to_string(),
                        },
                        default: crate::config::types::ConfigPath {
                            path: "/mock/default".to_string(),
                        },
                        system: crate::config::types::ConfigPath {
                            path: "/mock/system".to_string(),
                        },
                    },
                    ssh_defaults: Some(SshDefaults {
                        key_type: "ed25519".to_string(),
                        key_size: Some(256),
                    }),
                })),
                should_fail_load: false,
                should_fail_save: false,
                exists_returns: true,
            }
        }

        pub fn with_config(config: Config) -> Self {
            Self {
                config: Arc::new(Mutex::new(config)),
                should_fail_load: false,
                should_fail_save: false,
                exists_returns: true,
            }
        }
    }

    impl ConfigLoaderTrait for MockConfigLoader {
        fn load(&self) -> Result<Config> {
            if self.should_fail_load {
                return Err(GitSetupError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Mock load failure",
                )));
            }
            Ok(self.config.lock().unwrap().clone())
        }

        fn load_from(&self, _path: &Path) -> Result<Config> {
            self.load()
        }

        fn save(&self, config: &Config) -> Result<()> {
            if self.should_fail_save {
                return Err(GitSetupError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Mock save failure",
                )));
            }
            *self.config.lock().unwrap() = config.clone();
            Ok(())
        }

        fn save_to(&self, config: &Config, _path: &Path) -> Result<()> {
            self.save(config)
        }

        fn default_path(&self) -> Result<PathBuf> {
            Ok(PathBuf::from("/mock/config.toml"))
        }

        fn exists(&self) -> bool {
            self.exists_returns
        }

        fn create_default(&self) -> Config {
            Config {
                version: 1,
                defaults: None,
                profiles: Vec::new(),
                config_paths: ConfigPaths {
                    global: crate::config::types::ConfigPath {
                        path: "/mock/global".to_string(),
                    },
                    default: crate::config::types::ConfigPath {
                        path: "/mock/default".to_string(),
                    },
                    system: crate::config::types::ConfigPath {
                        path: "/mock/system".to_string(),
                    },
                },
                ssh_defaults: Some(SshDefaults {
                    key_type: "ed25519".to_string(),
                    key_size: Some(256),
                }),
            }
        }

        fn validate(&self, _config: &Config) -> Result<()> {
            Ok(())
        }

        fn migrate_if_needed(&self, _config: &mut Config) -> Result<bool> {
            Ok(false)
        }
    }

    #[test]
    fn test_mock_config_loader() {
        let mock = MockConfigLoader::new();

        // Test default behavior
        assert!(mock.exists());
        let config = mock.load().unwrap();
        assert_eq!(config.version, 1);
        assert!(config.profiles.is_empty());

        // Test save
        let mut new_config = config;
        new_config.version = 2;
        mock.save(&new_config).unwrap();

        let loaded = mock.load().unwrap();
        assert_eq!(loaded.version, 2);
    }

    #[test]
    fn test_mock_config_loader_failures() {
        let mut mock = MockConfigLoader::new();

        // Test load failure
        mock.should_fail_load = true;
        assert!(mock.load().is_err());

        // Test save failure
        mock.should_fail_load = false;
        mock.should_fail_save = true;
        let config = mock.create_default();
        assert!(mock.save(&config).is_err());
    }
}
