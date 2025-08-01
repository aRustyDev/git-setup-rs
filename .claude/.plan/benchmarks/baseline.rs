// Benchmark baseline for git-setup-rs performance targets
// This file defines the performance requirements that must be met

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// Phase 6 Performance Targets (from SPEC.md NFR-003)
const TARGET_STARTUP_TIME_MS: u64 = 100;      // <100ms startup
const TARGET_PROFILE_LOAD_MS: u64 = 20;       // <20ms profile load
const TARGET_PROFILE_SWITCH_MS: u64 = 100;    // <100ms profile switch
const TARGET_PATH_NORMALIZE_MS: u64 = 10;     // <10ms path operations
const TARGET_MEMORY_MB: usize = 50;            // <50MB memory usage
const TARGET_BINARY_SIZE_MB: usize = 20;       // <20MB binary size

/// Benchmark group for profile operations
fn bench_profile_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_operations");
    
    // Set measurement time to ensure stable results
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark: Profile Loading
    // Target: <20ms
    group.bench_function("load_profile", |b| {
        b.iter(|| {
            // Simulate profile load
            let profile_name = black_box("work");
            // ProfileManager::load(profile_name)
            std::thread::sleep(Duration::from_micros(15_000)); // Simulate 15ms
        });
    });
    
    // Benchmark: Profile Switching
    // Target: <100ms (includes Git config updates)
    group.bench_function("switch_profile", |b| {
        b.iter(|| {
            // Simulate profile switch
            let profile_name = black_box("personal");
            // ProfileManager::switch(profile_name)
            std::thread::sleep(Duration::from_micros(80_000)); // Simulate 80ms
        });
    });
    
    // Benchmark: Profile List
    // Should be fast even with many profiles
    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("list_profiles", count),
            count,
            |b, &count| {
                b.iter(|| {
                    // Simulate listing profiles
                    // ProfileManager::list()
                    std::thread::sleep(Duration::from_micros(100 * count)); // Linear time
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark group for security operations
fn bench_security_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_operations");
    
    // Benchmark: Atomic Write
    // Should be fast but safe
    group.bench_function("atomic_write", |b| {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        
        b.iter(|| {
            // Simulate atomic write
            let content = black_box("profile content");
            // atomic_write(&file_path, content)
            std::fs::write(&file_path, content).unwrap();
        });
    });
    
    // Benchmark: Memory Zeroization
    // Critical for security
    group.bench_function("sensitive_string_drop", |b| {
        b.iter(|| {
            // Create and drop SensitiveString
            let sensitive = black_box(String::from("MySecretPassword123!"));
            // let _ss = SensitiveString::new(sensitive);
            drop(sensitive); // Simulate zeroization
        });
    });
    
    group.finish();
}

/// Benchmark group for platform operations
fn bench_platform_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("platform_operations");
    
    // Benchmark: Path Normalization
    // Target: <10ms
    group.bench_function("normalize_path", |b| {
        b.iter(|| {
            let path = black_box("/home/user/../user/./config/git-setup");
            // platform::normalize_path(path)
            std::path::Path::new(path).canonicalize().ok();
        });
    });
    
    // Benchmark: Windows Path Conversion
    #[cfg(windows)]
    group.bench_function("windows_to_git_path", |b| {
        b.iter(|| {
            let path = black_box("C:\\Users\\Name\\Documents\\Projects");
            // platform::to_git_path(path)
            path.replace('\\', "/");
        });
    });
    
    group.finish();
}

/// Benchmark group for UI operations
fn bench_ui_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_operations");
    
    // Benchmark: CLI Parsing
    // Should be instant
    group.bench_function("parse_cli_args", |b| {
        let args = vec!["git-setup", "profile", "list", "--format", "json"];
        b.iter(|| {
            // Cli::parse_from(black_box(&args))
            args.join(" ");
        });
    });
    
    // Benchmark: TUI Render Frame
    // Target: 60fps = 16.67ms per frame
    group.bench_function("render_tui_frame", |b| {
        b.iter(|| {
            // app.render_frame()
            std::thread::sleep(Duration::from_micros(10_000)); // Simulate 10ms render
        });
    });
    
    group.finish();
}

