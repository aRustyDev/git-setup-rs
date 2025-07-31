# Non-Functional Requirements - git-setup-rs

## Performance Requirements

### Response Time
| Operation | Target | Degraded | Unacceptable |
|-----------|--------|----------|--------------|
| TUI Launch | <100ms | 100-200ms | >200ms |
| Profile Switch | <3s | 3-5s | >5s |
| Fuzzy Search (per keystroke) | <10ms | 10-25ms | >25ms |
| Profile List Load (100 items) | <50ms | 50-100ms | >100ms |
| 1Password Key List | <2s | 2-5s | >5s |
| Auto-detection Check | <100ms | 100-300ms | >300ms |
| Config File Parse | <20ms | 20-50ms | >50ms |
| Remote Profile Import | <5s | 5-10s | >10s |

### Resource Usage
| Metric | Target | Maximum | Measurement Point |
|--------|--------|---------|-------------------|
| Memory (Idle TUI) | <30MB | 50MB | After 5 min idle |
| Memory (Active) | <50MB | 100MB | During operations |
| CPU (Idle) | <1% | 2% | TUI open, no activity |
| CPU (Active) | <25% | 50% | During profile switch |
| Binary Size | <15MB | 25MB | Stripped release build |
| Config File Size | <1MB | 5MB | 100 profiles |

### Throughput
| Metric | Target | Minimum | Peak |
|--------|--------|---------|------|
| Concurrent TUI Instances | 5 | 3 | 10 |
| Profiles Manageable | 1000 | 100 | 5000 |
| Operations/Second | 50 | 25 | 100 |

## Reliability Requirements

### Availability
- Core Functions: 99.99% (local tool, no network dependency for basic ops)
- 1Password Integration: 99% (degrades gracefully when unavailable)
- Remote Profile Sources: 95% (with local caching fallback)

### Recovery
- Config Corruption Recovery: Automatic with <1s detection
- Backup Retention: Last 5 versions of profiles
- Crash Recovery: Restore to last stable state
- Transaction Rollback: All config changes are atomic

### Data Integrity
- Profile Validation: 100% of saves validated before write
- Atomic Writes: All configuration changes use write-rename pattern
- Checksum Verification: SHA-256 for remote profiles
- Version Control Compatible: Changes produce minimal, readable diffs

## Security Requirements

### Authentication
- 1Password: Biometric when available, password fallback
- No Service Account Tokens: Must use interactive auth only
- Session Management: Secure session caching for TUI lifetime
- Credential Timeout: Configurable, default 30 minutes

### Data Protection
- In Memory: Zeroize crate for all sensitive data
- At Rest: No private keys or passwords stored
- In Transit: TLS 1.3 for all remote operations
- Key References: Store only public keys and op:// URLs

### Access Control
- File Permissions: 600 for config files, 700 for directories
- Profile Isolation: Profiles cannot access other profiles' data
- Audit Trail: Optional logging of all profile changes
- Principle of Least Privilege: Request only needed permissions

### Security Standards
- OWASP Compliance: Input validation for all user data
- Supply Chain: Minimal dependencies, all audited
- Memory Safety: 100% safe Rust, no unsafe blocks in core
- Cryptography: Use only standard, audited implementations

## Usability Requirements

### User Experience
- Learning Curve: New user productive in <5 minutes
- Error Messages: Actionable, suggest fixes, no panic traces
- Help System: Context-sensitive help in TUI
- Keyboard Navigation: Full functionality without mouse
- Undo/Redo: Last 10 operations reversible

### Accessibility
- Terminal Compatibility: Works in all major terminals
- Color Schemes: Configurable, colorblind-friendly defaults
- Font Support: Works with standard monospace fonts
- Screen Reader: Compatible output for key operations
- Keyboard Shortcuts: Follows platform conventions

### Documentation
- In-App Help: Available for all features
- Man Page: Complete Unix manual page
- Online Docs: Searchable, versioned documentation
- Examples: 20+ real-world usage examples
- Video Tutorials: 5-minute quickstart guide

## Platform Requirements

### Operating System Support
| OS | Minimum Version | Priority | Notes |
|----|-----------------|----------|-------|
| macOS | 10.15 (Catalina) | Highest | Native arm64 + x86_64 |
| Linux | Kernel 4.19+ | High | glibc 2.27+, musl support |
| Windows | 10 version 1903 | Medium | Native, not WSL required |
| FreeBSD | 12.0 | Low | Community supported |

### Architecture Support
- x86_64: Full support, optimized
- arm64: Full support (Apple Silicon, AWS Graviton)
- armv7: Best effort for Raspberry Pi
- WASM: Future consideration

### Dependencies
- Git: 2.25.0 or higher
- 1Password CLI: 2.0.0 or higher (optional)
- Terminal: 256 color support recommended

## Compliance Requirements

### Standards
- GDPR: No telemetry without explicit consent
- SOC2: Audit trail capabilities
- HIPAA: No health data processing (N/A)
- PCI: No payment data processing (N/A)

### Telemetry (Opt-in)
- OTEL Compatible: OpenTelemetry protocol support
- Privacy: No PII collected
- Local First: Telemetry buffered locally
- Transparent: User can inspect all sent data

### Export Control
- Cryptography Notice: Uses standard crypto (exempt)
- Distribution: No geographic restrictions
- License: MIT or Apache 2.0 (user choice)

## Maintainability Requirements

### Code Quality
- Test Coverage: >80% overall, >95% for critical paths
- Documentation: All public APIs documented
- Linting: Zero clippy warnings on stable
- Formatting: Enforced via rustfmt

### Modularity
- Plugin Architecture: Core + optional features
- Feature Flags: Compile-time feature selection
- Clean Interfaces: Traits for all external integrations

### Monitoring
- Structured Logging: JSON output option
- Metrics: Prometheus-compatible metrics
- Tracing: Distributed tracing support
- Debug Mode: Verbose troubleshooting output

## Scalability Requirements

### Growth Dimensions
| Dimension | Current | Year 1 | Year 3 |
|-----------|---------|---------|---------|
| Users | 1 | 10,000 | 100,000 |
| Profiles/User | 5 | 20 | 50 |
| Features | 20 | 50 | 100 |
| Platforms | 3 | 4 | 6 |

### Performance Scaling
- Linear: Performance degrades linearly with profile count
- Lazy Loading: Load only active profile data
- Caching: Smart caching of frequently used data
- Background Tasks: Non-blocking 1Password operations

---
*These NFRs must be testable via automated tests and monitored in production.*