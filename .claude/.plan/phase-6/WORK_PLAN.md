# Phase 6: Health Monitoring System - Work Plan

## Prerequisites

Before starting Phase 6, ensure Phase 5 is complete and you understand system diagnostics.

**Required from Previous Phases**:
- ✅ Pattern matching system (Phase 5)
- ✅ Git integration working
- ✅ 1Password integration basics (Phase 4)
- ✅ File system operations (Phase 1)

**Required Knowledge**:
- **System Diagnostics**: Checking system state (*required*)
- **Error Handling**: Graceful degradation (*critical*)
- **Performance Monitoring**: Basic metrics (*helpful*)

💡 **Junior Dev Resources**:
- 📚 [Building Robust CLI Tools](https://rust-cli.github.io/book/) - Chapter on diagnostics
- 📖 [Health Checks in Practice](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) - Trait patterns
- 📖 [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) - Review
- 🔧 Examples: `examples/health/` directory
- 📊 [Performance Profiling Guide](../PERFORMANCE_PROFILING_GUIDE.md) - Profile health check performance
- 🔗 [Cross-Phase Integration Guide](../CROSS_PHASE_INTEGRATION_GUIDE.md) - Integrate with other phases

## Quick Reference

### Health Check Categories
1. **System Checks**: Git installed, version compatible
2. **Configuration Checks**: Profiles valid, paths exist
3. **Integration Checks**: 1Password connected, SSH available
4. **Performance Checks**: Operation timings, resource usage

## Overview

Phase 6 implements a comprehensive health monitoring system that helps users diagnose issues before they become problems. This proactive approach significantly reduces support burden.

**Key Deliverables**:
- Health check framework with multiple check types
- Diagnostic report generation
- Performance monitoring
- Self-healing mechanisms for common issues
- Integration with CLI/TUI

**Time Estimate**: 2 weeks (80 hours)
- Week 1: Basic health checks (40h)
- Week 2: Advanced diagnostics and reporting (40h)

## Week 1: Basic Health Checks

### 5B.1 Health Check Framework (16 hours)

#### Task 5B.1.1: Core Health Check System (8 hours)

💡 **Junior Dev Concept**: Health Check Systems
**What it is**: Code that verifies your application can work correctly
**Why we need it**: Users need to know what's wrong when things fail
**Real Example**: "Git not found" is better than "Command failed"

**Prerequisites**:
- [ ] Understand: Trait design patterns
- [ ] Review: Error handling from Phase 1
- [ ] Read: Command execution in Rust

**Step-by-Step Implementation**:

1. **Define Health Check Trait** (2 hours)
   ```rust
   // src/health/mod.rs
   
   use std::time::Duration;
   use serde::{Deserialize, Serialize};
   
   /// Status of a health check
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub enum HealthStatus {
       /// Everything working correctly
       Healthy,
       
       /// Working but with issues
       Warning(String),
       
       /// Not working
       Error(String),
       
       /// Check was skipped
       Skipped(String),
   }
   
   /// Result of a health check
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct HealthCheckResult {
       /// Name of the check
       pub name: String,
       
       /// Category (system, config, integration)
       pub category: String,
       
       /// Current status
       pub status: HealthStatus,
       
       /// Detailed message
       pub message: String,
       
       /// How long the check took
       pub duration: Duration,
       
       /// Suggested fix if unhealthy
       pub fix_suggestion: Option<String>,
   }
   
   /// Trait for implementing health checks
   #[async_trait]
   pub trait HealthCheck: Send + Sync {
       /// Unique name for this check
       fn name(&self) -> &str;
       
       /// Category of check
       fn category(&self) -> &str;
       
       /// Run the health check
       async fn check(&self) -> HealthCheckResult;
       
       /// Whether this check is critical
       fn is_critical(&self) -> bool {
           false
       }
   }
   ```
   
   💡 **Design Pattern**: Trait-based for extensibility

2. **Implement Basic System Checks** (3 hours)
   ```rust
   // src/health/system.rs
   
   /// Check if Git is installed and accessible
   pub struct GitInstalledCheck;
   
   #[async_trait]
   impl HealthCheck for GitInstalledCheck {
       fn name(&self) -> &str {
           "Git Installation"
       }
       
       fn category(&self) -> &str {
           "system"
       }
       
       fn is_critical(&self) -> bool {
           true // Can't work without Git
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = std::time::Instant::now();
           
           let result = match Command::new("git").arg("--version").output() {
               Ok(output) => {
                   if output.status.success() {
                       let version = String::from_utf8_lossy(&output.stdout);
                       let version = version.trim();
                       
                       // Check minimum version
                       if let Some(ver) = parse_git_version(&version) {
                           if ver >= (2, 25, 0) {
                               HealthStatus::Healthy
                           } else {
                               HealthStatus::Warning(format!(
                                   "Git {} is old, recommend 2.25.0+"
                               ))
                           }
                       } else {
                           HealthStatus::Warning("Could not parse Git version".into())
                       }
                   } else {
                       HealthStatus::Error("Git command failed".into())
                   }
               }
               Err(e) => {
                   HealthStatus::Error(format!("Git not found: {}", e))
               }
           };
           
           HealthCheckResult {
               name: self.name().to_string(),
               category: self.category().to_string(),
               status: result,
               message: "Checking Git installation".to_string(),
               duration: start.elapsed(),
               fix_suggestion: match &result {
                   HealthStatus::Error(_) => Some(
                       "Install Git from https://git-scm.com/downloads".into()
                   ),
                   _ => None,
               },
           }
       }
   }
   
   /// Parse Git version string
   fn parse_git_version(version: &str) -> Option<(u32, u32, u32)> {
       // "git version 2.34.1" -> (2, 34, 1)
       let parts: Vec<&str> = version.split_whitespace().collect();
       if parts.len() >= 3 {
           let version_str = parts[2];
           let nums: Vec<u32> = version_str
               .split('.')
               .filter_map(|s| s.parse().ok())
               .collect();
           
           if nums.len() >= 3 {
               return Some((nums[0], nums[1], nums[2]));
           }
       }
       None
   }
   ```

3. **Create Health Check Runner** (3 hours)
   ```rust
   /// Runs health checks and collects results
   pub struct HealthCheckRunner {
       checks: Vec<Box<dyn HealthCheck>>,
   }
   
   impl HealthCheckRunner {
       pub fn new() -> Self {
           Self {
               checks: Vec::new(),
           }
       }
       
       /// Add a health check
       pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
           self.checks.push(check);
       }
       
       /// Run all checks
       pub async fn run_all(&self) -> Vec<HealthCheckResult> {
           let mut results = Vec::new();
           
           for check in &self.checks {
               results.push(check.check().await);
           }
           
           results
       }
       
       /// Run only critical checks
       pub async fn run_critical(&self) -> Vec<HealthCheckResult> {
           let mut results = Vec::new();
           
           for check in &self.checks {
               if check.is_critical() {
                   results.push(check.check().await);
               }
           }
           
           results
       }
       
       /// Get overall health status
       pub fn overall_status(results: &[HealthCheckResult]) -> HealthStatus {
           let has_error = results.iter().any(|r| matches!(r.status, HealthStatus::Error(_)));
           let has_warning = results.iter().any(|r| matches!(r.status, HealthStatus::Warning(_)));
           
           if has_error {
               HealthStatus::Error("One or more checks failed".into())
           } else if has_warning {
               HealthStatus::Warning("Some checks have warnings".into())
           } else {
               HealthStatus::Healthy
           }
       }
   }
   ```

**Testing Your Implementation**:
```rust
#[tokio::test]
async fn test_git_check() {
    let check = GitInstalledCheck;
    let result = check.check().await;
    
    // Should pass on dev machines
    assert!(matches!(
        result.status, 
        HealthStatus::Healthy | HealthStatus::Warning(_)
    ));
}

#[tokio::test]
async fn test_health_runner() {
    let mut runner = HealthCheckRunner::new();
    runner.add_check(Box::new(GitInstalledCheck));
    
    let results = runner.run_all().await;
    assert!(!results.is_empty());
}
```

**Debugging Guide**:

**Issue**: Async trait not working
**Solution**: Make sure to add `#[async_trait]` attribute

**Issue**: Command execution hangs
**Solution**: Add timeout to command execution

**Visual Health Status**:
```
Health Check Report
─────────────────────────────────────────
✓ Git Installation          [system]     OK (15ms)
✓ Profile Directory         [config]     OK (2ms)
⚠ 1Password CLI            [integration] Warning: Not authenticated
✗ SSH Agent                [integration] Error: Not running

Overall Status: DEGRADED
3 checks passed, 1 warning, 1 error
```

**When You're Stuck**:
1. Check examples: `examples/health/basic_checks.rs`
2. Test individual checks in isolation
3. Add debug logging to check methods
4. Ask in Slack: #rust-diagnostics

#### Task 5B.1.2: Configuration Health Checks (8 hours)

💡 **Junior Dev Concept**: Configuration Validation
**What it is**: Verifying user configurations are valid and usable
**Why important**: Bad configs are a major source of issues
**Examples**: Missing files, invalid JSON, permission errors

**Implementation**:

1. **Profile Configuration Checks** (4 hours)
   ```rust
   // src/health/config.rs
   
   /// Check profile directory exists and is accessible
   pub struct ProfileDirectoryCheck {
       profile_dir: PathBuf,
   }
   
   #[async_trait]
   impl HealthCheck for ProfileDirectoryCheck {
       fn name(&self) -> &str {
           "Profile Directory"
       }
       
       fn category(&self) -> &str {
           "configuration"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = std::time::Instant::now();
           
           let status = if !self.profile_dir.exists() {
               HealthStatus::Error("Profile directory does not exist".into())
           } else if !self.profile_dir.is_dir() {
               HealthStatus::Error("Profile path is not a directory".into())
           } else {
               // Check permissions
               match fs::read_dir(&self.profile_dir) {
                   Ok(_) => HealthStatus::Healthy,
                   Err(e) => HealthStatus::Error(format!("Cannot read directory: {}", e)),
               }
           };
           
           HealthCheckResult {
               name: self.name().to_string(),
               category: self.category().to_string(),
               status,
               message: format!("Checking {}", self.profile_dir.display()),
               duration: start.elapsed(),
               fix_suggestion: match &status {
                   HealthStatus::Error(_) => Some(format!(
                       "Create directory: mkdir -p {}",
                       self.profile_dir.display()
                   )),
                   _ => None,
               },
           }
       }
   }
   
   /// Validate all profiles are properly formatted
   pub struct ProfileValidationCheck {
       profile_manager: Arc<dyn ProfileManager>,
   }
   
   #[async_trait]
   impl HealthCheck for ProfileValidationCheck {
       fn name(&self) -> &str {
           "Profile Validation"
       }
       
       fn category(&self) -> &str {
           "configuration"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = std::time::Instant::now();
           let mut errors = Vec::new();
           let mut warnings = Vec::new();
           
           // Get all profiles
           match self.profile_manager.list().await {
               Ok(profiles) => {
                   for profile_name in profiles {
                       match self.profile_manager.get(&profile_name).await {
                           Ok(profile) => {
                               // Validate profile fields
                               if profile.git.user_email.is_empty() {
                                   errors.push(format!("{}: missing email", profile_name));
                               } else if !is_valid_email(&profile.git.user_email) {
                                   warnings.push(format!("{}: invalid email format", profile_name));
                               }
                               
                               if profile.git.user_name.is_empty() {
                                   errors.push(format!("{}: missing name", profile_name));
                               }
                           }
                           Err(e) => {
                               errors.push(format!("{}: {}", profile_name, e));
                           }
                       }
                   }
               }
               Err(e) => {
                   return HealthCheckResult {
                       name: self.name().to_string(),
                       category: self.category().to_string(),
                       status: HealthStatus::Error(format!("Cannot list profiles: {}", e)),
                       message: "Failed to access profiles".to_string(),
                       duration: start.elapsed(),
                       fix_suggestion: Some("Check profile directory permissions".into()),
                   };
               }
           }
           
           let status = if !errors.is_empty() {
               HealthStatus::Error(format!("{} errors found", errors.len()))
           } else if !warnings.is_empty() {
               HealthStatus::Warning(format!("{} warnings found", warnings.len()))
           } else {
               HealthStatus::Healthy
           };
           
           HealthCheckResult {
               name: self.name().to_string(),
               category: self.category().to_string(),
               status,
               message: if errors.is_empty() && warnings.is_empty() {
                   "All profiles valid".to_string()
               } else {
                   format!("Issues: {}", errors.join(", "))
               },
               duration: start.elapsed(),
               fix_suggestion: if !errors.is_empty() {
                   Some("Run 'git-setup profile edit' to fix issues".into())
               } else {
                   None
               },
           }
       }
   }
   ```

2. **Git Configuration Checks** (4 hours)
   ```rust
   /// Check Git global configuration
   pub struct GitConfigCheck;
   
   #[async_trait]
   impl HealthCheck for GitConfigCheck {
       fn name(&self) -> &str {
           "Git Configuration"
       }
       
       fn category(&self) -> &str {
           "configuration"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = std::time::Instant::now();
           let mut issues = Vec::new();
           
           // Check global user.name
           match Command::new("git")
               .args(&["config", "--global", "user.name"])
               .output()
           {
               Ok(output) => {
                   if !output.status.success() || output.stdout.is_empty() {
                       issues.push("Global user.name not set");
                   }
               }
               Err(_) => issues.push("Cannot check Git config"),
           }
           
           // Check global user.email
           match Command::new("git")
               .args(&["config", "--global", "user.email"])
               .output()
           {
               Ok(output) => {
                   if !output.status.success() || output.stdout.is_empty() {
                       issues.push("Global user.email not set");
                   }
               }
               Err(_) => issues.push("Cannot check Git config"),
           }
           
           let status = if issues.is_empty() {
               HealthStatus::Healthy
           } else {
               HealthStatus::Warning(issues.join(", "))
           };
           
           HealthCheckResult {
               name: self.name().to_string(),
               category: self.category().to_string(),
               status,
               message: "Checking Git global configuration".to_string(),
               duration: start.elapsed(),
               fix_suggestion: if !issues.is_empty() {
                   Some("Set up a default profile with 'git-setup profile use --global'".into())
               } else {
                   None
               },
           }
       }
   }
   ```

---

### 🛑 CHECKPOINT 6.1: Basic Health System Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 16 hours + 4 hours review = 20 hours total

**Pre-Checkpoint Checklist**:
- [ ] Health check trait implemented
- [ ] System checks (Git, directories) working
- [ ] Configuration validation complete
- [ ] Runner executes all checks
- [ ] Results properly structured
- [ ] Fix suggestions included

**Review Focus**:
- Extensible design for new checks
- Clear error messages
- Actionable fix suggestions

---

### 5B.2 Integration Health Checks (24 hours)

#### Task 5B.2.1: External Service Checks (12 hours)

💡 **Junior Dev Concept**: Integration Testing
**What it is**: Verifying external services we depend on work
**Challenge**: Services might be slow or unavailable
**Solution**: Timeouts and graceful degradation

**Visual Integration Status**:
```
External Services Health
────────────────────────────────────────
Service          Status    Response Time
────────────────────────────────────────
1Password CLI    ✓ OK      125ms
SSH Agent        ✓ OK      12ms  
GPG Agent        ⚠ Warning Not running
GitHub API       ✓ OK      245ms
Network          ✓ OK      8ms
────────────────────────────────────────
```

**Implementation**:

1. **1Password Integration Check** (4 hours)
   ```rust
   // src/health/integrations.rs
   
   /// Check 1Password CLI availability and authentication
   pub struct OnePasswordCheck {
       op_cli: Arc<OnePasswordCli>,
   }
   
   #[async_trait]
   impl HealthCheck for OnePasswordCheck {
       fn name(&self) -> &str {
           "1Password CLI"
       }
       
       fn category(&self) -> &str {
           "integration"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = std::time::Instant::now();
           
           // First check if CLI is installed
           match Command::new("op").arg("--version").output() {
               Ok(output) if output.status.success() => {
                   // CLI exists, check authentication
                   match self.op_cli.whoami().await {
                       Ok(account) => HealthCheckResult {
                           name: self.name().to_string(),
                           category: self.category().to_string(),
                           status: HealthStatus::Healthy,
                           message: format!("Authenticated as {}", account),
                           duration: start.elapsed(),
                           fix_suggestion: None,
                       },
                       Err(_) => HealthCheckResult {
                           name: self.name().to_string(),
                           category: self.category().to_string(),
                           status: HealthStatus::Warning("Not authenticated".into()),
                           message: "1Password CLI installed but not authenticated".to_string(),
                           duration: start.elapsed(),
                           fix_suggestion: Some("Run 'op signin' to authenticate".into()),
                       },
                   }
               }
               _ => HealthCheckResult {
                   name: self.name().to_string(),
                   category: self.category().to_string(),
                   status: HealthStatus::Error("Not installed".into()),
                   message: "1Password CLI not found".to_string(),
                   duration: start.elapsed(),
                   fix_suggestion: Some(
                       "Install from https://developer.1password.com/docs/cli/get-started"
                           .into()
                   ),
               },
           }
       }
   }
   ```

2. **SSH Agent Check** (4 hours)
   ```rust
   /// Check SSH agent is running and has keys
   pub struct SshAgentCheck;
   
   #[async_trait]
   impl HealthCheck for SshAgentCheck {
       fn name(&self) -> &str {
           "SSH Agent"
       }
       
       fn category(&self) -> &str {
           "integration"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = std::time::Instant::now();
           
           // Check SSH_AUTH_SOCK environment variable
           if std::env::var("SSH_AUTH_SOCK").is_err() {
               return HealthCheckResult {
                   name: self.name().to_string(),
                   category: self.category().to_string(),
                   status: HealthStatus::Error("Not running".into()),
                   message: "SSH agent not detected".to_string(),
                   duration: start.elapsed(),
                   fix_suggestion: Some("Start SSH agent with 'eval $(ssh-agent)'".into()),
               };
           }
           
           // Check for loaded keys
           match Command::new("ssh-add").arg("-l").output() {
               Ok(output) => {
                   if output.status.success() {
                       let key_count = output.stdout.lines().count();
                       HealthCheckResult {
                           name: self.name().to_string(),
                           category: self.category().to_string(),
                           status: HealthStatus::Healthy,
                           message: format!("{} keys loaded", key_count),
                           duration: start.elapsed(),
                           fix_suggestion: None,
                       }
                   } else {
                       let stderr = String::from_utf8_lossy(&output.stderr);
                       if stderr.contains("The agent has no identities") {
                           HealthCheckResult {
                               name: self.name().to_string(),
                               category: self.category().to_string(),
                               status: HealthStatus::Warning("No keys loaded".into()),
                               message: "SSH agent running but no keys loaded".to_string(),
                               duration: start.elapsed(),
                               fix_suggestion: Some("Add keys with 'ssh-add'".into()),
                           }
                       } else {
                           HealthCheckResult {
                               name: self.name().to_string(),
                               category: self.category().to_string(),
                               status: HealthStatus::Error("Agent error".into()),
                               message: stderr.trim().to_string(),
                               duration: start.elapsed(),
                               fix_suggestion: None,
                           }
                       }
                   }
               }
               Err(e) => HealthCheckResult {
                   name: self.name().to_string(),
                   category: self.category().to_string(),
                   status: HealthStatus::Error("Cannot check agent".into()),
                   message: e.to_string(),
                   duration: start.elapsed(),
                   fix_suggestion: Some("Ensure ssh-add is in PATH".into()),
               },
           }
       }
   }
   ```

3. **Network Connectivity Check** (4 hours)
   ```rust
   /// Check network connectivity to common Git hosts
   pub struct NetworkCheck {
       hosts: Vec<(&'static str, u16)>, // (hostname, port)
   }
   
   impl Default for NetworkCheck {
       fn default() -> Self {
           Self {
               hosts: vec![
                   ("github.com", 443),
                   ("gitlab.com", 443),
                   ("bitbucket.org", 443),
               ],
           }
       }
   }
   
   #[async_trait]
   impl HealthCheck for NetworkCheck {
       fn name(&self) -> &str {
           "Network Connectivity"
       }
       
       fn category(&self) -> &str {
           "integration"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = std::time::Instant::now();
           let mut failed_hosts = Vec::new();
           
           for (host, port) in &self.hosts {
               let addr = format!("{}:{}", host, port);
               match tokio::time::timeout(
                   Duration::from_secs(5),
                   TcpStream::connect(&addr)
               ).await {
                   Ok(Ok(_)) => {
                       // Connected successfully
                   }
                   _ => {
                       failed_hosts.push(*host);
                   }
               }
           }
           
           let status = if failed_hosts.is_empty() {
               HealthStatus::Healthy
           } else if failed_hosts.len() < self.hosts.len() {
               HealthStatus::Warning(format!(
                   "Cannot reach: {}",
                   failed_hosts.join(", ")
               ))
           } else {
               HealthStatus::Error("No network connectivity".into())
           };
           
           HealthCheckResult {
               name: self.name().to_string(),
               category: self.category().to_string(),
               status,
               message: "Checking connectivity to Git hosts".to_string(),
               duration: start.elapsed(),
               fix_suggestion: if !failed_hosts.is_empty() {
                   Some("Check network connection and firewall settings".into())
               } else {
                   None
               },
           }
       }
   }
   ```

---

#### Task 5B.2.2: Performance Monitoring (12 hours)

💡 **Junior Dev Concept**: Performance Baselines
**What it is**: Measuring how fast operations should be
**Why monitor**: Slow operations frustrate users
**Goal**: Detect performance degradation early

**Implementation**:

1. **Performance Check Framework** (6 hours)
   ```rust
   // src/health/performance.rs
   
   /// Check performance of common operations
   pub struct PerformanceCheck {
       operations: Vec<Box<dyn PerformanceTest>>,
   }
   
   #[async_trait]
   pub trait PerformanceTest: Send + Sync {
       /// Name of the operation
       fn name(&self) -> &str;
       
       /// Expected duration (for baseline)
       fn expected_duration(&self) -> Duration;
       
       /// Run the performance test
       async fn run(&self) -> Result<Duration, Box<dyn Error>>;
   }
   
   /// Test profile loading performance
   pub struct ProfileLoadTest {
       profile_manager: Arc<dyn ProfileManager>,
   }
   
   #[async_trait]
   impl PerformanceTest for ProfileLoadTest {
       fn name(&self) -> &str {
           "Profile Loading"
       }
       
       fn expected_duration(&self) -> Duration {
           Duration::from_millis(50) // Should be fast
       }
       
       async fn run(&self) -> Result<Duration, Box<dyn Error>> {
           let start = Instant::now();
           
           // Load all profiles
           let profiles = self.profile_manager.list().await?;
           for name in profiles.iter().take(5) {
               self.profile_manager.get(name).await?;
           }
           
           Ok(start.elapsed())
       }
   }
   
   #[async_trait]
   impl HealthCheck for PerformanceCheck {
       fn name(&self) -> &str {
           "Performance"
       }
       
       fn category(&self) -> &str {
           "performance"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = Instant::now();
           let mut slow_operations = Vec::new();
           
           for test in &self.operations {
               match test.run().await {
                   Ok(duration) => {
                       let expected = test.expected_duration();
                       if duration > expected * 2 {
                           slow_operations.push(format!(
                               "{}: {}ms (expected <{}ms)",
                               test.name(),
                               duration.as_millis(),
                               expected.as_millis()
                           ));
                       }
                   }
                   Err(e) => {
                       slow_operations.push(format!("{}: failed - {}", test.name(), e));
                   }
               }
           }
           
           let status = if slow_operations.is_empty() {
               HealthStatus::Healthy
           } else {
               HealthStatus::Warning(format!(
                   "{} slow operations",
                   slow_operations.len()
               ))
           };
           
           HealthCheckResult {
               name: self.name().to_string(),
               category: self.category().to_string(),
               status,
               message: if slow_operations.is_empty() {
                   "All operations within expected performance".to_string()
               } else {
                   format!("Slow: {}", slow_operations.join(", "))
               },
               duration: start.elapsed(),
               fix_suggestion: if !slow_operations.is_empty() {
                   Some("Check system resources and disk space".into())
               } else {
                   None
               },
           }
       }
   }
   ```

2. **Resource Usage Monitoring** (6 hours)
   ```rust
   /// Monitor system resources
   pub struct ResourceCheck;
   
   #[async_trait]
   impl HealthCheck for ResourceCheck {
       fn name(&self) -> &str {
           "System Resources"
       }
       
       fn category(&self) -> &str {
           "performance"
       }
       
       async fn check(&self) -> HealthCheckResult {
           let start = Instant::now();
           let mut warnings = Vec::new();
           
           // Check disk space for profile directory
           if let Ok(home) = dirs::home_dir() {
               let profile_dir = home.join(".config/git-setup");
               
               #[cfg(unix)]
               {
                   use nix::sys::statvfs::statvfs;
                   
                   if let Ok(stat) = statvfs(&profile_dir) {
                       let available = stat.blocks_available() * stat.block_size();
                       let total = stat.blocks() * stat.block_size();
                       let percent_free = (available as f64 / total as f64) * 100.0;
                       
                       if percent_free < 5.0 {
                           warnings.push(format!("Low disk space: {:.1}% free", percent_free));
                       }
                   }
               }
           }
           
           // Check memory usage (simplified)
           if let Ok(mem_info) = sys_info::mem_info() {
               let percent_used = (mem_info.total - mem_info.avail) as f64 
                   / mem_info.total as f64 * 100.0;
               
               if percent_used > 90.0 {
                   warnings.push(format!("High memory usage: {:.1}%", percent_used));
               }
           }
           
           let status = if warnings.is_empty() {
               HealthStatus::Healthy
           } else {
               HealthStatus::Warning(warnings.join(", "))
           };
           
           HealthCheckResult {
               name: self.name().to_string(),
               category: self.category().to_string(),
               status,
               message: "Checking system resources".to_string(),
               duration: start.elapsed(),
               fix_suggestion: if !warnings.is_empty() {
                   Some("Free up system resources".into())
               } else {
                   None
               },
           }
       }
   }
   ```

---

### 🛑 CHECKPOINT 6.2: Integration Health Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 24 hours + 4 hours review = 28 hours total

**Pre-Checkpoint Checklist**:
- [ ] External service checks implemented
- [ ] Performance monitoring working
- [ ] Resource usage tracked
- [ ] Timeout handling correct
- [ ] All integration tests pass

**Review Focus**:
- Graceful handling of service failures
- Performance baselines reasonable
- Resource monitoring accurate

---

## Week 2: Advanced Diagnostics and Reporting

### 5B.3 Advanced Diagnostics System (20 hours)

#### Task 5B.3.1: Diagnostic Information Collection (10 hours)

💡 **Junior Dev Concept**: Diagnostic Reports
**What it is**: Detailed system information for troubleshooting
**Why needed**: Helps debug user issues remotely
**Privacy**: Never include sensitive data

**Visual Diagnostic Report**:
```
Git Setup Diagnostic Report
Generated: 2024-01-20 14:32:15
────────────────────────────────────────────

SYSTEM INFORMATION
├─ OS: macOS 13.5.2 (arm64)
├─ Shell: zsh 5.9
├─ Terminal: iTerm2 3.4.19
└─ Locale: en_US.UTF-8

GIT INFORMATION
├─ Version: 2.42.0
├─ Config Scope: global
├─ Core Editor: vim
└─ Credential Helper: osxkeychain

APPLICATION STATE
├─ Version: git-setup 1.0.0
├─ Config Dir: /Users/alice/.config/git-setup
├─ Profiles: 3 (work, personal, opensource)
└─ Active Profile: work

RECENT OPERATIONS
├─ [2024-01-20 14:30:05] Profile switched: personal -> work
├─ [2024-01-20 14:25:12] Health check run: All passed
└─ [2024-01-20 14:20:33] Profile created: opensource
```

**Implementation**:

1. **Diagnostic Data Collector** (5 hours)
   ```rust
   // src/health/diagnostics.rs
   
   use serde::{Deserialize, Serialize};
   use std::collections::HashMap;
   
   /// Complete diagnostic information
   #[derive(Debug, Serialize, Deserialize)]
   pub struct DiagnosticReport {
       /// When the report was generated
       pub timestamp: DateTime<Utc>,
       
       /// System information
       pub system: SystemInfo,
       
       /// Git configuration
       pub git: GitInfo,
       
       /// Application state
       pub app: AppInfo,
       
       /// Health check results
       pub health: Vec<HealthCheckResult>,
       
       /// Recent operations log
       pub recent_operations: Vec<OperationLog>,
       
       /// Performance metrics
       pub performance: PerformanceMetrics,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct SystemInfo {
       pub os: String,
       pub os_version: String,
       pub architecture: String,
       pub shell: Option<String>,
       pub terminal: Option<String>,
       pub locale: String,
       pub home_dir: PathBuf,
   }
   
   /// Collector for diagnostic information
   pub struct DiagnosticCollector {
       health_runner: HealthCheckRunner,
       operation_log: Arc<OperationLog>,
   }
   
   impl DiagnosticCollector {
       /// Collect all diagnostic information
       pub async fn collect(&self) -> Result<DiagnosticReport, DiagnosticError> {
           Ok(DiagnosticReport {
               timestamp: Utc::now(),
               system: self.collect_system_info()?,
               git: self.collect_git_info().await?,
               app: self.collect_app_info().await?,
               health: self.health_runner.run_all().await,
               recent_operations: self.operation_log.get_recent(20).await,
               performance: self.collect_performance_metrics().await?,
           })
       }
       
       /// Collect system information
       fn collect_system_info(&self) -> Result<SystemInfo, DiagnosticError> {
           Ok(SystemInfo {
               os: std::env::consts::OS.to_string(),
               os_version: sys_info::os_release()
                   .unwrap_or_else(|_| "unknown".to_string()),
               architecture: std::env::consts::ARCH.to_string(),
               shell: std::env::var("SHELL").ok(),
               terminal: std::env::var("TERM_PROGRAM").ok(),
               locale: std::env::var("LANG")
                   .unwrap_or_else(|_| "unknown".to_string()),
               home_dir: dirs::home_dir()
                   .ok_or(DiagnosticError::NoHomeDir)?,
           })
       }
       
       /// Collect Git information
       async fn collect_git_info(&self) -> Result<GitInfo, DiagnosticError> {
           let version = Command::new("git")
               .args(&["--version"])
               .output()
               .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
               .unwrap_or_else(|_| "unknown".to_string());
           
           let config_scope = self.detect_git_config_scope().await;
           let core_editor = self.get_git_config("core.editor").await;
           let credential_helper = self.get_git_config("credential.helper").await;
           
           Ok(GitInfo {
               version,
               config_scope,
               core_editor,
               credential_helper,
           })
       }
   }
   ```

2. **Privacy-Safe Reporting** (5 hours)
   ```rust
   /// Sanitize diagnostic report for sharing
   pub struct ReportSanitizer;
   
   impl ReportSanitizer {
       /// Remove sensitive information from report
       pub fn sanitize(report: &mut DiagnosticReport) {
           // Sanitize paths to use ~ for home
           if let Ok(home) = dirs::home_dir() {
               Self::sanitize_paths(&mut report.system.home_dir, &home);
               Self::sanitize_paths(&mut report.app.config_dir, &home);
           }
           
           // Remove sensitive environment info
           report.system.shell = report.system.shell.as_ref()
               .map(|s| s.split('/').last().unwrap_or("unknown").to_string());
           
           // Sanitize profile information
           for health_result in &mut report.health {
               if health_result.category == "configuration" {
                   // Remove email addresses from messages
                   health_result.message = Self::redact_emails(&health_result.message);
               }
           }
           
           // Redact operation details
           for op in &mut report.recent_operations {
               op.details = Self::redact_sensitive(&op.details);
           }
       }
       
       /// Redact email addresses
       fn redact_emails(text: &str) -> String {
           let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
               .unwrap();
           email_regex.replace_all(text, "[REDACTED_EMAIL]").to_string()
       }
       
       /// Replace home directory with ~
       fn sanitize_paths(path: &mut PathBuf, home: &Path) {
           if let Ok(relative) = path.strip_prefix(home) {
               *path = PathBuf::from("~").join(relative);
           }
       }
   }
   
   /// Export formats for diagnostic reports
   pub enum ExportFormat {
       Json,
       Yaml,
       Markdown,
       PlainText,
   }
   
   impl DiagnosticReport {
       /// Export report in specified format
       pub fn export(&self, format: ExportFormat) -> Result<String, DiagnosticError> {
           match format {
               ExportFormat::Json => {
                   serde_json::to_string_pretty(self)
                       .map_err(|e| DiagnosticError::Serialization(e.to_string()))
               }
               ExportFormat::Yaml => {
                   serde_yaml::to_string(self)
                       .map_err(|e| DiagnosticError::Serialization(e.to_string()))
               }
               ExportFormat::Markdown => Ok(self.to_markdown()),
               ExportFormat::PlainText => Ok(self.to_plain_text()),
           }
       }
       
       /// Generate markdown report
       fn to_markdown(&self) -> String {
           let mut md = String::new();
           
           md.push_str("# Git Setup Diagnostic Report\n\n");
           md.push_str(&format!("Generated: {}\n\n", self.timestamp));
           
           md.push_str("## System Information\n\n");
           md.push_str(&format!("- **OS**: {} {}\n", self.system.os, self.system.os_version));
           md.push_str(&format!("- **Architecture**: {}\n", self.system.architecture));
           if let Some(shell) = &self.system.shell {
               md.push_str(&format!("- **Shell**: {}\n", shell));
           }
           md.push_str("\n");
           
           md.push_str("## Health Check Results\n\n");
           md.push_str("| Check | Category | Status | Duration |\n");
           md.push_str("|-------|----------|--------|----------|\n");
           
           for result in &self.health {
               let status_emoji = match result.status {
                   HealthStatus::Healthy => "✅",
                   HealthStatus::Warning(_) => "⚠️",
                   HealthStatus::Error(_) => "❌",
                   HealthStatus::Skipped(_) => "⏭️",
               };
               
               md.push_str(&format!(
                   "| {} | {} | {} {} | {}ms |\n",
                   result.name,
                   result.category,
                   status_emoji,
                   match &result.status {
                       HealthStatus::Healthy => "Healthy",
                       HealthStatus::Warning(_) => "Warning",
                       HealthStatus::Error(_) => "Error",
                       HealthStatus::Skipped(_) => "Skipped",
                   },
                   result.duration.as_millis()
               ));
           }
           
           md
       }
   }
   ```

---

#### Task 5B.3.2: Self-Healing Mechanisms (10 hours)

💡 **Junior Dev Concept**: Self-Healing Systems
**What it is**: Automatically fixing common problems
**Why valuable**: Reduces support burden
**Important**: Always ask before making changes

**Implementation**:

1. **Auto-Fix Framework** (5 hours)
   ```rust
   // src/health/autofix.rs
   
   /// Trait for health checks that can fix themselves
   #[async_trait]
   pub trait AutoFixable: HealthCheck {
       /// Check if auto-fix is available
       fn can_auto_fix(&self) -> bool;
       
       /// Attempt to fix the issue
       async fn auto_fix(&self) -> Result<(), Box<dyn Error>>;
       
       /// Description of what the fix will do
       fn fix_description(&self) -> String;
   }
   
   /// Profile directory auto-fix
   pub struct ProfileDirectoryFixable {
       profile_dir: PathBuf,
   }
   
   #[async_trait]
   impl AutoFixable for ProfileDirectoryFixable {
       fn can_auto_fix(&self) -> bool {
           !self.profile_dir.exists()
       }
       
       async fn auto_fix(&self) -> Result<(), Box<dyn Error>> {
           fs::create_dir_all(&self.profile_dir)?;
           
           // Set proper permissions on Unix
           #[cfg(unix)]
           {
               use std::os::unix::fs::PermissionsExt;
               let permissions = fs::Permissions::from_mode(0o700);
               fs::set_permissions(&self.profile_dir, permissions)?;
           }
           
           Ok(())
       }
       
       fn fix_description(&self) -> String {
           format!("Create profile directory at {}", self.profile_dir.display())
       }
   }
   
   /// Auto-fix runner with user confirmation
   pub struct AutoFixRunner {
       fixes: Vec<Box<dyn AutoFixable>>,
       ui: Box<dyn ConfirmationUI>,
   }
   
   impl AutoFixRunner {
       /// Run health checks and offer fixes
       pub async fn run_with_fixes(&mut self) -> Result<(), AutoFixError> {
           let mut fixable_issues = Vec::new();
           
           // Run all checks
           for fix in &self.fixes {
               let result = fix.check().await;
               
               if matches!(result.status, HealthStatus::Error(_)) && fix.can_auto_fix() {
                   fixable_issues.push((fix.name(), fix.fix_description()));
               }
           }
           
           if fixable_issues.is_empty() {
               println!("✅ No issues found that can be auto-fixed");
               return Ok(());
           }
           
           // Show issues and ask for confirmation
           println!("\n🔧 Found {} issues that can be auto-fixed:", fixable_issues.len());
           for (name, description) in &fixable_issues {
               println!("  - {}: {}", name, description);
           }
           
           if self.ui.confirm("\nApply all fixes?", true)? {
               for fix in &self.fixes {
                   if fix.can_auto_fix() {
                       println!("Fixing {}...", fix.name());
                       fix.auto_fix().await?;
                   }
               }
               println!("✅ All fixes applied successfully");
           }
           
           Ok(())
       }
   }
   ```

2. **Common Auto-Fixes** (5 hours)
   ```rust
   /// Fix SSH agent not running
   pub struct SshAgentAutoFix;
   
   #[async_trait]
   impl AutoFixable for SshAgentAutoFix {
       fn can_auto_fix(&self) -> bool {
           std::env::var("SSH_AUTH_SOCK").is_err()
       }
       
       async fn auto_fix(&self) -> Result<(), Box<dyn Error>> {
           // Start SSH agent
           let output = Command::new("ssh-agent")
               .arg("-s")
               .output()?;
           
           if output.status.success() {
               // Parse output to set environment variables
               let agent_output = String::from_utf8_lossy(&output.stdout);
               
               for line in agent_output.lines() {
                   if line.starts_with("SSH_AUTH_SOCK=") {
                       let parts: Vec<&str> = line.split('=').collect();
                       if parts.len() >= 2 {
                           let value = parts[1].trim_end_matches(';');
                           std::env::set_var("SSH_AUTH_SOCK", value);
                       }
                   }
               }
               
               Ok(())
           } else {
               Err("Failed to start SSH agent".into())
           }
       }
       
       fn fix_description(&self) -> String {
           "Start SSH agent".to_string()
       }
   }
   
   /// Fix Git global config
   pub struct GitConfigAutoFix {
       profile_manager: Arc<dyn ProfileManager>,
   }
   
   #[async_trait]
   impl AutoFixable for GitConfigAutoFix {
       fn can_auto_fix(&self) -> bool {
           // Check if we have at least one profile to use
           true
       }
       
       async fn auto_fix(&self) -> Result<(), Box<dyn Error>> {
           // Get first available profile
           let profiles = self.profile_manager.list().await?;
           if let Some(profile_name) = profiles.first() {
               let profile = self.profile_manager.get(profile_name).await?;
               
               // Set global Git config
               Command::new("git")
                   .args(&["config", "--global", "user.name", &profile.git.user_name])
                   .output()?;
               
               Command::new("git")
                   .args(&["config", "--global", "user.email", &profile.git.user_email])
                   .output()?;
               
               Ok(())
           } else {
               Err("No profiles available".into())
           }
       }
       
       fn fix_description(&self) -> String {
           "Set Git global configuration from first available profile".to_string()
       }
   }
   ```

---

### 🛑 CHECKPOINT 6.3: Advanced Diagnostics Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**Workload**: 20 hours + 4 hours review = 24 hours total

**Pre-Checkpoint Checklist**:
- [ ] Diagnostic collection working
- [ ] Privacy sanitization implemented
- [ ] Export formats functional
- [ ] Auto-fix framework complete
- [ ] User confirmation flow working

**Review Focus**:
- No sensitive data in reports
- Auto-fixes are safe
- User control maintained

---

### 5B.4 Reporting and Visualization (20 hours)

#### Task 5B.4.1: CLI Reporting (10 hours)

💡 **Junior Dev Concept**: User-Friendly Reports
**What it is**: Presenting technical data clearly
**Challenge**: Making errors understandable
**Solution**: Progressive disclosure

**Implementation**:

1. **Report Formatter** (5 hours)
   ```rust
   // src/health/reporting.rs
   
   use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
   
   /// Format health check results for CLI display
   pub struct CliReporter {
       stdout: StandardStream,
       verbose: bool,
   }
   
   impl CliReporter {
       pub fn new(verbose: bool) -> Self {
           Self {
               stdout: StandardStream::stdout(ColorChoice::Auto),
               verbose,
           }
       }
       
       /// Display health check results
       pub fn report(&mut self, results: &[HealthCheckResult]) -> io::Result<()> {
           // Header
           self.write_header("Health Check Report")?;
           
           // Summary
           let (healthy, warnings, errors) = self.count_statuses(results);
           self.write_summary(healthy, warnings, errors)?;
           
           // Group by category
           let mut by_category: HashMap<String, Vec<&HealthCheckResult>> = HashMap::new();
           for result in results {
               by_category.entry(result.category.clone())
                   .or_default()
                   .push(result);
           }
           
           // Display each category
           for (category, results) in by_category {
               self.write_category(&category, &results)?;
           }
           
           // Show fix suggestions if any
           self.write_fix_suggestions(results)?;
           
           Ok(())
       }
       
       fn write_header(&mut self, title: &str) -> io::Result<()> {
           writeln!(&mut self.stdout)?;
           self.stdout.set_color(ColorSpec::new().set_bold(true))?;
           writeln!(&mut self.stdout, "{}", title)?;
           self.stdout.reset()?;
           writeln!(&mut self.stdout, "{}", "─".repeat(40))?;
           Ok(())
       }
       
       fn write_summary(&mut self, healthy: usize, warnings: usize, errors: usize) -> io::Result<()> {
           write!(&mut self.stdout, "Overall Status: ")?;
           
           if errors > 0 {
               self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
               writeln!(&mut self.stdout, "UNHEALTHY")?;
           } else if warnings > 0 {
               self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
               writeln!(&mut self.stdout, "DEGRADED")?;
           } else {
               self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
               writeln!(&mut self.stdout, "HEALTHY")?;
           }
           
           self.stdout.reset()?;
           writeln!(&mut self.stdout, "{} passed, {} warnings, {} errors", healthy, warnings, errors)?;
           writeln!(&mut self.stdout)?;
           
           Ok(())
       }
       
       fn write_category(&mut self, category: &str, results: &[&HealthCheckResult]) -> io::Result<()> {
           // Category header
           self.stdout.set_color(ColorSpec::new().set_bold(true))?;
           writeln!(&mut self.stdout, "{}:", category.to_uppercase())?;
           self.stdout.reset()?;
           
           // Results
           for result in results {
               self.write_result(result)?;
           }
           writeln!(&mut self.stdout)?;
           
           Ok(())
       }
       
       fn write_result(&mut self, result: &HealthCheckResult) -> io::Result<()> {
           // Status icon
           let (icon, color) = match &result.status {
               HealthStatus::Healthy => ("✓", Color::Green),
               HealthStatus::Warning(_) => ("⚠", Color::Yellow),
               HealthStatus::Error(_) => ("✗", Color::Red),
               HealthStatus::Skipped(_) => ("⏭", Color::Cyan),
           };
           
           self.stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
           write!(&mut self.stdout, "{} ", icon)?;
           self.stdout.reset()?;
           
           // Check name
           write!(&mut self.stdout, "{:<25}", result.name)?;
           
           // Status message
           match &result.status {
               HealthStatus::Healthy => {
                   self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                   write!(&mut self.stdout, "OK")?;
               }
               HealthStatus::Warning(msg) => {
                   self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                   write!(&mut self.stdout, "Warning: {}", msg)?;
               }
               HealthStatus::Error(msg) => {
                   self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                   write!(&mut self.stdout, "Error: {}", msg)?;
               }
               HealthStatus::Skipped(reason) => {
                   self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                   write!(&mut self.stdout, "Skipped: {}", reason)?;
               }
           }
           
           self.stdout.reset()?;
           
           // Duration in verbose mode
           if self.verbose {
               self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::DarkGray)))?;
               write!(&mut self.stdout, " ({}ms)", result.duration.as_millis())?;
               self.stdout.reset()?;
           }
           
           writeln!(&mut self.stdout)?;
           
           Ok(())
       }
   }
   ```

2. **Interactive Report Browser** (5 hours)
   ```rust
   /// Interactive TUI for browsing health reports
   pub struct InteractiveReporter {
       results: Vec<HealthCheckResult>,
       selected_index: usize,
       filter: Option<String>,
   }
   
   impl InteractiveReporter {
       pub fn new(results: Vec<HealthCheckResult>) -> Self {
           Self {
               results,
               selected_index: 0,
               filter: None,
           }
       }
       
       pub fn render(&self, f: &mut Frame, area: Rect) {
           let chunks = Layout::default()
               .direction(Direction::Vertical)
               .constraints([
                   Constraint::Length(3),  // Header
                   Constraint::Min(10),    // Results
                   Constraint::Length(3),  // Details
                   Constraint::Length(1),  // Help
               ])
               .split(area);
           
           // Header with summary
           self.render_header(f, chunks[0]);
           
           // Results list
           self.render_results_list(f, chunks[1]);
           
           // Selected result details
           self.render_details(f, chunks[2]);
           
           // Help line
           self.render_help(f, chunks[3]);
       }
   }
   ```

---

#### Task 5B.4.2: TUI Health Dashboard (10 hours)

💡 **Junior Dev Concept**: Real-time Dashboards
**What it is**: Live view of system health
**Challenge**: Updating without flickering
**Solution**: Smart rendering with diffs

**Visual Dashboard Mock**:
```
┌─ Git Setup Health Dashboard ─────────────────┐
│                                              │
│ System Health: ● HEALTHY                     │
│                                              │
│ ┌─ Checks (12/12 passed) ─────────────────┐ │
│ │ ✓ Git Installation        2.42.0  15ms  │ │
│ │ ✓ Profile Directory       OK      2ms   │ │
│ │ ✓ Git Configuration       OK      8ms   │ │
│ │ ✓ 1Password CLI          Auth     125ms │ │
│ │ ✓ SSH Agent              3 keys   12ms  │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ ┌─ Recent Activity ────────────────────────┐ │
│ │ 14:32:05 Profile switched to 'work'     │ │
│ │ 14:30:12 Health check completed         │ │
│ │ 14:28:44 Auto-fix applied: SSH Agent    │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ [R]efresh [F]ix Issues [E]xport [Q]uit       │
└──────────────────────────────────────────────┘
```

[Implementation continues with TUI dashboard code...]

---

### 🛑 FINAL CHECKPOINT 6: Health Monitoring Complete

#### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** to Phase 7 without approval.

**Total Phase Duration**: 2 weeks (80 hours)
- Week 1: Basic health checks and framework (40h)
- Week 2: Advanced diagnostics and reporting (40h)

**Final Deliverables**:
- ✅ Extensible health check framework
- ✅ System, config, and integration checks
- ✅ Performance monitoring
- ✅ Diagnostic report generation
- ✅ Privacy-safe export formats
- ✅ Self-healing mechanisms
- ✅ CLI and TUI reporting
- ✅ Complete test coverage

**Success Metrics**:
- All critical checks implemented
- Reports are privacy-safe
- Auto-fixes require confirmation
- Performance impact <50ms
- Zero false positives

**Key Achievements**:
1. **Proactive Issue Detection**: Catches problems before users hit them
2. **Self-Service Debugging**: Users can diagnose their own issues
3. **Privacy-First Design**: Safe to share reports
4. **Extensible Framework**: Easy to add new checks
5. **Great UX**: Clear, actionable feedback

---

## Common Issues and Solutions

### Health Check Issues

**Issue**: Check times out
**Solution**: Add timeout wrapper:
```rust
tokio::time::timeout(Duration::from_secs(5), check.check()).await
```

**Issue**: False positives
**Solution**: Add retry logic for network checks

### Reporting Issues

**Issue**: Terminal colors not working
**Solution**: Check TERM environment variable, use `ColorChoice::Auto`

**Issue**: Report too verbose
**Solution**: Implement progressive disclosure with --verbose flag

### Auto-Fix Issues

**Issue**: Fix fails silently
**Solution**: Always return detailed error messages
**Debug**: Add logging to each fix step

## Summary

Phase 6 provides comprehensive health monitoring that significantly reduces support burden and improves user experience. The implementation includes:

- Extensible health check framework
- Multiple check categories
- Advanced diagnostics
- Privacy-safe reporting
- Self-healing capabilities
- Beautiful CLI and TUI interfaces

The gradual complexity increase and proper checkpoints ensure junior developers can successfully implement this critical feature.

**Next**: Phase 7 - Signing Methods (SSH and GPG Basics)