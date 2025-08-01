# Performance Profiling Guide for git-setup-rs

## Overview

This guide provides comprehensive performance profiling examples for critical operations in git-setup-rs. Each example includes profiling setup, measurement techniques, and optimization strategies.

## Key Performance Areas Identified

Based on analysis of the phase plans, the following areas require performance profiling:

1. **File Operations** (Phase 1)
   - Atomic file writes
   - Configuration loading/saving
   - Permission setting

2. **Profile Management** (Phase 2)
   - Profile loading and caching
   - Profile switching
   - Profile validation

3. **UI Rendering** (Phase 3)
   - TUI render cycles
   - Event handling latency
   - Search operations

4. **1Password Integration** (Phase 4)
   - Subprocess spawning
   - JSON parsing
   - Credential lookup

5. **Pattern Detection** (Phase 5)
   - Path normalization
   - Rule matching
   - Auto-detection algorithms

6. **Health Checks** (Phase 6)
   - Concurrent check execution
   - Network operations
   - Resource monitoring

7. **Signing Operations** (Phase 7-8)
   - Key loading
   - Signature verification
   - Certificate validation

## Performance Profiling Examples

### 1. Atomic File Operations Profiling

**What to measure**: Write latency, syscall overhead, lock contention

```rust
// benches/file_operations.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use git_setup_rs::fs::{AtomicWrite, AtomicFileWriter};
use std::path::PathBuf;
use tempfile::TempDir;

fn benchmark_atomic_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_writes");
    let temp_dir = TempDir::new().unwrap();
    
    // Test different file sizes
    for size in [1024, 10240, 102400, 1024000].iter() {
        let content = vec![0u8; *size];
        
        group.bench_with_input(
            BenchmarkId::new("write_size", size),
            size,
            |b, _| {
                let path = temp_dir.path().join("test.toml");
                let writer = AtomicFileWriter::new();
                b.iter(|| {
                    writer.write_atomic(
                        black_box(&path),
                        black_box(&content)
                    ).unwrap();
                });
            }
        );
    }
    
    // Test concurrent writes
    group.bench_function("concurrent_writes", |b| {
        use rayon::prelude::*;
        let paths: Vec<_> = (0..10)
            .map(|i| temp_dir.path().join(format!("file_{}.toml", i)))
            .collect();
        
        b.iter(|| {
            paths.par_iter().for_each(|path| {
                let writer = AtomicFileWriter::new();
                writer.write_atomic(path, b"test content").unwrap();
            });
        });
    });
    
    group.finish();
}

// Profile syscalls with strace integration
#[cfg(target_os = "linux")]
fn profile_syscalls() {
    use std::process::Command;
    
    // Run with strace to see syscall patterns
    let output = Command::new("strace")
        .args(&["-c", "-e", "trace=file", "--", 
               "target/release/git-setup", "profile", "save"])
        .output()
        .expect("Failed to run strace");
    
    println!("Syscall profile:\n{}", String::from_utf8_lossy(&output.stderr));
}

criterion_group!(benches, benchmark_atomic_writes);
criterion_main!(benches);
```

**Optimization Guide**:
```rust
// Before: Multiple syscalls
pub fn write_atomic_slow(path: &Path, content: &[u8]) -> io::Result<()> {
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)?;  // write syscall
    fs::set_permissions(&temp_path, Permissions::from_mode(0o600))?;  // chmod syscall
    fs::rename(&temp_path, path)?;  // rename syscall
    Ok(())
}

// After: Reduced syscalls with pre-set permissions
pub fn write_atomic_fast(path: &Path, content: &[u8]) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    
    let temp_path = path.with_extension("tmp");
    
    // Open with permissions already set
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)  // Set permissions during creation
        .open(&temp_path)?;
    
    file.write_all(content)?;
    file.sync_all()?;  // Ensure data is on disk
    drop(file);  // Close before rename
    
    fs::rename(&temp_path, path)?;
    Ok(())
}
```

