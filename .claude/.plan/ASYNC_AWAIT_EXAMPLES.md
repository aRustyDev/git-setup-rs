# Async/Await Complexity Examples for git-setup-rs

## Overview

This guide helps junior developers understand and implement async/await patterns in the git-setup-rs project. Async programming allows handling multiple operations concurrently without blocking, crucial for responsive UIs and efficient I/O operations.

## Why Async in git-setup-rs?

1. **1Password CLI calls**: Can take seconds to complete
2. **File operations**: Reading/writing multiple profiles concurrently
3. **Git operations**: Multiple repositories can be processed in parallel
4. **TUI responsiveness**: UI remains interactive during long operations
5. **Network operations**: Fetching remote configurations

## Understanding Async/Await Basics

### Synchronous vs Asynchronous

```rust
// SYNCHRONOUS: Blocks until complete
fn read_profile_sync(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path) // Thread blocked here
        .map_err(|e| Error::FileRead(e))
}

// ASYNCHRONOUS: Returns a Future immediately
async fn read_profile_async(path: &Path) -> Result<String, Error> {
    tokio::fs::read_to_string(path).await // Thread can do other work
        .map_err(|e| Error::FileRead(e))
}

// Usage comparison
fn main() {
    // Sync: Sequential execution
    let profile1 = read_profile_sync("profile1.toml").unwrap();
    let profile2 = read_profile_sync("profile2.toml").unwrap(); // Waits for profile1
    
    // Async: Concurrent execution
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let (profile1, profile2) = tokio::join!(
            read_profile_async("profile1.toml"),
            read_profile_async("profile2.toml")
        ); // Both read at the same time!
    });
}
```

## Common Async Patterns in git-setup-rs

### Pattern 1: Basic Async Function

```rust
use tokio::fs;
use std::path::Path;

/// Load a profile asynchronously
pub async fn load_profile(name: &str) -> Result<Profile, Error> {
    // Build the path
    let path = profile_path(name)?;
    
    // Read file asynchronously
    let content = fs::read_to_string(&path).await
        .map_err(|e| Error::FileRead(format!("{}: {}", path.display(), e)))?;
    
    // Parse (CPU-bound, so not async)
    let profile: Profile = toml::from_str(&content)
        .map_err(|e| Error::ParseError(e.to_string()))?;
    
    Ok(profile)
}

// Important: async functions return Future
// They don't execute until awaited!
```

### Pattern 2: Concurrent Operations

```rust
use futures::future::try_join_all;

/// Load multiple profiles concurrently
pub async fn load_all_profiles() -> Result<Vec<Profile>, Error> {
    // List profile files
    let profile_dir = profile_directory()?;
    let mut entries = fs::read_dir(&profile_dir).await?;
    
    let mut futures = Vec::new();
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension() == Some(OsStr::new("toml")) {
            // Create future but don't await yet
            futures.push(load_profile_from_path(path));
        }
    }
    
    // Execute all futures concurrently
    let profiles = try_join_all(futures).await?;
    
    Ok(profiles)
}

// Alternative using tokio::join! for fixed number
pub async fn load_default_profiles() -> Result<(Profile, Profile), Error> {
    let (work, personal) = tokio::join!(
        load_profile("work"),
        load_profile("personal")
    );
    
    Ok((work?, personal?))
}
```

### Pattern 3: Timeouts and Cancellation

```rust
use tokio::time::{timeout, Duration};

/// Call 1Password with timeout
pub async fn get_1password_secret(reference: &str) -> Result<String, Error> {
    let future = async {
        let output = tokio::process::Command::new("op")
            .args(&["read", reference])
            .output()
            .await?;
        
        if !output.status.success() {
            return Err(Error::OnePasswordFailed);
        }
        
        String::from_utf8(output.stdout)
            .map_err(|_| Error::InvalidUtf8)
    };
    
    // Timeout after 30 seconds
    match timeout(Duration::from_secs(30), future).await {
        Ok(Ok(secret)) => Ok(secret),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(Error::Timeout("1Password operation timed out")),
    }
}

/// Cancellable operation using select!
pub async fn cancellable_operation(
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>
) -> Result<String, Error> {
    tokio::select! {
        result = do_long_operation() => {
            result
        }
        _ = &mut cancel_rx => {
            Err(Error::Cancelled)
        }
    }
}
```

