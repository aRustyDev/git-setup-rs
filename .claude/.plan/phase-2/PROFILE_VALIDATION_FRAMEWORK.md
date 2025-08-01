# Comprehensive Profile Validation Framework

## Overview

This document provides a complete implementation of the Profile Validation Framework for Phase 2, Task 2.1.3, with detailed examples, error handling, and junior developer guidance.

## Why Validation Matters

Profile validation prevents:
- Invalid Git configurations that break repositories
- Security vulnerabilities from malformed data
- User frustration from cryptic Git errors
- Data corruption from partial updates
- Integration failures with external tools

## Validation Architecture

### Multi-Layer Validation

```
┌─────────────────────────────────────────────────┐
│             Input Data (TOML/JSON)              │
└─────────────────────┬───────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────┐
│          1. Syntax Validation                   │
│         (Parse TOML/JSON structure)             │
└─────────────────────┬───────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────┐
│          2. Schema Validation                   │
│      (Required fields, data types)              │
└─────────────────────┬───────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────┐
│          3. Semantic Validation                 │
│    (Business rules, value constraints)          │
└─────────────────────┬───────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────┐
│          4. Cross-Field Validation              │
│      (Dependencies, conflicts)                  │
└─────────────────────┬───────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────┐
│          5. External Validation                 │
│     (File exists, key available)                │
└─────────────────────┬───────────────────────────┘
                      ▼
┌─────────────────────────────────────────────────┐
│            Valid Profile Object                 │
└─────────────────────────────────────────────────┘
```

## Implementation

### Core Validation Trait

```rust
// src/profiles/validation.rs

use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

/// Result of validation with detailed errors
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            suggestion: None,
        });
    }
    
    pub fn add_error_with_suggestion(
        &mut self, 
        field: impl Into<String>, 
        message: impl Into<String>,
        suggestion: impl Into<String>
    ) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            suggestion: Some(suggestion.into()),
        });
    }
    
    pub fn add_warning(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(ValidationWarning {
            field: field.into(),
            message: message.into(),
        });
    }
    
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// Trait for validatable types
pub trait Validate {
    fn validate(&self) -> ValidationResult;
}

/// Context for validation with external dependencies
pub struct ValidationContext {
    /// Check if files exist
    pub check_files: bool,
    /// Check if commands are available
    pub check_commands: bool,
    /// Check network resources
    pub check_network: bool,
    /// Custom validators
    pub custom_validators: HashMap<String, Box<dyn Fn(&dyn std::any::Any) -> ValidationResult>>,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            check_files: true,
            check_commands: true,
            check_network: false,
            custom_validators: HashMap::new(),
        }
    }
}
```

### Profile Validation Implementation

