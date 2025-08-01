# Cross-Phase Integration Guide

## Overview

This guide identifies and addresses integration gaps between phases, providing examples and patterns to ensure smooth component interaction across the entire git-setup-rs system.

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    git-setup-rs Integration Map                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Phase 1: Security Foundation                                   │
│  └─► Provides: SecureFS, SensitiveString, AtomicOps           │
│      Used by: ALL phases                                        │
│                                                                 │
│  Phase 2: Profile Management                                    │
│  └─► Provides: ProfileManager, Validation, Storage             │
│      Depends on: Phase 1 (SecureFS)                           │
│      Used by: Phases 3,4,5,6,7,8                              │
│                                                                 │
│  Phase 3: User Interfaces                                       │
│  └─► Provides: CLI, TUI, Events                               │
│      Depends on: Phases 1,2                                   │
│      Used by: All user-facing operations                      │
│                                                                 │
│  Phase 4: 1Password Integration                                │
│  └─► Provides: Credential Management                          │
│      Depends on: Phases 1,2                                   │
│      Used by: Phases 2,7,8 (signing)                         │
│                                                                 │
│  Phase 5: Pattern Detection                                     │
│  └─► Provides: Auto-detection, Rules                          │
│      Depends on: Phase 2                                      │
│      Used by: Phases 3,6                                     │
│                                                                 │
│  Phase 6: Health Monitoring                                     │
│  └─► Provides: Diagnostics, Auto-fix                          │
│      Depends on: Phases 1,2,4,5                              │
│      Used by: Phase 3 (UI)                                   │
│                                                                 │
│  Phase 7: Basic Signing                                         │
│  └─► Provides: SSH/GPG signing                                │
│      Depends on: Phases 1,2,4                                │
│      Used by: Phase 2 (profiles)                             │
│                                                                 │
│  Phase 8: Advanced Signing                                      │
│  └─► Provides: x509/Sigstore                                  │
│      Depends on: Phases 1,2,4,7                              │
│      Used by: Phase 2 (profiles)                             │
│                                                                 │
│  Phase 9: Platform & Distribution                               │
│  └─► Provides: Cross-platform, Performance                    │
│      Depends on: ALL phases                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Identified Integration Gaps

### Gap 1: Security Context Propagation

**Issue**: Security settings from Phase 1 need to flow through all phases consistently.

**Solution**:

```rust
// src/core/context.rs

use crate::security::{SecureFileSystem, SensitiveString};
use std::sync::Arc;

/// Application-wide security context
#[derive(Clone)]
pub struct SecurityContext {
    /// Secure file system operations
    pub fs: Arc<dyn SecureFileSystem>,
    
    /// Memory protection settings
    pub memory_protection: MemoryProtectionLevel,
    
    /// Audit configuration
    pub audit: AuditConfig,
}

impl SecurityContext {
    pub fn new() -> Self {
        Self {
            fs: Arc::new(DefaultSecureFS::new()),
            memory_protection: MemoryProtectionLevel::default(),
            audit: AuditConfig::default(),
        }
    }
    
    /// Create a restricted context for untrusted operations
    pub fn restricted(&self) -> Self {
        Self {
            fs: Arc::new(RestrictedFS::new(self.fs.clone())),
            memory_protection: MemoryProtectionLevel::Maximum,
            audit: self.audit.clone().with_verbose(true),
        }
    }
}

/// Thread-local security context
thread_local! {
    static SECURITY_CONTEXT: RefCell<Option<SecurityContext>> = RefCell::new(None);
}

/// Run code with a specific security context
pub fn with_security_context<T, F>(ctx: SecurityContext, f: F) -> T
where
    F: FnOnce() -> T,
{
    SECURITY_CONTEXT.with(|c| {
        let prev = c.borrow_mut().replace(ctx);
        let result = f();
        *c.borrow_mut() = prev;
        result
    })
}

/// Get current security context
pub fn current_security_context() -> Option<SecurityContext> {
    SECURITY_CONTEXT.with(|c| c.borrow().clone())
}

// Integration example: ProfileManager using security context
impl ProfileManager {
    pub fn save_profile(&self, profile: &Profile) -> Result<(), ProfileError> {
        let ctx = current_security_context()
            .ok_or(ProfileError::NoSecurityContext)?;
        
        // Use secure FS from context
        let path = self.profile_path(&profile.name);
        let content = toml::to_string(profile)?;
        
        ctx.fs.write_atomic(&path, content.as_bytes())?;
        
        // Audit the operation
        ctx.audit.log_operation(AuditEvent::ProfileSaved {
            name: profile.name.clone(),
            user: std::env::var("USER").ok(),
        });
        
        Ok(())
    }
}
```