### Pattern 4: Error Handling in Async

```rust
/// Async error handling with context
pub async fn apply_profile(name: &str) -> Result<(), Error> {
    // Load profile
    let profile = load_profile(name).await
        .map_err(|e| Error::ProfileLoadFailed(name.to_string(), Box::new(e)))?;
    
    // Apply Git config (might fail)
    apply_git_config(&profile).await
        .map_err(|e| Error::GitConfigFailed(Box::new(e)))?;
    
    // Handle signing config
    if let Some(signing) = &profile.signing {
        setup_signing(signing).await
            .map_err(|e| Error::SigningSetupFailed(Box::new(e)))?;
    }
    
    Ok(())
}

/// Using ? operator with async
pub async fn process_profiles() -> Result<(), Error> {
    let profiles = load_all_profiles().await?; // Early return on error
    
    for profile in profiles {
        validate_profile(&profile)?; // Sync function, still works
        save_profile(&profile).await?; // Async function
    }
    
    Ok(())
}

/// Handling multiple errors
pub async fn process_with_partial_failures() -> Result<Vec<String>, Vec<Error>> {
    let profile_names = vec!["work", "personal", "client"];
    let mut successes = Vec::new();
    let mut errors = Vec::new();
    
    for name in profile_names {
        match load_profile(name).await {
            Ok(profile) => successes.push(profile.name),
            Err(e) => errors.push(e),
        }
    }
    
    if errors.is_empty() {
        Ok(successes)
    } else {
        Err(errors)
    }
}
```

### Pattern 5: Async Traits

```rust
use async_trait::async_trait;

/// Async trait for profile storage backends
#[async_trait]
pub trait AsyncProfileStorage: Send + Sync {
    async fn load(&self, name: &str) -> Result<Profile, Error>;
    async fn save(&self, profile: &Profile) -> Result<(), Error>;
    async fn delete(&self, name: &str) -> Result<(), Error>;
    async fn list(&self) -> Result<Vec<String>, Error>;
}

/// File system implementation
pub struct FileSystemStorage {
    base_path: PathBuf,
}

#[async_trait]
impl AsyncProfileStorage for FileSystemStorage {
    async fn load(&self, name: &str) -> Result<Profile, Error> {
        let path = self.base_path.join(format!("{}.toml", name));
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(toml::from_str(&content)?)
    }
    
    async fn save(&self, profile: &Profile) -> Result<(), Error> {
        let path = self.base_path.join(format!("{}.toml", &profile.name));
        let content = toml::to_string_pretty(profile)?;
        
        // Atomic write
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, &content).await?;
        tokio::fs::rename(&temp_path, &path).await?;
        
        Ok(())
    }
    
    async fn delete(&self, name: &str) -> Result<(), Error> {
        let path = self.base_path.join(format!("{}.toml", name));
        tokio::fs::remove_file(&path).await?;
        Ok(())
    }
    
    async fn list(&self) -> Result<Vec<String>, Error> {
        let mut profiles = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.base_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".toml") {
                    profiles.push(name.trim_end_matches(".toml").to_string());
                }
            }
        }
        
        Ok(profiles)
    }
}
```

### Pattern 6: Channels and Message Passing

```rust
use tokio::sync::{mpsc, oneshot};

/// Event types for async communication
#[derive(Debug)]
pub enum ProfileEvent {
    Load { name: String, reply: oneshot::Sender<Result<Profile, Error>> },
    Save { profile: Profile, reply: oneshot::Sender<Result<(), Error>> },
    Delete { name: String, reply: oneshot::Sender<Result<(), Error>> },
}

/// Profile manager actor
pub struct ProfileActor {
    receiver: mpsc::Receiver<ProfileEvent>,
    storage: Box<dyn AsyncProfileStorage>,
}

impl ProfileActor {
    pub async fn run(mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event {
                ProfileEvent::Load { name, reply } => {
                    let result = self.storage.load(&name).await;
                    let _ = reply.send(result);
                }
                ProfileEvent::Save { profile, reply } => {
                    let result = self.storage.save(&profile).await;
                    let _ = reply.send(result);
                }
                ProfileEvent::Delete { name, reply } => {
                    let result = self.storage.delete(&name).await;
                    let _ = reply.send(result);
                }
            }
        }
    }
}

/// Handle to communicate with the actor
#[derive(Clone)]
pub struct ProfileHandle {
    sender: mpsc::Sender<ProfileEvent>,
}

impl ProfileHandle {
    pub async fn load(&self, name: &str) -> Result<Profile, Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        
        self.sender.send(ProfileEvent::Load {
            name: name.to_string(),
            reply: reply_tx,
        }).await.map_err(|_| Error::ActorDied)?;
        
        reply_rx.await.map_err(|_| Error::ActorDied)?
    }
}
```

