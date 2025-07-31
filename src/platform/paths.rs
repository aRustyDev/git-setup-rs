use std::path::PathBuf;

pub trait PlatformPaths: std::fmt::Debug + Send + Sync {
    fn home_dir(&self) -> Result<PathBuf, std::io::Error>;
    fn config_dir(&self) -> Result<PathBuf, std::io::Error>;
    fn default_ssh_program(&self) -> &'static str;
    fn default_gpg_program(&self) -> &'static str;
    fn expand_path(&self, path: &str) -> String;
}

#[derive(Debug)]
pub struct SystemPlatform;

impl PlatformPaths for SystemPlatform {
    fn home_dir(&self) -> Result<PathBuf, std::io::Error> {
        #[cfg(windows)]
        {
            std::env::var("USERPROFILE")
                .map(PathBuf::from)
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "USERPROFILE not found")
                })
        }

        #[cfg(not(windows))]
        {
            std::env::var("HOME")
                .map(PathBuf::from)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not found"))
        }
    }

    fn config_dir(&self) -> Result<PathBuf, std::io::Error> {
        let home = self.home_dir()?;

        #[cfg(windows)]
        {
            Ok(home.join("AppData").join("Roaming").join("git-setup"))
        }

        #[cfg(not(windows))]
        {
            let config_home = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".config"));
            Ok(config_home.join("git").join("setup"))
        }
    }

    fn default_ssh_program(&self) -> &'static str {
        #[cfg(windows)]
        {
            "C:\\Program Files\\1Password\\app\\8\\op-ssh-sign.exe"
        }

        #[cfg(not(windows))]
        {
            "/usr/local/bin/op-ssh-sign"
        }
    }

    fn default_gpg_program(&self) -> &'static str {
        #[cfg(windows)]
        {
            "gpg.exe"
        }

        #[cfg(not(windows))]
        {
            "gpg"
        }
    }

    fn expand_path(&self, path: &str) -> String {
        if path.starts_with('~') {
            if let Ok(home) = self.home_dir() {
                return path.replacen('~', &home.to_string_lossy(), 1);
            }
        }
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_dir_detection() {
        let platform = SystemPlatform;
        let home = platform.home_dir().unwrap();
        assert!(home.exists());
        assert!(home.is_dir());
    }

    #[test]
    fn test_config_dir_construction() {
        let platform = SystemPlatform;
        let config_dir = platform.config_dir().unwrap();
        let config_str = config_dir.to_string_lossy();
        assert!(config_str.contains("git"));

        // Should be under home directory or XDG_CONFIG_HOME
        let home = platform.home_dir().unwrap();
        let home_str = home.to_string_lossy();
        assert!(config_str.starts_with(&*home_str) || std::env::var("XDG_CONFIG_HOME").is_ok());
    }

    #[test]
    fn test_path_expansion() {
        let platform = SystemPlatform;
        let expanded = platform.expand_path("~/test");
        assert!(!expanded.starts_with('~'));
        assert!(expanded.contains("test"));

        // Test path without tilde should remain unchanged
        let unchanged = platform.expand_path("/absolute/path");
        assert_eq!(unchanged, "/absolute/path");
    }

    #[test]
    fn test_path_expansion_with_home() {
        let platform = SystemPlatform;
        let home = platform.home_dir().unwrap();
        let home_str = home.to_string_lossy();

        let expanded = platform.expand_path("~/documents");
        assert!(expanded.starts_with(&*home_str));
        assert!(expanded.ends_with("documents"));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_ssh_program() {
        let platform = SystemPlatform;
        let ssh_program = platform.default_ssh_program();
        assert!(ssh_program.ends_with(".exe"));
        assert!(ssh_program.contains("op-ssh-sign"));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_unix_ssh_program() {
        let platform = SystemPlatform;
        let ssh_program = platform.default_ssh_program();
        assert!(!ssh_program.ends_with(".exe"));
        assert!(ssh_program.contains("op-ssh-sign"));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_gpg_program() {
        let platform = SystemPlatform;
        let gpg_program = platform.default_gpg_program();
        assert!(gpg_program.ends_with(".exe"));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_unix_gpg_program() {
        let platform = SystemPlatform;
        let gpg_program = platform.default_gpg_program();
        assert!(!gpg_program.ends_with(".exe"));
        assert_eq!(gpg_program, "gpg");
    }

    #[test]
    fn test_home_dir_environment_variables() {
        let platform = SystemPlatform;
        let home = platform.home_dir().unwrap();

        #[cfg(windows)]
        {
            // On Windows, should use USERPROFILE
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                assert_eq!(home.to_string_lossy(), userprofile);
            }
        }

        #[cfg(not(windows))]
        {
            // On Unix-like systems, should use HOME
            if let Ok(home_env) = std::env::var("HOME") {
                assert_eq!(home.to_string_lossy(), home_env);
            }
        }
    }

    #[test]
    fn test_config_dir_structure() {
        let platform = SystemPlatform;
        let config_dir = platform.config_dir().unwrap();

        #[cfg(windows)]
        {
            let config_str = config_dir.to_string_lossy();
            assert!(config_str.contains("AppData"));
            assert!(config_str.contains("Roaming"));
            assert!(config_str.contains("git-setup"));
        }

        #[cfg(not(windows))]
        {
            let config_str = config_dir.to_string_lossy();
            assert!(config_str.contains("git"));
            assert!(config_str.contains("setup"));

            // Should either be ~/.config/git/setup or $XDG_CONFIG_HOME/git/setup
            assert!(config_str.contains(".config") || std::env::var("XDG_CONFIG_HOME").is_ok());
        }
    }
}
