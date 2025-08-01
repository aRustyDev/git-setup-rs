# Rust Ownership Concepts for Junior Developers

## Overview

This guide explains Rust's ownership system using practical examples from the git-setup-rs project. If you're coming from languages like Python, JavaScript, or Java, ownership might be the most challenging Rust concept to grasp - but it's also what makes Rust memory-safe without a garbage collector.

## Why Ownership Matters

In other languages:
- **Python/JavaScript/Java**: Garbage collector handles memory automatically
- **C/C++**: You manually allocate and free memory (prone to bugs)
- **Rust**: Ownership rules ensure memory safety at compile time

## The Three Rules of Ownership

1. **Each value has a single owner**
2. **When the owner goes out of scope, the value is dropped**
3. **There can only be one owner at a time**

Let's explore each rule with examples.

## Rule 1: Each Value Has a Single Owner

### Example: Profile Names

```rust
// BAD: This won't compile
fn main() {
    let profile_name = String::from("work");
    let another_name = profile_name;  // ownership moved here
    
    println!("Profile: {}", profile_name); // ERROR: value borrowed after move
}

// GOOD: Clone when you need a copy
fn main() {
    let profile_name = String::from("work");
    let another_name = profile_name.clone(); // explicit copy
    
    println!("Profile: {}", profile_name);    // ✓ Works
    println!("Another: {}", another_name);     // ✓ Works
}

// BETTER: Use references when possible
fn main() {
    let profile_name = String::from("work");
    let another_name = &profile_name; // borrow, don't take ownership
    
    println!("Profile: {}", profile_name);    // ✓ Works
    println!("Another: {}", another_name);     // ✓ Works
}
```

### Real-World Example: Profile Management

```rust
// From our ProfileManager
pub struct ProfileManager {
    profiles: HashMap<String, Profile>,
}

impl ProfileManager {
    // BAD: This would take ownership of the profile
    pub fn add_profile_bad(mut self, profile: Profile) -> Self {
        self.profiles.insert(profile.name.clone(), profile);
        self // have to return self because caller lost ownership!
    }
    
    // GOOD: Take a reference to self
    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.insert(profile.name.clone(), profile);
        // self is borrowed mutably, caller keeps ownership
    }
}

// Usage
let mut manager = ProfileManager::new();
let work_profile = Profile::new("work");

// With the bad version:
// manager = manager.add_profile_bad(work_profile); // awkward!

// With the good version:
manager.add_profile(work_profile); // natural!
```

## Rule 2: Values Are Dropped When Owner Goes Out of Scope

### Example: Automatic Cleanup

```rust
// Sensitive data that needs cleanup
pub struct SensitiveString {
    data: String,
}

impl Drop for SensitiveString {
    fn drop(&mut self) {
        // Zero out the memory when dropped
        println!("Zeroing sensitive data...");
        self.data.clear();
        // In real code, we'd use the zeroize crate
    }
}

fn process_password() {
    let password = SensitiveString {
        data: String::from("secret123"),
    };
    // Use password...
} // password is dropped here, memory is zeroed automatically

fn main() {
    process_password();
    // Password memory is already cleaned up!
}
```

### Real-World Example: File Handles

```rust
use std::fs::File;
use std::io::Write;

// File is automatically closed when it goes out of scope
fn write_profile(path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    // No need to call file.close() - happens automatically!
} // file is dropped and closed here

// Our AtomicFileWriter uses this pattern
pub struct AtomicFileWriter {
    temp_file: Option<tempfile::NamedTempFile>,
}

impl AtomicFileWriter {
    pub fn write(&mut self, content: &[u8]) -> Result<(), std::io::Error> {
        let temp = tempfile::NamedTempFile::new()?;
        // temp file will be automatically deleted if we don't persist it
        
        // ... write content ...
        
        self.temp_file = Some(temp);
        Ok(())
    }
} // If write fails, temp file is automatically cleaned up!
```

## Rule 3: Only One Owner at a Time (But Many Borrowers)

### Example: Borrowing vs Ownership

```rust
// A profile that multiple parts of our app need to read
#[derive(Clone)]
struct Profile {
    name: String,
    email: String,
}

// Function that needs to read the profile
fn validate_profile(profile: &Profile) -> bool {
    // Borrowing with & means we can read but not take ownership
    !profile.email.is_empty() && profile.email.contains('@')
}

// Function that needs to modify the profile
fn normalize_email(profile: &mut Profile) {
    // Mutable borrow with &mut means we can modify
    profile.email = profile.email.to_lowercase();
}

fn main() {
    let mut my_profile = Profile {
        name: String::from("Alice"),
        email: String::from("ALICE@EXAMPLE.COM"),
    };
    
    // Multiple immutable borrows are OK
    let is_valid = validate_profile(&my_profile);
    let also_valid = validate_profile(&my_profile); // ✓ Works!
    
    // But only one mutable borrow at a time
    normalize_email(&mut my_profile);
    // Can't use my_profile here until normalize_email is done
    
    println!("Profile email: {}", my_profile.email); // alice@example.com
}
```