```rust
// Common validation patterns
lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
    
    static ref PROFILE_NAME_REGEX: Regex = Regex::new(
        r"^[a-z0-9][a-z0-9-]*$"
    ).unwrap();
    
    static ref GIT_CONFIG_KEY_REGEX: Regex = Regex::new(
        r"^[a-zA-Z][a-zA-Z0-9.-]*\.[a-zA-Z][a-zA-Z0-9.-]*$"
    ).unwrap();
    
    static ref URL_REGEX: Regex = Regex::new(
        r"^(https?|git|ssh)://[^\s/$.?#].[^\s]*$"
    ).unwrap();
}

impl Validate for Profile {
    fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // 1. Validate profile name
        self.validate_name(&mut result);
        
        // 2. Validate Git configuration
        if let Some(git) = &self.git {
            self.validate_git_config(git, &mut result);
        } else {
            result.add_error("git", "Git configuration is required");
        }
        
        // 3. Validate signing configuration
        if let Some(signing) = &self.signing {
            self.validate_signing_config(signing, &mut result);
        }
        
        // 4. Validate detection rules
        self.validate_detection_rules(&mut result);
        
        // 5. Validate parent profile reference
        if let Some(extends) = &self.extends {
            self.validate_extends(extends, &mut result);
        }
        
        result
    }
}

impl Profile {
    fn validate_name(&self, result: &mut ValidationResult) {
        if self.name.is_empty() {
            result.add_error("name", "Profile name cannot be empty");
            return;
        }
        
        if self.name.len() > 50 {
            result.add_error("name", "Profile name too long (max 50 characters)");
        }
        
        if !PROFILE_NAME_REGEX.is_match(&self.name) {
            result.add_error_with_suggestion(
                "name",
                format!("Invalid profile name '{}'. Must contain only lowercase letters, numbers, and hyphens", self.name),
                format!("Try: {}", self.name.to_lowercase().replace("_", "-").replace(" ", "-"))
            );
        }
        
        // Reserved names
        let reserved = ["default", "global", "system", "local"];
        if reserved.contains(&self.name.as_str()) {
            result.add_error(
                "name",
                format!("'{}' is a reserved profile name", self.name)
            );
        }
    }
    
    fn validate_git_config(&self, git: &GitConfig, result: &mut ValidationResult) {
        // Validate user name
        if git.user_name.is_empty() {
            result.add_error("git.user_name", "Git user name is required");
        } else if git.user_name.len() > 100 {
            result.add_error("git.user_name", "Git user name too long (max 100 characters)");
        } else if git.user_name.trim() != git.user_name {
            result.add_warning("git.user_name", "Git user name has leading/trailing whitespace");
        }
        
        // Validate email
        if git.user_email.is_empty() {
            result.add_error("git.user_email", "Git user email is required");
        } else if !EMAIL_REGEX.is_match(&git.user_email) {
            result.add_error_with_suggestion(
                "git.user_email",
                format!("Invalid email address '{}'", git.user_email),
                "Email must be in format: user@example.com"
            );
        }
        
        // Validate custom Git config
        for (key, value) in &git.extra {
            if !GIT_CONFIG_KEY_REGEX.is_match(key) {
                result.add_error(
                    format!("git.extra.{}", key),
                    "Invalid Git config key format. Must be 'section.key'"
                );
            }
            
            // Validate specific Git config values
            match key.as_str() {
                "core.editor" => {
                    if value.is_empty() {
                        result.add_error("git.extra.core.editor", "Editor cannot be empty");
                    }
                }
                "core.autocrlf" => {
                    let valid_values = ["true", "false", "input"];
                    if !valid_values.contains(&value.as_str()) {
                        result.add_error(
                            "git.extra.core.autocrlf",
                            format!("Invalid value '{}'. Must be one of: true, false, input", value)
                        );
                    }
                }
                _ => {}
            }
        }
    }
    
    fn validate_signing_config(&self, signing: &SigningConfig, result: &mut ValidationResult) {
        match signing.method {
            SigningMethod::Ssh => {
                if signing.ssh_key_ref.is_none() {
                    result.add_error(
                        "signing.ssh_key_ref",
                        "SSH key reference is required for SSH signing"
                    );
                } else if let Some(key_ref) = &signing.ssh_key_ref {
                    self.validate_ssh_key_ref(key_ref, result);
                }
            }
            SigningMethod::Gpg => {
                if signing.gpg_key_id.is_none() {
                    result.add_error(
                        "signing.gpg_key_id",
                        "GPG key ID is required for GPG signing"
                    );
                } else if let Some(key_id) = &signing.gpg_key_id {
                    self.validate_gpg_key_id(key_id, result);
                }
            }
            _ => {
                // X509 and Sigstore validation in Phase 8
            }
        }
    }
    
    fn validate_ssh_key_ref(&self, key_ref: &str, result: &mut ValidationResult) {
        if key_ref.starts_with("op://") {
            // 1Password reference
            let parts: Vec<&str> = key_ref.strip_prefix("op://").unwrap().split('/').collect();
            if parts.len() != 3 {
                result.add_error_with_suggestion(
                    "signing.ssh_key_ref",
                    "Invalid 1Password reference format",
                    "Format: op://vault/item/field"
                );
            }
        } else if key_ref.starts_with("~") || key_ref.starts_with("/") {
            // File path
            // Note: Actual file existence check done in ValidationContext
        } else {
            result.add_error(
                "signing.ssh_key_ref",
                "SSH key must be a file path or 1Password reference (op://...)"
            );
        }
    }
    
    fn validate_gpg_key_id(&self, key_id: &str, result: &mut ValidationResult) {
        // GPG key ID can be:
        // - Short form: 8 hex chars (ABCD1234)
        // - Long form: 16 hex chars
        // - Fingerprint: 40 hex chars
        // - Email address
        
        let hex_lengths = [8, 16, 40];
        let is_hex = key_id.chars().all(|c| c.is_ascii_hexdigit());
        
        if is_hex && hex_lengths.contains(&key_id.len()) {
            // Valid hex key ID
            return;
        }
        
        if EMAIL_REGEX.is_match(key_id) {
            // Email format
            return;
        }
        
        result.add_error_with_suggestion(
            "signing.gpg_key_id",
            format!("Invalid GPG key ID '{}'", key_id),
            "Use key ID (8/16/40 hex chars) or email address"
        );
    }
    
    fn validate_detection_rules(&self, result: &mut ValidationResult) {
        // Path patterns
        for (i, pattern) in self.path_patterns.iter().enumerate() {
            if pattern.is_empty() {
                result.add_error(
                    format!("path_patterns[{}]", i),
                    "Path pattern cannot be empty"
                );
            }
            
            // Validate glob pattern
            if let Err(e) = glob::Pattern::new(pattern) {
                result.add_error(
                    format!("path_patterns[{}]", i),
                    format!("Invalid glob pattern: {}", e)
                );
            }
        }
        
        // Remote patterns
        for (i, remote) in self.remotes.iter().enumerate() {
            if let Some(url) = &remote.url {
                if !URL_REGEX.is_match(url) && !url.contains("*") {
                    result.add_warning(
                        format!("remotes[{}].url", i),
                        "Remote URL doesn't look like a valid Git URL"
                    );
                }
            }
            
            if let Some(host) = &remote.host {
                if host.is_empty() {
                    result.add_error(
                        format!("remotes[{}].host", i),
                        "Remote host cannot be empty"
                    );
                }
            }
        }
    }
    
    fn validate_extends(&self, parent: &str, result: &mut ValidationResult) {
        if parent == self.name {
            result.add_error(
                "extends",
                "Profile cannot extend itself"
            );
        }
        
        if !PROFILE_NAME_REGEX.is_match(parent) {
            result.add_error(
                "extends",
                format!("Invalid parent profile name '{}'", parent)
            );
        }
    }
}
```