### Gap 2: Event Bus for Cross-Phase Communication

**Issue**: Phases need to communicate events without tight coupling.

**Solution**:

```rust
// src/core/events.rs

use tokio::sync::broadcast;
use std::sync::Arc;

/// System-wide events
#[derive(Clone, Debug)]
pub enum SystemEvent {
    // Profile events
    ProfileCreated { name: String },
    ProfileUpdated { name: String },
    ProfileDeleted { name: String },
    ProfileSwitched { from: Option<String>, to: String },
    
    // Signing events
    SigningConfigured { method: SigningMethod },
    SigningFailed { error: String },
    
    // Health events
    HealthCheckStarted,
    HealthCheckCompleted { issues: Vec<HealthIssue> },
    HealthIssueFixed { issue: HealthIssue },
    
    // 1Password events
    OnePasswordAuthenticated,
    OnePasswordDisconnected,
    CredentialAccessed { reference: String },
    
    // UI events
    UIStarted { mode: UIMode },
    UIStopped,
    UserAction { action: String },
}

/// Event bus for system-wide communication
pub struct EventBus {
    sender: broadcast::Sender<SystemEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }
    
    /// Publish an event
    pub fn publish(&self, event: SystemEvent) {
        let _ = self.sender.send(event);
    }
    
    /// Subscribe to events
    pub fn subscribe(&self) -> EventSubscriber {
        EventSubscriber {
            receiver: self.sender.subscribe(),
        }
    }
}

pub struct EventSubscriber {
    receiver: broadcast::Receiver<SystemEvent>,
}

impl EventSubscriber {
    /// Receive next event
    pub async fn recv(&mut self) -> Option<SystemEvent> {
        self.receiver.recv().await.ok()
    }
    
    /// Filter specific event types
    pub fn filter<F>(self, predicate: F) -> FilteredSubscriber<F>
    where
        F: Fn(&SystemEvent) -> bool,
    {
        FilteredSubscriber {
            subscriber: self,
            predicate,
        }
    }
}

// Integration example: Health monitor reacting to profile changes
pub struct HealthMonitor {
    event_bus: Arc<EventBus>,
}

impl HealthMonitor {
    pub async fn start(&self) {
        let mut subscriber = self.event_bus.subscribe();
        
        while let Some(event) = subscriber.recv().await {
            match event {
                SystemEvent::ProfileSwitched { to, .. } => {
                    // Run health checks for new profile
                    self.check_profile_health(&to).await;
                }
                SystemEvent::SigningConfigured { method } => {
                    // Verify signing setup
                    self.verify_signing_method(method).await;
                }
                _ => {}
            }
        }
    }
}
```

### Gap 3: Unified Error Handling

**Issue**: Each phase has its own error types, making cross-phase error handling complex.

**Solution**:

```rust
// src/core/error.rs

use thiserror::Error;

/// Root error type for all phases
#[derive(Debug, Error)]
pub enum GitSetupError {
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Profile error: {0}")]
    Profile(#[from] ProfileError),
    
    #[error("UI error: {0}")]
    UI(#[from] UIError),
    
    #[error("1Password error: {0}")]
    OnePassword(#[from] OnePasswordError),
    
    #[error("Signing error: {0}")]
    Signing(#[from] SigningError),
    
    #[error("Health check error: {0}")]
    Health(#[from] HealthError),
    
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),
    
    #[error("Integration error: {message}")]
    Integration { message: String, source: Box<dyn Error> },
}

/// Error context for better diagnostics
pub struct ErrorContext {
    pub phase: &'static str,
    pub operation: String,
    pub details: HashMap<String, String>,
}

impl GitSetupError {
    /// Add context to error
    pub fn with_context(self, ctx: ErrorContext) -> Self {
        match self {
            GitSetupError::Integration { message, source } => {
                GitSetupError::Integration {
                    message: format!("{} (in {} during {})", message, ctx.phase, ctx.operation),
                    source,
                }
            }
            _ => GitSetupError::Integration {
                message: format!("Error in {} during {}", ctx.phase, ctx.operation),
                source: Box::new(self),
            }
        }
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, GitSetupError>;

// Integration example: Converting between error types
impl From<std::io::Error> for GitSetupError {
    fn from(err: std::io::Error) -> Self {
        SecurityError::Io(err).into()
    }
}
```

