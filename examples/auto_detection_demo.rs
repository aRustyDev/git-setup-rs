use git_setup_rs::{
    AutoDetector, DetectionConfig, ProfileDetector,
    profile::ProfileManager,
    external::git::MockGitWrapper,
    config::types::{Profile, KeyType, Scope},
};
use std::sync::Arc;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Git Setup Auto Detection Demo");
    println!("==============================");

    // Create some test profiles
    let work_profile = Profile {
        name: "work".to_string(),
        git_user_name: Some("John Doe".to_string()),
        git_user_email: "john.doe@company.com".to_string(),
        key_type: KeyType::Ssh,
        signing_key: Some("ssh-ed25519 AAAAC3...work-key".to_string()),
        repos: vec![
            "git@github.com:company/*".to_string(),
            "git@gitlab.company.com:*".to_string(),
        ],
        match_patterns: vec!["*/work/*".to_string(), "*/company/*".to_string()],
        include_if_dirs: vec!["/home/user/work".to_string()],
        host_patterns: vec!["work-laptop-*".to_string()],
        scope: Some(Scope::Local),
        vault_name: None,
        ssh_key_title: None,
        ssh_key_source: None,
        ssh_key_path: None,
        allowed_signers: None,
        one_password: false,
    };

    let personal_profile = Profile {
        name: "personal".to_string(),
        git_user_name: Some("John Doe".to_string()),
        git_user_email: "john@personal.dev".to_string(),
        key_type: KeyType::Gpg,
        signing_key: Some("B5690EEEBB952194".to_string()),
        repos: vec![
            "git@github.com:johndoe/*".to_string(),
            "https://github.com/johndoe/*".to_string(),
        ],
        match_patterns: vec!["*/personal/*".to_string(), "*/github/*".to_string()],
        include_if_dirs: vec!["/home/user/personal".to_string()],
        host_patterns: vec!["home-*".to_string(), "personal-*".to_string()],
        scope: Some(Scope::Global),
        vault_name: Some("Personal".to_string()),
        ssh_key_title: None,
        ssh_key_source: None,
        ssh_key_path: None,
        allowed_signers: None,
        one_password: false,
    };

    let opensource_profile = Profile {
        name: "opensource".to_string(),
        git_user_name: Some("John Doe".to_string()),
        git_user_email: "johndoe@contributor.dev".to_string(),
        key_type: KeyType::Ssh,
        signing_key: Some("ssh-ed25519 AAAAC3...opensource-key".to_string()),
        repos: vec![
            "git@github.com:apache/*".to_string(),
            "git@github.com:rust-lang/*".to_string(),
            "https://github.com/*/rust-*".to_string(),
        ],
        match_patterns: vec!["*/opensource/*".to_string(), "*/projects/rust/*".to_string()],
        include_if_dirs: vec!["/home/user/opensource".to_string()],
        host_patterns: vec![],
        scope: Some(Scope::Local),
        vault_name: None,
        ssh_key_title: None,
        ssh_key_source: None,
        ssh_key_path: None,
        allowed_signers: None,
        one_password: false,
    };

    // Create a mock profile manager with our test profiles
    let profiles = vec![work_profile, personal_profile, opensource_profile];
    let profile_manager = Arc::new(create_mock_profile_manager(profiles));

    // Demo 1: Detection by remote URL
    println!("\n1. Detection by Remote URL");
    println!("---------------------------");

    let mut git_config = HashMap::new();
    git_config.insert("remote.origin.url".to_string(), "git@github.com:company/awesome-project.git".to_string());
    let git = Arc::new(MockGitWrapper::new().with_config(git_config));

    let detector = AutoDetector::new(profile_manager.clone(), git);

    // Simulate being in a company repository
    let temp_dir = tempfile::TempDir::new()?;
    std::fs::create_dir(temp_dir.path().join(".git"))?;

    match detector.detect_in(temp_dir.path())? {
        Some(result) => {
            println!("✓ Detected profile: '{}' (confidence: {:.0}%)",
                     result.profile.name, result.confidence * 100.0);
            println!("  Reason: {}", result.reason);
            println!("  Email: {}", result.profile.git_user_email);
        }
        None => println!("✗ No profile detected"),
    }

    // Demo 2: Detection by directory pattern
    println!("\n2. Detection by Directory Pattern");
    println!("-----------------------------------");

    let git2 = Arc::new(MockGitWrapper::new());
    let detector2 = AutoDetector::new(profile_manager.clone(), git2);

    let work_path = std::path::PathBuf::from("/home/user/work/my-project");
    match detector2.detect_in(&work_path)? {
        Some(result) => {
            println!("✓ Detected profile: '{}' (confidence: {:.0}%)",
                     result.profile.name, result.confidence * 100.0);
            println!("  Reason: {}", result.reason);
            println!("  Key type: {:?}", result.profile.key_type);
        }
        None => println!("✗ No profile detected"),
    }

    // Demo 3: Detection by include-if directory
    println!("\n3. Detection by Include-If Directory");
    println!("-------------------------------------");

    let personal_path = std::path::PathBuf::from("/home/user/personal/cool-project");
    match detector2.detect_in(&personal_path)? {
        Some(result) => {
            println!("✓ Detected profile: '{}' (confidence: {:.0}%)",
                     result.profile.name, result.confidence * 100.0);
            println!("  Reason: {}", result.reason);
            println!("  Scope: {:?}", result.profile.scope);
        }
        None => println!("✗ No profile detected"),
    }

    // Demo 4: Multiple matches - show all results
    println!("\n4. All Possible Matches");
    println!("------------------------");

    let mut multiple_config = HashMap::new();
    multiple_config.insert("remote.origin.url".to_string(), "git@github.com:rust-lang/rust.git".to_string());
    multiple_config.insert("user.email".to_string(), "johndoe@contributor.dev".to_string());
    let git3 = Arc::new(MockGitWrapper::new().with_config(multiple_config));

    let detector3 = AutoDetector::new(profile_manager.clone(), git3.clone());
    let _opensource_path = std::path::PathBuf::from("/home/user/opensource/rust-project");

    let results = detector3.detect_all()?;
    if results.is_empty() {
        println!("✗ No profiles detected");
    } else {
        println!("Found {} potential matches:", results.len());
        for (i, result) in results.iter().enumerate() {
            println!("  {}. '{}' (confidence: {:.0}%)",
                     i + 1, result.profile.name, result.confidence * 100.0);
            println!("     Reason: {}", result.reason);
            println!("     Matched rules: {}",
                     result.matched_rules.iter()
                         .map(|r| format!("{} ({:.0}%)", r.rule_name, r.confidence * 100.0))
                         .collect::<Vec<_>>()
                         .join(", "));
        }

        if let Some(best) = results.first() {
            println!("\n→ Recommended profile: '{}'", best.profile.name);
        }
    }

    // Demo 5: Custom detection configuration
    println!("\n5. Custom Detection Configuration");
    println!("----------------------------------");

    let custom_config = DetectionConfig {
        min_confidence: 0.8,  // Higher threshold
        check_remote_url: true,
        check_directory: false,  // Disable directory matching
        check_include_if: true,
        check_hostname: false,   // Disable hostname matching
        check_git_config: true,
        enable_cache: false,
    };

    let detector4 = AutoDetector::with_config(profile_manager, git3, custom_config);

    let results = detector4.detect_all()?;
    println!("With custom config (higher threshold, limited rules):");
    if results.is_empty() {
        println!("✗ No profiles met the higher confidence threshold");
    } else {
        for result in results {
            println!("  '{}' (confidence: {:.0}%)",
                     result.profile.name, result.confidence * 100.0);
        }
    }

    println!("\n✅ Auto Detection Demo Complete!");
    Ok(())
}

// Helper function to create a mock profile manager
fn create_mock_profile_manager(profiles: Vec<Profile>) -> impl ProfileManager {
    use git_setup_rs::profile::mock::MockProfileManager;
    MockProfileManager::with_profiles(profiles)
}
