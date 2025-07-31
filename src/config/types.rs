use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    pub defaults: Option<Profile>,
    pub profiles: Vec<Profile>,
    pub config_paths: ConfigPaths,
    pub ssh_defaults: Option<SshDefaults>,
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub name: String,
    pub git_user_name: Option<String>,
    pub git_user_email: String,
    pub key_type: KeyType,
    pub signing_key: Option<String>,
    pub vault_name: Option<String>,
    pub ssh_key_title: Option<String>,
    pub scope: Option<Scope>,
    pub ssh_key_source: Option<SshKeySource>,
    pub ssh_key_path: Option<String>,
    pub allowed_signers: Option<String>,
    #[serde(default)]
    pub match_patterns: Vec<String>,
    #[serde(default)]
    pub repos: Vec<String>,
    #[serde(default)]
    pub include_if_dirs: Vec<String>,
    #[serde(default)]
    pub host_patterns: Vec<String>,
    #[serde(default)]
    pub one_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    #[default]
    Ssh,
    Gpg,
    X509,
    Gitsign,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    #[default]
    Local,
    Global,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SshKeySource {
    #[default]
    OnePassword,
    AuthorizedKeys,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPaths {
    pub global: ConfigPath,
    pub default: ConfigPath,
    pub system: ConfigPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPath {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshDefaults {
    pub key_type: String,
    pub key_size: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_toml_roundtrip() {
        let profile = Profile {
            name: "test".to_string(),
            git_user_name: Some("Test User".to_string()),
            git_user_email: "test@example.com".to_string(),
            key_type: KeyType::Ssh,
            signing_key: Some("key123".to_string()),
            vault_name: Some("vault1".to_string()),
            ssh_key_title: Some("Test Key".to_string()),
            scope: Some(Scope::Local),
            ssh_key_source: Some(SshKeySource::OnePassword),
            ssh_key_path: Some("/path/to/key".to_string()),
            allowed_signers: Some("signers.txt".to_string()),
            match_patterns: vec!["pattern1".to_string(), "pattern2".to_string()],
            repos: vec!["repo1".to_string(), "repo2".to_string()],
            include_if_dirs: vec!["dir1".to_string()],
            host_patterns: vec!["*.example.com".to_string()],
            one_password: true,
        };

        let toml_str = toml::to_string(&profile).unwrap();
        let parsed: Profile = toml::from_str(&toml_str).unwrap();
        assert_eq!(profile.name, parsed.name);
        assert_eq!(profile.git_user_email, parsed.git_user_email);
        assert_eq!(profile.match_patterns, parsed.match_patterns);
        assert_eq!(profile.repos, parsed.repos);
        assert_eq!(profile.one_password, parsed.one_password);
    }

    #[test]
    fn test_profile_default_vecs() {
        let toml_str = r#"
            name = "test"
            git_user_email = "test@example.com"
            key_type = "ssh"
        "#;

        let profile: Profile = toml::from_str(toml_str).unwrap();
        assert_eq!(profile.name, "test");
        assert_eq!(profile.git_user_email, "test@example.com");
        assert!(profile.match_patterns.is_empty());
        assert!(profile.repos.is_empty());
        assert!(profile.include_if_dirs.is_empty());
        assert!(profile.host_patterns.is_empty());
        assert!(!profile.one_password); // Should default to false
    }

    #[test]
    fn test_keytype_deserialization() {
        let test_cases = vec![
            ("ssh", KeyType::Ssh),
            ("gpg", KeyType::Gpg),
            ("x509", KeyType::X509),
            ("gitsign", KeyType::Gitsign),
        ];

        for (input, expected) in test_cases {
            let toml_str = format!(
                r#"
                name = "test"
                git_user_email = "test@example.com"
                key_type = "{}"
            "#,
                input
            );
            let profile: Profile = toml::from_str(&toml_str).unwrap();
            assert!(matches!(
                (profile.key_type, &expected),
                (KeyType::Ssh, KeyType::Ssh)
                    | (KeyType::Gpg, KeyType::Gpg)
                    | (KeyType::X509, KeyType::X509)
                    | (KeyType::Gitsign, KeyType::Gitsign)
            ));
        }
    }

    #[test]
    fn test_scope_deserialization() {
        let test_cases = vec![
            ("local", Scope::Local),
            ("global", Scope::Global),
            ("system", Scope::System),
        ];

        for (input, expected) in test_cases {
            let toml_str = format!(
                r#"
                name = "test"
                git_user_email = "test@example.com"
                key_type = "ssh"
                scope = "{}"
            "#,
                input
            );
            let profile: Profile = toml::from_str(&toml_str).unwrap();
            assert!(profile.scope.is_some());
            let scope = profile.scope.unwrap();
            assert!(matches!(
                (scope, &expected),
                (Scope::Local, Scope::Local)
                    | (Scope::Global, Scope::Global)
                    | (Scope::System, Scope::System)
            ));
        }
    }

    #[test]
    fn test_ssh_key_source_deserialization() {
        let test_cases = vec![
            ("onepassword", SshKeySource::OnePassword),
            ("authorizedkeys", SshKeySource::AuthorizedKeys),
            ("file", SshKeySource::File),
        ];

        for (input, expected) in test_cases {
            let toml_str = format!(
                r#"
                name = "test"
                git_user_email = "test@example.com"
                key_type = "ssh"
                ssh_key_source = "{}"
            "#,
                input
            );
            let profile: Profile = toml::from_str(&toml_str).unwrap();
            assert!(profile.ssh_key_source.is_some());
            let ssh_key_source = profile.ssh_key_source.unwrap();
            assert!(matches!(
                (ssh_key_source, &expected),
                (SshKeySource::OnePassword, SshKeySource::OnePassword)
                    | (SshKeySource::AuthorizedKeys, SshKeySource::AuthorizedKeys)
                    | (SshKeySource::File, SshKeySource::File)
            ));
        }
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let config = Config {
            version: 1,
            defaults: Some(Profile {
                name: "default".to_string(),
                git_user_name: None,
                git_user_email: "default@example.com".to_string(),
                key_type: KeyType::Ssh,
                signing_key: None,
                vault_name: None,
                ssh_key_title: None,
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
            profiles: vec![],
            config_paths: ConfigPaths {
                global: ConfigPath {
                    path: "/global/config".to_string(),
                },
                default: ConfigPath {
                    path: "/default/config".to_string(),
                },
                system: ConfigPath {
                    path: "/system/config".to_string(),
                },
            },
            ssh_defaults: Some(SshDefaults {
                key_type: "ed25519".to_string(),
                key_size: Some(256),
            }),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert!(parsed.defaults.is_some());
        assert_eq!(parsed.config_paths.global.path, "/global/config");
        assert!(parsed.ssh_defaults.is_some());
    }

    #[test]
    fn test_invalid_enum_handling() {
        let invalid_key_type = r#"
            name = "test"
            git_user_email = "test@example.com"
            key_type = "invalid"
        "#;
        let result: Result<Profile, _> = toml::from_str(invalid_key_type);
        assert!(result.is_err());

        let invalid_scope = r#"
            name = "test"
            git_user_email = "test@example.com"
            key_type = "ssh"
            scope = "invalid"
        "#;
        let result: Result<Profile, _> = toml::from_str(invalid_scope);
        assert!(result.is_err());

        let invalid_ssh_source = r#"
            name = "test"
            git_user_email = "test@example.com"
            key_type = "ssh"
            ssh_key_source = "invalid"
        "#;
        let result: Result<Profile, _> = toml::from_str(invalid_ssh_source);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_validation_minimal() {
        let toml_str = r#"
            name = "minimal"
            git_user_email = "minimal@example.com"
            key_type = "ssh"
        "#;

        let profile: Profile = toml::from_str(toml_str).unwrap();
        assert_eq!(profile.name, "minimal");
        assert_eq!(profile.git_user_email, "minimal@example.com");
        assert!(matches!(profile.key_type, KeyType::Ssh));
        assert!(profile.git_user_name.is_none());
        assert!(profile.signing_key.is_none());
    }

    #[test]
    fn test_config_paths_structure() {
        let toml_str = r#"
            [global]
            path = "/global"

            [default]
            path = "/default"

            [system]
            path = "/system"
        "#;

        let config_paths: ConfigPaths = toml::from_str(toml_str).unwrap();
        assert_eq!(config_paths.global.path, "/global");
        assert_eq!(config_paths.default.path, "/default");
        assert_eq!(config_paths.system.path, "/system");
    }
}