### 2. Profile Loading Performance

**What to measure**: Parse time, I/O latency, cache effectiveness

```rust
// benches/profile_loading.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_setup_rs::profiles::{Profile, ProfileManager};
use std::sync::Arc;
use parking_lot::RwLock;

fn benchmark_profile_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_operations");
    
    // Setup test profiles
    let manager = ProfileManager::new().unwrap();
    
    // Benchmark cold load
    group.bench_function("cold_load", |b| {
        b.iter(|| {
            // Clear any caches
            drop(ProfileManager::new().unwrap());
            
            let manager = ProfileManager::new().unwrap();
            let profile = manager.load_profile(black_box("work")).unwrap();
            black_box(profile);
        });
    });
    
    // Benchmark warm load (cached)
    group.bench_function("warm_load", |b| {
        let manager = ProfileManager::new().unwrap();
        // Prime the cache
        manager.load_profile("work").unwrap();
        
        b.iter(|| {
            let profile = manager.load_profile(black_box("work")).unwrap();
            black_box(profile);
        });
    });
    
    // Benchmark profile switching
    group.bench_function("profile_switch", |b| {
        let manager = ProfileManager::new().unwrap();
        
        b.iter(|| {
            manager.switch_profile(black_box("work")).unwrap();
            manager.switch_profile(black_box("personal")).unwrap();
        });
    });
    
    // Benchmark concurrent access
    group.bench_function("concurrent_access", |b| {
        use rayon::prelude::*;
        let manager = Arc::new(ProfileManager::new().unwrap());
        
        b.iter(|| {
            (0..10).into_par_iter().for_each(|i| {
                let profile_name = if i % 2 == 0 { "work" } else { "personal" };
                let _ = manager.load_profile(profile_name);
            });
        });
    });
    
    group.finish();
}

// Memory profiling for cache efficiency
fn profile_memory_usage() {
    use jemalloc_ctl::{stats, epoch};
    
    // Force a garbage collection
    epoch::advance().unwrap();
    
    let allocated_before = stats::allocated::read().unwrap();
    
    // Load many profiles
    let manager = ProfileManager::new().unwrap();
    for i in 0..1000 {
        let _ = manager.load_profile(&format!("profile_{}", i));
    }
    
    epoch::advance().unwrap();
    let allocated_after = stats::allocated::read().unwrap();
    
    println!("Memory used by 1000 profiles: {} bytes", 
             allocated_after - allocated_before);
}

criterion_group!(benches, benchmark_profile_operations);
criterion_main!(benches);
```

**Optimization Guide**:
```rust
// Profile cache with LRU eviction
use lru::LruCache;
use parking_lot::RwLock;

pub struct ProfileCache {
    cache: RwLock<LruCache<String, Arc<Profile>>>,
}

impl ProfileCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(LruCache::new(capacity)),
        }
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<Profile>> {
        self.cache.write().get(name).cloned()
    }
    
    pub fn insert(&self, name: String, profile: Arc<Profile>) {
        self.cache.write().put(name, profile);
    }
}

// Lazy parsing optimization
pub struct LazyProfile {
    path: PathBuf,
    parsed: OnceCell<Profile>,
}

impl LazyProfile {
    pub fn get(&self) -> Result<&Profile> {
        self.parsed.get_or_try_init(|| {
            Profile::from_file(&self.path)
        })
    }
}
```

### 3. TUI Rendering Performance

**What to measure**: Frame time, event latency, render efficiency

