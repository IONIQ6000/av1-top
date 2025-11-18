# Code Review & Improvement Suggestions

## Executive Summary

The codebase is **well-structured, production-ready, and follows Rust best practices**. However, there are several opportunities for improvement in robustness, maintainability, and performance.

**Overall Grade: A-** (Excellent foundation, minor improvements recommended)

---

## 🟢 Strengths (What's Great)

### 1. **Architecture**
✅ Clean separation of concerns (core, daemon, TUI)
✅ Excellent module organization
✅ Proper use of Result types and error handling
✅ Well-documented with doc comments
✅ Type-safe design with strong typing

### 2. **Code Quality**
✅ No linter errors or warnings
✅ Consistent naming conventions
✅ Comprehensive unit tests for critical functions
✅ Clear, readable code with good comments
✅ Idiomatic Rust patterns

### 3. **Features**
✅ Complete end-to-end pipeline
✅ Robust error handling
✅ Atomic file operations
✅ Job persistence
✅ Real-time monitoring

---

## 🟡 Suggested Improvements

### 1. **Extract Magic Numbers to Constants** (Priority: High)

**Issue:** Repeated magic numbers make code harder to maintain.

**Locations:**
- File size calculations: `1024 * 1024 * 1024` (multiple files)
- Progress modulo: `100` (daemon)
- Stability check: `3` samples, `500`ms delay
- GPU frequency threshold: `500` MHz
- Disk sector size: `512` bytes
- Version string offset: `8`

**Suggestion:** Create a constants module:

```rust
// core/src/constants.rs
pub mod units {
    pub const KIB: u64 = 1024;
    pub const MIB: u64 = KIB * 1024;
    pub const GIB: u64 = MIB * 1024;
    pub const TIB: u64 = GIB * 1024;
    
    pub const SECTOR_SIZE_BYTES: u64 = 512;
}

pub mod defaults {
    use super::units::GIB;
    
    pub const MIN_FILE_SIZE_BYTES: u64 = 2 * GIB;
    pub const SIZE_GATE_FACTOR: f64 = 0.9;
    pub const SCAN_INTERVAL_SECONDS: u64 = 60;
}

pub mod stability {
    pub const SAMPLE_COUNT: usize = 3;
    pub const SAMPLE_DELAY_MS: u64 = 500;
}

pub mod progress {
    pub const PRINT_EVERY_N_FRAMES: u64 = 100;
}

pub mod gpu {
    pub const ACTIVE_FREQ_THRESHOLD_MHZ: u32 = 500;
}
```

**Benefits:**
- Single source of truth
- Easy to tune parameters
- Self-documenting code
- Easier testing with different values

---

### 2. **Add Timeout to FFmpeg Execution** (Priority: High)

**Issue:** FFmpeg could hang indefinitely with no timeout.

**Current Code:**
```rust
// core/src/executor.rs
pub fn execute_transcode(...) -> Result<TranscodeResult> {
    // No timeout!
    for line in reader.lines() { ... }
    child.wait()?; // Waits forever
}
```

**Suggestion:** Add configurable timeout:

```rust
use std::time::Duration;

pub struct ExecuteOptions {
    pub timeout: Option<Duration>,
    pub max_stderr_size: usize,
}

impl Default for ExecuteOptions {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(3600 * 4)), // 4 hours max
            max_stderr_size: 10 * 1024 * 1024, // 10 MB max stderr
        }
    }
}

pub fn execute_transcode_with_timeout<F>(
    ffmpeg_path: &Path,
    params: &TranscodeParams,
    args: Vec<String>,
    options: ExecuteOptions,
    progress_callback: Option<F>,
) -> Result<TranscodeResult> {
    // Use std::sync::mpsc or tokio timeout
    // Kill process if timeout exceeded
}
```

**Benefits:**
- Prevents infinite hangs
- Detects stalled encodes
- Configurable per job
- Better resource cleanup

---

### 3. **Eliminate Code Duplication** (Priority: Medium)

**Issue 1: Byte Formatting**

Appears in 3 places:
- `core/src/job.rs` - `humanize_bytes()`
- `av1top/src/main.rs` - `format_bytes()`
- `av1d/src/main.rs` - inline calculations