### Pattern 7: Stream Processing

```rust
use tokio_stream::{Stream, StreamExt};
use futures::stream;

/// Watch profile directory for changes
pub fn watch_profiles() -> impl Stream<Item = ProfileChange> {
    // In real implementation, use notify crate
    stream::unfold(0, |state| async move {
        // Simulate changes
        tokio::time::sleep(Duration::from_secs(1)).await;
        Some((ProfileChange::Modified("work".to_string()), state + 1))
    })
}

/// Process profile changes
pub async fn handle_profile_changes() -> Result<(), Error> {
    let mut changes = watch_profiles();
    
    while let Some(change) = changes.next().await {
        match change {
            ProfileChange::Created(name) => {
                println!("Profile created: {}", name);
            }
            ProfileChange::Modified(name) => {
                println!("Profile modified: {}", name);
                // Reload profile
                if let Ok(profile) = load_profile(&name).await {
                    // Update UI, etc.
                }
            }
            ProfileChange::Deleted(name) => {
                println!("Profile deleted: {}", name);
            }
        }
    }
    
    Ok(())
}

/// Batch process profiles
pub async fn batch_process_profiles<F, Fut>(
    profiles: Vec<String>,
    batch_size: usize,
    processor: F,
) -> Result<Vec<Profile>, Error>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<Profile, Error>>,
{
    let mut results = Vec::new();
    
    // Process in batches to avoid overwhelming the system
    for chunk in profiles.chunks(batch_size) {
        let futures: Vec<_> = chunk.iter()
            .map(|name| processor(name.clone()))
            .collect();
        
        let batch_results = try_join_all(futures).await?;
        results.extend(batch_results);
        
        // Small delay between batches
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Ok(results)
}
```

### Pattern 8: Async Mutex and Shared State

```rust
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;

/// Shared profile cache
pub struct ProfileCache {
    profiles: Arc<RwLock<HashMap<String, Profile>>>,
    storage: Arc<dyn AsyncProfileStorage>,
}

impl ProfileCache {
    pub fn new(storage: Arc<dyn AsyncProfileStorage>) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }
    
    /// Get profile with caching
    pub async fn get(&self, name: &str) -> Result<Profile, Error> {
        // Try read lock first (multiple readers allowed)
        {
            let cache = self.profiles.read().await;
            if let Some(profile) = cache.get(name) {
                return Ok(profile.clone());
            }
        }
        
        // Not in cache, need write lock
        let mut cache = self.profiles.write().await;
        
        // Double-check (another task might have loaded it)
        if let Some(profile) = cache.get(name) {
            return Ok(profile.clone());
        }
        
        // Load from storage
        let profile = self.storage.load(name).await?;
        cache.insert(name.to_string(), profile.clone());
        
        Ok(profile)
    }
    
    /// Invalidate cache entry
    pub async fn invalidate(&self, name: &str) {
        let mut cache = self.profiles.write().await;
        cache.remove(name);
    }
}

/// Async counter with mutex
pub struct AsyncCounter {
    value: Arc<Mutex<u64>>,
}

impl AsyncCounter {
    pub fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(0)),
        }
    }
    
    pub async fn increment(&self) -> u64 {
        let mut value = self.value.lock().await;
        *value += 1;
        *value
    }
    
    pub async fn get(&self) -> u64 {
        let value = self.value.lock().await;
        *value
    }
}
```

