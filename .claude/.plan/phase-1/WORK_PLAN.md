# Phase 1: Foundation & Security - Work Plan

## Prerequisites

Before starting Phase 1, ensure you have the foundation in place.

**Required Knowledge**:
- **Rust Fundamentals**: Ownership, error handling, traits (*critical*)
- **Security Concepts**: File permissions, memory safety (*critical*)
- **Testing**: Unit and integration testing in Rust (*required*)
- **Cross-Platform**: Basic Windows/Unix differences (*helpful*)

💡 **Junior Dev Resources**:
- 📚 [The Rust Book](https://doc.rust-lang.org/book/) - Complete before starting
- 📖 [Rust Error Handling Guide](https://doc.rust-lang.org/book/ch09-00-error-handling.html) - Comprehensive guide
- 📖 [Security Best Practices](https://anssi-fr.github.io/rust-guide/) - Read sections 1-3
- 🔧 [Rust Playground](https://play.rust-lang.org/) - Practice concepts
- 📝 [Project Context](../../resources/CONTEXT.md) - Understand what we're building
- ⚠️ [Critical Issues](../../.reviews/RECOMMENDATIONS_2024_01.md) - Must-fix problems
- 🔐 [Security Implementation Examples](../SECURITY_IMPLEMENTATION_EXAMPLES.md) - **MUST READ** for Phase 1
- 🦀 [Rust Ownership Examples](../RUST_OWNERSHIP_EXAMPLES.md) - Read if new to Rust ownership

## Quick Reference - Essential Resources

### File System & Security
- [std::fs documentation](https://doc.rust-lang.org/std/fs/) - File operations
- [tempfile crate](https://docs.rs/tempfile/) - Safe temporary files
- [zeroize crate](https://docs.rs/zeroize/) - Secure memory wiping
- [nix crate](https://docs.rs/nix/) - Unix permissions

### Project Resources
- **[SPEC.md](../../.spec/SPEC.md)** - See NFR-002 Security Requirements
- **[TDD Guide](../../resources/guides/TDD_GUIDE.md)** - MANDATORY: Test-driven development practices
- **[Rust Patterns](../../resources/patterns/RUST_PATTERNS.md)** - Project-specific patterns to follow
- **[Architecture](../../.spec/architecture/ARCHITECTURE.md)** - System design overview
- **[Known Issues](../../.reviews/SENIOR_DEV_REVIEW.md)** - Critical problems to avoid
- **[Rust Glossary](../../resources/rust-glossary.md)** - Quick reference for Rust concepts

### Commands
- `cargo test` - Run all tests
- `cargo test -- --nocapture` - See test output
- `cargo tarpaulin --out Html` - Generate coverage report
- `cargo clippy -- -D warnings` - Strict linting

## Overview

Phase 1 establishes the secure foundation for git-setup-rs, implementing critical security infrastructure that all other phases depend on. This phase is security-critical - mistakes here affect the entire project.

**Key Deliverables**:
- Atomic file operations with rollback
- Memory-safe handling of sensitive data
- Cross-platform configuration system
- Comprehensive error handling

**Checkpoint Strategy**: 4 mandatory checkpoints with external review

**Time Estimate**: 2 weeks (80 hours)

## Development Methodology: Test-Driven Development (TDD)

💡 **Junior Dev Concept**: Test-Driven Development (TDD)
**What it is**: Write tests before writing the actual code
**Why we use it**: Ensures we build exactly what's needed and catch bugs early
**The Process**:
1. 🔴 **Red**: Write a failing test
2. 🟢 **Green**: Write minimal code to pass
3. 🔵 **Refactor**: Improve code while keeping tests green

## Done Criteria Checklist

Phase 1 is complete when ALL criteria are met:
- [ ] Atomic file operations working on all platforms
- [ ] Memory zeroization verified for all sensitive data
- [ ] Configuration loads from TOML/YAML/JSON/env
- [ ] File permissions automatically set to 600/700
- [ ] Cross-platform paths handled correctly
- [ ] Test coverage ≥90% on security-critical paths
- [ ] cargo audit shows no vulnerabilities
- [ ] Documentation complete with security notes
- [ ] All 4 checkpoints reviewed and approved

## Work Breakdown with Review Checkpoints

### 1.1 Secure File System Module (20 hours)

**Complexity**: High - Critical security component
**Files**: `src/fs/mod.rs`, `src/fs/atomic.rs`, `src/fs/permissions.rs`

#### Task 1.1.1: Design Atomic File Operations API (4 hours)

💡 **Junior Dev Concept**: Atomic File Operations
**What it is**: File writes that either completely succeed or completely fail - no partial writes
**Why we use it**: Prevents corrupted config files if the program crashes mid-write
**Real Example**: If power fails while saving a profile, the old profile remains intact

**Prerequisites**:
- [ ] Read: [Atomic File Operations Explained](https://lwn.net/Articles/457667/)
- [ ] Understand: Rust's `Result<T, E>` for error handling
- [ ] Practice: Try the tempfile crate examples

**Step-by-Step Implementation**:

1. **Create the module structure** (30 minutes)
   ```bash
   mkdir -p src/fs
   touch src/fs/mod.rs src/fs/atomic.rs src/fs/permissions.rs
   ```
   
   In `src/fs/mod.rs`:
   ```rust
   pub mod atomic;
   pub mod permissions;
   
   pub use atomic::AtomicWrite;
   pub use permissions::SecurePermissions;
   ```

2. **Design the trait API** (1 hour)
   ```rust
   // src/fs/atomic.rs
   use std::path::Path;
   use std::io;
   
   /// Trait for atomic file operations
   pub trait AtomicWrite {
       /// Write content atomically - all or nothing
       fn write_atomic(&self, path: &Path, content: &[u8]) -> io::Result<()>;
       
       /// Write with automatic backup of existing file
       fn write_with_backup(&self, path: &Path, content: &[u8]) -> io::Result<PathBuf>;
   }
   ```
   
   💡 **Tip**: Use `&[u8]` instead of `String` to handle binary data too

3. **Write the trait documentation** (1 hour)
   Add comprehensive docs explaining:
   - When to use each method
   - Error conditions
   - Platform-specific behaviors
   
   ⚠️ **Common Mistake**: Forgetting to document platform differences
   ✅ **Instead**: Add a "Platform Notes" section to your docs

4. **Create the test plan** (1.5 hours)
   ```rust
   // src/fs/atomic.rs
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_atomic_write_success() {
           // TODO: Test successful write
       }
       
       #[test]
       fn test_atomic_write_rollback_on_error() {
           // TODO: Test that partial writes don't happen
       }
       
       #[test]
       fn test_concurrent_writes_are_safe() {
           // TODO: Test multiple writers don't corrupt
       }
   }
   ```

#### Task 1.1.2: Implement Atomic Write Operations (8 hours)

**Prerequisites**:
- [ ] Completed Task 1.1.1
- [ ] Read: [tempfile crate docs](https://docs.rs/tempfile/)
- [ ] Understand: File system rename operations

**Step-by-Step TDD Implementation**:

1. **Write the first failing test** (30 minutes)
   ```rust
   #[test]
   fn test_atomic_write_creates_file() {
       let temp_dir = tempfile::tempdir().unwrap();
       let file_path = temp_dir.path().join("test.txt");
       
       let writer = AtomicFileWriter::new();
       writer.write_atomic(&file_path, b"Hello, World!").unwrap();
       
       assert!(file_path.exists());
       assert_eq!(std::fs::read(&file_path).unwrap(), b"Hello, World!");
   }
   ```
   
   🔴 **This test will fail** - we haven't implemented `AtomicFileWriter` yet!

2. **Implement minimal code to pass** (2 hours)
   ```rust
   use std::fs;
   use std::io;
   use std::path::Path;
   use tempfile::NamedTempFile;
   
   pub struct AtomicFileWriter;
   
   impl AtomicFileWriter {
       pub fn new() -> Self {
           AtomicFileWriter
       }
   }
   
   impl AtomicWrite for AtomicFileWriter {
       fn write_atomic(&self, path: &Path, content: &[u8]) -> io::Result<()> {
           // Create temp file in same directory as target
           let dir = path.parent().ok_or_else(|| {
               io::Error::new(io::ErrorKind::InvalidInput, "Invalid path")
           })?;
           
           let mut temp_file = NamedTempFile::new_in(dir)?;
           
           // Write content to temp file
           temp_file.write_all(content)?;
           
           // Sync to disk (important!)
           temp_file.as_file().sync_all()?;
           
           // Atomic rename
           temp_file.persist(path)?;
           
           Ok(())
       }
   }
   ```
   
   💡 **Why temp file + rename?**: The rename operation is atomic on most file systems

3. **Add error handling tests** (2 hours)
   ```rust
   #[test]
   fn test_atomic_write_cleans_up_on_error() {
       // Test that temp files are cleaned up if write fails
   }
   
   #[test]
   fn test_atomic_write_preserves_permissions() {
       // Test that existing file permissions are maintained
   }
   ```

4. **Implement rollback logic** (3.5 hours)
   - Handle write failures
   - Clean up temp files
   - Preserve original file on error
   
   ⚠️ **Common Mistake**: Leaking temp files on error
   ✅ **Instead**: Use RAII - temp files auto-delete on drop

#### Task 1.1.3: Implement Cross-Platform Permissions (4 hours)

💡 **Junior Dev Concept**: File Permissions
**What it is**: Control who can read/write/execute files
**Why we need it**: Protect sensitive data like SSH keys from other users
**Platform Differences**:
- Unix: Simple numeric modes (600 = owner read/write only)
- Windows: Complex ACLs (Access Control Lists)

**Implementation Guide**:

1. **Unix permissions implementation** (2 hours)
   ```rust
   #[cfg(unix)]
   pub fn set_secure_permissions(path: &Path) -> io::Result<()> {
       use std::os::unix::fs::PermissionsExt;
       
       let metadata = fs::metadata(path)?;
       let mut perms = metadata.permissions();
       
       // 0o600 = read/write for owner only
       perms.set_mode(0o600);
       fs::set_permissions(path, perms)?;
       
       Ok(())
   }
   ```

2. **Windows permissions implementation** (2 hours)
   ```rust
   #[cfg(windows)]
   pub fn set_secure_permissions(path: &Path) -> io::Result<()> {
       // Windows is more complex - may need windows-acl crate
       // For now, we'll rely on default NTFS permissions
       Ok(())
   }
   ```
   
   📚 **Learn More**: [Windows File Security](https://docs.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights)

#### Task 1.1.4: Integration Tests (4 hours)

**Test Scenarios to Cover**:

1. **Concurrent access test** (1 hour)
   ```rust
   #[test]
   fn test_multiple_writers_dont_corrupt() {
       // Spawn multiple threads writing to same file
       // Verify no corruption occurs
   }
   ```

2. **Crash recovery test** (1 hour)
   ```rust
   #[test]
   fn test_interrupted_write_doesnt_corrupt() {
       // Simulate process crash during write
       // Verify original file intact
   }
   ```

3. **Permission preservation test** (1 hour)
   ```rust
   #[test]
   fn test_permissions_preserved_after_write() {
       // Create file with specific permissions
       // Write atomically
       // Verify permissions unchanged
   }
   ```

4. **Cross-platform path test** (1 hour)
   ```rust
   #[test]
   fn test_handles_platform_paths() {
       // Test Windows paths: C:\Users\...
       // Test Unix paths: /home/user/...
       // Test relative paths: ./config
   }
   ```

---

## 🛑 CHECKPOINT 1: Secure File System Complete

### ⚠️ MANDATORY STOP POINT ⚠️

**DO NOT PROCEED** past this checkpoint without completing the review process and obtaining all required approvals.

### Pre-Checkpoint Checklist

Before requesting review, ensure ALL items are complete:

- [ ] All code committed and pushed to feature branch
- [ ] All tests passing locally (`cargo test --all`)
- [ ] Code coverage ≥ 95% for fs module: `cargo tarpaulin --out Html`
- [ ] No compiler warnings: `cargo build --all-targets`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation complete for all public APIs
- [ ] Security analysis document written
- [ ] Benchmarks show <10ms overhead

### Review Process

1. **Create Pull Request**
   ```bash
   git checkout -b checkpoint/phase-1-1-secure-file-system
   git add .
   git commit -m "CHECKPOINT: Phase 1.1 - Secure File System Complete"
   git push origin checkpoint/phase-1-1-secure-file-system
   ```
   
   PR Title: `CHECKPOINT: Phase 1.1 - Secure File System Complete`
   
   PR Description MUST include:
   - [ ] Link to this checkpoint in WORK_PLAN.md
   - [ ] Summary of atomic operations implementation
   - [ ] Platform-specific handling notes
   - [ ] Performance benchmark results
   - [ ] Security considerations addressed

2. **Run Automated Checks**
   The following CI checks MUST pass:
   - [ ] Build and test on Windows, macOS, Linux
   - [ ] Code coverage ≥ 95%
   - [ ] Security scan (cargo audit)
   - [ ] Benchmarks within targets

3. **Request Reviews**
   Tag required reviewers in PR:
   - @tech-lead - Architecture and code quality
   - @security-engineer - Security implementation
   - @qa-engineer - Test coverage and quality

### Review Criteria

#### Code Quality (Tech Lead)
- [ ] Atomic operations truly atomic
- [ ] Error handling comprehensive
- [ ] No resource leaks
- [ ] Clean abstractions

#### Security (Security Engineer)
- [ ] File permissions correctly set
- [ ] Temp files secure
- [ ] No race conditions
- [ ] Rollback works correctly

#### Test Quality (QA Engineer)
- [ ] All edge cases covered
- [ ] Integration tests comprehensive
- [ ] Platform differences tested
- [ ] Benchmarks included

### Approval Requirements

The following approvals are REQUIRED:

- [ ] **Tech Lead**: Approved ✅
- [ ] **Security Engineer**: Approved ✅
- [ ] **QA Engineer**: Approved ✅
- [ ] **CI Pipeline**: All checks passed ✅

### Consequences of Skipping

**If you proceed without approval**:
- Entire phase 1 work rejected
- Security vulnerabilities in foundation
- Rework of dependent phases
- Project delay of 1-2 weeks
- Performance review impact

---

### 1.2 Memory Safety Module (16 hours)

**Complexity**: High - Security critical
**Files**: `src/security/mod.rs`, `src/security/zeroize.rs`, `src/security/sensitive.rs`

💡 **Junior Dev Concept**: Memory Zeroization
**What it is**: Overwriting sensitive data in memory with zeros before freeing it
**Why we need it**: Prevents passwords/keys from being read from memory dumps
**Example**: If your program crashes, passwords might still be in RAM unless zeroized

📖 **Essential Reading**: See [Security Implementation Examples - Layer 1: Memory Security](../SECURITY_IMPLEMENTATION_EXAMPLES.md#layer-1-memory-security) for complete examples

#### Task 1.2.1: Design Secure String Types (4 hours)

**Prerequisites**:
- [ ] Read: [zeroize crate documentation](https://docs.rs/zeroize/)
- [ ] Understand: Rust's Drop trait
- [ ] Learn: Why String::clear() isn't enough

**Step-by-Step Implementation**:

1. **Understand the security problem** (30 minutes)
   ```rust
   // ❌ BAD: Password stays in memory after drop
   {
       let password = String::from("super-secret");
       // ... use password ...
   } // password memory might still contain "super-secret"
   
   // ✅ GOOD: Password zeroized on drop
   {
       let password = SensitiveString::from("super-secret");
       // ... use password ...
   } // memory guaranteed to be zeroed
   ```

2. **Design the secure types** (1.5 hours)
   ```rust
   use zeroize::{Zeroize, ZeroizeOnDrop};
   
   /// A String that automatically zeros memory when dropped
   #[derive(Zeroize, ZeroizeOnDrop)]
   pub struct SensitiveString(String);
   
   impl SensitiveString {
       pub fn new(s: String) -> Self {
           SensitiveString(s)
       }
       
       /// Get the value - be careful not to clone!
       pub fn expose(&self) -> &str {
           &self.0
       }
   }
   
   /// A buffer for binary sensitive data (like keys)
   pub struct SecureBuffer {
       data: Vec<u8>,
   }
   
   impl Drop for SecureBuffer {
       fn drop(&mut self) {
           self.data.zeroize();
       }
   }
   ```
   
   💡 **Design Decision**: We use newtype pattern to prevent accidental misuse

3. **Write comprehensive tests** (2 hours)
   ```rust
   #[test]
   fn test_sensitive_string_zeroizes_on_drop() {
       let ptr: *const u8;
       let len: usize;
       
       {
           let sensitive = SensitiveString::new("password123".to_string());
           ptr = sensitive.expose().as_ptr();
           len = sensitive.expose().len();
       } // sensitive dropped here
       
       // SAFETY: We're just reading memory we know was allocated
       unsafe {
           let slice = std::slice::from_raw_parts(ptr, len);
           assert!(slice.iter().all(|&b| b == 0));
       }
   }
   ```
   
   ⚠️ **Warning**: This test uses unsafe - be very careful!

#### Task 1.2.2: Implement Zeroize Integration (6 hours)

[Continue with similar detailed breakdowns for remaining tasks...]

## Common Issues & Solutions

### Issue: Tests Pass Locally but Fail in CI
**Symptom**: `cargo test` works on your machine but fails in GitHub Actions
**Likely Cause**: Platform-specific code not properly gated
**Solution**:
```rust
#[cfg(unix)]
#[test]
fn test_unix_permissions() {
    // Unix-only test
}

#[cfg(windows)]
#[test]
fn test_windows_permissions() {
    // Windows-only test
}
```

### Issue: Atomic Write Fails with "Cross-device link"
**Symptom**: Error when temp directory is on different file system
**Cause**: Can't rename across file system boundaries
**Solution**: Ensure temp file is in same directory as target:
```rust
let temp_file = NamedTempFile::new_in(target_dir)?;
```

## Performance Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| Atomic write (1KB) | <5ms | <10ms |
| Permission set | <1ms | <5ms |
| Zeroize (1KB) | <0.1ms | <1ms |
| Config load | <10ms | <20ms |

## Security Checklist

Before completing Phase 1, verify:
- [ ] No sensitive data in logs
- [ ] All sensitive strings use SensitiveString
- [ ] File permissions restrictive by default
- [ ] Atomic operations truly atomic
- [ ] Memory zeroization verified
- [ ] No hardcoded paths or secrets

## Learning Resources

### 📚 Recommended Reading Order
1. [The Rust Book - Chapter 9: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
2. [Secure Coding in Rust](https://anssi-fr.github.io/rust-guide/)
3. [File System Programming](https://riptutorial.com/rust/topic/4080/file-i-o)

### 📖 Additional Tutorials
- [Rust Error Handling Best Practices](https://doc.rust-lang.org/rust-by-example/error.html)
- [Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)

### 💬 Getting Help
- Post questions in #rust-beginners channel
- Tag @rust-mentor for pairing sessions
- Check examples/ directory for working code

## Next Phase Preview

Phase 2 will build upon this security foundation to implement:
- Profile data structures
- CRUD operations with validation
- Git configuration management
- Import/export functionality

Make sure Phase 1 is solid - everything else depends on it!

---

*Last updated: 2025-07-30*