### Gap 4: Configuration Management

**Issue**: Each phase has configuration needs that must be coordinated.

**Solution**:

```rust
// src/core/config.rs

use serde::{Deserialize, Serialize};

/// Master configuration combining all phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSetupConfig {
    /// Phase 1: Security settings
    pub security: SecurityConfig,
    
    /// Phase 2: Profile settings
    pub profiles: ProfileConfig,
    
    /// Phase 3: UI preferences
    pub ui: UIConfig,
    
    /// Phase 4: 1Password settings
    pub onepassword: OnePasswordConfig,
    
    /// Phase 5: Detection rules
    pub detection: DetectionConfig,
    
    /// Phase 6: Health monitoring
    pub health: HealthConfig,
    
    /// Phase 7-8: Signing preferences
    pub signing: SigningConfig,
    
    /// Phase 9: Platform-specific
    pub platform: PlatformConfig,
}

impl GitSetupConfig {
    /// Load configuration with cascading priority
    pub fn load() -> Result<Self> {
        let mut config = Config::builder();
        
        // 1. Built-in defaults
        config = config.add_source(Config::try_from(&Self::default())?);
        
        // 2. System-wide config
        if let Some(path) = Self::system_config_path() {
            if path.exists() {
                config = config.add_source(File::new(path.to_str().unwrap(), FileFormat::Toml));
            }
        }
        
        // 3. User config
        if let Some(path) = Self::user_config_path() {
            if path.exists() {
                config = config.add_source(File::new(path.to_str().unwrap(), FileFormat::Toml));
            }
        }
        
        // 4. Environment variables
        config = config.add_source(Environment::with_prefix("GIT_SETUP"));
        
        // 5. Build and validate
        let config: GitSetupConfig = config.build()?.try_deserialize()?;
        config.validate()?;
        
        Ok(config)
    }
    
    /// Validate cross-phase configuration consistency
    fn validate(&self) -> Result<()> {
        // Example: If using 1Password, ensure it's enabled
        if self.profiles.allow_onepassword_refs && !self.onepassword.enabled {
            return Err(GitSetupError::Integration {
                message: "1Password references allowed but 1Password not enabled".to_string(),
                source: Box::new(ValidationError),
            });
        }
        
        // Example: Signing method must match available tools
        if self.signing.default_method == SigningMethod::Gpg && !self.platform.gpg_available {
            return Err(GitSetupError::Integration {
                message: "GPG signing configured but GPG not available".to_string(),
                source: Box::new(ValidationError),
            });
        }
        
        Ok(())
    }
}

/// Configuration hot-reloading
pub struct ConfigWatcher {
    config: Arc<RwLock<GitSetupConfig>>,
    watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn watch(config_path: PathBuf) -> Result<Self> {
        let config = Arc::new(RwLock::new(GitSetupConfig::load()?));
        let config_clone = config.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    if let Ok(new_config) = GitSetupConfig::load() {
                        *config_clone.write().unwrap() = new_config;
                    }
                }
            }
        })?;
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        Ok(Self { config, watcher })
    }
}
```

### Gap 5: Dependency Injection Container

**Issue**: Phases need access to shared services without tight coupling.

**Solution**:

