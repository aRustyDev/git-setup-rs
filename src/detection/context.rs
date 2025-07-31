//! Repository context extraction for auto-detection.
//!
//! This module provides functionality to extract relevant information from
//! the current repository and environment for use in profile detection.

use crate::{
    error::Result,
    external::git::{GitConfigScope, GitWrapper},
    platform::{PlatformPaths, SystemPlatform},
};
use std::env;
use std::path::{Path, PathBuf};

/// Repository context information
#[derive(Debug, Clone)]
pub struct RepositoryContext {
    /// Current working directory
    pub working_dir: PathBuf,

    /// Git repository root (if in a git repo)
    pub repo_root: Option<PathBuf>,

    /// Remote URLs (origin, upstream, etc.)
    pub remotes: Vec<RemoteInfo>,

    /// Current git user.email (if set)
    pub current_email: Option<String>,

    /// Current git user.name (if set)
    pub current_name: Option<String>,

    /// Hostname of current machine
    pub hostname: String,

    /// Parent directories up to home
    pub parent_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    pub push_url: Option<String>,
}

pub struct ContextExtractor<G: GitWrapper> {
    git: G,
    platform: Box<dyn PlatformPaths>,
}

impl<G: GitWrapper> ContextExtractor<G> {
    pub fn new(git: G) -> Self {
        Self {
            git,
            platform: Box::new(SystemPlatform),
        }
    }

    pub fn with_platform(git: G, platform: Box<dyn PlatformPaths>) -> Self {
        Self { git, platform }
    }

    pub fn extract(&self) -> Result<RepositoryContext> {
        let current_dir = env::current_dir()?;
        self.extract_in(&current_dir)
    }

    pub fn extract_in(&self, path: &Path) -> Result<RepositoryContext> {
        let working_dir = path.to_path_buf();

        // Find git repository root
        let repo_root = self.find_repo_root(&working_dir)?;

        // Extract git information if in a repository
        let (remotes, current_email, current_name) = if repo_root.is_some() {
            (
                self.get_remotes()?,
                self.git
                    .get_config("user.email", Some(GitConfigScope::Local))
                    .unwrap_or(None),
                self.git
                    .get_config("user.name", Some(GitConfigScope::Local))
                    .unwrap_or(None),
            )
        } else {
            (Vec::new(), None, None)
        };

        // Get hostname
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        // Build parent directory list
        let parent_dirs = self.build_parent_dirs(&working_dir)?;

        Ok(RepositoryContext {
            working_dir,
            repo_root,
            remotes,
            current_email,
            current_name,
            hostname,
            parent_dirs,
        })
    }

    fn find_repo_root(&self, start: &Path) -> Result<Option<PathBuf>> {
        let mut current = start;

        loop {
            if current.join(".git").exists() {
                return Ok(Some(current.to_path_buf()));
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => return Ok(None),
            }
        }
    }

    fn get_remotes(&self) -> Result<Vec<RemoteInfo>> {
        // First try to get all remote URLs using git config --list
        let config_output = self
            .git
            .get_all_config(Some(GitConfigScope::Local))
            .unwrap_or_default();

        let mut remotes = Vec::new();

        // Parse remote URLs from git config output
        for (key, url) in &config_output {
            if let Some(remote_name) = key
                .strip_prefix("remote.")
                .and_then(|s| s.strip_suffix(".url"))
            {
                let push_url = config_output
                    .get(&format!("remote.{}.pushurl", remote_name))
                    .cloned();

                remotes.push(RemoteInfo {
                    name: remote_name.to_string(),
                    url: url.clone(),
                    push_url,
                });
            }
        }

        Ok(remotes)
    }

    fn build_parent_dirs(&self, start: &Path) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        let mut current = start;
        let home = self.platform.home_dir()?;

        while current != home && current.parent().is_some() {
            dirs.push(current.to_path_buf());
            current = current.parent().unwrap();
        }

        Ok(dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external::git::MockGitWrapper;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // Mock platform for testing
    #[derive(Debug)]
    struct MockPlatformPaths {
        home_dir: PathBuf,
    }

    impl MockPlatformPaths {
        fn new(home_dir: PathBuf) -> Self {
            Self { home_dir }
        }
    }

    impl PlatformPaths for MockPlatformPaths {
        fn home_dir(&self) -> std::result::Result<PathBuf, std::io::Error> {
            Ok(self.home_dir.clone())
        }

        fn config_dir(&self) -> std::result::Result<PathBuf, std::io::Error> {
            Ok(self.home_dir.join(".config"))
        }

        fn default_ssh_program(&self) -> &'static str {
            "ssh"
        }

        fn default_gpg_program(&self) -> &'static str {
            "gpg"
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

    #[test]
    fn test_context_extraction_in_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();

        let mut config = HashMap::new();
        config.insert("user.email".to_string(), "test@example.com".to_string());
        config.insert("user.name".to_string(), "Test User".to_string());
        config.insert(
            "remote.origin.url".to_string(),
            "git@github.com:user/repo.git".to_string(),
        );

        let mock_git = MockGitWrapper::new().with_config(config);
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let context = extractor.extract_in(temp_dir.path()).unwrap();

        assert!(context.repo_root.is_some());
        assert_eq!(context.current_email, Some("test@example.com".to_string()));
        assert_eq!(context.current_name, Some("Test User".to_string()));
        assert_eq!(context.remotes.len(), 1);
        assert_eq!(context.remotes[0].url, "git@github.com:user/repo.git");
        assert_eq!(context.remotes[0].name, "origin");
    }