### Context-Aware Validation

```rust
impl Profile {
    /// Validate with external checks
    pub fn validate_with_context(&self, ctx: &ValidationContext) -> ValidationResult {
        let mut result = self.validate();
        
        // File existence checks
        if ctx.check_files {
            if let Some(signing) = &self.signing {
                if let Some(ssh_key) = &signing.ssh_key_ref {
                    if !ssh_key.starts_with("op://") {
                        let path = shellexpand::tilde(ssh_key);
                        if !std::path::Path::new(path.as_ref()).exists() {
                            result.add_error(
                                "signing.ssh_key_ref",
                                format!("SSH key file not found: {}", ssh_key)
                            );
                        }
                    }
                }
            }
        }
        
        // Command availability checks
        if ctx.check_commands {
            if let Some(git) = &self.git {
                if let Some(editor) = git.extra.get("core.editor") {
                    if which::which(editor).is_err() {
                        result.add_warning(
                            "git.extra.core.editor",
                            format!("Editor '{}' not found in PATH", editor)
                        );
                    }
                }
            }
            
            if let Some(signing) = &self.signing {
                match signing.method {
                    SigningMethod::Gpg => {
                        if which::which("gpg").is_err() {
                            result.add_error(
                                "signing.method",
                                "GPG is not installed or not in PATH"
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        
        result
    }
}
```