```rust
// benches/tui_rendering.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_setup_rs::ui::{App, render_frame};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn benchmark_tui_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("tui_rendering");
    
    // Setup test terminal
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    // Benchmark frame rendering
    group.bench_function("render_frame", |b| {
        let mut app = App::new();
        
        b.iter(|| {
            terminal.draw(|f| {
                render_frame(black_box(&mut app), black_box(f));
            }).unwrap();
        });
    });
    
    // Benchmark event handling
    group.bench_function("handle_keypress", |b| {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = App::new();
        
        let events = vec![
            KeyEvent::new(KeyCode::Down, KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Up, KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        ];
        
        b.iter(|| {
            for event in &events {
                app.handle_key(black_box(*event));
            }
        });
    });
    
    // Benchmark search performance
    group.bench_function("fuzzy_search", |b| {
        let mut app = App::new();
        // Add 1000 test profiles
        for i in 0..1000 {
            app.add_profile(format!("profile_{}", i));
        }
        
        b.iter(|| {
            app.search(black_box("prof"));
            black_box(&app.search_results);
        });
    });
    
    group.finish();
}

// Profile render bottlenecks
fn profile_render_pipeline() {
    use std::time::Instant;
    use tracing::{info_span, instrument};
    
    #[instrument]
    fn render_with_spans(app: &mut App, terminal: &mut Terminal<TestBackend>) {
        let _span = info_span!("render_pipeline").entered();
        
        let start = Instant::now();
        {
            let _span = info_span!("prepare_widgets").entered();
            app.prepare_widgets();
        }
        let prepare_time = start.elapsed();
        
        let start = Instant::now();
        {
            let _span = info_span!("terminal_draw").entered();
            terminal.draw(|f| {
                render_frame(app, f);
            }).unwrap();
        }
        let draw_time = start.elapsed();
        
        println!("Prepare: {:?}, Draw: {:?}", prepare_time, draw_time);
    }
}

criterion_group!(benches, benchmark_tui_rendering);
criterion_main!(benches);
```

**Optimization Guide**:
```rust
// Widget caching for static content
use ratatui::widgets::{Block, Borders, Widget};

pub struct CachedWidget<W> {
    widget: W,
    cache_key: u64,
    rendered: OnceCell<Buffer>,
}

impl<W: Widget> CachedWidget<W> {
    pub fn render(&self, area: Rect) -> &Buffer {
        self.rendered.get_or_init(|| {
            let mut buffer = Buffer::empty(area);
            self.widget.render(area, &mut buffer);
            buffer
        })
    }
}

// Incremental search with debouncing
use tokio::time::{Duration, Instant, sleep};

pub struct SearchDebouncer {
    last_query: RwLock<String>,
    last_search: RwLock<Instant>,
    delay: Duration,
}

impl SearchDebouncer {
    pub async fn search(&self, query: String) -> Vec<String> {
        {
            let mut last_query = self.last_query.write();
            let mut last_search = self.last_search.write();
            
            *last_query = query.clone();
            *last_search = Instant::now();
        }
        
        // Debounce
        sleep(self.delay).await;
        
        // Check if query changed
        let current_query = self.last_query.read().clone();
        if current_query != query {
            return vec![];  // Query changed, skip
        }
        
        // Perform actual search
        self.perform_search(&query)
    }
}
```

### 4. 1Password Integration Performance

**What to measure**: Process spawn time, JSON parsing, IPC latency