    #[test]
    fn test_context_extraction_no_repo() {
        let temp_dir = TempDir::new().unwrap();
        let mock_git = MockGitWrapper::new();
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let context = extractor.extract_in(temp_dir.path()).unwrap();

        assert!(context.repo_root.is_none());
        assert!(context.remotes.is_empty());
        assert!(context.current_email.is_none());
        assert!(context.current_name.is_none());
    }

    #[test]
    fn test_context_extraction_multiple_remotes() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();

        let mut config = HashMap::new();
        config.insert(
            "remote.origin.url".to_string(),
            "git@github.com:user/repo.git".to_string(),
        );
        config.insert(
            "remote.upstream.url".to_string(),
            "git@github.com:upstream/repo.git".to_string(),
        );
        config.insert(
            "remote.origin.pushurl".to_string(),
            "git@github.com:user/repo-fork.git".to_string(),
        );

        let mock_git = MockGitWrapper::new().with_config(config);
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let context = extractor.extract_in(temp_dir.path()).unwrap();

        assert_eq!(context.remotes.len(), 2);

        let origin = context
            .remotes
            .iter()
            .find(|r| r.name == "origin")
            .unwrap();
        assert_eq!(origin.url, "git@github.com:user/repo.git");
        assert_eq!(
            origin.push_url,
            Some("git@github.com:user/repo-fork.git".to_string())
        );

        let upstream = context
            .remotes
            .iter()
            .find(|r| r.name == "upstream")
            .unwrap();
        assert_eq!(upstream.url, "git@github.com:upstream/repo.git");
        assert!(upstream.push_url.is_none());
    }

    #[test]
    fn test_find_repo_root_nested() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path();
        let nested_dir = repo_root.join("src").join("deep").join("nested");
        std::fs::create_dir_all(&nested_dir).unwrap();

        let git_dir = repo_root.join(".git");
        std::fs::create_dir(&git_dir).unwrap();

        let mock_git = MockGitWrapper::new();
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let result = extractor.find_repo_root(&nested_dir).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap(), repo_root);
    }

    #[test]
    fn test_find_repo_root_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mock_git = MockGitWrapper::new();
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let result = extractor.find_repo_root(temp_dir.path()).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_build_parent_dirs() {
        let mock_git = MockGitWrapper::new();
        let home_dir = PathBuf::from("/home/test");
        let mock_platform = Box::new(MockPlatformPaths::new(home_dir.clone()));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let start_path = PathBuf::from("/home/test/projects/work/repo");

        let parent_dirs = extractor.build_parent_dirs(&start_path).unwrap();

        assert!(parent_dirs.contains(&PathBuf::from("/home/test/projects/work/repo")));
        assert!(parent_dirs.contains(&PathBuf::from("/home/test/projects/work")));
        assert!(parent_dirs.contains(&PathBuf::from("/home/test/projects")));
        // Should stop at home directory
        assert!(!parent_dirs.contains(&home_dir));
    }

    #[test]
    fn test_hostname_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let mock_git = MockGitWrapper::new();
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let context = extractor.extract_in(temp_dir.path()).unwrap();

        // Hostname should be extracted (might be "unknown" in test environments)
        assert!(!context.hostname.is_empty());
    }

    #[test]
    fn test_context_debug_display() {
        let context = RepositoryContext {
            working_dir: PathBuf::from("/test"),
            repo_root: None,
            remotes: vec![],
            current_email: None,
            current_name: None,
            hostname: "test-host".to_string(),
            parent_dirs: vec![],
        };

        let debug_str = format!("{:?}", context);
        assert!(debug_str.contains("RepositoryContext"));
        assert!(debug_str.contains("test-host"));
    }

    #[test]
    fn test_remote_info_debug_display() {
        let remote = RemoteInfo {
            name: "origin".to_string(),
            url: "git@github.com:user/repo.git".to_string(),
            push_url: Some("git@github.com:user/fork.git".to_string()),
        };

        let debug_str = format!("{:?}", remote);
        assert!(debug_str.contains("RemoteInfo"));
        assert!(debug_str.contains("origin"));
        assert!(debug_str.contains("git@github.com:user/repo.git"));
    }

    #[test]
    fn test_context_extractor_new() {
        let mock_git = MockGitWrapper::new();
        let _extractor = ContextExtractor::new(mock_git);
        // Should create successfully with default platform
    }

    #[test]
    fn test_context_extractor_with_platform() {
        let mock_git = MockGitWrapper::new();
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));
        let _extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        // Should create successfully with custom platform
    }

    #[test]
    fn test_extract_current_directory() {
        let mock_git = MockGitWrapper::new();
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);

        // This might succeed or fail depending on the test environment
        // The important thing is it doesn't panic
        let _result = extractor.extract();
    }

    #[test]
    fn test_git_config_failure_handling() {
        let mock_git = MockGitWrapper::new().with_failure();
        let mock_platform = Box::new(MockPlatformPaths::new(PathBuf::from("/home/test")));

        let extractor = ContextExtractor::with_platform(mock_git, mock_platform);
        let temp_dir = TempDir::new().unwrap();

        // Should still work even if git commands fail
        let context = extractor.extract_in(temp_dir.path()).unwrap();
        assert!(context.remotes.is_empty());
        assert!(context.current_email.is_none());
    }
}
