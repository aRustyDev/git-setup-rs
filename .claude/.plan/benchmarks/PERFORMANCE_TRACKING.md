# Performance Tracking Template

Use this template to track performance metrics across development phases.

## Current Performance Metrics

*Last Updated: [DATE]*

### Startup Performance
| Metric | Target | Current | Status | Notes |
|--------|--------|---------|--------|-------|
| Cold Start | <100ms | ___ms | ⬜ | First run |
| Warm Start | <50ms | ___ms | ⬜ | With cache |
| TUI Launch | <150ms | ___ms | ⬜ | To interactive |
| CLI Response | <50ms | ___ms | ⬜ | Simple command |

### Profile Operations
| Operation | Target | Current | Status | Notes |
|-----------|--------|---------|--------|-------|
| Load Profile | <20ms | ___ms | ⬜ | Single profile |
| Switch Profile | <100ms | ___ms | ⬜ | Including Git config |
| List Profiles | <10ms | ___ms | ⬜ | Up to 100 profiles |
| Create Profile | <50ms | ___ms | ⬜ | With validation |
| Delete Profile | <20ms | ___ms | ⬜ | With cleanup |

### Security Operations
| Operation | Target | Current | Status | Notes |
|-----------|--------|---------|--------|-------|
| Atomic Write | <30ms | ___ms | ⬜ | Average file |
| Credential Load | <50ms | ___ms | ⬜ | From 1Password |
| Memory Clear | <1ms | ___μs | ⬜ | SensitiveString drop |
| Path Validation | <5ms | ___ms | ⬜ | Complex path |

### Platform Operations
| Platform | Operation | Target | Current | Status |
|----------|-----------|--------|---------|--------|
| Windows | Path Normalize | <10ms | ___ms | ⬜ |
| Windows | Long Path | <15ms | ___ms | ⬜ |
| macOS | Universal Binary | +0% | __% | ⬜ |
| Linux | Path Operations | <5ms | ___ms | ⬜ |

### Resource Usage
| Resource | Target | Current | Status | Notes |
|----------|--------|---------|--------|-------|
| Memory (Idle) | <20MB | ___MB | ⬜ | After startup |
| Memory (Active) | <50MB | ___MB | ⬜ | During operation |
| CPU (Idle) | <1% | __% | ⬜ | When waiting |
| Binary Size | <20MB | ___MB | ⬜ | Stripped release |

## Performance History

### Version 0.1.0 - Phase 1-2 Complete
| Metric | Value | Change | Notes |
|--------|-------|--------|-------|
| Startup | 150ms | Baseline | Initial implementation |
| Profile Load | 25ms | Baseline | No optimization |
| Memory | 45MB | Baseline | Debug symbols included |

### Version 0.2.0 - Phase 3-4 Complete
| Metric | Value | Change | Notes |
|--------|-------|--------|-------|
| Startup | | | |
| Profile Load | | | |
| Memory | | | |

### Version 0.3.0 - Phase 5 Complete
| Metric | Value | Change | Notes |
|--------|-------|--------|-------|
| Startup | | | |
| Profile Load | | | |
| Memory | | | |

### Version 1.0.0 - Phase 6 Complete
| Metric | Value | Change | Notes |
|--------|-------|--------|-------|
| Startup | | | |
| Profile Load | | | |
| Memory | | | |

## Optimization Log

### [DATE] - Optimization Description
**Target**: Reduce profile loading time
**Approach**: Cache parsed TOML data
**Result**: 25ms → 15ms (-40%)
**Trade-off**: +2MB memory usage

### [DATE] - Optimization Description
**Target**: 
**Approach**: 
**Result**: 
**Trade-off**: 

## Performance Regression Tracking

### Regression Template
```
Date: [DATE]
Version: [COMMIT/VERSION]
Metric: [WHAT GOT SLOWER]
Baseline: [PREVIOUS VALUE]
Current: [NEW VALUE]
Regression: [+X%]
Cause: [WHAT CHANGED]
Fix: [HOW TO FIX]
Priority: [High/Medium/Low]
```

### Active Regressions
*None currently tracked*

### Resolved Regressions
*None yet*

## Profiling Results

### CPU Profile Summary
*Date: [DATE]*
*Tool: flamegraph/perf/Instruments*

Top 5 Hot Spots:
1. Function: ___% time
2. Function: ___% time
3. Function: ___% time
4. Function: ___% time
5. Function: ___% time

### Memory Profile Summary
*Date: [DATE]*
*Tool: heaptrack/valgrind/Instruments*

Top 5 Allocations:
1. Type: ___MB (___%)
2. Type: ___MB (___%)
3. Type: ___MB (___%)
4. Type: ___MB (___%)
5. Type: ___MB (___%)

## Action Items

### Immediate (This Sprint)
- [ ] Investigate: [WHAT]
- [ ] Optimize: [WHAT]
- [ ] Benchmark: [WHAT]

### Short Term (Next Month)
- [ ] Profile: [WHAT]
- [ ] Consider: [OPTIMIZATION]
- [ ] Research: [TECHNIQUE]

### Long Term
- [ ] Evaluate: [MAJOR CHANGE]
- [ ] Plan: [ARCHITECTURE CHANGE]

## Notes

### Performance Testing Environment
- **CPU**: [MODEL]
- **RAM**: [AMOUNT]
- **OS**: [VERSION]
- **Rust**: [VERSION]
- **Build**: `cargo build --release`

### Benchmark Configuration
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

---

*Track performance throughout development. What gets measured gets improved!*