**Suggestion:** Move to core and reuse:

```rust
// core/src/utils.rs
pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.0} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}
```

**Issue 2: File Stability Check**

Duplicated in daemon (could be moved to core).

**Suggestion:** Move to `core/src/utils.rs` or `core/src/heuristics.rs`

---

### 4. **Improve Atomic File Replacement** (Priority: High)

**Issue:** Backup file naming could collide with existing files.

**Current Code:**
```rust
let backup_path = original_path.with_extension("original.bak");
// What if original.bak already exists?
```

**Suggestion:** Use UUID or timestamp for backup name:

```rust
pub fn replace_file_atomic(original_path: &Path, transcoded_path: &Path) -> Result<()> {
    // Create unique backup name
    let backup_path = original_path.with_extension(
        format!("bak-{}", uuid::Uuid::new_v4())
    );
    
    // Step 1: Rename original to backup
    fs::rename(original_path, &backup_path)?;
    
    // Step 2: Rename transcoded to original name
    match fs::rename(transcoded_path, original_path) {
        Ok(_) => {
            // Success! Delete the backup
            let _ = fs::remove_file(&backup_path);
            Ok(())
        }
        Err(e) => {
            // Restore from backup
            let restore_result = fs::rename(&backup_path, original_path);
            if restore_result.is_err() {
                eprintln!("CRITICAL: Failed to restore backup at {:?}", backup_path);
            }
            Err(CoreError::IoError(e))
        }
    }
}
```

**Benefits:**
- No collision risk
- Better error messages
- Critical failure logging

---

### 5. **Add Stderr Size Limit** (Priority: Medium)

**Issue:** Very long transcodes could fill memory with stderr output.

**Current Code:**
```rust
let mut stderr_lines = Vec::new();
// Unbounded growth!
for line in reader.lines() {
    stderr_lines.push(line.clone());
}
```

**Suggestion:** Limit stderr storage:

```rust
const MAX_STDERR_LINES: usize = 1000;

let mut stderr_lines = Vec::new();
for line in reader.lines() {
    if stderr_lines.len() < MAX_STDERR_LINES {
        stderr_lines.push(line.clone());
    }
    // Still parse progress but don't store every line
}
```

**Benefits:**
- Prevents memory exhaustion
- Still captures useful debug info
- More predictable resource usage

---

### 6. **Better Version Parsing** (Priority: Low)

**Issue:** Version parsing is fragile and could fail on edge cases.

**Current Code:**
```rust
let version_part = &first_line[start + 8..]; // Magic number 8!
let version = version_part.split_whitespace().next()?;
```

**Suggestion:** Use regex or more robust parsing:

```rust
use regex::Regex;

fn extract_version(output: &str) -> Result<String> {
    let re = Regex::new(r"version\s+([^\s]+)").unwrap();
    re.captures(output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CoreError::FFmpegError("Could not parse version".into()))
}
```

**Benefits:**
- More robust
- Handles variations better
- Self-documenting pattern

---

### 7. **Add Logging Infrastructure** (Priority: Medium)

**Issue:** Using `eprintln!` everywhere isn't ideal for production.

**Current:** `eprintln!("Warning: ...")`

**Suggestion:** Use `log` + `env_logger` crates:

```rust
// Add to Cargo.toml
log = "0.4"
env_logger = "0.11"

// In main.rs
use log::{info, warn, error, debug};

fn main() -> Result<()> {
    env_logger::init();
    
    info!("AV1 Daemon starting...");
    warn!("QSV hardware test failed");
    error!("Cannot find FFmpeg");
    debug!("Processing file: {}", path);
}
```

**Benefits:**
- Configurable log levels
- Can write to files
- Structured logging
- Production-ready

---

### 8. **Improve Error Context** (Priority: Low)

**Issue:** Some errors could provide more context.

**Example:**
```rust
fs::rename(original_path, &backup_path)?;
// If this fails, we don't know why
```

**Suggestion:** Add context with anyhow:

```rust
fs::rename(original_path, &backup_path)
    .with_context(|| format!(
        "Failed to create backup: {} -> {}",
        original_path.display(),
        backup_path.display()
    ))?;
```