```rust
// benches/onepassword_integration.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_setup_rs::integrations::OpCli;
use serde_json::Value;

fn benchmark_op_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("1password_ops");
    
    // Benchmark subprocess spawning
    group.bench_function("spawn_op_process", |b| {
        b.iter(|| {
            let output = std::process::Command::new("op")
                .args(&["--version"])
                .output()
                .unwrap();
            black_box(output);
        });
    });
    
    // Benchmark JSON parsing
    group.bench_function("parse_op_json", |b| {
        let json_str = r#"
        {
            "id": "abc123",
            "title": "GitHub SSH Key",
            "category": "SSH_KEY",
            "fields": [
                {"label": "private_key", "value": "-----BEGIN RSA..."}
            ]
        }"#;
        
        b.iter(|| {
            let parsed: Value = serde_json::from_str(black_box(json_str)).unwrap();
            black_box(parsed);
        });
    });
    
    // Benchmark full credential lookup
    group.bench_function("lookup_credential", |b| {
        let op_cli = OpCli::new();
        
        b.iter(|| {
            // Mock the actual op call for benchmarking
            let result = op_cli.get_item_mock(black_box("GitHub SSH Key"));
            black_box(result);
        });
    });
    
    group.finish();
}

// Process pool for op CLI
pub struct OpProcessPool {
    pool: Vec<Child>,
    available: Arc<Mutex<Vec<usize>>>,
}

impl OpProcessPool {
    pub fn new(size: usize) -> Self {
        let mut pool = Vec::with_capacity(size);
        let mut available = Vec::with_capacity(size);
        
        for i in 0..size {
            // Start persistent op process
            let child = Command::new("op")
                .args(&["signin", "--raw"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            
            pool.push(child);
            available.push(i);
        }
        
        Self {
            pool,
            available: Arc::new(Mutex::new(available)),
        }
    }
    
    pub async fn execute(&self, cmd: &str) -> Result<String> {
        // Get available process
        let idx = {
            let mut available = self.available.lock().unwrap();
            available.pop().ok_or("No available processes")?
        };
        
        // Use process...
        
        // Return to pool
        self.available.lock().unwrap().push(idx);
        
        Ok(output)
    }
}

criterion_group!(benches, benchmark_op_operations);
criterion_main!(benches);
```

### 5. Pattern Detection Performance

**What to measure**: Path normalization, rule matching speed, cache hit rates

```rust
// benches/pattern_detection.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_setup_rs::detection::{PatternMatcher, ProfileDetector};
use std::path::PathBuf;

fn benchmark_pattern_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_detection");
    
    // Setup test patterns
    let detector = ProfileDetector::new()
        .add_rule("*/work/*", "work")
        .add_rule("*/personal/*", "personal")
        .add_rule("*/client-*/", "client");
    
    // Benchmark path normalization
    group.bench_function("normalize_path", |b| {
        let paths = vec![
            PathBuf::from("/home/user/work/../work/project"),
            PathBuf::from("C:\\Users\\Name\\Documents\\..\\Work"),
            PathBuf::from("~/projects/./client-acme/"),
        ];
        
        b.iter(|| {
            for path in &paths {
                let normalized = detector.normalize_path(black_box(path));
                black_box(normalized);
            }
        });
    });
    
    // Benchmark pattern matching
    group.bench_function("match_patterns", |b| {
        let test_paths = vec![
            "/home/user/work/project",
            "/home/user/personal/blog",
            "/home/user/client-acme/app",
        ];
        
        b.iter(|| {
            for path in &test_paths {
                let profile = detector.detect_profile(black_box(path));
                black_box(profile);
            }
        });
    });
    
    // Benchmark with many rules
    group.bench_function("many_rules", |b| {
        let mut detector = ProfileDetector::new();
        // Add 1000 rules
        for i in 0..1000 {
            detector.add_rule(&format!("*/project_{}/", i), &format!("profile_{}", i));
        }
        
        b.iter(|| {
            let profile = detector.detect_profile(black_box("/home/user/project_500/src"));
            black_box(profile);
        });
    });
    
    group.finish();
}

// Optimized pattern matching with trie
use trie_rs::{Trie, TrieBuilder};

pub struct OptimizedPatternMatcher {
    trie: Trie<u8>,
    patterns: Vec<(String, String)>,
}

impl OptimizedPatternMatcher {
    pub fn new() -> Self {
        Self {
            trie: TrieBuilder::new().build(),
            patterns: Vec::new(),
        }
    }
    
    pub fn add_pattern(&mut self, pattern: &str, profile: &str) {
        // Convert pattern to searchable form
        let key = pattern.replace("*", "");
        self.trie.insert(key.as_bytes());
        self.patterns.push((pattern.to_string(), profile.to_string()));
    }
    
    pub fn match_path(&self, path: &str) -> Option<&str> {
        // Fast prefix matching
        let matches = self.trie.common_prefix_search(path.as_bytes());
        
        if !matches.is_empty() {
            // Return first match
            Some(&self.patterns[0].1)
        } else {
            None
        }
    }
}

criterion_group!(benches, benchmark_pattern_detection);
criterion_main!(benches);
```