### Pattern 9: Complex Async Orchestration

```rust
/// Complete profile setup with multiple async operations
pub async fn setup_complete_profile(
    name: &str,
    config: ProfileConfig,
) -> Result<(), Error> {
    // Step 1: Validate configuration
    validate_config(&config)?;
    
    // Step 2: Create profile
    let profile = create_profile(name, config).await?;
    
    // Step 3: Parallel operations
    let (git_result, ssh_result, gpg_result) = tokio::join!(
        setup_git_config(&profile),
        setup_ssh_signing(&profile),
        setup_gpg_signing(&profile)
    );
    
    // Check results
    git_result.map_err(|e| Error::GitSetupFailed(Box::new(e)))?;
    
    // SSH and GPG are optional, log errors but don't fail
    if let Err(e) = ssh_result {
        log::warn!("SSH setup failed: {}", e);
    }
    if let Err(e) = gpg_result {
        log::warn!("GPG setup failed: {}", e);
    }
    
    // Step 4: Save profile
    save_profile(&profile).await?;
    
    // Step 5: Verify setup
    verify_profile_setup(&profile).await?;
    
    Ok(())
}

/// Async retry with exponential backoff
pub async fn retry_async<F, Fut, T, E>(
    mut operation: F,
    max_attempts: u32,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_attempts => return Err(e),
            Err(e) => {
                log::warn!("Attempt {} failed: {}", attempt, e);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

// Usage
pub async fn reliable_1password_call(reference: &str) -> Result<String, Error> {
    retry_async(
        || get_1password_secret(reference),
        3,
    ).await
}
```

### Pattern 10: Testing Async Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test] // Note: Not async
    fn test_sync_function() {
        // Regular test for sync code
    }
    
    #[tokio::test] // Macro creates runtime
    async fn test_async_function() {
        let result = load_profile("test").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_operations() {
        let (r1, r2) = tokio::join!(
            load_profile("work"),
            load_profile("personal")
        );
        
        assert!(r1.is_ok());
        assert!(r2.is_ok());
    }
    
    #[tokio::test]
    async fn test_with_timeout() {
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            load_profile("test")
        ).await;
        
        assert!(result.is_ok());
    }
    
    /// Mock async trait implementation
    struct MockStorage {
        profiles: Arc<Mutex<HashMap<String, Profile>>>,
    }
    
    #[async_trait]
    impl AsyncProfileStorage for MockStorage {
        async fn load(&self, name: &str) -> Result<Profile, Error> {
            let profiles = self.profiles.lock().await;
            profiles.get(name)
                .cloned()
                .ok_or(Error::ProfileNotFound)
        }
        
        async fn save(&self, profile: &Profile) -> Result<(), Error> {
            let mut profiles = self.profiles.lock().await;
            profiles.insert(profile.name.clone(), profile.clone());
            Ok(())
        }
        
        // ... other methods
    }
}
```

## Common Async Pitfalls and Solutions

### Pitfall 1: Blocking in Async Code

```rust
// BAD: Blocks the async runtime
async fn bad_async_function() {
    std::thread::sleep(Duration::from_secs(1)); // BLOCKS!
    std::fs::read_to_string("file.txt").unwrap(); // BLOCKS!
}

// GOOD: Use async equivalents
async fn good_async_function() {
    tokio::time::sleep(Duration::from_secs(1)).await; // Non-blocking
    tokio::fs::read_to_string("file.txt").await.unwrap(); // Non-blocking
}

// When you must use blocking code
async fn handle_blocking_operation() -> Result<String, Error> {
    // Move to blocking thread pool
    tokio::task::spawn_blocking(|| {
        // Now safe to block
        std::thread::sleep(Duration::from_secs(1));
        expensive_computation()
    }).await.map_err(|e| Error::TaskJoinError(e))?
}
```

### Pitfall 2: Forgetting to Await

```rust
// BAD: Future created but not executed
async fn forgot_await() {
    load_profile("work"); // Warning: unused future!
    println!("Done"); // Prints immediately, load not started
}

// GOOD: Await the future
async fn proper_await() {
    load_profile("work").await; // Actually executes
    println!("Done"); // Prints after load completes
}