### Validation Error Display

```rust
use colored::*;

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() && self.warnings.is_empty() {
            writeln!(f, "{} Profile validation passed!", "✓".green())?;
            return Ok(());
        }
        
        if !self.errors.is_empty() {
            writeln!(f, "{} {} validation error(s) found:", 
                "✗".red(), 
                self.errors.len()
            )?;
            
            for error in &self.errors {
                writeln!(f, "  {} {}: {}", 
                    "•".red(),
                    error.field.yellow(),
                    error.message
                )?;
                
                if let Some(suggestion) = &error.suggestion {
                    writeln!(f, "    {} {}", 
                        "→".blue(),
                        suggestion.italic()
                    )?;
                }
            }
        }
        
        if !self.warnings.is_empty() {
            writeln!(f, "\n{} {} warning(s):", 
                "⚠".yellow(), 
                self.warnings.len()
            )?;
            
            for warning in &self.warnings {
                writeln!(f, "  {} {}: {}", 
                    "•".yellow(),
                    warning.field,
                    warning.message.dimmed()
                )?;
            }
        }
        
        Ok(())
    }
}
```

### Validation Helpers

```rust
/// Common validation helpers
pub mod validators {
    use super::*;
    
    /// Validate string length
    pub fn validate_length(
        value: &str, 
        min: usize, 
        max: usize, 
        field: &str, 
        result: &mut ValidationResult
    ) {
        if value.len() < min {
            result.add_error(
                field,
                format!("Too short (minimum {} characters)", min)
            );
        } else if value.len() > max {
            result.add_error(
                field,
                format!("Too long (maximum {} characters)", max)
            );
        }
    }
    
    /// Validate against a list of allowed values
    pub fn validate_enum<T: PartialEq + Display>(
        value: &T,
        allowed: &[T],
        field: &str,
        result: &mut ValidationResult
    ) {
        if !allowed.contains(value) {
            let allowed_str = allowed.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            
            result.add_error(
                field,
                format!("Invalid value '{}'. Must be one of: {}", value, allowed_str)
            );
        }
    }
    
    /// Validate URL format
    pub fn validate_url(url: &str, field: &str, result: &mut ValidationResult) {
        if !URL_REGEX.is_match(url) {
            result.add_error(
                field,
                format!("Invalid URL format: {}", url)
            );
        }
    }
    
    /// Validate file path
    pub fn validate_path(path: &str, field: &str, result: &mut ValidationResult) {
        if path.is_empty() {
            result.add_error(field, "Path cannot be empty");
            return;
        }
        
        // Check for dangerous patterns
        if path.contains("..") {
            result.add_error(
                field,
                "Path cannot contain '..' for security reasons"
            );
        }
        
        if path.contains('\0') {
            result.add_error(
                field,
                "Path contains invalid null character"
            );
        }
    }
}
```