### 6. Concurrent Health Checks

**What to measure**: Parallel execution time, resource contention, timeout handling

```rust
// benches/health_checks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_setup_rs::health::{HealthCheck, HealthCheckRunner};
use tokio::runtime::Runtime;

fn benchmark_health_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("health_checks");
    let runtime = Runtime::new().unwrap();
    
    // Benchmark sequential vs parallel execution
    group.bench_function("sequential_checks", |b| {
        let runner = HealthCheckRunner::new();
        
        b.iter(|| {
            runtime.block_on(async {
                let results = runner.run_sequential().await;
                black_box(results);
            });
        });
    });
    
    group.bench_function("parallel_checks", |b| {
        let runner = HealthCheckRunner::new();
        
        b.iter(|| {
            runtime.block_on(async {
                let results = runner.run_parallel().await;
                black_box(results);
            });
        });
    });
    
    // Benchmark with timeout handling
    group.bench_function("checks_with_timeout", |b| {
        use tokio::time::{timeout, Duration};
        let runner = HealthCheckRunner::new();
        
        b.iter(|| {
            runtime.block_on(async {
                let result = timeout(
                    Duration::from_secs(5),
                    runner.run_all()
                ).await;
                black_box(result);
            });
        });
    });
    
    group.finish();
}

// Optimized concurrent execution
use futures::stream::{FuturesUnordered, StreamExt};

pub struct OptimizedHealthRunner {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl OptimizedHealthRunner {
    pub async fn run_optimized(&self) -> Vec<HealthResult> {
        // Use FuturesUnordered for better performance
        let mut futures = FuturesUnordered::new();
        
        for check in &self.checks {
            futures.push(async move {
                check.run().await
            });
        }
        
        // Collect results as they complete
        let mut results = Vec::with_capacity(self.checks.len());
        while let Some(result) = futures.next().await {
            results.push(result);
        }
        
        results
    }
    
    // With semaphore to limit concurrency
    pub async fn run_limited(&self, max_concurrent: usize) -> Vec<HealthResult> {
        use tokio::sync::Semaphore;
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        
        let futures: Vec<_> = self.checks.iter().map(|check| {
            let permit = semaphore.clone();
            async move {
                let _permit = permit.acquire().await.unwrap();
                check.run().await
            }
        }).collect();
        
        futures::future::join_all(futures).await
    }
}

criterion_group!(benches, benchmark_health_checks);
criterion_main!(benches);
```

### 7. Signing Operations Performance

**What to measure**: Key loading time, signature generation/verification

```rust
// benches/signing_operations.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_setup_rs::signing::{SshSigner, GpgSigner, SigningMethod};

fn benchmark_signing_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signing_ops");
    
    // Benchmark SSH key loading
    group.bench_function("load_ssh_key", |b| {
        use ssh_key::PrivateKey;
        
        b.iter(|| {
            let key = PrivateKey::from_openssh(black_box(SSH_KEY_DATA)).unwrap();
            black_box(key);
        });
    });
    
    // Benchmark SSH signing
    group.bench_function("ssh_sign_commit", |b| {
        let signer = SshSigner::new("~/.ssh/id_ed25519").unwrap();
        let commit_data = b"tree abc123\nparent def456\nauthor Test\n\nTest commit";
        
        b.iter(|| {
            let signature = signer.sign(black_box(commit_data)).unwrap();
            black_box(signature);
        });
    });
    
    // Benchmark GPG operations
    group.bench_function("gpg_sign_commit", |b| {
        let signer = GpgSigner::new("test@example.com").unwrap();
        let commit_data = b"tree abc123\nparent def456\nauthor Test\n\nTest commit";
        
        b.iter(|| {
            let signature = signer.sign(black_box(commit_data)).unwrap();
            black_box(signature);
        });
    });
    
    // Benchmark signature verification
    group.bench_function("verify_signature", |b| {
        let verifier = SignatureVerifier::new();
        let signed_data = include_bytes!("../test_data/signed_commit.txt");
        
        b.iter(|| {
            let valid = verifier.verify(black_box(signed_data)).unwrap();
            black_box(valid);
        });
    });
    
    group.finish();
}

// Cached key loading
use dashmap::DashMap;

pub struct CachedKeySigner {
    cache: DashMap<String, Arc<PrivateKey>>,
}

impl CachedKeySigner {
    pub fn load_key(&self, path: &str) -> Result<Arc<PrivateKey>> {
        if let Some(key) = self.cache.get(path) {
            return Ok(key.clone());
        }
        
        let key = Arc::new(PrivateKey::from_file(path)?);
        self.cache.insert(path.to_string(), key.clone());
        Ok(key)
    }
}

criterion_group!(benches, benchmark_signing_operations);
criterion_main!(benches);
```

