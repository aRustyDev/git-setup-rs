# Comprehensive Integration Test Plan

## Overview

This document outlines the comprehensive integration testing strategy for git-setup-rs, covering all phases and ensuring the system works correctly as a whole.

## Test Categories

### 1. End-to-End User Workflows

#### Test Suite: Complete User Journey
```rust
#[tokio::test]
async fn test_complete_user_setup_flow() {
    // Test the entire flow from installation to daily use
    
    // 1. First run experience
    let app = GitSetup::new();
    assert!(app.is_first_run());
    
    // 2. Create first profile
    let profile = create_test_profile("work");
    app.profile_manager.create(profile).await.unwrap();
    
    // 3. Apply profile to repository
    let test_repo = create_test_repo();
    app.apply_profile("work", &test_repo).await.unwrap();
    
    // 4. Verify Git configuration
    let config = read_git_config(&test_repo);
    assert_eq!(config.get("user.email"), Some("work@example.com"));
    
    // 5. Test auto-detection
    add_remote(&test_repo, "origin", "https://github.com/work/project");
    let detected = app.auto_detect(&test_repo).await.unwrap();
    assert_eq!(detected, Some("work"));
}
```

### 2. Cross-Platform Integration Tests

#### Platform Test Matrix
| Platform | Shell | Terminal | 1Password | Git Version |
|----------|-------|----------|-----------|-------------|
| Windows 11 | PowerShell | Terminal | ✓ | 2.42+ |
| Windows 11 | WSL2 | Terminal | ✓ | 2.42+ |
| macOS 13+ | zsh | Terminal.app | ✓ | 2.42+ |
| macOS 13+ | bash | iTerm2 | ✓ | 2.42+ |
| Ubuntu 22.04 | bash | GNOME | ✓ | 2.34+ |
| Ubuntu 22.04 | zsh | Alacritty | ✓ | 2.34+ |

#### Platform-Specific Tests
```rust
#[cfg(windows)]
mod windows_tests {
    #[test]
    fn test_windows_path_handling() {
        // Test backslash handling
        let path = ProfilePath::new("C:\\Users\\Alice\\.config\\git-setup");
        assert!(path.is_valid());
        
        // Test UNC paths
        let unc = ProfilePath::new("\\\\server\\share\\git-setup");
        assert!(unc.is_valid());
    }
    
    #[test]
    fn test_windows_credential_manager() {
        // Test Windows Credential Manager integration
        let cred_helper = GitCredentialHelper::windows();
        assert!(cred_helper.is_available());
    }
}

#[cfg(target_os = "macos")]
mod macos_tests {
    #[test]
    fn test_keychain_integration() {
        // Test macOS Keychain integration
        let keychain = MacKeychain::new();
        assert!(keychain.is_available());
    }
    
    #[test]
    fn test_touch_id_prompt() {
        // Test Touch ID for 1Password
        if OnePasswordCli::supports_biometric() {
            assert!(test_biometric_auth().is_ok());
        }
    }
}
```

### 3. Security Integration Tests

#### Security Test Scenarios
```rust
mod security_tests {
    #[test]
    fn test_no_credential_leaks_in_logs() {
        // Set up logging capture
        let log_capture = TestLogCapture::new();
        
        // Perform operations with sensitive data
        let sensitive = SensitiveString::new("super-secret-key");
        let profile = Profile {
            signing: Some(SigningConfig {
                ssh_key: Some(sensitive),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        // Apply profile with logging
        apply_profile_with_logging(profile);
        
        // Verify no secrets in logs
        let logs = log_capture.get_all();
        assert!(!logs.contains("super-secret-key"));
        assert!(logs.contains("[REDACTED]"));
    }
    
    #[tokio::test]
    async fn test_remote_config_security() {
        // Start malicious server
        let server = MockServer::start().await;
        server.mock(|when, then| {
            when.method(GET).path("/evil.toml");
            then.status(200)
                .body(r#"
                [profile]
                name = "../../../etc/passwd"
                "#);
        });
        
        // Try to import malicious config
        let result = RemoteImporter::new()
            .import_profile(&server.url("/evil.toml"))
            .await;
        
        // Should reject path traversal
        assert!(matches!(result, Err(RemoteError::ValidationError(_))));
    }
}
```

### 4. Integration with External Tools

