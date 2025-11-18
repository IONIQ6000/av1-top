
# AV1 Daemon + TUI Monitor Project Spec (for Cursor)

You are my **senior Rust engineer** and TUI architect. I want to build a Rust project that does two main things:

1. A **daemon-like tool** that watches my media library and runs **AV1 transcodes via ffmpeg + Intel QSV**, following specific rules.
2. A **btop-style TUI monitor** that shows both **system metrics** and **the state of these transcode jobs** (queue, running, done, failed, size savings, etc).

Think of it as:  
> “btop, but aware of my AV1 transcoder.”

We are targeting **Linux**, running in a server context (likely in Docker). The machine has an **Intel GPU** (Arc A310) and a custom ffmpeg at `/external-ffmpeg/ffmpeg`.

---

## Overall Project Shape

Create a **Rust workspace** with:

- A root `Cargo.toml` (workspace).
- One library crate for shared logic: `core`.
- Two binary crates:
  - `av1d` (daemon) → watches folders, schedules and runs transcodes.
  - `av1top` (TUI) → shows system metrics & transcoder jobs, btop-style.

Directory layout:

- `Cargo.toml` (workspace)
- `core/`
  - `Cargo.toml`
  - `src/lib.rs`
- `av1d/`
  - `Cargo.toml`
  - `src/main.rs`
- `av1top/`
  - `Cargo.toml`
  - `src/main.rs`

Use **Rust stable** and idiomatic patterns. Avoid unnecessary cleverness.

---

## Crates / Dependencies

In the workspace, use these crates (add what you need, but at minimum):

**Common (core/):**

- `serde`, `serde_json`, `serde_derive` or `serde` with `derive` feature
- `thiserror` for error types
- `chrono` for timestamps

**Daemon (av1d/):**

- `notify` for filesystem watching, or implement a simple polling loop (your call, but design APIs so we can swap later).
- `tokio` or `async-std` is fine, but you can also start with a simple threaded model if easier.
- `sysinfo` for basic system metrics if needed in daemon (CPU load, etc).
- `anyhow` for easy error bubbling in main.

**TUI (av1top/):**

- `ratatui` (successor of `tui-rs`) for TUI layout and widgets.
- `crossterm` for terminal backend.
- `sysinfo` for CPU, memory, disk, process info.

Try to keep versions relatively recent/stable.

---

## Core crate (shared logic)

Design `core` to hold:

### 1. Config types

- `TranscodeConfig`:
  - path to ffmpeg binary (default `/external-ffmpeg/ffmpeg`)
  - watched library roots: `Vec<PathBuf>`
  - file size threshold (bytes) for minimum source size (e.g. 2 GiB)
  - size gate factor for output (e.g. 0.9 = must be ≤ 90% of original)
  - QSV-specific settings (global_quality per resolution bucket)
- `PathsConfig`:
  - logs directory
  - job state directory

Config should be serializable/deserializable via `serde`.

Use a simple TOML config file (e.g. `~/.config/av1janitor/config.toml`) but for now it’s OK to hard-code defaults and leave `TODO` markers where config reading will go.

### 2. Job model

Define a `TranscodeJob` struct that represents **one file’s conversion attempt**, with fields like:

```rust
pub enum JobStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

pub struct JobReason(pub String); // textual explanation

pub struct TranscodeJob {
    pub id: String,                // some unique id (e.g. UUID or hash of path+time)
    pub source_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub reason: Option<JobReason>,
    pub original_bytes: Option<u64>,
    pub new_bytes: Option<u64>,
    pub is_webrip_like: bool,      // based on heuristics
}
```

Also define helper methods like:

- `duration()`
- `size_savings_ratio()` etc.

### 3. ffprobe / metadata utilities

Add helper functions:

```rust
pub struct VideoStreamInfo {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub is_default: bool,
    pub avg_frame_rate: String,
    pub r_frame_rate: String,
}

pub struct FileMetadata {
    pub video_streams: Vec<VideoStreamInfo>,
    pub format_name: String,
    pub tags_muxing_app: Option<String>,
    pub tags_major_brand: Option<String>,
    pub tags_compatible_brands: Option<String>,
    pub size: Option<u64>,
}

pub fn run_ffprobe(ffmpeg_path: &Path, file: &Path) -> Result<FileMetadata, CoreError>;
```

Implementation detail:

- Call `ffprobe` via ffmpeg’s binary or `ffprobe` sibling.
- For now, you can mock or stub some details if needed; just design the API clearly.

### 4. Heuristics

Implement pure functions that express our rules *exactly* like we discussed:

- `fn is_webrip_like(meta: &FileMetadata) -> bool;`

  WebRip-like if:
  - `format_name` includes `mp4`, `mov`, or `webm`, OR
  - any video stream is VFR (avg_frame_rate != r_frame_rate), OR
  - any video stream has odd dimensions (width or height is not divisible by 2).

- `fn should_skip_for_size(bytes: u64, min_bytes: u64) -> bool;`
- `fn is_already_av1(meta: &FileMetadata) -> bool;`
- `fn choose_quality(height: u32) -> u8;`  
  - <1080 → 25  
  - =1080 → 24  
  - ≥1440 → 23
- `fn choose_surface(bit_depth: u8) -> &'static str;`
  - ≥10 → `p010`  
  - else → `nv12`

These should be deterministic and easy to test.

### 5. Job state serialization

Define a way to serialize `TranscodeJob` to JSON and write it to a `*.json` file, e.g.:

- jobs directory: `state/jobs/`
- per-job file: `<job_id>.json`

This is what `av1top` will read to display the job list.

Provide functions:

```rust
pub fn save_job_state(job: &TranscodeJob, dir: &Path) -> Result<(), CoreError>;
pub fn load_all_jobs(dir: &Path) -> Result<Vec<TranscodeJob>, CoreError>;
```

---

## Daemon crate: `av1d`

Purpose:  
A long-running process that:

1. Scans configured directories for candidate media files.
2. Applies logic:
   - Skip if:
     - file has `<basename>.av1skip`
     - below min size
     - no video
     - already AV1
     - file is currently “unstable” (size changing across several seconds)
   - Otherwise, queue a `TranscodeJob`.
3. Executes ffmpeg with external binary `/external-ffmpeg/ffmpeg` using **Intel QSV AV1** and our rules.
4. Applies size gate & WebRip safety.
5. On completion, updates job status JSON and writes `.why.txt` / `.av1skip` when needed.
6. Atomically replaces the source file on success.

### Daemon behavior

Implement, at minimum, a simple loop:

- Every N seconds (e.g. 60):
  - Recursively scan configured folders for `.mkv`, `.mp4`, `.avi`, etc. (keep the extension list short and configurable)
  - For each file:
    - If there is a job JSON already with status `Success` and no newer mtime on the file, skip.
    - If there is `.av1skip`, skip.
    - Else, consider it for processing.

- Process one job at a time for now (concurrency can be added later).

### Transcode pipeline

Use the **exact encoding ideas** we've already debugged:

- Use external binary: `/external-ffmpeg/ffmpeg`
- Initial guard:
  - Verify `ffmpeg -version` reports `8.x` or `n8.x`.
  - Verify `av1_qsv` is present in `-encoders`.
  - Run a small QSV test (e.g. `testsrc2` → `null`) once at startup and cache the result.

- For each job:

  1. Run ffprobe to get metadata.
  2. Apply heuristics:
     - Skip if:
       - < min size (e.g. 2 GiB)
       - already AV1
  3. Check the file’s size stability:
     - Sample `stat` size, wait a few seconds, sample again, repeat 3–5 times.
     - If still changing, mark as `Skipped` with reason “file still copying”.
  4. Decide:
     - `vord` (video index) based on default disposition.
     - `qual` (23/24/25).
     - `surface` (`p010` / `nv12`).
     - `webLikely` based on heuristics.

  5. Build the ffmpeg command line that roughly matches:

     - Input flags:
       - `-y -v verbose -stats -benchmark -benchmark_all`
       - `-hwaccel none`
       - `-init_hw_device qsv=hw -filter_hw_device hw`
       - `-analyzeduration 50M -probesize 50M`
       - plus `-fflags +genpts -copyts -start_at_zero` if `webLikely`.

     - Mapping & filters:
       - `-map 0`
       - `-map -0:v`
       - `-map -0:t`
       - `-map 0:v:<vord>`
       - `-map 0:a?`
       - `-map -0:a:m:language:rus`
       - `-map -0:a:m:language:ru`
       - `-map 0:s?`
       - `-map -0:s:m:language:rus`
       - `-map -0:s:m:language:ru`
       - `-map_chapters 0`
       - If `webLikely`, add `-vsync 0 -avoid_negative_ts make_zero`.
       - `-vf:v:0 pad=ceil(iw/2)*2:ceil(ih/2)*2,setsar=1,format=<surface>,hwupload=extra_hw_frames=64`
       - `-c:v:0 av1_qsv`
       - `-global_quality:v:0 <qual>`
       - `-preset:v:0 medium`
       - `-look_ahead 1`
       - `-c:a copy`
       - `-c:s copy`
       - `-max_muxing_queue_size 2048`
       - `-map_metadata 0`
       - `-f matroska`
       - `-movflags +faststart`

     - Output file:
       - Same directory as source
       - Name: `<basename>.av1-tmp.mkv` during processing
       - On success & size gate pass, rename to `<basename>.mkv` with atomic replace (or `<basename>.av1.mkv` and then swap/rename the original elsewhere).

  6. Size gate:
     - Compare new vs original bytes
     - If new > 90% of original:
       - Write `.why.txt` with reason.
       - Write `.av1skip`.
       - Delete converted file.
       - Mark job as `Skipped` with appropriate reason.
     - Else:
       - Replace original file atomically.
       - Mark job `Success`, fill in `new_bytes`.

  7. ALWAYS update the `TranscodeJob` JSON status file as you go:
     - On `Pending` → `Running` → `Success`/`Failed`/`Skipped`.

