# Changelog

All notable changes to the AV1 Daemon + TUI Monitor project.

## [Phase 2] - Continuation Session - Full Media Analysis

### Added

#### Core Library
- **FFprobe Integration** (`core/src/ffprobe.rs`)
  - Complete ffprobe execution with JSON parsing
  - Extracts video stream metadata (codec, dimensions, bit depth, frame rates)
  - Parses container format information and tags
  - Intelligent bit depth detection from pixel format names
  - Handles Matroska and MP4 tag variations
  - Fallback to filesystem for file size when ffprobe doesn't provide it
  - Comprehensive error handling with helpful context

#### Daemon (av1d)
- **Full File Analysis** (`av1d/src/main.rs`)
  - `analyze_file_full()` - Complete analysis using ffprobe
  - `is_file_stable()` - Multi-sample stability check (detects files being copied)
  - Applies all heuristics to real media metadata
  - Determines encoding parameters (quality, surface, WebRip handling)
  - Detailed output showing what would be done and why
  
- **FFmpeg/QSV Validation**
  - `validate_ffmpeg_environment()` - Comprehensive startup validation
  - `extract_ffmpeg_version()` - Version parsing and checking
  - `test_qsv_hardware()` - QSV hardware initialization test
  - Verifies ffmpeg version 8.x or n8.x
  - Confirms ffprobe availability
  - Checks for av1_qsv encoder in build
  - Optional QSV hardware test (testsrc2 → null)
  - Clear status reporting with warnings

#### TUI (av1top)
- **Real Job Loading** (`av1top/src/main.rs`)
  - Loads actual job files from `~/.local/share/av1janitor/jobs/`
  - Auto-reloads jobs every 2 seconds
  - Falls back to demo data if no jobs exist
  - Shows job load status in status bar
  - Displays error messages if loading fails
  - Sorts jobs by creation date (newest first)
  - Job count display in status bar

### Changed

#### Daemon
- Transformed from simple stub to full analysis tool
- Now provides detailed per-file analysis with ffprobe integration
- Added file stability checking before analysis
- Enhanced output formatting with more information

#### TUI
- Status bar now shows actual job count and load status
- Auto-refreshing job data instead of static dummy data
- Better error handling and user feedback

### Technical Details

**New Dependencies:**
- All existing dependencies remain

**New Functions:**
- `core::run_ffprobe()` - Execute ffprobe and parse output
- `core::convert_ffprobe_output()` - Convert ffprobe JSON to FileMetadata
- `core::parse_bit_depth()` - Intelligent bit depth detection
- `av1d::validate_ffmpeg_environment()` - Complete ffmpeg validation
- `av1d::extract_ffmpeg_version()` - Parse ffmpeg version string
- `av1d::test_qsv_hardware()` - Test QSV initialization
- `av1d::analyze_file_full()` - Full file analysis with ffprobe
- `av1d::is_file_stable()` - File stability check
- `av1top::App::reload_jobs()` - Reload jobs from disk

**Code Quality:**
- Zero linter errors
- All functions have doc comments
- Comprehensive error handling
- ~2,500+ total lines of code

## [Phase 1] - Initial Scaffold

### Added

#### Project Structure
- Created Rust workspace with three crates
- Root `Cargo.toml` with workspace dependencies
- `.gitignore` for Rust projects
- Comprehensive documentation (README.md, IMPLEMENTATION_STATUS.md)

#### Core Library (`core/`)
- **Configuration** (`config.rs`)
  - `TranscodeConfig` - FFmpeg settings, directories, thresholds
  - `PathsConfig` - Logs and job directories
  - `QsvQualitySettings` - Resolution-based quality settings
  - Default implementations with sensible values

- **Error Handling** (`error.rs`)
  - `CoreError` enum with thiserror
  - Covers all error cases (FFmpeg, IO, JSON, metadata)
  - Proper error context and messages

- **Job Model** (`job.rs`)
  - `JobStatus` enum (Pending, Running, Success, Failed, Skipped)
  - `JobReason` - Textual explanations
  - `TranscodeJob` - Complete job state tracking
  - Helper methods for duration and size savings
  - Human-readable formatting functions

- **Metadata Types** (`metadata.rs`)
  - `VideoStreamInfo` - Stream details
  - `FileMetadata` - Complete file information
  - Helper methods for VFR detection, odd dimensions, resolution labels

- **Heuristics** (`heuristics.rs`)
  - `is_webrip_like()` - WebRip detection
  - `should_skip_for_size()` - Size threshold check
  - `is_already_av1()` - Codec detection
  - `choose_quality()` - Resolution-based quality selection
  - `choose_surface()` - Bit-depth based format selection
  - Comprehensive unit tests

- **Persistence** (`persistence.rs`)
  - `save_job_state()` - Serialize job to JSON
  - `load_all_jobs()` - Load all jobs from directory
  - Creates directories as needed
  - Handles corrupt files gracefully
  - Unit tests with tempfile

- **FFprobe Stub** (`ffprobe.rs`)
  - Function signatures and types defined
  - Clear TODO markers for implementation

#### Daemon (`av1d/`)
- Main entry point with configuration display
- Directory scanning (recursive)
- File filtering by extension
- `.av1skip` marker detection
- Size threshold checking
- Basic analysis reporting

#### TUI (`av1top/`)
- Complete ratatui-based interface
- CPU and memory gauges
- Disk usage panel
- Job table with 8 columns (uppercase headers)
- Color-coded status indicators
- Keyboard controls (q to quit, r to refresh)
- 1-second refresh rate
- Demo data for display testing

### Testing
- Unit tests for heuristics functions
- Unit tests for persistence functions
- All tests passing

---

## Version History

- **Phase 2**: Full media analysis capabilities
- **Phase 1**: Initial scaffold with all types and structures

## Next Release Goals

- [ ] Actual ffmpeg transcoding execution
- [ ] Job state management during transcoding
- [ ] Size gate implementation
- [ ] Atomic file replacement
- [ ] Continuous daemon loop
- [ ] Configuration file loading