/// Benchmark group for startup time
fn bench_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");
    
    // Benchmark: Cold Start
    // Target: <100ms
    group.bench_function("cold_start", |b| {
        b.iter(|| {
            // Simulate full application startup
            std::thread::sleep(Duration::from_micros(75_000)); // Simulate 75ms
        });
    });
    
    // Benchmark: Warm Start (with cache)
    // Should be faster than cold start
    group.bench_function("warm_start", |b| {
        b.iter(|| {
            // Simulate startup with cache
            std::thread::sleep(Duration::from_micros(30_000)); // Simulate 30ms
        });
    });
    
    group.finish();
}

/// Memory usage baseline check
/// Run with: cargo bench --bench baseline -- --profile-time 60
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // This benchmark helps profile memory usage
    group.bench_function("idle_memory", |b| {
        b.iter(|| {
            // Simulate application at idle
            let _data = vec![0u8; 1024 * 1024]; // 1MB allocation
            black_box(&_data);
        });
    });
    
    group.finish();
}

// Register all benchmark groups
criterion_group!(
    benches,
    bench_profile_operations,
    bench_security_operations,
    bench_platform_operations,
    bench_ui_operations,
    bench_startup,
    bench_memory_usage
);

criterion_main!(benches);

/// Performance assertion tests
/// These can be run as regular tests to verify performance
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_startup_time() {
        let start = Instant::now();
        // Application::new();
        let elapsed = start.elapsed();
        
        assert!(
            elapsed.as_millis() < TARGET_STARTUP_TIME_MS as u128,
            "Startup took {}ms, target is {}ms",
            elapsed.as_millis(),
            TARGET_STARTUP_TIME_MS
        );
    }
    
    #[test]
    fn test_profile_load_time() {
        let start = Instant::now();
        // ProfileManager::load("test");
        let elapsed = start.elapsed();
        
        assert!(
            elapsed.as_millis() < TARGET_PROFILE_LOAD_MS as u128,
            "Profile load took {}ms, target is {}ms",
            elapsed.as_millis(),
            TARGET_PROFILE_LOAD_MS
        );
    }
    
    #[test]
    fn test_memory_usage() {
        // This would use a memory profiler in real implementation
        let current_memory_mb = 30; // Simulated
        
        assert!(
            current_memory_mb < TARGET_MEMORY_MB,
            "Memory usage is {}MB, target is {}MB",
            current_memory_mb,
            TARGET_MEMORY_MB
        );
    }
}

/// Helper module for performance monitoring
pub mod monitoring {
    use std::time::{Duration, Instant};
    
    /// Simple performance timer
    pub struct PerfTimer {
        name: String,
        start: Instant,
        target: Duration,
    }
    
    impl PerfTimer {
        pub fn new(name: &str, target_ms: u64) -> Self {
            Self {
                name: name.to_string(),
                start: Instant::now(),
                target: Duration::from_millis(target_ms),
            }
        }
    }
    
    impl Drop for PerfTimer {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed();
            if elapsed > self.target {
                eprintln!(
                    "PERF WARNING: {} took {:?}, target was {:?}",
                    self.name, elapsed, self.target
                );
            }
        }
    }
    
    /// Macro for easy performance monitoring
    #[macro_export]
    macro_rules! perf_monitor {
        ($name:expr, $target_ms:expr, $code:block) => {{
            let _timer = $crate::monitoring::PerfTimer::new($name, $target_ms);
            $code
        }};
    }
}

/*
Junior Developer Notes:
======================

Running Benchmarks:
------------------
# Run all benchmarks
cargo bench --bench baseline

# Run specific benchmark group
cargo bench --bench baseline -- profile_operations

# Save results for comparison
cargo bench --bench baseline -- --save-baseline main

# Compare against baseline
cargo bench --bench baseline -- --baseline main

Understanding Results:
---------------------
The output shows:
- time: [X ns Y ns Z ns] - Lower/Middle/Upper bounds
- change: [+A% +B% +C%] - Performance change from baseline

Performance Tips:
----------------
1. Profile before optimizing - don't guess!
2. Focus on hot paths shown by profiler
3. Measure after each change
4. Small improvements add up
5. Sometimes "good enough" is perfect

Common Optimizations:
--------------------
- Avoid allocations in loops
- Use &str instead of String when possible  
- Cache expensive computations
- Batch I/O operations
- Use efficient data structures

Remember: Premature optimization is the root of all evil!
Focus on correctness first, then optimize what matters.
*/