### Real-World Example: Shared Configuration

```rust
use std::sync::Arc;

// Configuration that needs to be shared across threads
pub struct GitSetupConfig {
    pub profiles_dir: PathBuf,
    pub default_profile: String,
}

// Arc (Atomic Reference Counting) allows multiple owners
pub struct Application {
    config: Arc<GitSetupConfig>,
    profile_manager: ProfileManager,
    ui_handler: UiHandler,
}

impl Application {
    pub fn new(config: GitSetupConfig) -> Self {
        let config = Arc::new(config); // Wrap in Arc
        
        Self {
            config: Arc::clone(&config), // Cheap clone of Arc, not the data
            profile_manager: ProfileManager::new(Arc::clone(&config)),
            ui_handler: UiHandler::new(Arc::clone(&config)),
        }
    }
}

// Each component can have its own Arc to the same config
impl ProfileManager {
    pub fn new(config: Arc<GitSetupConfig>) -> Self {
        Self { config }
    }
    
    pub fn list_profiles(&self) -> Vec<String> {
        // Can read config through Arc
        let dir = &self.config.profiles_dir;
        // ... list files in dir ...
        vec![]
    }
}
```

## Common Ownership Patterns in git-setup-rs

### Pattern 1: Builder Pattern (Consuming Ownership)

```rust
// ProfileBuilder consumes values to build a Profile
pub struct ProfileBuilder {
    name: Option<String>,
    email: Option<String>,
}

impl ProfileBuilder {
    pub fn new() -> Self {
        Self { name: None, email: None }
    }
    
    // These methods take self by value, consuming the builder
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self // Return self for chaining
    }
    
    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    // build() consumes the builder
    pub fn build(self) -> Result<Profile, String> {
        Ok(Profile {
            name: self.name.ok_or("Name required")?,
            email: self.email.ok_or("Email required")?,
        })
    }
}

// Usage
let profile = ProfileBuilder::new()
    .name(String::from("work"))
    .email(String::from("work@example.com"))
    .build()?; // Builder is consumed here
// Can't use builder after build()!
```

### Pattern 2: Interior Mutability (Multiple Owners, Mutable)

```rust
use std::cell::RefCell;
use std::rc::Rc;

// For single-threaded sharing with mutation
pub struct EventBus {
    listeners: Rc<RefCell<Vec<Box<dyn Fn(&Event)>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(Vec::new())),
        }
    }
    
    // Can clone the Rc to share ownership
    pub fn subscribe(&self) -> EventSubscriber {
        EventSubscriber {
            listeners: Rc::clone(&self.listeners),
        }
    }
    
    pub fn emit(&self, event: &Event) {
        // Borrow the RefCell's contents mutably
        let listeners = self.listeners.borrow();
        for listener in listeners.iter() {
            listener(event);
        }
    }
}
```

### Pattern 3: Lifetime Annotations (Borrowing Relationships)

```rust
// This struct borrows data, doesn't own it
pub struct ProfileValidator<'a> {
    existing_profiles: &'a [String],
}

impl<'a> ProfileValidator<'a> {
    // The 'a lifetime says: this validator can't outlive the slice it borrows
    pub fn new(existing: &'a [String]) -> Self {
        Self { existing_profiles: existing }
    }
    
    pub fn is_name_taken(&self, name: &str) -> bool {
        self.existing_profiles.contains(&name.to_string())
    }
}

// Usage
fn validate_new_profile(name: &str) -> bool {
    let existing = vec!["work".to_string(), "personal".to_string()];
    let validator = ProfileValidator::new(&existing);
    
    !validator.is_name_taken(name)
} // validator and existing are both dropped here
```

## Common Ownership Mistakes and Solutions

### Mistake 1: Fighting the Borrow Checker

```rust
// BAD: Trying to mutate while borrowed
fn broken_update(profiles: &mut Vec<Profile>) {
    for profile in profiles.iter() {
        if profile.name == "old" {
            profiles.push(Profile::new("new")); // ERROR: can't mutate while iterating
        }
    }
}

// GOOD: Collect changes, then apply
fn working_update(profiles: &mut Vec<Profile>) {
    let needs_new = profiles.iter().any(|p| p.name == "old");
    if needs_new {
        profiles.push(Profile::new("new")); // ✓ Works
    }
}

// BETTER: Use indices when you need to mutate during iteration
fn better_update(profiles: &mut Vec<Profile>) {
    let mut i = 0;
    while i < profiles.len() {
        if profiles[i].name == "old" {
            profiles.push(Profile::new("new")); // ✓ Works
        }
        i += 1;
    }
}
```

### Mistake 2: Unnecessary Cloning

