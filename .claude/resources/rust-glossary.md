# Rust Concepts Glossary for git-setup-rs

This glossary explains Rust concepts used throughout the phase plans, aimed at developers with 6-12 months of Rust experience.

## Table of Contents
- [Ownership & Borrowing](#ownership--borrowing)
- [Smart Pointers](#smart-pointers)
- [Traits & Generics](#traits--generics)
- [Error Handling](#error-handling)
- [Async Programming](#async-programming)
- [Testing](#testing)
- [Security Concepts](#security-concepts)
- [Common Patterns](#common-patterns)

## Ownership & Borrowing

### `Arc<T>` - Atomic Reference Counter
**What**: Allows multiple owners of the same data
**When to use**: When you need to share read-only data between threads
**Example in project**:
```rust
pub profile_manager: Arc<dyn ProfileManager>
// Multiple parts of the TUI can read the profile manager
```
**Learn more**: [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)

### `Mutex<T>` - Mutual Exclusion
**What**: Ensures only one thread can access data at a time
**When to use**: When shared data needs to be modified
**Example in project**:
```rust
pub profile_manager: Arc<Mutex<dyn ProfileManager>>
// TUI can safely modify profiles from any component
```
**Learn more**: [std::sync::Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html)

### Lifetime Parameters (`'a`)
**What**: Tells Rust how long references are valid
**When to use**: When storing references in structs
**Example**:
```rust
struct ProfileRef<'a> {
    name: &'a str,  // This reference lives as long as 'a
}
```
**Common confusion**: Lifetimes don't affect runtime, only compile-time checks

## Smart Pointers

### `Box<T>` - Heap Allocation
**What**: Stores data on the heap instead of stack
**When to use**: For large data or recursive types
**Example**:
```rust
Box::new(Profile::new())  // Profile allocated on heap
```

### `Rc<T>` - Reference Counter (single-threaded)
**What**: Like Arc but for single-threaded use
**When to use**: Shared ownership within one thread
**Prefer Arc** in most cases for future-proofing

### `RefCell<T>` - Interior Mutability
**What**: Allows mutation of data even with immutable reference
**When to use**: When you need to mutate data inside an immutable struct
**Warning**: Can panic at runtime if borrowing rules violated

## Traits & Generics

### Trait Bounds
**What**: Constraints on generic types
**Example**:
```rust
fn process<T: Send + Sync>(data: T) { }
// T must implement both Send and Sync traits
```

### Common Trait Bounds in Project:

#### `Send`
**What**: Type can be transferred between threads
**Most types are Send** except Rc, raw pointers

#### `Sync`
**What**: Type can be shared between threads
**Rule**: T is Sync if &T is Send

#### `Clone` vs `Copy`
- **Clone**: Explicit duplication with `.clone()`
- **Copy**: Implicit copy on assignment (only for small types)

### `dyn Trait` - Dynamic Dispatch
**What**: Trait object for runtime polymorphism
**When to use**: When you don't know concrete type at compile time
**Example**:
```rust
pub profile_manager: Arc<Mutex<dyn ProfileManager>>
// Can use any type implementing ProfileManager
```
**Cost**: Small runtime overhead vs static dispatch

## Error Handling

### `Result<T, E>`
**What**: Type for operations that might fail
**Convention**: 
- `Ok(value)` for success
- `Err(error)` for failure

### Error Propagation with `?`
**What**: Shorthand for returning errors early
**Equivalent to**:
```rust
// Using ?
let content = fs::read_to_string(path)?;

// Equivalent to
let content = match fs::read_to_string(path) {
    Ok(c) => c,
    Err(e) => return Err(e.into()),
};
```

### Custom Error Types with `thiserror`
**What**: Macro for creating error enums
**Example**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("Profile not found: {name}")]
    NotFound { name: String },
    
    #[error("Invalid profile format")]
    InvalidFormat(#[from] toml::de::Error),
}
```

## Async Programming

### `async`/`await`
**What**: Syntax for asynchronous code
**When to use**: For I/O operations that might block
**Example**:
```rust
async fn load_profile(name: &str) -> Result<Profile> {
    let content = tokio::fs::read_to_string(path).await?;
    //                                                ^^^^^ waits without blocking thread
    Ok(toml::from_str(&content)?)
}
```

### `#[tokio::main]` and `#[tokio::test]`
**What**: Macros to set up async runtime
**Use**: Required for running async code in main() or tests

### Future Trait
**What**: Core trait for async values
**Usually hidden**: async functions return `impl Future<Output = T>`

## Testing

### `#[cfg(test)]`
**What**: Conditional compilation for test code
**Result**: Test code not included in release binary

### Test Organization
```rust
// Unit tests go in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() { }
}

// Integration tests go in tests/ directory
// tests/integration_test.rs
```

### Test Utilities

#### `tempfile` crate
**What**: Creates temporary files/directories for testing
**Cleanup**: Automatically deleted when dropped
```rust
let temp_dir = tempfile::tempdir()?;
let temp_path = temp_dir.path().join("test.txt");
```

## Security Concepts

### Zeroization
**What**: Overwriting memory with zeros before freeing
**Why**: Prevents sensitive data from persisting in RAM
**How**:
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
struct Password(String);  // Automatically zeroed when dropped
```

### Atomic Operations (File System)
**What**: Operations that fully complete or don't happen at all
**Implementation**: Write to temp file, then rename
**Why rename?**: Rename is atomic on most file systems

## Common Patterns

### Newtype Pattern
**What**: Wrapping a type in a struct for type safety
**Example**:
```rust
pub struct ProfileName(String);
// Can't accidentally pass a regular String where ProfileName expected
```

### Builder Pattern
**What**: Construct complex objects step by step
**Example**:
```rust
let profile = ProfileBuilder::new("work")
    .email("work@company.com")
    .signing_key(key)
    .build()?;
```

### RAII (Resource Acquisition Is Initialization)
**What**: Resources cleaned up when object dropped
**Examples**:
- Files closed
- Locks released
- Memory freed
**In Rust**: Automatic via Drop trait

### Type State Pattern
**What**: Encode state in type system
**Example**:
```rust
struct Profile<State> {
    data: ProfileData,
    _state: PhantomData<State>,
}

struct Valid;
struct Invalid;

// Can only save valid profiles
impl Profile<Valid> {
    fn save(&self) -> Result<()> { }
}
```

## Quick Reference

### When to use which smart pointer?
- **Single owner, heap**: `Box<T>`
- **Shared, immutable**: `Arc<T>`
- **Shared, mutable**: `Arc<Mutex<T>>`
- **Single thread shared**: `Rc<T>` or `Rc<RefCell<T>>`

### When to use which error handling?
- **Recoverable errors**: `Result<T, E>`
- **Unrecoverable errors**: `panic!()` (rare)
- **Option when no value**: `Option<T>`
- **Custom errors**: `thiserror` crate

### Common trait combinations
- **Thread-safe**: `Send + Sync`
- **Cloneable**: `Clone + Send + Sync`
- **Serializable**: `Serialize + Deserialize` (serde)
- **Comparable**: `PartialEq + Eq + PartialOrd + Ord`

## Getting Help

### Where to learn more:
1. **The Rust Book**: [doc.rust-lang.org/book](https://doc.rust-lang.org/book/)
2. **Rust by Example**: [doc.rust-lang.org/rust-by-example](https://doc.rust-lang.org/rust-by-example/)
3. **std docs**: [doc.rust-lang.org/std](https://doc.rust-lang.org/std/)
4. **This Week in Rust**: [this-week-in-rust.org](https://this-week-in-rust.org/)

### Project-specific help:
- `#rust-beginners` - General Rust questions
- `#git-setup-dev` - Project-specific questions
- `@rust-mentor` - Request pairing session
- `examples/` - Working code examples

## Common Gotchas

1. **Mutex poisoning**: If thread panics while holding lock, mutex becomes "poisoned"
2. **Async in traits**: Requires `async-trait` crate or manual impl Future
3. **Clone vs reference**: Cloning is often fine - don't over-optimize
4. **String vs &str**: Use `String` for owned data, `&str` for borrowed
5. **unwrap() in production**: Almost always wrong - use `?` or `expect()` with message

---

*Remember: It's okay to not understand everything immediately. Rust's complexity is front-loaded - once concepts click, development becomes much smoother.*