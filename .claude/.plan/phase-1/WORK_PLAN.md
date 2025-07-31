# Phase 1: Foundation & Security - Work Plan

## Prerequisites

This is the first phase establishing the secure foundation for git-setup-rs.

**Required Knowledge**:
- **Rust Fundamentals**: Ownership, error handling, traits (*critical*)
- **Security Concepts**: Memory safety, file permissions, secure coding (*critical*)
- **File Systems**: Atomic operations, cross-platform paths (*required*)
- **Testing**: Unit tests, integration tests, TDD methodology (*required*)

**Required Tools**:
- Rust 1.75+ with cargo
- Git 2.25+
- IDE with Rust analyzer
- cargo-audit for security scanning

## Quick Reference - Essential Resources

### Crate Documentation
- [zeroize](https://docs.rs/zeroize) - Secure memory handling
- [figment](https://docs.rs/figment) - Configuration management
- [nix](https://docs.rs/nix) - Unix file permissions
- [tempfile](https://docs.rs/tempfile) - Atomic file operations

### Project Resources
- **[SPEC.md](../../.spec/SPEC.md)** - See sections: System Architecture, NFR-002 Security
- **[TDD Guide](../../resources/guides/TDD_GUIDE.md)** - MANDATORY: Test-driven development practices
- **[Rust Patterns](../../resources/patterns/RUST_PATTERNS.md)** - Project-specific Rust patterns
- **[Architecture](../../.spec/architecture/ARCHITECTURE.md)** - System architecture overview
- **[Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)** - Security best practices
- **[Known Issues](../../.reviews/SENIOR_DEV_REVIEW.md)** - Critical issues to avoid

### Commands
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo tarpaulin` - Generate coverage report
- `cargo audit` - Security vulnerability scan
- `cargo clippy` - Linting

## Overview

Phase 1 establishes the secure foundation for git-setup-rs, implementing atomic file operations, memory-safe credential handling, and configuration infrastructure. This phase is critical as all future phases depend on these security primitives.

**Deliverables**:
- Atomic file system operations with rollback
- Secure memory handling with zeroize
- Configuration layer with Figment
- Cross-platform path handling
- Comprehensive test suite with 90%+ coverage

**Checkpoint Strategy**: 4 checkpoints aligned with major components

**Time Estimate**: 2 weeks (80 hours)

## Development Methodology: Test-Driven Development (TDD)

**CRITICAL**: Every feature must follow strict TDD practices. See [TDD Guide](../../resources/guides/TDD_GUIDE.md) for detailed methodology.

1. **Write failing tests first** - Define security properties and behavior
2. **Implement minimum code** - Make tests pass with simplest solution
3. **Refactor with confidence** - Improve code while tests ensure correctness
4. **Security review** - Verify no credentials in memory after operations

⚠️ **WARNING**: The [Senior Dev Review](../../.reviews/SENIOR_DEV_REVIEW.md) found major issues with skipping TDD. Do NOT skip tests!

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

Design the public API for atomic file operations with rollback support.

**Deliverables**:
- API design document with trait definitions
- Security threat model for file operations
- Test plan covering failure scenarios

```rust
// Example API structure
pub trait AtomicWrite {
    fn write_atomic(&self, path: &Path, content: &[u8]) -> Result<()>;
    fn write_with_backup(&self, path: &Path, content: &[u8]) -> Result<PathBuf>;
}
```

#### Task 1.1.2: Implement Atomic Write Operations (8 hours)

**TDD Steps**:
1. Write tests for atomic write success cases
2. Write tests for rollback on failure
3. Write tests for concurrent access handling
4. Implement using tempfile crate with rename

**Key Requirements**:
- Use write-rename pattern for atomicity
- Preserve file permissions on existing files
- Handle cross-device moves gracefully
- Clean up temp files on all error paths

#### Task 1.1.3: Implement Cross-Platform Permissions (4 hours)

**Platform-Specific Requirements**:
- Unix: Use nix crate for chmod 600/700
- Windows: Use windows-acl crate for ACLs
- Validate permissions after setting

#### Task 1.1.4: Integration Tests (4 hours)

Write comprehensive integration tests:
- Concurrent write attempts
- Power failure simulation (process kill)
- Cross-platform path edge cases
- Permission preservation

---

### CHECKPOINT 1: Secure File System Complete

**Review Requirements**:
- Demonstrate atomic writes with rollback
- Show permission setting on all platforms
- Verify temp file cleanup in all scenarios
- Security review of implementation

**Deliverables for Review**:
- Working atomic file operations
- 95%+ test coverage
- Benchmarks showing <10ms overhead
- Security analysis document

---

### 1.2 Memory Safety Module (16 hours)

**Complexity**: High - Security critical
**Files**: `src/security/mod.rs`, `src/security/zeroize.rs`, `src/security/sensitive.rs`

#### Task 1.2.1: Design Secure String Types (4 hours)

Design wrapper types for sensitive data that auto-zeroize.

```rust
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SensitiveString(String);

pub struct SecureBuffer {
    data: Vec<u8>,
    // Auto-zeroizes on drop
}
```

#### Task 1.2.2: Implement Zeroize Integration (6 hours)

**TDD Requirements**:
- Test memory is actually zeroed (use unsafe to verify)
- Test all drop paths trigger zeroization
- Test cloning is prevented or secure

**Implementation Notes**:
- Integrate zeroize crate
- Implement Drop traits correctly
- Prevent accidental exposure via Debug/Display

#### Task 1.2.3: Create Secure Credential Store (6 hours)

Build in-memory credential store with automatic cleanup:
- Time-based expiration
- Secure storage with zeroize
- Access logging for audit

---

### CHECKPOINT 2: Memory Safety Complete

**Review Requirements**:
- Demonstrate zeroization with memory dump
- Show no credentials persisted after scope
- Verify secure types prevent leaks
- Performance impact analysis

**Deliverables for Review**:
- Secure string and buffer types
- Memory safety test suite
- Security audit trail
- Documentation with examples

---

### 1.3 Configuration Infrastructure (20 hours)

**Complexity**: Medium - Foundation for profiles
**Files**: `src/config/mod.rs`, `src/config/loader.rs`, `src/config/schema.rs`

#### Task 1.3.1: Define Configuration Schema (4 hours)

Create strongly-typed configuration structures:

```rust
#[derive(Deserialize, Serialize, Debug)]
pub struct AppConfig {
    pub profiles_dir: PathBuf,
    pub security: SecurityConfig,
    pub defaults: DefaultsConfig,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SecurityConfig {
    pub credential_timeout: Duration,
    pub require_confirmation: bool,
}
```

#### Task 1.3.2: Implement Figment Loader (8 hours)

**Requirements**:
- Load from multiple sources in order:
  1. Default values
  2. Config file (TOML/YAML/JSON)
  3. Environment variables
  4. Command-line overrides
- Validate all loaded values
- Secure handling of sensitive configs

#### Task 1.3.3: Configuration Persistence (4 hours)

Implement secure config writing:
- Use atomic file operations from 1.1
- Set secure permissions automatically
- Validate before writing

#### Task 1.3.4: Configuration Tests (4 hours)

Comprehensive test coverage:
- Multi-source merging
- Invalid configuration handling
- Security validation
- Migration from old formats

---

### CHECKPOINT 3: Configuration System Complete

**Review Requirements**:
- Demo loading from all sources
- Show secure persistence
- Verify validation works
- Profile configuration ready

**Deliverables for Review**:
- Working Figment integration
- Configuration schema defined
- Load/save operations
- Migration support

---

### 1.4 Foundation Integration & CI/CD (24 hours)

**Complexity**: Medium - Brings everything together
**Files**: `src/lib.rs`, `.github/workflows/`, `Cargo.toml`

#### Task 1.4.1: Create Public API Module (8 hours)

Design and implement the foundation crate's public API:
- Export secure types
- Export file operations
- Export configuration types
- Comprehensive documentation

#### Task 1.4.2: Setup CI/CD Pipeline (8 hours)

GitHub Actions workflow with:
- Multi-platform testing matrix (macOS, Linux, Windows)
- Security scanning with cargo-audit
- Coverage reporting with tarpaulin
- Clippy linting with strict settings
- Documentation building

#### Task 1.4.3: Integration Test Suite (4 hours)

End-to-end tests combining all modules:
- Config + secure storage
- File operations + permissions
- Memory safety verification

#### Task 1.4.4: Documentation & Examples (4 hours)

- README with security notes
- Example usage for each module
- Security best practices guide
- API documentation

---

### CHECKPOINT 4: Phase 1 Complete

**Final Review Requirements**:
- All modules integrated and working
- CI/CD pipeline green on all platforms
- Security audit passed
- Documentation complete
- Ready for Phase 2 to build upon

**Deliverables for Review**:
- Complete foundation crate
- 90%+ test coverage report
- Security analysis document
- Performance benchmarks
- Documentation site

## Troubleshooting

### Common Issues

**Atomic Operations Failing on Windows**
- Cause: Cross-device moves not supported
- Solution: Implement copy+delete fallback

**Zeroize Not Working**
- Cause: Compiler optimizations
- Solution: Use `black_box` to prevent optimization

**Permission Errors on CI**
- Cause: GitHub Actions filesystem restrictions
- Solution: Skip permission tests in CI environment

### Debugging Tools
- `RUST_LOG=debug` - Enable debug logging
- `cargo expand` - See macro expansions
- `valgrind` - Verify memory is cleared
- `strace/procmon` - Debug file operations

## Security Checklist

Before marking any task complete:
- [ ] No sensitive data in logs
- [ ] All credentials use secure types
- [ ] File permissions are restrictive
- [ ] Error messages don't leak info
- [ ] Temp files are cleaned up
- [ ] Memory is zeroized after use

## Next Phase Preview

Phase 2 (Core Profile Management) will build upon this foundation to implement:
- Profile CRUD operations using secure file system
- Git configuration management
- Profile validation and templates

The secure foundation from Phase 1 ensures all profile operations are safe by default.

---

*Last updated: 2025-07-30*