#### Git Integration Tests
```rust
mod git_integration_tests {
    use std::process::Command;
    
    #[test]
    fn test_git_commit_signing() {
        let repo = TempRepo::new();
        
        // Configure SSH signing
        configure_ssh_signing(&repo);
        
        // Create signed commit
        create_test_file(&repo, "test.txt", "content");
        let output = Command::new("git")
            .current_dir(&repo.path())
            .args(&["commit", "-S", "-m", "Signed commit"])
            .output()
            .unwrap();
        
        assert!(output.status.success());
        
        // Verify signature
        let verify = Command::new("git")
            .current_dir(&repo.path())
            .args(&["log", "--show-signature", "-1"])
            .output()
            .unwrap();
        
        let output_str = String::from_utf8_lossy(&verify.stdout);
        assert!(output_str.contains("Good signature"));
    }
}
```

#### 1Password Integration Tests
```rust
mod onepassword_integration_tests {
    #[tokio::test]
    async fn test_1password_credential_fetch() {
        // Skip if 1Password not available
        if !OnePasswordCli::is_available() {
            return;
        }
        
        // Create test vault item
        let item_id = create_test_1password_item().await;
        
        // Fetch via our integration
        let op_client = OnePasswordCli::new();
        let credential = op_client
            .get_credential(&format!("op://Test/{}/password", item_id))
            .await
            .unwrap();
        
        // Verify (without logging the value!)
        assert!(!credential.expose_secret().is_empty());
        
        // Cleanup
        cleanup_test_item(item_id).await;
    }
}
```

### 5. Performance Integration Tests

#### Performance Benchmarks
```rust
mod performance_tests {
    use criterion::{criterion_group, criterion_main, Criterion};
    
    fn bench_profile_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("profile_operations");
        
        // Benchmark profile switching
        group.bench_function("switch_profile", |b| {
            let app = setup_test_app();
            let repo = setup_test_repo();
            
            b.iter(|| {
                app.apply_profile("work", &repo).unwrap();
                app.apply_profile("personal", &repo).unwrap();
            });
        });
        
        // Benchmark auto-detection
        group.bench_function("auto_detect", |b| {
            let detector = setup_detector_with_100_patterns();
            let repo = setup_repo_with_remote();
            
            b.iter(|| {
                detector.detect(&repo).unwrap();
            });
        });
        
        group.finish();
    }
}
```

### 6. Error Recovery Integration Tests

#### Error Handling Scenarios
```rust
mod error_recovery_tests {
    #[tokio::test]
    async fn test_recovery_from_corrupted_profile() {
        // Corrupt a profile file
        let profile_path = get_profile_path("work");
        fs::write(&profile_path, "invalid toml {{{").unwrap();
        
        // Should handle gracefully
        let manager = ProfileManager::new();
        let result = manager.get("work").await;
        
        assert!(matches!(result, Err(ProfileError::ParseError(_))));
        
        // Should still list other profiles
        let profiles = manager.list().await.unwrap();
        assert!(profiles.contains(&"personal".to_string()));
    }
    
    #[test]
    fn test_git_not_installed() {
        // Temporarily rename git executable
        let _guard = temporarily_hide_git();
        
        // Run health check
        let health = HealthCheckRunner::new();
        let results = health.run_all().await;
        
        // Should report git missing
        let git_check = results.iter()
            .find(|r| r.name == "Git Installation")
            .unwrap();
        
        assert!(matches!(git_check.status, HealthStatus::Error(_)));
        assert!(git_check.fix_suggestion.is_some());
    }
}
```

### 7. UI Integration Tests

#### TUI Integration Tests
```rust
mod tui_integration_tests {
    use ratatui::backend::TestBackend;
    
    #[tokio::test]
    async fn test_tui_profile_management() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        
        // Simulate user interactions
        app.handle_key(KeyCode::Char('n')).await.unwrap(); // New profile
        app.handle_key(KeyCode::Char('w')).await.unwrap();
        app.handle_key(KeyCode::Char('o')).await.unwrap();
        app.handle_key(KeyCode::Char('r')).await.unwrap();
        app.handle_key(KeyCode::Char('k')).await.unwrap();
        app.handle_key(KeyCode::Enter).await.unwrap();
        
        // Verify profile created
        assert!(app.profiles.contains(&"work".to_string()));
        
        // Test rendering
        terminal.draw(|f| app.render(f, f.size())).unwrap();
        
        // Verify UI state
        assert_eq!(app.screen, Screen::ProfileView);
    }
}
```

### 8. Data Migration Tests

