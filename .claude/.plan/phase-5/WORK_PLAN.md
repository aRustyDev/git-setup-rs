# Phase 5: Pattern Matching & Auto-Detection - Work Plan

## Prerequisites

Before starting Phase 5, ensure you're comfortable with basic Rust patterns.

**Required from Previous Phases**:
- ✅ Profile management system (Phase 2)
- ✅ Git integration working (Phase 2)
- ✅ Basic CLI/TUI (Phase 3)
- ✅ Error handling patterns

**Required Knowledge**:
- **Rust Patterns**: Match expressions, pattern syntax (*critical*)
- **Regular Expressions**: Basic regex understanding (*helpful*)
- **Git Remotes**: How Git stores remote URLs (*required*)

💡 **Junior Dev Resources**:
- 📚 [Rust Book Ch 18](https://doc.rust-lang.org/book/ch18-00-patterns.html) - Patterns and Matching
- 📖 [Pattern Matching Guide](https://doc.rust-lang.org/rust-by-example/flow_control/match.html) - Comprehensive intro
- 📖 [Regex Tutorial](https://regexone.com/) - Interactive regex learning
- 🔧 [Rust Playground](https://play.rust-lang.org/) - Test patterns online
- 📝 Examples: See `examples/patterns/` directory
- 📊 [Performance Profiling Guide](../PERFORMANCE_PROFILING_GUIDE.md) - Optimize pattern matching
- 🔗 [Cross-Phase Integration Guide](../CROSS_PHASE_INTEGRATION_GUIDE.md) - Pattern detection integration

## Quick Reference

### Git Commands
```bash
# List remotes with URLs
git remote -v

# Get URL of specific remote
git remote get-url origin

# Add new remote
git remote add upstream https://github.com/org/repo.git
```

### Pattern Types We'll Implement
1. **Exact Match**: `github.com/myorg/myrepo`
2. **Wildcard**: `github.com/myorg/*`
3. **Regex**: `github\.com/myorg/project-\d+`

## Overview

Phase 5 implements intelligent profile auto-detection based on Git repository remote URLs. This feature significantly improves user experience by automatically suggesting the correct profile.

**Key Deliverables**:
- Pattern matching system with multiple pattern types
- Repository remote URL detection
- Profile suggestion based on patterns
- User confirmation flow
- Pattern priority system

**Time Estimate**: 2 weeks (80 hours)
- Week 1: Pattern matching implementation (40h)
- Week 2: Auto-detection and integration (40h)

## Week 1: Pattern Matching Fundamentals

### 5A.1 Pattern System Design (16 hours)

#### Task 5A.1.1: Understanding Pattern Matching (4 hours)

💡 **Junior Dev Concept**: Pattern Matching in Rust
**What it is**: Rust's way of checking if data matches specific patterns
**Why we use it**: More powerful than if/else, compiler ensures all cases handled
**Real Example**: Matching "github.com/work/*" to suggest work profile

**Prerequisites**:
- [ ] Complete: Rust Book Chapter 18.1-18.2
- [ ] Try: Pattern matching exercises in playground
- [ ] Understand: Exhaustive matching concept

**Visual Concept**:
```
Git Remote URL                    Pattern                     Result
──────────────                   ─────────                   ──────
"github.com/acme/api"      matches "github.com/acme/*"  →  ✓ Match
"github.com/personal/blog" matches "github.com/acme/*"  →  ✗ No match
"gitlab.com/acme/api"      matches "*.com/acme/*"        →  ✓ Match
```

**Step-by-Step Implementation**:

1. **Create Pattern Enum** (1 hour)
   ```rust
   // src/profile/pattern.rs
   
   use regex::Regex;
   use std::fmt;
   
   /// Different types of patterns for matching URLs
   #[derive(Debug, Clone)]
   pub enum UrlPattern {
       /// Exact string match
       Exact(String),
       
       /// Simple wildcard pattern (only * supported)
       Wildcard(String),
       
       /// Full regex pattern
       Regex(Regex),
   }
   
   impl UrlPattern {
       /// Create pattern from string with auto-detection
       pub fn from_str(pattern: &str) -> Result<Self, PatternError> {
           if pattern.contains('*') {
               Ok(UrlPattern::Wildcard(pattern.to_string()))
           } else if pattern.starts_with('^') || pattern.contains('$') {
               let regex = Regex::new(pattern)
                   .map_err(|e| PatternError::InvalidRegex(e.to_string()))?;
               Ok(UrlPattern::Regex(regex))
           } else {
               Ok(UrlPattern::Exact(pattern.to_string()))
           }
       }
   }
   ```
   
   💡 **Design Choice**: Auto-detect pattern type from string format

2. **Implement Pattern Matching** (1.5 hours)
   ```rust
   impl UrlPattern {
       /// Check if URL matches this pattern
       pub fn matches(&self, url: &str) -> bool {
           match self {
               UrlPattern::Exact(pattern) => url == pattern,
               UrlPattern::Wildcard(pattern) => self.matches_wildcard(url, pattern),
               UrlPattern::Regex(regex) => regex.is_match(url),
           }
       }
       
       /// Handle wildcard matching
       fn matches_wildcard(&self, url: &str, pattern: &str) -> bool {
           // Split pattern by * to get parts
           let parts: Vec<&str> = pattern.split('*').collect();
           
           if parts.is_empty() {
               return true; // Just "*" matches everything
           }
           
           let mut url_remainder = url;
           
           for (i, part) in parts.iter().enumerate() {
               if part.is_empty() {
                   continue; // Skip empty parts from consecutive *
               }
               
               if i == 0 {
                   // First part must match start
                   if !url_remainder.starts_with(part) {
                       return false;
                   }
                   url_remainder = &url_remainder[part.len()..];
               } else if i == parts.len() - 1 && !pattern.ends_with('*') {
                   // Last part must match end (if no trailing *)
                   return url_remainder.ends_with(part);
               } else {
                   // Middle parts can appear anywhere
                   if let Some(pos) = url_remainder.find(part) {
                       url_remainder = &url_remainder[pos + part.len()..];
                   } else {
                       return false;
                   }
               }
           }
           
           true
       }
   }
   ```
   
   ⚠️ **Common Mistake**: Not handling edge cases like "**" or "*abc*def*"
   ✅ **Solution**: Test thoroughly with various patterns

3. **Write Comprehensive Tests** (1.5 hours)
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_exact_matching() {
           let pattern = UrlPattern::Exact("github.com/myorg/repo".to_string());
           
           assert!(pattern.matches("github.com/myorg/repo"));
           assert!(!pattern.matches("github.com/myorg/other"));
           assert!(!pattern.matches("gitlab.com/myorg/repo"));
       }
       
       #[test]
       fn test_wildcard_matching() {
           // Test cases with expected results
           let test_cases = vec![
               ("github.com/*", "github.com/anything", true),
               ("github.com/*", "gitlab.com/anything", false),
               ("*/myorg/*", "github.com/myorg/repo", true),
               ("*/myorg/*", "gitlab.com/myorg/repo", true),
               ("github.com/*/api", "github.com/org/api", true),
               ("github.com/*/api", "github.com/org/web", false),
           ];
           
           for (pattern_str, url, expected) in test_cases {
               let pattern = UrlPattern::from_str(pattern_str).unwrap();
               assert_eq!(
                   pattern.matches(url), 
                   expected,
                   "Pattern '{}' matching '{}' expected {}", 
                   pattern_str, url, expected
               );
           }
       }
       
       #[test]
       fn test_regex_matching() {
           let pattern = UrlPattern::from_str(r"^github\.com/myorg/project-\d+$").unwrap();
           
           assert!(pattern.matches("github.com/myorg/project-123"));
           assert!(pattern.matches("github.com/myorg/project-1"));
           assert!(!pattern.matches("github.com/myorg/project-abc"));
           assert!(!pattern.matches("github.com/myorg/project-"));
       }
   }
   ```

**Testing Your Work**:
```bash
# Run pattern tests
cargo test pattern::tests

# Test with examples
cargo run --example pattern_matching

# Benchmark pattern performance
cargo bench pattern_matching
```

**Debugging Guide**:

**Issue**: Wildcard pattern not matching expected URLs
**Debug Steps**:
1. Print pattern parts: `dbg!(pattern.split('*').collect::<Vec<_>>())`
2. Add logging in matches_wildcard
3. Test with simpler patterns first

**Issue**: Regex pattern fails to compile
**Solution**: Test regex at [regex101.com](https://regex101.com/) first

**When You're Stuck**:
1. Try the example: `examples/patterns/wildcard_demo.rs`
2. Use println! debugging in matches_wildcard
3. Ask in Slack: #rust-patterns channel

#### Task 5A.1.2: Pattern Priority System (4 hours)

💡 **Junior Dev Concept**: Priority Systems
**What it is**: Determining which pattern to use when multiple match
**Why needed**: User might have overlapping patterns
**Real Example**: Both "github.com/*" and "github.com/work/*" match

**Implementation**:

1. **Add Priority to Patterns** (1.5 hours)
   ```rust
   /// Pattern with priority for ordering
   #[derive(Debug, Clone)]
   pub struct PrioritizedPattern {
       pub pattern: UrlPattern,
       pub priority: i32,
       pub profile_name: String,
   }
   
   impl PrioritizedPattern {
       pub fn new(pattern: UrlPattern, priority: i32, profile_name: String) -> Self {
           Self { pattern, priority, profile_name }
       }
   }
   
   /// Collection of patterns with matching logic
   pub struct PatternMatcher {
       patterns: Vec<PrioritizedPattern>,
   }
   
   impl PatternMatcher {
       pub fn new() -> Self {
           Self { patterns: Vec::new() }
       }
       
       pub fn add_pattern(&mut self, pattern: PrioritizedPattern) {
           self.patterns.push(pattern);
           // Keep sorted by priority (highest first)
           self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
       }
       
       /// Find all matching patterns for a URL
       pub fn find_matches(&self, url: &str) -> Vec<&PrioritizedPattern> {
           self.patterns
               .iter()
               .filter(|p| p.pattern.matches(url))
               .collect()
       }
       
       /// Get best match (highest priority)
       pub fn best_match(&self, url: &str) -> Option<&PrioritizedPattern> {
           self.find_matches(url).into_iter().next()
       }
   }
   ```

2. **Pattern Specificity Calculation** (1.5 hours)
   ```rust
   impl UrlPattern {
       /// Calculate specificity score (higher = more specific)
       pub fn specificity(&self) -> u32 {
           match self {
               UrlPattern::Exact(_) => 1000,  // Most specific
               UrlPattern::Regex(_) => 500,   // Medium specific
               UrlPattern::Wildcard(p) => {
                   // More segments = more specific
                   let segments = p.split('/').count() as u32;
                   let wildcards = p.matches('*').count() as u32;
                   segments * 10 - wildcards * 5
               }
           }
       }
   }
   ```

3. **Integration Tests** (1 hour)
   ```rust
   #[test]
   fn test_pattern_priority() {
       let mut matcher = PatternMatcher::new();
       
       // Add patterns with different priorities
       matcher.add_pattern(PrioritizedPattern::new(
           UrlPattern::from_str("github.com/*").unwrap(),
           10,
           "personal".to_string(),
       ));
       
       matcher.add_pattern(PrioritizedPattern::new(
           UrlPattern::from_str("github.com/work/*").unwrap(),
           20,
           "work".to_string(),
       ));
       
       // More specific pattern should win
       let best = matcher.best_match("github.com/work/project").unwrap();
       assert_eq!(best.profile_name, "work");
   }
   ```

---

### 🛑 CHECKPOINT 5.1: Pattern System Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 16 hours + 4 hours review = 20 hours total

**Pre-Checkpoint Checklist**:
- [ ] Pattern types (Exact, Wildcard, Regex) implemented
- [ ] Pattern matching works correctly
- [ ] Priority system functioning
- [ ] All tests passing
- [ ] Performance acceptable (<1ms per match)

**Review Focus**:
- Pattern matching correctness
- Edge case handling
- Performance characteristics

---

### 5A.2 Repository Detection (12 hours)

#### Task 5A.2.1: Git Remote Detection (6 hours)

💡 **Junior Dev Concept**: Git Remotes
**What it is**: Git's record of where to push/pull code
**Why we need it**: To know which profile to suggest
**Real Example**: `origin` pointing to your GitHub fork

**Prerequisites**:
- [ ] Understand: Git remote concepts
- [ ] Practice: `git remote` commands
- [ ] Read: git2-rs documentation on remotes

**Implementation**:

1. **Create Remote Detector** (2 hours)
   ```rust
   // src/git/remote.rs
   
   use git2::{Repository, Remote};
   use std::path::Path;
   
   /// Detects remotes in a Git repository
   pub struct RemoteDetector {
       repo: Repository,
   }
   
   impl RemoteDetector {
       /// Open repository at path
       pub fn open(path: &Path) -> Result<Self, DetectorError> {
           let repo = Repository::open(path)
               .map_err(|e| DetectorError::NotARepository(e.to_string()))?;
           Ok(Self { repo })
       }
       
       /// Get all remote URLs
       pub fn get_remote_urls(&self) -> Result<Vec<(String, String)>, DetectorError> {
           let mut urls = Vec::new();
           
           let remotes = self.repo.remotes()
               .map_err(|e| DetectorError::GitError(e.to_string()))?;
           
           for remote_name in remotes.iter() {
               if let Some(name) = remote_name {
                   let remote = self.repo.find_remote(name)
                       .map_err(|e| DetectorError::GitError(e.to_string()))?;
                   
                   if let Some(url) = remote.url() {
                       urls.push((name.to_string(), url.to_string()));
                   }
               }
           }
           
           Ok(urls)
       }
       
       /// Get URL for specific remote (usually "origin")
       pub fn get_remote_url(&self, name: &str) -> Result<Option<String>, DetectorError> {
           match self.repo.find_remote(name) {
               Ok(remote) => Ok(remote.url().map(|s| s.to_string())),
               Err(_) => Ok(None),
           }
       }
   }
   ```

2. **URL Normalization** (2 hours)
   ```rust
   /// Normalize Git URLs for consistent matching
   pub fn normalize_git_url(url: &str) -> String {
       let mut normalized = url.to_string();
       
       // Remove protocol
       if let Some(pos) = normalized.find("://") {
           normalized = normalized[pos + 3..].to_string();
       }
       
       // Remove git@ prefix
       if normalized.starts_with("git@") {
           normalized = normalized[4..].to_string();
           // Convert : to / after hostname
           if let Some(pos) = normalized.find(':') {
               normalized.replace_range(pos..pos + 1, "/");
           }
       }
       
       // Remove .git suffix
       if normalized.ends_with(".git") {
           normalized.truncate(normalized.len() - 4);
       }
       
       // Remove trailing slash
       normalized.trim_end_matches('/').to_string()
   }
   
   #[test]
   fn test_url_normalization() {
       let cases = vec![
           ("https://github.com/user/repo.git", "github.com/user/repo"),
           ("git@github.com:user/repo.git", "github.com/user/repo"),
           ("ssh://git@github.com/user/repo", "github.com/user/repo"),
           ("https://github.com/user/repo/", "github.com/user/repo"),
       ];
       
       for (input, expected) in cases {
           assert_eq!(normalize_git_url(input), expected);
       }
   }
   ```

3. **Integration with Pattern Matcher** (2 hours)
   ```rust
   /// Auto-detect profile based on repository
   pub struct ProfileAutoDetector {
       pattern_matcher: PatternMatcher,
   }
   
   impl ProfileAutoDetector {
       pub fn new(profiles: Vec<Profile>) -> Self {
           let mut matcher = PatternMatcher::new();
           
           // Build patterns from profiles
           for profile in profiles {
               for remote in &profile.remotes {
                   if let Ok(pattern) = UrlPattern::from_str(&remote.pattern) {
                       matcher.add_pattern(PrioritizedPattern::new(
                           pattern,
                           remote.priority,
                           profile.name.clone(),
                       ));
                   }
               }
           }
           
           Self { pattern_matcher: matcher }
       }
       
       /// Detect profile for repository at path
       pub fn detect(&self, repo_path: &Path) -> Result<Option<String>, DetectorError> {
           let detector = RemoteDetector::open(repo_path)?;
           let urls = detector.get_remote_urls()?;
           
           // Check each remote URL
           for (_, url) in urls {
               let normalized = normalize_git_url(&url);
               
               if let Some(match_) = self.pattern_matcher.best_match(&normalized) {
                   return Ok(Some(match_.profile_name.clone()));
               }
           }
           
           Ok(None)
       }
   }
   ```

**Testing Your Work**:
```bash
# Test with a real repository
cd ~/your-project
cargo run --example detect_profile .

# Run detection tests
cargo test detection::tests
```

---

### 🛑 CHECKPOINT 5.2: Detection System Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 12 hours + 4 hours review = 16 hours total

**Pre-Checkpoint Checklist**:
- [ ] Remote URL detection working
- [ ] URL normalization handles all formats
- [ ] Integration with patterns complete
- [ ] Tests cover various Git configurations

---

## Week 2: Auto-Detection Integration

### 5A.3 User Interaction Flow (20 hours)

#### Task 5A.3.1: Confirmation Dialog Implementation (8 hours)

💡 **Junior Dev Concept**: User Confirmation Flow
**What it is**: Asking users before making changes
**Why important**: Never surprise users with automatic actions
**Real Example**: "Detected work profile for this repo. Use it? [Y/n]"

**Prerequisites**:
- [ ] Review: CLI prompt patterns
- [ ] Understand: User experience principles
- [ ] Practice: Terminal input handling

**Visual Flow**:
```
User enters repo     System detects       User confirms       Profile applied
────────────────     ──────────────      ─────────────      ───────────────
$ cd work-project -> Analyze remotes  -> Show suggestion -> Configure Git
                     Match patterns       "Use 'work'?"      Success message
                     Find best match      [Y/n]: y          ✓ Applied
```

**Step-by-Step Implementation**:

1. **Create Confirmation UI Trait** (2 hours)
   ```rust
   // src/ui/confirmation.rs
   
   use std::io::{self, Write};
   use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
   
   /// Trait for user confirmations across CLI and TUI
   pub trait ConfirmationUI {
       /// Ask for yes/no confirmation
       fn confirm(&mut self, message: &str, default: bool) -> Result<bool, UiError>;
       
       /// Show options and get selection
       fn select_option(&mut self, message: &str, options: &[String]) -> Result<usize, UiError>;
   }
   
   /// CLI implementation of confirmation
   pub struct CliConfirmation;
   
   impl ConfirmationUI for CliConfirmation {
       fn confirm(&mut self, message: &str, default: bool) -> Result<bool, UiError> {
           // Color the message for visibility
           execute!(
               io::stdout(),
               SetForegroundColor(Color::Cyan),
               Print(message),
               ResetColor
           )?;
           
           // Show default
           let prompt = if default { " [Y/n]: " } else { " [y/N]: " };
           print!("{}", prompt);
           io::stdout().flush()?;
           
           // Read input
           let mut input = String::new();
           io::stdin().read_line(&mut input)?;
           
           // Parse response
           let response = input.trim().to_lowercase();
           Ok(match response.as_str() {
               "y" | "yes" => true,
               "n" | "no" => false,
               "" => default,
               _ => {
                   println!("Please answer 'y' or 'n'");
                   self.confirm(message, default)?
               }
           })
       }
       
       fn select_option(&mut self, message: &str, options: &[String]) -> Result<usize, UiError> {
           println!("{}", message);
           
           // Display options
           for (i, option) in options.iter().enumerate() {
               println!("  {}. {}", i + 1, option);
           }
           
           print!("Select (1-{}): ", options.len());
           io::stdout().flush()?;
           
           // Read selection
           let mut input = String::new();
           io::stdin().read_line(&mut input)?;
           
           // Parse selection
           match input.trim().parse::<usize>() {
               Ok(n) if n > 0 && n <= options.len() => Ok(n - 1),
               _ => {
                   println!("Invalid selection. Please try again.");
                   self.select_option(message, options)?
               }
           }
       }
   }
   ```
   
   💡 **Design Pattern**: Trait allows swapping CLI/TUI implementations

2. **Create Auto-Detection Flow** (3 hours)
   ```rust
   // src/profile/auto_detect.rs
   
   /// Handles the auto-detection user flow
   pub struct AutoDetectionFlow {
       detector: ProfileAutoDetector,
       profile_manager: Box<dyn ProfileManager>,
       ui: Box<dyn ConfirmationUI>,
   }
   
   impl AutoDetectionFlow {
       /// Run auto-detection for current directory
       pub async fn run(&mut self) -> Result<Option<String>, FlowError> {
           let current_dir = std::env::current_dir()?;
           
           // Check if already configured
           if self.is_already_configured(&current_dir)? {
               return Ok(None);
           }
           
           // Detect profile
           match self.detector.detect(&current_dir)? {
               Some(profile_name) => {
                   self.handle_detection(profile_name).await
               }
               None => {
                   self.handle_no_detection().await
               }
           }
       }
       
       /// Handle when a profile is detected
       async fn handle_detection(&mut self, profile_name: String) -> Result<Option<String>, FlowError> {
           // Get profile details
           let profile = self.profile_manager.get(&profile_name).await?;
           
           // Build confirmation message
           let message = format!(
               "🎯 Detected profile '{}' for this repository\n   Email: {}\n   Use this profile?",
               profile.name,
               profile.git.user_email
           );
           
           if self.ui.confirm(&message, true)? {
               // Apply profile
               self.apply_profile(&profile_name).await?;
               println!("✅ Profile '{}' applied successfully!", profile_name);
               Ok(Some(profile_name))
           } else {
               // Offer alternatives
               self.offer_alternatives().await
           }
       }
       
       /// Handle when no profile matches
       async fn handle_no_detection(&mut self) -> Result<Option<String>, FlowError> {
           if self.ui.confirm("No profile detected. Would you like to select one?", true)? {
               self.manual_selection().await
           } else {
               Ok(None)
           }
       }
       
       /// Offer alternative profiles
       async fn offer_alternatives(&mut self) -> Result<Option<String>, FlowError> {
           let profiles = self.profile_manager.list().await?;
           
           if profiles.is_empty() {
               println!("No profiles found. Create one with: git-setup profile create");
               return Ok(None);
           }
           
           // Add "None" option
           let mut options = profiles.clone();
           options.push("None (skip)".to_string());
           
           let selection = self.ui.select_option(
               "Select a different profile:",
               &options
           )?;
           
           if selection < profiles.len() {
               let profile_name = &profiles[selection];
               self.apply_profile(profile_name).await?;
               println!("✅ Profile '{}' applied successfully!", profile_name);
               Ok(Some(profile_name.clone()))
           } else {
               Ok(None)
           }
       }
   }
   ```

3. **Integration with Git Hooks** (3 hours)
   ```rust
   /// Git hook integration for auto-detection
   pub struct HookIntegration;
   
   impl HookIntegration {
       /// Install post-checkout hook for auto-detection
       pub fn install_hook(repo_path: &Path) -> Result<(), FlowError> {
           let hook_path = repo_path.join(".git/hooks/post-checkout");
           
           // Create hook script
           let hook_content = r#"#!/bin/sh
   # git-setup-rs auto-detection hook
   # Runs after checkout to suggest profile
   
   # Only run on branch checkout (not file checkout)
   if [ "$3" = "1" ]; then
       git-setup detect --quiet 2>/dev/null || true
   fi
   "#;
           
           // Write hook file
           fs::write(&hook_path, hook_content)?;
           
           // Make executable
           #[cfg(unix)]
           {
               use std::os::unix::fs::PermissionsExt;
               let mut perms = fs::metadata(&hook_path)?.permissions();
               perms.set_mode(0o755);
               fs::set_permissions(&hook_path, perms)?;
           }
           
           Ok(())
       }
   }
   ```

**Testing Auto-Detection**:
```rust
#[tokio::test]
async fn test_auto_detection_flow() {
    // Create test repo with remote
    let test_repo = TestRepo::new();
    test_repo.add_remote("origin", "https://github.com/work/project.git");
    
    // Create mock UI that auto-confirms
    let mut mock_ui = MockConfirmationUI::new();
    mock_ui.expect_confirm()
        .returning(|_, _| Ok(true));
    
    // Run detection
    let mut flow = AutoDetectionFlow::new(
        detector,
        profile_manager,
        Box::new(mock_ui)
    );
    
    let result = flow.run().await.unwrap();
    assert_eq!(result, Some("work".to_string()));
}
```

**Debugging Guide**:

**Issue**: Detection not triggering
**Debug**: Add logging to see which patterns are checked
```rust
log::debug!("Checking URL '{}' against patterns", normalized_url);
```

**Issue**: Wrong profile detected
**Solution**: Check pattern priorities and specificity

---

#### Task 5A.3.2: Settings Persistence (6 hours)

💡 **Junior Dev Concept**: Remembering User Choices
**What it is**: Saving user's "don't ask again" preferences
**Why needed**: Avoid annoying users with repeated prompts
**Storage**: Local `.git/config` or global settings

**Implementation**:

1. **Create Settings Storage** (2 hours)
   ```rust
   // src/profile/auto_detect_settings.rs
   
   use serde::{Deserialize, Serialize};
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AutoDetectSettings {
       /// Disabled for specific repos (by path)
       pub disabled_repos: Vec<String>,
       
       /// Always use specific profile for patterns
       pub pattern_overrides: HashMap<String, String>,
       
       /// Global auto-detect enabled
       pub enabled: bool,
       
       /// Ask before applying
       pub confirm_before_apply: bool,
   }
   
   impl Default for AutoDetectSettings {
       fn default() -> Self {
           Self {
               disabled_repos: Vec::new(),
               pattern_overrides: HashMap::new(),
               enabled: true,
               confirm_before_apply: true,
           }
       }
   }
   
   impl AutoDetectSettings {
       /// Load from config file
       pub fn load() -> Result<Self, ConfigError> {
           let config_path = dirs::config_dir()
               .ok_or(ConfigError::NoConfigDir)?
               .join("git-setup")
               .join("auto-detect.toml");
           
           if config_path.exists() {
               let content = fs::read_to_string(&config_path)?;
               toml::from_str(&content)
                   .map_err(|e| ConfigError::ParseError(e.to_string()))
           } else {
               Ok(Self::default())
           }
       }
       
       /// Save to config file
       pub fn save(&self) -> Result<(), ConfigError> {
           let config_dir = dirs::config_dir()
               .ok_or(ConfigError::NoConfigDir)?
               .join("git-setup");
           
           fs::create_dir_all(&config_dir)?;
           
           let config_path = config_dir.join("auto-detect.toml");
           let content = toml::to_string_pretty(self)?;
           fs::write(config_path, content)?;
           
           Ok(())
       }
       
       /// Check if auto-detect is disabled for repo
       pub fn is_disabled_for(&self, repo_path: &Path) -> bool {
           let path_str = repo_path.to_string_lossy().to_string();
           self.disabled_repos.contains(&path_str)
       }
       
       /// Disable auto-detect for specific repo
       pub fn disable_for(&mut self, repo_path: &Path) {
           let path_str = repo_path.to_string_lossy().to_string();
           if !self.disabled_repos.contains(&path_str) {
               self.disabled_repos.push(path_str);
           }
       }
   }
   ```

2. **Integrate Settings with Flow** (2 hours)
   ```rust
   impl AutoDetectionFlow {
       /// Run with settings check
       pub async fn run_with_settings(&mut self) -> Result<Option<String>, FlowError> {
           let current_dir = std::env::current_dir()?;
           
           // Load settings
           let mut settings = AutoDetectSettings::load()
               .unwrap_or_default();
           
           // Check if disabled
           if !settings.enabled || settings.is_disabled_for(&current_dir) {
               return Ok(None);
           }
           
           // Run detection
           match self.run().await? {
               Some(profile) => Ok(Some(profile)),
               None => {
                   // Ask about disabling for this repo
                   if self.ui.confirm(
                       "Disable auto-detection for this repository?",
                       false
                   )? {
                       settings.disable_for(&current_dir);
                       settings.save()?;
                       println!("Auto-detection disabled for this repository.");
                   }
                   Ok(None)
               }
           }
       }
   }
   ```

3. **CLI Commands for Settings** (2 hours)
   ```rust
   /// CLI commands for auto-detect settings
   #[derive(Subcommand)]
   pub enum AutoDetectCommands {
       /// Enable auto-detection globally
       Enable,
       
       /// Disable auto-detection globally
       Disable,
       
       /// Show current settings
       Status,
       
       /// Reset to defaults
       Reset,
   }
   ```

---

#### Task 5A.3.3: TUI Integration (6 hours)

💡 **Junior Dev Concept**: TUI Auto-Detection
**What it is**: Showing detection results in the TUI
**Challenge**: Non-blocking UI while detecting
**Solution**: Background task with status updates

**Visual TUI Mock**:
```
┌─ Git Setup - Auto Detection ────────────┐
│                                         │
│  🔍 Analyzing repository...             │
│                                         │
│  Remote: github.com/acme/project        │
│                                         │
│  ✓ Profile detected: work               │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │ Email: alice@acme.com           │   │
│  │ Name: Alice Smith               │   │
│  │ Signing: SSH                    │   │
│  └─────────────────────────────────┘   │
│                                         │
│  [ Apply ]  [ Choose Other ]  [ Skip ]  │
│                                         │
│  Press Enter to apply, Tab to navigate  │
└─────────────────────────────────────────┘
```

**Implementation**:

1. **Create TUI Detection Widget** (3 hours)
   ```rust
   // src/tui/widgets/auto_detect.rs
   
   use ratatui::{
       layout::{Alignment, Constraint, Direction, Layout, Rect},
       style::{Color, Modifier, Style},
       text::{Line, Span},
       widgets::{Block, Borders, Paragraph, Wrap},
       Frame,
   };
   
   #[derive(Debug, Clone)]
   pub enum DetectionState {
       Checking,
       Found(String, ProfileSummary),
       NotFound,
       Error(String),
   }
   
   pub struct AutoDetectWidget {
       state: DetectionState,
       selected_action: usize,
       actions: Vec<&'static str>,
   }
   
   impl AutoDetectWidget {
       pub fn new() -> Self {
           Self {
               state: DetectionState::Checking,
               selected_action: 0,
               actions: vec!["Apply", "Choose Other", "Skip"],
           }
       }
       
       pub fn render(&self, f: &mut Frame, area: Rect) {
           let chunks = Layout::default()
               .direction(Direction::Vertical)
               .constraints([
                   Constraint::Length(3),   // Title
                   Constraint::Min(10),     // Content
                   Constraint::Length(3),   // Actions
                   Constraint::Length(1),   // Help
               ])
               .split(area);
           
           // Title
           let title = Paragraph::new("🔍 Auto-Detection")
               .block(Block::default().borders(Borders::BOTTOM))
               .alignment(Alignment::Center);
           f.render_widget(title, chunks[0]);
           
           // Content based on state
           match &self.state {
               DetectionState::Checking => {
                   self.render_checking(f, chunks[1]);
               }
               DetectionState::Found(name, profile) => {
                   self.render_found(f, chunks[1], name, profile);
               }
               DetectionState::NotFound => {
                   self.render_not_found(f, chunks[1]);
               }
               DetectionState::Error(msg) => {
                   self.render_error(f, chunks[1], msg);
               }
           }
           
           // Actions
           self.render_actions(f, chunks[2]);
           
           // Help
           let help = Paragraph::new("Press Enter to select, Tab to navigate")
               .style(Style::default().fg(Color::DarkGray))
               .alignment(Alignment::Center);
           f.render_widget(help, chunks[3]);
       }
       
       fn render_found(&self, f: &mut Frame, area: Rect, name: &str, profile: &ProfileSummary) {
           let content = vec![
               Line::from(""),
               Line::from(vec![
                   Span::raw("✓ Profile detected: "),
                   Span::styled(name, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
               ]),
               Line::from(""),
               Line::from(vec![
                   Span::raw("  Email: "),
                   Span::styled(&profile.email, Style::default().fg(Color::Cyan)),
               ]),
               Line::from(vec![
                   Span::raw("  Name: "),
                   Span::raw(&profile.name),
               ]),
               Line::from(vec![
                   Span::raw("  Signing: "),
                   Span::raw(&profile.signing_method),
               ]),
           ];
           
           let paragraph = Paragraph::new(content)
               .block(Block::default().borders(Borders::ALL))
               .wrap(Wrap { trim: true });
           f.render_widget(paragraph, area);
       }
   }
   ```

2. **Background Detection Task** (3 hours)
   ```rust
   /// Run detection in background
   pub async fn start_detection(
       repo_path: PathBuf,
       tx: mpsc::Sender<DetectionResult>,
   ) {
       tokio::spawn(async move {
           // Simulate async detection
           let detector = ProfileAutoDetector::new().await;
           
           match detector.detect(&repo_path).await {
               Ok(Some(profile_name)) => {
                   // Get profile details
                   let profile = load_profile(&profile_name).await;
                   tx.send(DetectionResult::Found(profile_name, profile)).await.ok();
               }
               Ok(None) => {
                   tx.send(DetectionResult::NotFound).await.ok();
               }
               Err(e) => {
                   tx.send(DetectionResult::Error(e.to_string())).await.ok();
               }
           }
       });
   }
   ```

---

### 🛑 CHECKPOINT 5.3: Week 2 Functionality Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 20 hours + 4 hours review = 24 hours total

**Pre-Checkpoint Checklist**:
- [ ] Confirmation dialog working in CLI
- [ ] Settings persistence functional
- [ ] TUI integration complete
- [ ] User preferences saved
- [ ] Background detection working
- [ ] All Week 2 tests passing

**Review Focus**:
- User experience flow
- Settings storage security
- TUI responsiveness

**What Makes This Checkpoint Important**:
This checkpoint ensures the user interaction layer is polished before moving to the final integration phase. The confirmation flow is critical for user trust.

---

## Week 2 (Continued): Final Integration and Polish

### 5A.4 Integration and Testing (20 hours)

#### Task 5A.4.1: Git Hook Integration (8 hours)

💡 **Junior Dev Concept**: Git Hooks
**What they are**: Scripts that run automatically on Git events
**Why use them**: Auto-detect when user clones or switches branches
**Example**: Post-checkout hook runs after `git checkout`

**Implementation**:

1. **Hook Manager** (4 hours)
   ```rust
   // src/git/hooks.rs
   
   pub struct GitHookManager {
       repo_path: PathBuf,
   }
   
   impl GitHookManager {
       /// Install auto-detection hooks
       pub fn install_auto_detect_hooks(&self) -> Result<(), HookError> {
           self.install_post_checkout_hook()?;
           self.install_post_clone_hook()?;
           Ok(())
       }
       
       /// Create post-checkout hook
       fn install_post_checkout_hook(&self) -> Result<(), HookError> {
           let hook_content = include_str!("../../hooks/post-checkout.sh");
           self.install_hook("post-checkout", hook_content)
       }
       
       /// Install a hook with given content
       fn install_hook(&self, name: &str, content: &str) -> Result<(), HookError> {
           let hooks_dir = self.repo_path.join(".git/hooks");
           fs::create_dir_all(&hooks_dir)?;
           
           let hook_path = hooks_dir.join(name);
           
           // Check if hook already exists
           if hook_path.exists() {
               // Backup existing hook
               let backup_path = hooks_dir.join(format!("{}.backup", name));
               fs::copy(&hook_path, backup_path)?;
           }
           
           // Write new hook
           fs::write(&hook_path, content)?;
           
           // Make executable on Unix
           #[cfg(unix)]
           {
               use std::os::unix::fs::PermissionsExt;
               let mut perms = fs::metadata(&hook_path)?.permissions();
               perms.set_mode(0o755);
               fs::set_permissions(&hook_path, perms)?;
           }
           
           Ok(())
       }
   }
   ```

2. **Hook Scripts** (2 hours)
   ```bash
   # hooks/post-checkout.sh
   #!/bin/sh
   # Git-setup auto-detection hook
   
   # Skip if disabled
   if [ "$GIT_SETUP_DISABLE_AUTO_DETECT" = "1" ]; then
       exit 0
   fi
   
   # Only run on branch checkout (not file checkout)
   if [ "$3" = "1" ]; then
       # Run detection quietly
       git-setup detect --quiet 2>/dev/null || true
   fi
   ```

3. **Testing Hooks** (2 hours)
   ```rust
   #[test]
   fn test_hook_installation() {
       let temp_repo = TempRepo::new();
       let manager = GitHookManager::new(&temp_repo.path());
       
       // Install hooks
       manager.install_auto_detect_hooks().unwrap();
       
       // Verify hook exists and is executable
       let hook_path = temp_repo.path().join(".git/hooks/post-checkout");
       assert!(hook_path.exists());
       
       #[cfg(unix)]
       {
           let metadata = fs::metadata(&hook_path).unwrap();
           let permissions = metadata.permissions();
           assert!(permissions.mode() & 0o111 != 0); // Check executable
       }
   }
   ```

---

#### Task 5A.4.2: Performance Optimization (8 hours)

💡 **Junior Dev Concept**: Performance Matters
**Why optimize**: Detection should be instant (<100ms)
**Key areas**: Pattern matching, Git operations, caching
**Goal**: Invisible to user

**Optimization Areas**:

1. **Pattern Caching** (3 hours)
   ```rust
   /// Cached pattern matcher for performance
   pub struct CachedPatternMatcher {
       patterns: Vec<PrioritizedPattern>,
       regex_cache: HashMap<String, Regex>,
   }
   
   impl CachedPatternMatcher {
       /// Pre-compile all regex patterns
       pub fn new(patterns: Vec<PrioritizedPattern>) -> Self {
           let mut regex_cache = HashMap::new();
           
           // Pre-compile regex patterns
           for pattern in &patterns {
               if let UrlPattern::Regex(regex) = &pattern.pattern {
                   regex_cache.insert(
                       regex.as_str().to_string(),
                       regex.clone()
                   );
               }
           }
           
           Self { patterns, regex_cache }
       }
   }
   ```

2. **Lazy Git Operations** (3 hours)
   ```rust
   /// Only check Git when needed
   pub struct LazyGitDetector {
       repo_path: PathBuf,
       remotes: OnceCell<Vec<(String, String)>>,
   }
   
   impl LazyGitDetector {
       pub fn new(repo_path: PathBuf) -> Self {
           Self {
               repo_path,
               remotes: OnceCell::new(),
           }
       }
       
       /// Get remotes, caching result
       pub fn get_remotes(&self) -> Result<&Vec<(String, String)>, DetectorError> {
           self.remotes.get_or_try_init(|| {
               RemoteDetector::open(&self.repo_path)?
                   .get_remote_urls()
           })
       }
   }
   ```

3. **Benchmark Suite** (2 hours)
   ```rust
   #[bench]
   fn bench_pattern_matching(b: &mut Bencher) {
       let matcher = create_test_matcher();
       let url = "github.com/org/repo";
       
       b.iter(|| {
           black_box(matcher.best_match(url));
       });
   }
   ```

---

#### Task 5A.4.3: Integration Testing (4 hours)

💡 **Junior Dev Concept**: End-to-End Testing
**What it is**: Testing the complete flow from detection to application
**Why critical**: Ensures all parts work together
**Approach**: Simulate real user scenarios

**Test Scenarios**:

```rust
#[tokio::test]
async fn test_complete_auto_detection_flow() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let repo = init_test_repo(&temp_dir);
    add_remote(&repo, "origin", "https://github.com/work/project");
    
    // Create profiles
    let profile_manager = create_test_profile_manager();
    profile_manager.create(Profile {
        name: "work".to_string(),
        git: GitConfig {
            user_email: "alice@work.com".to_string(),
            user_name: "Alice Work".to_string(),
        },
        remotes: vec![RemotePattern {
            pattern: "github.com/work/*".to_string(),
            priority: 10,
        }],
        ..Default::default()
    }).await.unwrap();
    
    // Run detection
    let mut flow = AutoDetectionFlow::new(
        ProfileAutoDetector::new(profile_manager.clone()),
        profile_manager,
        Box::new(AutoConfirmUI::new(true)), // Auto-confirm
    );
    
    let result = flow.run().await.unwrap();
    assert_eq!(result, Some("work".to_string()));
    
    // Verify Git config was updated
    let config = read_git_config(&repo);
    assert_eq!(config.get("user.email"), Some("alice@work.com"));
}
```

---

### 🛑 FINAL CHECKPOINT 5: Pattern Matching & Auto-Detection Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** to Phase 6 without approval.

**Total Phase Duration**: 2 weeks (80 hours)
- Week 1: Core pattern system (40h)
- Week 2: User interaction and integration (40h)

**Final Deliverables**:
- ✅ Pattern matching with 3 types (Exact, Wildcard, Regex)
- ✅ Repository remote detection
- ✅ Auto-detection flow with confirmation
- ✅ Settings persistence
- ✅ Git hook integration
- ✅ Performance optimized (<100ms)
- ✅ Complete test coverage
- ✅ Documentation

**Success Metrics**:
- Detection accuracy: >95% for configured patterns
- Performance: <100ms for detection
- User satisfaction: Smooth, non-intrusive flow
- Code coverage: >90% for critical paths

**Key Achievements**:
1. **Junior-Friendly Implementation**: Every concept explained
2. **Robust Pattern System**: Handles edge cases
3. **Great UX**: Non-intrusive with user control
4. **Performance**: Instant detection
5. **Well-Tested**: Comprehensive test suite

---

## Common Issues and Solutions

### Pattern Matching Issues

**Issue**: Wildcard patterns not matching as expected
**Debug**:
```rust
// Add debug logging
log::debug!("Pattern parts: {:?}", pattern.split('*').collect::<Vec<_>>());
log::debug!("URL being matched: {}", url);
```
**Solution**: Ensure URL normalization is consistent

**Issue**: Regex patterns causing panics
**Solution**: Always validate regex at pattern creation:
```rust
Regex::new(pattern).map_err(|e| PatternError::InvalidRegex(e.to_string()))?
```

### Performance Issues

**Issue**: Detection takes >100ms
**Profile**: Use `cargo flamegraph` to find bottlenecks
**Common fixes**:
1. Cache compiled regex patterns
2. Avoid repeated Git operations
3. Use lazy initialization

### Git Hook Issues

**Issue**: Hooks not executing
**Debug checklist**:
1. Check hook has executable permissions: `ls -la .git/hooks/`
2. Verify shebang line: `#!/bin/sh`
3. Test manually: `.git/hooks/post-checkout`
4. Check Git version supports hooks

## Summary

Phase 5 provides intelligent auto-detection that significantly improves user experience. The implementation is thoroughly documented for junior developers with:

- Clear concept explanations
- Visual diagrams
- Step-by-step implementation
- Comprehensive testing
- Performance optimization
- Proper error handling

The gradual complexity increase and well-paced checkpoints ensure developers can successfully implement this feature without feeling overwhelmed.

#### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** to Phase 6 without approval.

**Final Deliverables**:
- Pattern matching system with 3 types
- Repository detection working
- User confirmation flow
- Settings persistence
- All tests passing
- Documentation complete

**Success Metrics**:
- Detection accuracy >95%
- Performance <100ms
- User satisfaction with flow

---

## Common Issues and Solutions

### Issue: Pattern matching too slow
**Symptom**: Detection takes >100ms
**Solution**: 
1. Cache compiled regex patterns
2. Order patterns by frequency of use
3. Short-circuit on first match if appropriate

### Issue: Git remotes not detected
**Symptom**: RemoteDetector returns empty list
**Debug**:
1. Check if directory is a Git repo: `git status`
2. List remotes manually: `git remote -v`
3. Check git2 version compatibility

## Summary

Phase 5 provides a solid foundation for auto-detection with extensive junior developer support. The gradual complexity increase and frequent checkpoints ensure steady progress without overwhelming learners.

**Next**: Phase 6 - Health Monitoring System (Basic Diagnostics)