```rust
// src/core/container.rs

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Service container for dependency injection
pub struct ServiceContainer {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    /// Register a service
    pub fn register<T: Any + Send + Sync + 'static>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }
    
    /// Register a shared service
    pub fn register_shared<T: Any + Send + Sync + 'static>(&mut self, service: Arc<T>) {
        self.services.insert(TypeId::of::<Arc<T>>(), Box::new(service));
    }
    
    /// Get a service
    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref())
    }
    
    /// Get a shared service
    pub fn get_shared<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.services
            .get(&TypeId::of::<Arc<T>>())
            .and_then(|s| s.downcast_ref::<Arc<T>>())
            .cloned()
    }
}

/// Application builder with dependency injection
pub struct AppBuilder {
    container: ServiceContainer,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            container: ServiceContainer::new(),
        }
    }
    
    /// Configure Phase 1: Security
    pub fn with_security(mut self, ctx: SecurityContext) -> Self {
        self.container.register(ctx);
        self
    }
    
    /// Configure Phase 2: Profiles
    pub fn with_profile_manager(mut self, manager: Arc<ProfileManager>) -> Self {
        self.container.register_shared(manager);
        self
    }
    
    /// Configure Phase 3: UI
    pub fn with_ui(mut self, ui_mode: UIMode) -> Self {
        match ui_mode {
            UIMode::CLI => self.container.register(CLIHandler::new()),
            UIMode::TUI => self.container.register(TUIHandler::new()),
        }
        self
    }
    
    /// Configure Phase 4: 1Password
    pub fn with_onepassword(mut self, provider: Arc<dyn OnePasswordProvider>) -> Self {
        self.container.register_shared(provider);
        self
    }
    
    /// Build the application
    pub fn build(self) -> Result<Application> {
        // Validate all required services are registered
        self.validate_dependencies()?;
        
        Ok(Application {
            container: Arc::new(self.container),
        })
    }
    
    fn validate_dependencies(&self) -> Result<()> {
        // Check required services
        if self.container.get::<SecurityContext>().is_none() {
            return Err(GitSetupError::Integration {
                message: "Security context not configured".to_string(),
                source: Box::new(ConfigError),
            });
        }
        
        Ok(())
    }
}

// Usage example
pub async fn main() -> Result<()> {
    let app = AppBuilder::new()
        .with_security(SecurityContext::new())
        .with_profile_manager(Arc::new(ProfileManager::new()?))
        .with_ui(UIMode::TUI)
        .with_onepassword(Arc::new(OnePasswordCli::new()?))
        .build()?;
    
    app.run().await
}
```

### Gap 6: Cross-Phase Testing Framework

**Issue**: Testing integration between phases requires complex setup.

**Solution**:

```rust
// src/testing/integration.rs

/// Integration test helper
pub struct IntegrationTestContext {
    pub security: MockSecurityContext,
    pub profiles: MockProfileManager,
    pub ui: MockUI,
    pub onepassword: MockOnePasswordProvider,
    pub event_bus: EventBus,
    pub config: GitSetupConfig,
}

impl IntegrationTestContext {
    pub fn new() -> Self {
        Self {
            security: MockSecurityContext::new(),
            profiles: MockProfileManager::new(),
            ui: MockUI::new(),
            onepassword: MockOnePasswordProvider::new(),
            event_bus: EventBus::new(),
            config: GitSetupConfig::default(),
        }
    }
    
    /// Setup a complete test environment
    pub fn setup(self) -> TestEnvironment {
        let container = ServiceContainer::new();
        container.register(self.security);
        container.register(self.profiles);
        container.register(self.ui);
        container.register(self.onepassword);
        container.register(self.event_bus.clone());
        container.register(self.config);
        
        TestEnvironment {
            container: Arc::new(container),
            event_bus: self.event_bus,
        }
    }
    
    /// Create test profile with all integrations
    pub fn create_test_profile(&self) -> Profile {
        Profile {
            name: "test".to_string(),
            git: Some(GitConfig {
                user_name: "Test User".to_string(),
                user_email: "test@example.com".to_string(),
                extra: HashMap::new(),
            }),
            signing: Some(SigningConfig {
                method: SigningMethod::Ssh,
                ssh_key_ref: Some("op://Test/SSH Key/private".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

// Integration test example
#[tokio::test]
async fn test_profile_switch_with_signing() {
    let ctx = IntegrationTestContext::new();
    let env = ctx.setup();
    
    // Create profile with SSH signing
    let profile = ctx.create_test_profile();
    env.profiles.create_profile(profile.clone()).await.unwrap();
    
    // Mock 1Password response
    env.onepassword.set_secret(
        "op://Test/SSH Key/private",
        SensitiveString::from("test-key")
    ).await;
    
    // Switch profile
    env.profiles.switch_profile("test").await.unwrap();
    
    // Verify events were published
    let mut subscriber = env.event_bus.subscribe();
    let event = subscriber.recv().await.unwrap();
    assert!(matches!(event, SystemEvent::ProfileSwitched { to, .. } if to == "test"));
}
```

