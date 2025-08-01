# Performance Benchmarks

This directory contains performance benchmarks for git-setup-rs organized by phase.

## Structure

```
benchmarks/
├── README.md                 # This file
├── phase_1_security.rs      # File operations, memory safety
├── phase_2_profiles.rs      # Profile loading, validation
├── phase_3_ui.rs            # TUI rendering, event handling
├── phase_4_1password.rs     # Integration performance
├── phase_5_detection.rs     # Pattern matching, auto-detection
├── phase_6_health.rs        # Health checks, diagnostics
├── phase_7_signing.rs       # Signing operations
├── phase_8_advanced.rs      # Advanced features
└── integration.rs           # Cross-phase benchmarks
```

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific phase benchmarks
cargo bench --bench phase_1_security

# Generate performance report
cargo bench -- --save-baseline main
cargo bench -- --baseline main

# Profile with flamegraph
cargo flamegraph --bench phase_2_profiles
```

## Key Metrics

Each benchmark measures:
- **Latency**: Time to complete operation
- **Throughput**: Operations per second
- **Memory**: Allocations and peak usage
- **Concurrency**: Scaling with parallel operations

## Performance Targets

See [PERFORMANCE_PROFILING_GUIDE.md](../PERFORMANCE_PROFILING_GUIDE.md) for detailed targets.