## Testing the Validation Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_profile() {
        let profile = Profile {
            name: "work".to_string(),
            git: Some(GitConfig {
                user_name: "Alice Developer".to_string(),
                user_email: "alice@company.com".to_string(),
                extra: HashMap::new(),
            }),
            ..Default::default()
        };
        
        let result = profile.validate();
        assert!(result.is_valid());
        assert!(result.warnings.is_empty());
    }
    
    #[test]
    fn test_invalid_email() {
        let profile = Profile {
            name: "test".to_string(),
            git: Some(GitConfig {
                user_name: "Test".to_string(),
                user_email: "not-an-email".to_string(),
                extra: HashMap::new(),
            }),
            ..Default::default()
        };
        
        let result = profile.validate();
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "git.user_email");
    }
    
    #[test]
    fn test_reserved_name() {
        let profile = Profile {
            name: "default".to_string(),
            git: Some(GitConfig {
                user_name: "Test".to_string(),
                user_email: "test@example.com".to_string(),
                extra: HashMap::new(),
            }),
            ..Default::default()
        };
        
        let result = profile.validate();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.field == "name"));
    }
    
    #[test]
    fn test_ssh_signing_validation() {
        let profile = Profile {
            name: "ssh-test".to_string(),
            git: Some(GitConfig {
                user_name: "Test".to_string(),
                user_email: "test@example.com".to_string(),
                extra: HashMap::new(),
            }),
            signing: Some(SigningConfig {
                method: SigningMethod::Ssh,
                ssh_key_ref: Some("~/.ssh/id_ed25519.pub".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let result = profile.validate();
        assert!(result.is_valid());
    }
    
    #[test]
    fn test_validation_with_context() {
        let profile = Profile {
            name: "context-test".to_string(),
            git: Some(GitConfig {
                user_name: "Test".to_string(),
                user_email: "test@example.com".to_string(),
                extra: {
                    let mut extra = HashMap::new();
                    extra.insert("core.editor".to_string(), "nonexistent-editor".to_string());
                    extra
                },
            }),
            ..Default::default()
        };
        
        let ctx = ValidationContext {
            check_commands: true,
            ..Default::default()
        };
        
        let result = profile.validate_with_context(&ctx);
        assert!(result.warnings.iter().any(|w| w.field == "git.extra.core.editor"));
    }
}
```

## Usage Examples

### Basic Validation

```rust
fn create_profile(name: String, email: String) -> Result<Profile, ValidationResult> {
    let profile = Profile {
        name,
        git: Some(GitConfig {
            user_name: "Developer".to_string(),
            user_email: email,
            extra: HashMap::new(),
        }),
        ..Default::default()
    };
    
    let validation = profile.validate();
    if validation.is_valid() {
        Ok(profile)
    } else {
        Err(validation)
    }
}
```

### Interactive Validation

```rust
fn interactive_profile_creation() -> Result<Profile, Box<dyn Error>> {
    loop {
        print!("Profile name: ");
        io::stdout().flush()?;
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        
        print!("Email: ");
        io::stdout().flush()?;
        let mut email = String::new();
        io::stdin().read_line(&mut email)?;
        
        let profile = Profile {
            name: name.trim().to_string(),
            git: Some(GitConfig {
                user_name: "Developer".to_string(),
                user_email: email.trim().to_string(),
                extra: HashMap::new(),
            }),
            ..Default::default()
        };
        
        let validation = profile.validate();
        if validation.is_valid() {
            return Ok(profile);
        } else {
            println!("\n{}", validation);
            println!("\nPlease fix the errors and try again.\n");
        }
    }
}
```

### Batch Validation

```rust
fn validate_all_profiles(profiles: Vec<Profile>) -> HashMap<String, ValidationResult> {
    profiles.into_iter()
        .map(|p| {
            let name = p.name.clone();
            let result = p.validate();
            (name, result)
        })
        .collect()
}
```

## Best Practices

1. **Validate Early**: Check data as soon as it enters the system
2. **Provide Context**: Give users actionable error messages
3. **Suggest Fixes**: When possible, suggest corrections
4. **Separate Warnings**: Not all issues are errors
5. **Performance**: Cache regex compilation with lazy_static
6. **Extensibility**: Use the Validate trait for custom types
7. **Testing**: Test both valid and invalid cases

## Junior Developer Tips

1. **Start Simple**: Begin with basic field validation
2. **Use Regex Carefully**: Test patterns thoroughly
3. **Error Messages Matter**: Write them for users, not developers
4. **Don't Over-Validate**: Balance strictness with usability
5. **Test Edge Cases**: Empty strings, special characters, etc.
6. **Document Constraints**: Why is a field required?
7. **Consider Context**: Not all validation can be done offline

This comprehensive validation framework ensures data integrity while providing excellent user experience through clear error messages and helpful suggestions.