### Gap 7: Performance Monitoring Across Phases

**Issue**: Need to track performance across phase boundaries.

**Solution**:

```rust
// src/core/metrics.rs

use prometheus::{Counter, Histogram, Registry};

/// Application-wide metrics
pub struct Metrics {
    // Phase 1: Security metrics
    pub secure_writes: Counter,
    pub secure_write_duration: Histogram,
    
    // Phase 2: Profile metrics
    pub profile_loads: Counter,
    pub profile_load_duration: Histogram,
    
    // Phase 4: 1Password metrics
    pub op_calls: Counter,
    pub op_call_duration: Histogram,
    
    // Cross-phase metrics
    pub total_operations: Counter,
    pub operation_duration: Histogram,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        Ok(Self {
            secure_writes: Counter::new("secure_writes_total", "Total secure write operations")?,
            secure_write_duration: Histogram::new("secure_write_duration_seconds", "Secure write duration")?,
            profile_loads: Counter::new("profile_loads_total", "Total profile load operations")?,
            profile_load_duration: Histogram::new("profile_load_duration_seconds", "Profile load duration")?,
            op_calls: Counter::new("op_calls_total", "Total 1Password CLI calls")?,
            op_call_duration: Histogram::new("op_call_duration_seconds", "1Password call duration")?,
            total_operations: Counter::new("operations_total", "Total operations across all phases")?,
            operation_duration: Histogram::new("operation_duration_seconds", "Operation duration")?,
        })
    }
}

/// Performance tracking wrapper
pub async fn track_operation<F, T>(
    metrics: &Metrics,
    operation: &str,
    phase: &str,
    f: F,
) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let start = Instant::now();
    metrics.total_operations.inc();
    
    let result = f.await;
    
    let duration = start.elapsed().as_secs_f64();
    metrics.operation_duration.observe(duration);
    
    // Log slow operations
    if duration > 1.0 {
        warn!("Slow operation: {} in {} took {:.2}s", operation, phase, duration);
    }
    
    result
}
```

## Integration Testing Scenarios

### Scenario 1: Full Profile Lifecycle

```rust
#[tokio::test]
async fn test_full_profile_lifecycle() {
    let env = IntegrationTestContext::new().setup();
    
    // Phase 2: Create profile
    let profile = Profile {
        name: "lifecycle-test".to_string(),
        // ... configuration
    };
    env.profiles.create_profile(profile.clone()).await.unwrap();
    
    // Phase 3: UI lists profiles
    let profiles = env.ui.list_profiles().await.unwrap();
    assert!(profiles.contains(&"lifecycle-test".to_string()));
    
    // Phase 5: Pattern detection
    let detected = env.detection.detect_profile("/home/user/work/project").await.unwrap();
    assert_eq!(detected, Some("lifecycle-test".to_string()));
    
    // Phase 4 + 7: Apply profile with signing
    env.profiles.apply_profile("lifecycle-test").await.unwrap();
    
    // Phase 6: Health check
    let health = env.health.check_all().await.unwrap();
    assert!(health.is_healthy());
}
```

### Scenario 2: Error Propagation

```rust
#[tokio::test]
async fn test_error_propagation() {
    let env = IntegrationTestContext::new().setup();
    
    // Simulate 1Password failure
    env.onepassword.trigger_error_on(
        "read_secret",
        OnePasswordError::NotAuthenticated
    ).await;
    
    // Try to use profile with 1Password reference
    let result = env.profiles.apply_profile("test").await;
    
    // Error should propagate with context
    assert!(matches!(
        result,
        Err(GitSetupError::Integration { message, .. }) if message.contains("1Password")
    ));
}
```

## Best Practices for Cross-Phase Integration

1. **Use Dependency Injection**: Avoid hard dependencies between phases
2. **Event-Driven Communication**: Use event bus for loose coupling
3. **Unified Error Handling**: Consistent error types and propagation
4. **Shared Configuration**: Single source of truth for settings
5. **Security Context**: Thread security settings through all operations
6. **Performance Monitoring**: Track metrics across phase boundaries
7. **Integration Testing**: Test phase interactions explicitly

## Conclusion

This guide addresses the major integration gaps between phases, providing patterns and examples for building a cohesive, well-integrated system. The solutions ensure that each phase can work independently while seamlessly integrating into the larger application.