## Running Performance Tests

### Setup

```bash
# Install profiling tools
cargo install flamegraph
cargo install cargo-criterion

# Linux specific
sudo apt-get install linux-tools-common linux-tools-generic
sudo apt-get install valgrind heaptrack

# macOS specific
brew install instruments
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench profile_loading

# Generate flamegraph
cargo flamegraph --bench profile_loading

# Profile with perf (Linux)
perf record --call-graph=dwarf target/release/git-setup profile switch work
perf report

# Profile with Instruments (macOS)
instruments -t "Time Profiler" target/release/git-setup
```

### Continuous Performance Monitoring

```rust
// tests/performance_regression.rs
use criterion::{Criterion, criterion_group, criterion_main};

fn performance_regression_tests(c: &mut Criterion) {
    // Set baseline thresholds
    let mut group = c.benchmark_group("regression");
    group.significance_level(0.1)
          .sample_size(100)
          .measurement_time(std::time::Duration::from_secs(10));
    
    // Critical path: profile switching
    group.bench_function("critical_profile_switch", |b| {
        let manager = ProfileManager::new().unwrap();
        b.iter(|| {
            manager.switch_profile("work").unwrap();
        });
    });
    
    // Assert performance requirements
    assert!(group.throughput() > 1000.0, "Profile switch too slow!");
    
    group.finish();
}

criterion_group!(regression, performance_regression_tests);
criterion_main!(regression);
```

## Performance Optimization Checklist

### Before Optimizing
- [ ] Profile first - identify actual bottlenecks
- [ ] Establish baseline measurements
- [ ] Set performance targets
- [ ] Consider algorithmic improvements first

### Common Optimizations
- [ ] Cache frequently accessed data
- [ ] Use efficient data structures (HashMap vs Vec)
- [ ] Minimize allocations in hot paths
- [ ] Batch I/O operations
- [ ] Use async/parallel execution where appropriate
- [ ] Implement lazy loading
- [ ] Pre-compile regular expressions
- [ ] Use zero-copy operations

### After Optimizing
- [ ] Verify improvements with benchmarks
- [ ] Check for regressions in other areas
- [ ] Document optimization rationale
- [ ] Add performance tests to CI

## Performance Targets

Based on the phase plans, these are the performance targets:

| Operation | Target | Maximum | Notes |
|-----------|--------|---------|-------|
| Profile Load | <20ms | <100ms | Cold load |
| Profile Switch | <50ms | <200ms | Including Git config |
| File Write | <10ms | <50ms | Atomic operation |
| Pattern Match | <1ms | <10ms | Per path |
| Health Check | <100ms | <500ms | Per check |
| TUI Render | <16ms | <33ms | 60fps target |
| 1Password Lookup | <200ms | <1s | Including subprocess |

## Conclusion

This guide provides comprehensive performance profiling examples for all critical paths in git-setup-rs. Use these examples as templates for measuring and optimizing performance throughout the development process. Remember: always measure before optimizing, and focus on the actual bottlenecks identified through profiling.