#### Upgrade Path Testing
```rust
mod migration_tests {
    #[test]
    fn test_profile_format_migration() {
        // Create old format profile
        let old_profile = r#"
        name = "work"
        email = "old@format.com"
        "#;
        
        write_old_format_profile("work", old_profile);
        
        // Run migration
        let migrator = ProfileMigrator::new();
        migrator.migrate_all().unwrap();
        
        // Verify new format
        let manager = ProfileManager::new();
        let profile = manager.get("work").await.unwrap();
        
        assert_eq!(profile.git.user_email, "old@format.com");
    }
}
```

### 9. Edge Case Integration Tests

#### Edge Case Scenarios
```rust
mod edge_case_tests {
    #[test]
    fn test_unicode_in_profiles() {
        let profile = Profile {
            name: "работа".to_string(), // Russian
            git: GitConfig {
                user_name: "北京用户".to_string(), // Chinese
                user_email: "user@例え.jp".to_string(), // Japanese domain
            },
            ..Default::default()
        };
        
        let manager = ProfileManager::new();
        manager.create(profile.clone()).await.unwrap();
        
        let loaded = manager.get("работа").await.unwrap();
        assert_eq!(loaded.git.user_name, "北京用户");
    }
    
    #[test]
    fn test_extremely_long_paths() {
        // Windows has 260 char path limit
        let long_name = "a".repeat(255);
        let result = Profile::new(&long_name);
        
        #[cfg(windows)]
        assert!(result.is_err());
        
        #[cfg(not(windows))]
        assert!(result.is_ok());
    }
}
```

### 10. Stress Testing

#### Load Testing
```rust
mod stress_tests {
    #[tokio::test]
    async fn test_many_profiles() {
        let manager = ProfileManager::new();
        
        // Create 1000 profiles
        for i in 0..1000 {
            let profile = Profile {
                name: format!("profile_{}", i),
                ..Default::default()
            };
            manager.create(profile).await.unwrap();
        }
        
        // List should still be fast
        let start = Instant::now();
        let profiles = manager.list().await.unwrap();
        let duration = start.elapsed();
        
        assert_eq!(profiles.len(), 1000);
        assert!(duration < Duration::from_secs(1));
    }
    
    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager = Arc::new(ProfileManager::new());
        let mut handles = vec![];
        
        // Spawn 100 concurrent operations
        for i in 0..100 {
            let m = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let profile = Profile {
                    name: format!("concurrent_{}", i),
                    ..Default::default()
                };
                m.create(profile).await
            });
            handles.push(handle);
        }
        
        // All should succeed
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }
    }
}
```

## Test Data Generation

### Generated Test Data
```rust
mod test_data_generation {
    use fake::{Fake, Faker};
    use proptest::prelude::*;
    
    prop_compose! {
        fn arb_profile()
            (name in "[a-z]{3,20}",
             email in r"[a-z]+@[a-z]+\.(com|org|net)",
             signing in prop::option::of(arb_signing_config()))
            -> Profile {
            Profile {
                name,
                git: GitConfig {
                    user_email: email,
                    user_name: Faker.fake(),
                },
                signing,
                ..Default::default()
            }
        }
    }
    
    proptest! {
        #[test]
        fn test_profile_roundtrip(profile in arb_profile()) {
            let manager = ProfileManager::new();
            
            // Save and load
            manager.create(profile.clone()).await.unwrap();
            let loaded = manager.get(&profile.name).await.unwrap();
            
            // Should match
            assert_eq!(loaded, profile);
        }
    }
}
```

## Continuous Integration Setup

### CI Test Matrix
```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        include:
          - os: ubuntu-latest
            rust: nightly
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      
      - name: Install 1Password CLI
        run: |
          # Platform-specific installation
      
      - name: Run Integration Tests
        run: cargo test --features integration-tests
      
      - name: Run Benchmarks
        run: cargo bench --no-run
```

## Test Coverage Requirements

### Coverage Targets
- Unit Tests: >90%
- Integration Tests: >80%
- End-to-End Tests: >70%
- Security Tests: 100%

### Critical Path Coverage
All critical paths must have 100% coverage:
- Profile creation and application
- Credential handling
- Security validations
- Error recovery

## Summary

This comprehensive integration test plan ensures git-setup-rs works correctly across all platforms, integrates properly with external tools, handles errors gracefully, and maintains security. The test suite covers normal operations, edge cases, error conditions, and performance requirements.

Regular execution of these tests in CI ensures the system remains reliable as it evolves.