No need to implement every tiny detail at once, but design the module structure to support this logic cleanly.

---

## TUI crate: `av1top`

Purpose:  
A **btop-style** terminal UI that displays:

- CPU usage, memory usage, maybe load average.
- Disk usage summary (at least for library paths).
- A table of **AV1 jobs**, reading from the job JSON files written by the daemon:
  - Columns like: `STATUS`, `FILE`, `RES`, `ORIG_SIZE`, `NEW_SIZE`, `SAVINGS`, `DURATION`, `REASON`.

### Libraries

- Use `ratatui` + `crossterm`.
- Use `sysinfo` for CPU, memory, disks.
- Use `serde_json` to read job JSONs from the jobs directory.

### Layout (first version)

- Top row: CPU & memory bars.
- Middle: two panels:
  - Left: Disk usage (per mount or per configured library path).
  - Right: Job list (table).
- Bottom: A status line:
  - Show “daemon status” (just assume running for now or check PID file later).
  - Show hotkeys (e.g. `q` to quit, `r` to refresh).

### Behavior

- Refresh every 1 second.
- On each tick:
  - Re-scan job JSON directory.
  - Parse job entries and show:
    - Sort by `created_at` or `started_at`, newest first.
  - Update system metrics.

- Implement keyboard handling:
  - `q` → quit.
  - Future: up/down, filtering, etc. but not required in first version.

---

## Coding Style & Non-functional Requirements

- Use **clear, boring, explicit code**. I want readability > cleverness.
- Every public function in `core` should have a doc comment that explains what it does in plain language.
- Handle errors explicitly. Use `thiserror` in `core` to define a `CoreError` enum.
- For long-running loops (in the daemon), log important events:
  - when a job is discovered
  - when a job starts
  - when ffmpeg starts and ends
  - when a job succeeds or fails
  - when size gate rejects a file
- For now, use simple `eprintln!` or `log` crate with env_logger; we can wire proper logging later.

---

## What I want you (Cursor) to do first

1. **Set up the workspace skeleton**:
   - Workspace `Cargo.toml`
   - `core`, `av1d`, `av1top` crates with their `Cargo.toml` files.
   - Add the dependencies listed above.

2. Implement in `core`:
   - `TranscodeConfig`, `PathsConfig`, basic `TranscodeJob` & `JobStatus`.
   - `FileMetadata` & `VideoStreamInfo` types.
   - Stubs for:
     - `run_ffprobe(...)`
     - `is_webrip_like(...)`
     - `should_skip_for_size(...)`
     - `is_already_av1(...)`
     - `choose_quality(...)`
     - `choose_surface(...)`
   - Implement JSON serialization for `TranscodeJob` and functions to:
     - `save_job_state(job: &TranscodeJob, dir: &Path) -> Result<(), CoreError>`
     - `load_all_jobs(dir: &Path) -> Result<Vec<TranscodeJob>, CoreError>`

3. In `av1d`:
   - Implement a simple `main` that:
     - Reads config (or uses hard-coded defaults).
     - Scans the library directory once.
     - For each file, prints a line about whether it would be queued or skipped based on core heuristics (no ffmpeg yet).
   - This is just to validate the wiring.

4. In `av1top`:
   - Implement a simple `ratatui`-based UI that:
     - Draws placeholder panels and a dummy job table (hardcoded rows) to verify layout & input handling.

After that skeleton is working and compilable, we can iterate:

- Flesh out `run_ffprobe`.
- Add the ffmpeg invocation to `av1d`.
- Replace dummy jobs in `av1top` with reading real job JSONs.

---

Take this document as the **project spec**. In Cursor, treat this as the authoritative description of what I want to build, and start by generating the workspace and initial code, keeping the code clean, commented, and modular so we can keep extending it.
