# Implementation Status

## Summary

This document tracks the implementation status of the AV1 Daemon + TUI Monitor project scaffold.

**Date:** Initial scaffold completed
**Status:** ✅ All planned features for Phase 1 complete

## Completed Tasks

### 1. Workspace Structure ✅
- Created root `Cargo.toml` with workspace configuration
- Set up three crates: `core`, `av1d`, `av1top`
- Configured dependencies with workspace-level version management
- Added `.gitignore` for Rust projects

### 2. Core Library (`core/`) ✅

#### Configuration (`config.rs`)
- ✅ `TranscodeConfig` with ffmpeg path, watched directories, thresholds, QSV settings
- ✅ `PathsConfig` for logs and job directories
- ✅ Default implementations with sensible values
- ✅ Serde serialization support (ready for TOML loading)

#### Error Handling (`error.rs`)
- ✅ `CoreError` enum with thiserror
- ✅ Covers all error cases: FFmpeg, IO, JSON, metadata, config
- ✅ Proper error context and messages

#### Job Model (`job.rs`)
- ✅ `JobStatus` enum: Pending, Running, Success, Failed, Skipped
- ✅ `JobReason` wrapper for textual explanations
- ✅ `TranscodeJob` struct with all required fields
- ✅ Helper methods: `duration()`, `size_savings_ratio()`, `size_savings_bytes()`
- ✅ Human-readable formatting functions
- ✅ UUID-based job IDs

#### Metadata Types (`metadata.rs`)
- ✅ `VideoStreamInfo` with codec, dimensions, bit depth, frame rates
- ✅ `FileMetadata` with streams, format, tags, size
- ✅ Helper methods: `is_vfr()`, `has_odd_dimensions()`, `resolution_string()`
- ✅ Default stream selection logic

#### Heuristics (`heuristics.rs`)
- ✅ `is_webrip_like()` - checks format, VFR, odd dimensions
- ✅ `should_skip_for_size()` - size threshold check
- ✅ `is_already_av1()` - codec detection
- ✅ `choose_quality()` - resolution-based quality (23/24/25)
- ✅ `choose_surface()` - bit-depth based format (p010/nv12)
- ✅ Comprehensive unit tests

#### FFprobe Integration (`ffprobe.rs`)
- ✅ `run_ffprobe()` function signature and stub
- ✅ FFprobe output structures defined
- ✅ TODO markers for full implementation
- ✅ Clear documentation of planned implementation

#### Persistence (`persistence.rs`)
- ✅ `save_job_state()` - serialize job to JSON
- ✅ `load_all_jobs()` - deserialize all jobs from directory
- ✅ Creates directories as needed
- ✅ Handles errors gracefully (warns on corrupt files)
- ✅ Unit tests with tempfile

### 3. Daemon (`av1d/`) ✅

- ✅ Main entry point with command-line interface
- ✅ Configuration loading (currently defaults, file loading TODO)
- ✅ Directory scanning (recursive)
- ✅ File filtering by extension
- ✅ `.av1skip` marker detection
- ✅ Size threshold checking
- ✅ Reports what would be done (queue vs skip)
- ✅ Environment variable support (`AV1_TEST_DIR`)
- ✅ Clear output with reasoning for each decision
- ✅ TODO markers for ffprobe integration

### 4. TUI Monitor (`av1top/`) ✅

#### UI Layout
- ✅ Top section: CPU and memory gauges
- ✅ Middle left: Disk usage panel
- ✅ Middle right: Job table with 8 columns
- ✅ Bottom: Status bar with hotkeys
- ✅ Color-coded status indicators

#### Functionality
- ✅ Real-time system metrics via sysinfo
- ✅ 1-second refresh rate
- ✅ Keyboard handling: `q` to quit, `r` to refresh
- ✅ Dummy job data for display testing
- ✅ Human-readable sizes and durations
- ✅ Proper terminal setup and cleanup
- ✅ Uppercase column headers (per user preference)

#### Job Table Columns
- ✅ STATUS - color-coded
- ✅ FILE - truncated if too long
- ✅ RES - resolution label
- ✅ ORIG_SIZE - formatted bytes
- ✅ NEW_SIZE - formatted bytes
- ✅ SAVINGS - percentage and absolute
- ✅ DURATION - human-readable time
- ✅ REASON - truncated if too long

### 5. Documentation ✅

- ✅ Comprehensive README.md
- ✅ Project structure documentation
- ✅ Build and run instructions
- ✅ Architecture notes
- ✅ Configuration documentation
- ✅ FFmpeg command structure reference
- ✅ This status document

### 6. Code Quality ✅

- ✅ All code compiles without errors
- ✅ No linter warnings
- ✅ Extensive comments (per user preference [[memory:8757823]])
- ✅ Unit tests for core heuristics and persistence
- ✅ Clear, boring, explicit code (as requested)
- ✅ Proper error handling throughout
- ✅ Idiomatic Rust patterns

## Next Steps (Future Iterations)

The following features are designed and documented but not yet implemented:

1. **FFprobe Implementation**
   - Execute ffprobe and parse JSON output
   - Extract video stream metadata
   - Handle errors properly

2. **FFmpeg Transcoding**
   - Implement full command-line construction
   - Process execution and monitoring
   - Progress tracking
   - Log capture

3. **Size Gate & File Replacement**
   - Compare output vs input size
   - Write .av1skip and .why.txt on rejection
   - Atomic file replacement on success

4. **File Stability Check**
   - Detect files still being copied
   - Multiple size samples with delays

5. **Configuration File Loading**
   - Read from ~/.config/av1janitor/config.toml
   - Override defaults with user settings

6. **Real Job Loading in TUI**
   - Replace dummy data with core::load_all_jobs()
   - Auto-refresh on job changes

7. **Daemon Loop**
   - Continuous scanning (not just once)
   - Concurrent job processing
   - FFmpeg validation at startup

8. **Logging**
   - Replace eprintln! with proper logging
   - Log levels and filtering
   - Rotate log files

## Testing

To test the current implementation:

```bash
# Test core library
cd core && cargo test

# Test daemon scanning (point to a media directory)
AV1_TEST_DIR=/path/to/media cargo run --bin av1d

# Test TUI
cargo run --bin av1top
# (Press 'q' to quit)
```

## Notes

- All TODO markers are clearly labeled in the code
- The architecture is designed to support incremental development
- Each module has clear separation of concerns
- The project follows the spec exactly as requested
- Code is well-commented for future development