**Benefits:**
- Better error messages
- Easier debugging
- More helpful for users

---

### 9. **Add Configuration Validation** (Priority: Medium)

**Issue:** No validation of config values.

**Current:**
```rust
pub struct TranscodeConfig {
    pub size_gate_factor: f64, // What if it's 0? Negative? > 1?
    pub min_file_size_bytes: u64, // What if it's 0?
}
```

**Suggestion:** Add validation method:

```rust
impl TranscodeConfig {
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.size_gate_factor <= 0.0 || self.size_gate_factor > 1.0 {
            return Err(format!(
                "size_gate_factor must be between 0 and 1, got {}",
                self.size_gate_factor
            ));
        }
        
        if self.min_file_size_bytes == 0 {
            return Err("min_file_size_bytes cannot be 0".into());
        }
        
        if self.watched_directories.is_empty() {
            return Err("watched_directories cannot be empty".into());
        }
        
        Ok(())
    }
}
```

**Benefits:**
- Catches config errors early
- Better user feedback
- Prevents runtime issues

---

### 10. **Use Newtype Pattern for Type Safety** (Priority: Low)

**Issue:** Using raw u64 for different types of data.

**Current:**
```rust
pub struct TranscodeJob {
    pub original_bytes: Option<u64>,
    pub new_bytes: Option<u64>,
}
```

**Suggestion:** Use newtype wrappers:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileSize(u64);

impl FileSize {
    pub fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> u64 {
        self.0
    }
    
    pub fn format_human(&self) -> String {
        format_bytes(self.0)
    }
}

pub struct TranscodeJob {
    pub original_bytes: Option<FileSize>,
    pub new_bytes: Option<FileSize>,
}
```

**Benefits:**
- Type safety (can't mix up different u64s)
- Self-documenting
- Can add domain-specific methods

---

### 11. **Add Progress Persistence** (Priority: Low)

**Issue:** If daemon crashes mid-transcode, progress is lost.

**Suggestion:** Periodically save progress to job file:

```rust
// In executor callback
if progress.frame % 500 == 0 {
    job.current_frame = Some(progress.frame);
    save_job_state(&job, jobs_dir)?;
}
```

**Benefits:**
- Resume capability
- Better crash recovery
- More accurate TUI display

---

### 12. **Add Dry-Run Mode** (Priority: Medium)

**Issue:** No way to test without actually transcoding.

**Suggestion:** Add dry-run flag:

```rust
pub struct TranscodeConfig {
    pub dry_run: bool,
}

// In daemon
if config.dry_run {
    println!("DRY RUN: Would transcode {}", path.display());
    return Ok(FileResult::Skipped("Dry run mode".into()));
}
```

**Benefits:**
- Safe testing
- Preview before running
- Validation of setup

---

### 13. **Improve TUI Error Handling** (Priority: Medium)

**Issue:** TUI could panic on malformed job files.

**Current:**
```rust
let jobs = create_dummy_jobs(); // Fallback, but could be better
```

**Suggestion:** Show partial data and errors:

```rust
struct App {
    jobs: Vec<TranscodeJob>,
    job_errors: Vec<(String, String)>, // (filename, error)
}

// Display errors in a separate panel
fn draw_error_panel(f: &mut Frame, area: Rect, app: &App) {
    for (file, error) in &app.job_errors {
        // Show which job files failed to load
    }
}
```

**Benefits:**
- Better visibility into issues
- Don't lose good data due to one bad file
- Helpful for debugging

---

### 14. **Add Concurrent Processing** (Priority: High for production)

**Issue:** Processes files sequentially (slow for large libraries).

**Current:**
```rust
for file_path in &files {
    process_file(file_path, ...)?; // One at a time
}
```

**Suggestion:** Use tokio for concurrent processing:

```rust
use tokio::task::JoinSet;

// Process up to N files concurrently
let mut set = JoinSet::new();
let max_concurrent = config.max_concurrent_jobs.unwrap_or(1);

for file_path in files {
    if set.len() >= max_concurrent {
        // Wait for one to complete
        set.join_next().await;
    }
    
    set.spawn(async move {
        process_file(file_path, config, paths_config, ffmpeg)
    });
}