// Sometimes you want to start without awaiting
async fn concurrent_start() {
    let future1 = load_profile("work"); // Start
    let future2 = load_profile("personal"); // Start
    
    // Do other work...
    
    // Now await both
    let (r1, r2) = tokio::join!(future1, future2);
}
```

### Pitfall 3: Lifetime Issues with Async

```rust
// BAD: Borrowing across await points
async fn lifetime_issue() {
    let data = String::from("hello");
    let reference = &data;
    
    some_async_function().await; // Error: reference must live across await
    
    println!("{}", reference);
}

// GOOD: Clone or restructure
async fn lifetime_fixed() {
    let data = String::from("hello");
    let cloned = data.clone();
    
    some_async_function().await;
    
    println!("{}", cloned);
}

// BETTER: Use Arc for shared ownership
async fn lifetime_arc() {
    let data = Arc::new(String::from("hello"));
    let data_clone = Arc::clone(&data);
    
    tokio::spawn(async move {
        println!("{}", data_clone);
    });
    
    some_async_function().await;
    println!("{}", data);
}
```

### Pitfall 4: Deadlocks with Async Mutex

```rust
// BAD: Can deadlock
async fn potential_deadlock(cache: &ProfileCache) {
    let guard = cache.profiles.write().await;
    
    // Don't call async functions while holding lock!
    let profile = load_profile("work").await; // May deadlock
    
    // ... use guard
}

// GOOD: Minimize lock scope
async fn no_deadlock(cache: &ProfileCache) {
    let profile = load_profile("work").await;
    
    // Only lock when necessary
    {
        let mut guard = cache.profiles.write().await;
        guard.insert("work".to_string(), profile);
    } // Lock released here
}
```

## Real-World Example: Profile Manager Service

```rust
use tokio::sync::{broadcast, mpsc};
use std::collections::HashMap;
use std::sync::Arc;

/// Complete async profile manager
pub struct AsyncProfileManager {
    storage: Arc<dyn AsyncProfileStorage>,
    cache: ProfileCache,
    event_tx: broadcast::Sender<ProfileEvent>,
    command_rx: mpsc::Receiver<ProfileCommand>,
}

impl AsyncProfileManager {
    pub fn new(
        storage: Arc<dyn AsyncProfileStorage>,
    ) -> (Self, ProfileManagerHandle) {
        let (event_tx, _) = broadcast::channel(100);
        let (command_tx, command_rx) = mpsc::channel(32);
        
        let manager = Self {
            storage: Arc::clone(&storage),
            cache: ProfileCache::new(storage),
            event_tx: event_tx.clone(),
            command_rx,
        };
        
        let handle = ProfileManagerHandle {
            command_tx,
            event_tx,
        };
        
        (manager, handle)
    }
    
    pub async fn run(mut self) {
        // Initial load
        if let Err(e) = self.load_all_profiles().await {
            log::error!("Failed to load profiles: {}", e);
        }
        
        // Process commands
        while let Some(command) = self.command_rx.recv().await {
            match command {
                ProfileCommand::Load { name, reply } => {
                    let result = self.cache.get(&name).await;
                    let _ = reply.send(result);
                }
                
                ProfileCommand::Create { profile, reply } => {
                    let result = self.create_profile(profile).await;
                    let _ = reply.send(result);
                }
                
                ProfileCommand::Update { profile, reply } => {
                    let result = self.update_profile(profile).await;
                    let _ = reply.send(result);
                }
                
                ProfileCommand::Delete { name, reply } => {
                    let result = self.delete_profile(&name).await;
                    let _ = reply.send(result);
                }
                
                ProfileCommand::RefreshAll { reply } => {
                    let result = self.load_all_profiles().await;
                    let _ = reply.send(result);
                }
            }
        }
    }
    
    async fn create_profile(&self, profile: Profile) -> Result<(), Error> {
        // Validate
        profile.validate()?;
        
        // Check if exists
        if self.cache.get(&profile.name).await.is_ok() {
            return Err(Error::ProfileAlreadyExists);
        }
        
        // Save to storage
        self.storage.save(&profile).await?;
        
        // Update cache
        self.cache.profiles.write().await
            .insert(profile.name.clone(), profile.clone());
        
        // Emit event
        let _ = self.event_tx.send(ProfileEvent::Created(profile));
        
        Ok(())
    }
    