```rust
// BAD: Cloning when borrowing would work
fn print_profile_name(profile: Profile) { // Takes ownership!
    println!("{}", profile.name);
} // profile is dropped here

// GOOD: Borrow instead
fn print_profile_name(profile: &Profile) { // Just borrows
    println!("{}", profile.name);
} // profile still valid in caller

// When to clone:
// 1. When you need to store the data
// 2. When lifetimes get too complex
// 3. When the data is small (like bool, i32)
```

### Mistake 3: String vs &str

```rust
// BAD: Taking String when &str would work
fn validate_email(email: String) -> bool { // Forces caller to give up ownership
    email.contains('@')
}

// GOOD: Take &str for read-only string operations
fn validate_email(email: &str) -> bool { // Works with String or &str
    email.contains('@')
}

// Usage
let email = String::from("test@example.com");
let is_valid = validate_email(&email); // ✓ Can still use email after
```

## Ownership in Async Code

```rust
// Async functions and ownership
async fn fetch_profile(name: String) -> Result<Profile, Error> {
    // name is moved into the async function
    let url = format!("https://api.example.com/profiles/{}", name);
    // ... fetch from URL ...
    Ok(Profile::new(&name))
}

// Sharing data between async tasks
use tokio::sync::Mutex;
use std::sync::Arc;

async fn concurrent_profile_updates() {
    let shared_profiles = Arc::new(Mutex::new(Vec::<Profile>::new()));
    
    let mut tasks = vec![];
    
    for i in 0..10 {
        let profiles = Arc::clone(&shared_profiles); // Clone the Arc, not the data
        
        let task = tokio::spawn(async move {
            let profile = Profile::new(&format!("user{}", i));
            let mut profiles = profiles.lock().await;
            profiles.push(profile);
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }
}
```

## Practice Exercises

### Exercise 1: Fix the Ownership Error
```rust
// This code has ownership issues. Fix it!
struct Config {
    path: String,
}

fn setup() -> Config {
    let config = Config {
        path: String::from("/home/user/.config"),
    };
    
    validate_config(config);
    
    config // ERROR: value used after move
}

fn validate_config(config: Config) {
    println!("Validating: {}", config.path);
}
```

### Exercise 2: Implement a Cache
```rust
// Implement a simple cache that doesn't take ownership of keys
use std::collections::HashMap;

struct Cache<T> {
    data: HashMap<String, T>,
}

impl<T> Cache<T> {
    fn new() -> Self {
        Self { data: HashMap::new() }
    }
    
    // Fix this method signature
    fn get(&self, key: String) -> Option<&T> {
        self.data.get(&key)
    }
    
    // Fix this method signature  
    fn insert(&mut self, key: String, value: T) {
        self.data.insert(key, value);
    }
}
```

### Exercise 3: Lifetime Relationships
```rust
// Make this compile by adding the right lifetime annotations
struct ProfileRef {
    name: &str,
    email: &str,
}

impl ProfileRef {
    fn new(name: &str, email: &str) -> Self {
        Self { name, email }
    }
    
    fn format(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
}
```

## Solutions to Exercises

### Solution 1: Fix the Ownership Error
```rust
fn setup() -> Config {
    let config = Config {
        path: String::from("/home/user/.config"),
    };
    
    validate_config(&config); // Borrow instead of move
    
    config
}

fn validate_config(config: &Config) { // Take a reference
    println!("Validating: {}", config.path);
}
```

### Solution 2: Implement a Cache
```rust
impl<T> Cache<T> {
    fn new() -> Self {
        Self { data: HashMap::new() }
    }
    
    // Take &str to avoid taking ownership
    fn get(&self, key: &str) -> Option<&T> {
        self.data.get(key)
    }
    
    // Take String for the key (we need to store it)
    fn insert(&mut self, key: String, value: T) {
        self.data.insert(key, value);
    }
}
```

### Solution 3: Lifetime Relationships
```rust
struct ProfileRef<'a> {
    name: &'a str,
    email: &'a str,
}

impl<'a> ProfileRef<'a> {
    fn new(name: &'a str, email: &'a str) -> Self {
        Self { name, email }
    }
    
    fn format(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
}
```

## Key Takeaways

1. **Think about ownership from the start** - Who owns this data? Who needs to access it?
2. **Prefer borrowing over owning** - Use `&T` instead of `T` when possible
3. **Clone is okay** - Sometimes cloning is the clearest solution
4. **Use smart pointers when needed** - `Arc`, `Rc`, `RefCell` solve specific problems
5. **The compiler is your friend** - Ownership errors prevent runtime crashes

## Next Steps

- Read [The Rust Book Chapter 4](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- Practice with the exercises in `examples/ownership/`
- Try refactoring some Python/JavaScript code to Rust
- Ask questions in #rust-beginners when stuck!

Remember: Everyone struggles with ownership at first. It gets easier with practice, and the safety it provides is worth the initial learning curve.