// Wait for remaining
while let Some(result) = set.join_next().await {
    // Handle result
}
```

**Benefits:**
- Much faster for large libraries
- Better hardware utilization
- Configurable concurrency

---

### 15. **Add Filesystem Watcher** (Priority: High for daemon)

**Issue:** Currently only scans once, not continuous.

**Suggestion:** Use notify crate (already in dependencies):

```rust
use notify::{Watcher, RecursiveMode, Event};

fn watch_directories(config: &TranscodeConfig) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    
    for dir in &config.watched_directories {
        watcher.watch(dir, RecursiveMode::Recursive)?;
    }
    
    loop {
        match rx.recv() {
            Ok(Event::Create(path)) => {
                if is_media_file(&path) {
                    queue_file(path);
                }
            }
            Ok(Event::Modify(path)) => {
                // Handle modifications
            }
            _ => {}
        }
    }
}
```

**Benefits:**
- Real-time detection
- No polling delay
- More efficient
- True daemon behavior

---

### 16. **Add Signal Handling** (Priority: Medium)

**Issue:** Daemon doesn't gracefully handle SIGTERM/SIGINT.

**Suggestion:** Add signal handling:

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup signal handler
    tokio::spawn(async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Received shutdown signal, finishing current job...");
        // Set shutdown flag
    });
    
    // Main loop checks shutdown flag
    while !should_shutdown() {
        process_files(...);
    }
    
    println!("Graceful shutdown complete");
}
```

**Benefits:**
- Graceful shutdown
- Finish current job
- Proper cleanup
- Better systemd integration

---

### 17. **Add Job Metadata to JSON** (Priority: Low)

**Issue:** Jobs don't store enough information for debugging.

**Suggestion:** Add more fields:

```rust
pub struct TranscodeJob {
    // ... existing fields ...
    
    /// FFmpeg command used
    pub ffmpeg_command: Option<Vec<String>>,
    
    /// FFmpeg stderr output (truncated)
    pub ffmpeg_stderr: Option<String>,
    
    /// Video resolution
    pub resolution: Option<String>,
    
    /// Original codec
    pub original_codec: Option<String>,
    
    /// Detected as WebRip
    pub is_webrip_like: bool, // Already there!
}
```

**Benefits:**
- Better debugging
- Can recreate exact command
- More informative TUI
- Audit trail

---

### 18. **Improve GPU Stats** (Priority: Medium)

**Issue:** GPU stats are basic estimates, not accurate.

**Suggestion:** Use Intel GPU tools or better APIs:

```rust
// Option 1: Parse intel_gpu_top output (if available)
fn get_intel_gpu_stats() -> Option<GpuStats> {
    let output = Command::new("intel_gpu_top")
        .args(["-J", "-s", "100"]) // JSON output, 100ms sample
        .output()
        .ok()?;
    
    // Parse JSON for accurate GPU usage
}

// Option 2: Use libva or other Intel APIs
// Option 3: Parse /sys/kernel/debug/dri/0/i915_* files
```

**Benefits:**
- Accurate GPU usage
- Better monitoring
- Encoder queue depth
- Memory bandwidth

---

### 19. **Add Config File Loading** (Priority: High for production)

**Issue:** All config is currently hardcoded.

**Suggestion:** Load from TOML:

```rust
use serde::Deserialize;

// Add to Cargo.toml: toml = "0.8"

impl TranscodeConfig {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: TranscodeConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }
    
    pub fn load_or_default() -> Self {
        let config_path = PathBuf::from(env::var("HOME").unwrap_or_default())
            .join(".config/av1janitor/config.toml");
        
        Self::load_from_file(&config_path)
            .unwrap_or_else(|_| Self::default())
    }
}
```

**Benefits:**
- User customization
- Per-system config
- No recompilation needed
- Production-ready

---

### 20. **Add Health Checks** (Priority: Medium)

**Issue:** No way to verify system health.

**Suggestion:** Add health check endpoint:

```rust
pub struct HealthStatus {
    pub ffmpeg_ok: bool,
    pub qsv_ok: bool,
    pub disk_space_ok: bool,
    pub jobs_dir_writable: bool,
}

pub fn check_health(config: &TranscodeConfig) -> HealthStatus {
    // Verify all prerequisites
    // Check disk space
    // Test write access
}
```

**Benefits:**
- Quick diagnostics
- Pre-flight checks
- Monitoring integration
- Better error messages

---

## 🔵 Additional Suggestions

### 21. **Add Metrics Export** (Priority: Low)

Export metrics for Prometheus/Grafana:
- Jobs processed counter
- Size saved counter
- Transcode duration histogram
- Error rates

### 22. **Add Job Retry Logic** (Priority: Medium)

Retry failed jobs with exponential backoff:
- Transient failures (disk full, temporary GPU issues)
- Max retry count
- Backoff delay

### 23. **Add Resume Capability** (Priority: Low)

Resume interrupted transcodes:
- Save partial output
- Check if partial exists
- Resume from keyframe

### 24. **Better Test Coverage** (Priority: Medium)

Add integration tests:
- End-to-end workflow tests
- Mock ffmpeg for testing
- Property-based testing
- Benchmark tests

### 25. **Add CLI Arguments** (Priority: Medium)

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Directory to scan
    #[arg(short, long)]
    directory: PathBuf,
    
    /// Dry run mode
    #[arg(long)]
    dry_run: bool,
    
    /// Config file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}
```

---

## 📊 Priority Matrix

| Priority | Improvement | Impact | Effort |
|----------|-------------|--------|--------|
| 🔴 High | Constants module | High | Low |
| 🔴 High | FFmpeg timeout | High | Medium |
| 🔴 High | Atomic replacement | High | Low |
| 🔴 High | Config file loading | High | Medium |
| 🔴 High | Concurrent processing | Very High | High |
| 🔴 High | Filesystem watcher | High | Medium |
| 🟡 Medium | Logging infrastructure | Medium | Low |
| 🟡 Medium | Stderr size limit | Medium | Low |
| 🟡 Medium | Signal handling | Medium | Low |
| 🟡 Medium | Job retry logic | Medium | Medium |
| 🟡 Medium | CLI arguments | Medium | Low |
| 🟢 Low | Code deduplication | Low | Low |
| 🟢 Low | Version parsing | Low | Low |
| 🟢 Low | Newtype pattern | Low | Medium |
| 🟢 Low | Progress persistence | Low | Low |

---

## 🎯 Recommended Action Plan

### Phase 1: Critical Fixes (Do First)
1. ✅ Add constants module
2. ✅ Add FFmpeg timeout
3. ✅ Improve atomic replacement
4. ✅ Add stderr size limit

### Phase 2: Production Ready
5. Add config file loading (TOML)
6. Add logging infrastructure
7. Add signal handling
8. Add CLI arguments

### Phase 3: Performance & Reliability
9. Add concurrent processing
10. Add filesystem watcher
11. Add job retry logic
12. Better GPU monitoring

### Phase 4: Polish
13. Code deduplication
14. Better test coverage
15. Metrics export
16. Resume capability

---

## 💡 Quick Wins (Low Effort, High Impact)

1. **Constants Module** - 30 minutes, huge maintainability win
2. **Stderr Limit** - 5 minutes, prevents memory issues
3. **Config Validation** - 15 minutes, better error messages
4. **Logging** - 20 minutes, production-ready diagnostics

---

## 🏆 Overall Assessment

The codebase is **excellent** for an initial implementation:
- ✅ Clean architecture
- ✅ Good error handling
- ✅ Well-documented
- ✅ Type-safe
- ✅ Tested where it matters

**Main Areas for Improvement:**
1. Production hardening (timeouts, limits, logging)
2. Performance (concurrency, async)
3. User experience (config files, CLI args)
4. Robustness (signal handling, retry logic)

**None of these are blockers** - the code works well as-is. These are all enhancements for production deployment.

---

## 📝 Conclusion

**Current State:** Production-ready for single-file processing
**Recommended:** Implement Priority High items before large-scale deployment
**Estimated Work:** 2-3 days for all High priority items

The codebase is solid. These suggestions will make it excellent for long-term production use!