    async fn update_profile(&self, profile: Profile) -> Result<(), Error> {
        // Validate
        profile.validate()?;
        
        // Save to storage
        self.storage.save(&profile).await?;
        
        // Update cache
        self.cache.profiles.write().await
            .insert(profile.name.clone(), profile.clone());
        
        // Emit event
        let _ = self.event_tx.send(ProfileEvent::Updated(profile));
        
        Ok(())
    }
    
    async fn delete_profile(&self, name: &str) -> Result<(), Error> {
        // Delete from storage
        self.storage.delete(name).await?;
        
        // Remove from cache
        self.cache.invalidate(name).await;
        
        // Emit event
        let _ = self.event_tx.send(ProfileEvent::Deleted(name.to_string()));
        
        Ok(())
    }
    
    async fn load_all_profiles(&self) -> Result<(), Error> {
        let names = self.storage.list().await?;
        let mut profiles = HashMap::new();
        
        // Load in batches
        for chunk in names.chunks(10) {
            let futures: Vec<_> = chunk.iter()
                .map(|name| self.storage.load(name))
                .collect();
            
            let results = try_join_all(futures).await?;
            
            for profile in results {
                profiles.insert(profile.name.clone(), profile);
            }
        }
        
        // Update cache
        *self.cache.profiles.write().await = profiles;
        
        Ok(())
    }
}

/// Handle for external communication
#[derive(Clone)]
pub struct ProfileManagerHandle {
    command_tx: mpsc::Sender<ProfileCommand>,
    event_tx: broadcast::Sender<ProfileEvent>,
}

impl ProfileManagerHandle {
    pub async fn load_profile(&self, name: &str) -> Result<Profile, Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        
        self.command_tx.send(ProfileCommand::Load {
            name: name.to_string(),
            reply: reply_tx,
        }).await.map_err(|_| Error::ServiceDied)?;
        
        reply_rx.await.map_err(|_| Error::ServiceDied)?
    }
    
    pub async fn create_profile(&self, profile: Profile) -> Result<(), Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        
        self.command_tx.send(ProfileCommand::Create {
            profile,
            reply: reply_tx,
        }).await.map_err(|_| Error::ServiceDied)?;
        
        reply_rx.await.map_err(|_| Error::ServiceDied)?
    }
    
    pub fn subscribe_events(&self) -> broadcast::Receiver<ProfileEvent> {
        self.event_tx.subscribe()
    }
}

// Usage example
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create storage backend
    let storage = Arc::new(FileSystemStorage::new("/home/user/.config/git-setup"));
    
    // Create and start manager
    let (manager, handle) = AsyncProfileManager::new(storage);
    
    // Spawn manager task
    let manager_task = tokio::spawn(manager.run());
    
    // Subscribe to events
    let mut events = handle.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            println!("Profile event: {:?}", event);
        }
    });
    
    // Use the handle
    let profile = Profile::new("work");
    handle.create_profile(profile).await?;
    
    let loaded = handle.load_profile("work").await?;
    println!("Loaded profile: {}", loaded.name);
    
    // Wait for manager to finish (it won't in this example)
    manager_task.await.map_err(|e| Error::TaskJoinError(e))?;
    
    Ok(())
}
```

## Key Takeaways

1. **Async is contagious**: Once you have one async function, callers must be async too
2. **Don't block the runtime**: Use async versions of I/O operations
3. **Concurrency != Parallelism**: Async is about efficiency, not speed
4. **Error handling is crucial**: Async errors can be harder to debug
5. **Test thoroughly**: Async bugs often appear under load

## Common Async Dependencies for git-setup-rs

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
futures = "0.3"
tokio-stream = "0.1"

[dev-dependencies]
tokio-test = "0.4"
```

## Next Steps

1. Read the [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
2. Practice with the async exercises in `examples/async/`
3. Start with simple async file operations
4. Progress to concurrent operations
5. Implement a small async service

Remember: Async programming is complex but powerful. Start simple, test thoroughly, and gradually increase complexity as you gain confidence.