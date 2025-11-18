# AV1Top - Comprehensive TUI Monitor

## Overview

The enhanced AV1Top TUI provides **comprehensive, real-time monitoring** of your transcoding system, inspired by tdarr's detailed interface. It displays everything you need to know about your system's performance and transcode operations.

## Layout Structure

```
┌─────────────────────────────────────────────────────────────┐
│ HEADER: Queue Summary (Pending, Running, Completed, etc.)  │
├─────────────────────────────────────────────────────────────┤
│              SYSTEM STATISTICS (4 Panels)                   │
│  ┌───────┐  ┌───────┐  ┌───────┐  ┌───────┐              │
│  │  CPU  │  │  GPU  │  │ Memory│  │  I/O  │              │
│  │ Gauge │  │ Gauge │  │ Gauge │  │ Stats │              │
│  └───────┘  └───────┘  └───────┘  └───────┘              │
├─────────────────────────────────────────────────────────────┤
│              CURRENT TRANSCODE DETAILS                      │
│  Progress bar, file sizes, ratio, duration                 │
├─────────────────────────────────────────────────────────────┤
│              TRANSCODE HISTORY TABLE                        │
│  Status │ File │ Original │ Output │ Savings │ etc.       │
├─────────────────────────────────────────────────────────────┤
│ FOOTER: Controls and status messages                       │
└─────────────────────────────────────────────────────────────┘
```

## Features Breakdown

### 1. Header - Queue Overview
Displays at-a-glance queue statistics:
- **Queue**: Number of pending jobs waiting to process
- **Running**: Currently active transcodes (typically 1)
- **✓ Completed**: Successfully transcoded files
- **✗ Failed**: Failed transcode attempts
- **⊘ Skipped**: Files skipped (size gate, too small, etc.)

**Example:**
```
Queue: 5 │ Running: 1 │ ✓ 42 │ ✗ 2 │ ⊘ 8
```

### 2. System Statistics Panels

#### CPU Panel
- **Gauge**: Visual representation of CPU usage (0-100%)
  - Green: < 80% (normal)
  - Red: > 80% (high load)
- **Usage %**: Current CPU utilization
- **Cores**: Number of CPU cores/threads

**Color Coding:**
- 🟢 Green gauge: CPU usage < 80%
- 🔴 Red gauge: CPU usage ≥ 80%

#### GPU Panel (Intel QSV)
- **Gauge**: Visual representation of GPU usage
  - Magenta gauge shows GPU activity
  - Red if > 80%
- **Usage %**: GPU utilization (estimated from frequency)
- **VRAM**: Video memory usage in MB
- **Encoder**: Status indicator
  - 🟢 Green "Active": Currently encoding
  - ⚪ Gray "Idle": Not encoding

**Intel GPU Detection:**
- On Linux: Reads from `/sys/class/drm/card0/gt_cur_freq_mhz`
- On macOS: Shows placeholder (no direct Intel GPU stats)

#### Memory Panel
- **Gauge**: Visual memory usage
  - Green: < 80%
  - Red: ≥ 80%
- **Used/Total**: RAM usage in GiB (e.g., "12.5 / 32.0 GiB")
- **Swap**: Swap memory usage in GiB

#### I/O Panel
- **Read MB/s**: Disk read throughput
  - 🔵 Cyan color
- **Write MB/s**: Disk write throughput
  - 🟡 Yellow color
- **Disks**: Number of disks detected

**I/O Detection:**
- On Linux: Reads from `/proc/diskstats` for accurate I/O
- On macOS: Shows 0.0 (requires different API)

### 3. Current Transcode Details

Shows detailed information about the **actively running** transcode:

**Displayed Information:**
- **FILE**: Full filename of the file being transcoded
- **Progress Gauge**: Visual progress bar (0-100%)
  - Green gauge showing estimated completion
- **Original Size**: Source file size (e.g., "5.23 GiB")
- **Current Size**: Output file size so far (yellow text)
- **Ratio**: Current output/input ratio
  - 🟢 Green: ≤ 90% (will pass size gate)
  - 🔴 Red: > 90% (will fail size gate)
- **Duration**: Elapsed time (e.g., "15m 23s")
- **Status**: "TRANSCODING" in bold green

**When Idle:**
- Shows "No active transcodes" in gray

**Example:**
```
┌─ Current Transcode ──────────────────────────────────────┐
│ ████████████████░░░░░░░░░░░░ 68%                        │
│ FILE: big_movie_4k.mkv                                   │
│ Original: 8.45 GiB  │  Current: 5.23 GiB  │  Ratio: 61.9%│
│ Duration: 18m 34s  │  Status: TRANSCODING                │
└──────────────────────────────────────────────────────────┘
```

### 4. Transcode History Table

Comprehensive table showing all transcode jobs (sorted newest first):

**Columns:**
1. **STATUS**
   - 🟢 SUCCESS: Completed successfully
   - 🔵 RUNNING: Currently processing
   - 🔴 FAILED: Encoding failed
   - 🟡 SKIPPED: Did not transcode (size gate, etc.)
   - ⚪ PENDING: Waiting in queue

2. **FILE**: Filename (truncated if > 20 chars)

3. **ORIGINAL**: Source file size
   - Format: "5.23 GiB", "1.2 GiB", etc.

4. **OUTPUT**: Transcoded file size
   - Shows "N/A" for pending/running jobs
   - Format: "3.45 GiB"

5. **SAVINGS**: Absolute space saved
   - Format: "1.78 GiB"
   - Shows actual GiB saved

6. **RATIO**: Output/Input percentage
   - Format: "65.9%"
   - Lower is better (more compression)
   - Shows what % of original size remains

7. **DURATION**: Time taken to transcode
   - Format: "18m 34s", "1h 23m 45s"
   - Shows "N/A" for pending jobs

8. **REASON**: Why skipped/failed
   - "Size gate failed"
   - "File too small"
   - "FFmpeg error"
   - "-" for successful jobs

**Color Coding:**
- 🟢 Green rows: Successful transcodes
- 🔵 Cyan rows (bold): Currently running
- 🔴 Red rows: Failed
- 🟡 Yellow rows: Skipped
- ⚪ Gray rows: Pending

### 5. Footer - Controls & Status

**Left Side: Controls**
- `q`: Quit application
- `r`: Force refresh all data

**Right Side: Status**
- ✓ "X jobs loaded" (green): Jobs loaded successfully
- ⚠ "Error message" (yellow): If job loading failed

## Real-Time Updates

- **System Stats**: Update every 1 second
  - CPU, GPU, Memory, I/O all refresh
- **Jobs**: Reload every 2 seconds
  - Reads job JSON files from disk
  - Updates table automatically
- **Render**: 100ms tick rate
  - Smooth UI updates

## Data Sources

### System Metrics
- **CPU**: `sysinfo` crate - cross-platform
- **Memory**: `sysinfo` crate - cross-platform
- **GPU**: Linux: `/sys/class/drm/card0/*`, macOS: not available
- **I/O**: Linux: `/proc/diskstats`, macOS: not implemented
- **Disks**: `sysinfo` Disks API

### Transcode Data
- **Job Files**: `~/.local/share/av1janitor/jobs/*.json`
- **Format**: JSON with job status, timestamps, sizes
- **Updates**: Written by daemon as jobs progress

## Usage

```bash
# Run the TUI
cargo run --bin av1top

# Or after building
./target/release/av1top
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit |
| `r` | Force refresh |

## Visual Examples

### Active Transcode Session
```
┌─ AV1 Transcoding Monitor ────────────────────────────────────┐
│ AV1 Janitor │ Queue: 3 │ Running: 1 │ ✓ 45 │ ✗ 2 │ ⊘ 7      │
├───────────────────────────────────────────────────────────────┤
│ ┌─ CPU ───┐ ┌─ GPU (Intel QSV) ─┐ ┌─ Memory ┐ ┌─ I/O Stats ┐│
│ │███░░ 65% │ │████░ 85%         │ │███░ 72% │ │Read: 45MB/s││
│ │Usage: 65%│ │Usage: 85%        │ │12.3/32GiB│ │Write: 120  ││
│ │Cores: 16 │ │VRAM: 245 MB      │ │Swap: 0GiB│ │Disks: 2    ││
│ │          │ │Encoder: Active    │ │          │ │            ││
│ └──────────┘ └───────────────────┘ └──────────┘ └────────────┘│
├───────────────────────────────────────────────────────────────┤
│ ┌─ Current Transcode ──────────────────────────────────────┐ │
│ │ ████████████░░░░░░░ 72%                                  │ │
│ │ FILE: amazing_movie_1080p.mkv                            │ │
│ │ Original: 5.45 GiB │ Current: 3.67 GiB │ Ratio: 67.3%   │ │
│ │ Duration: 12m 45s │ Status: TRANSCODING                  │ │
│ └──────────────────────────────────────────────────────────┘ │
├───────────────────────────────────────────────────────────────┤
│ ┌─ Transcode History ──────────────────────────────────────┐ │
│ │STATUS  │FILE       │ORIGINAL│OUTPUT │SAVINGS│RATIO│DUR...││
│ │RUNNING │amazing... │5.45 GiB│3.67Gi.│N/A    │67.3%│12m45s││
│ │SUCCESS │movie1.mkv │8.23 GiB│5.12Gi.│3.11Gi.│62.2%│23m12s││
│ │SUCCESS │episode... │3.45 GiB│2.34Gi.│1.11Gi.│67.8%│8m34s ││
│ │SKIPPED │small.avi  │1.2 GiB │N/A    │N/A    │N/A  │N/A   ││
│ └──────────────────────────────────────────────────────────┘ │
├───────────────────────────────────────────────────────────────┤
│  q  Quit   r  Refresh  │  ✓ 52 jobs loaded                   │
└───────────────────────────────────────────────────────────────┘
```

### Idle (No Active Transcodes)
```
┌─ AV1 Transcoding Monitor ────────────────────────────────────┐
│ AV1 Janitor │ Queue: 0 │ Running: 0 │ ✓ 45 │ ✗ 2 │ ⊘ 7      │
├───────────────────────────────────────────────────────────────┤
│ [System stats panels with low usage...]                       │
├───────────────────────────────────────────────────────────────┤
│ ┌─ Current Transcode ──────────────────────────────────────┐ │
│ │                                                           │ │
│ │              No active transcodes                         │ │
│ │                                                           │ │
│ └──────────────────────────────────────────────────────────┘ │
├───────────────────────────────────────────────────────────────┤
│ [Transcode history showing completed jobs...]                 │
└───────────────────────────────────────────────────────────────┘
```

## Comparison to Tdarr

### Features Ported from Tdarr:
✅ Queue statistics (pending, running, completed)
✅ Real-time system metrics (CPU, GPU, RAM)
✅ Current job details with progress
✅ File size comparison (original vs output)
✅ Compression ratio calculation
✅ I/O throughput monitoring
✅ GPU encoder status
✅ Comprehensive job history
✅ Color-coded status indicators
✅ Duration tracking

### AV1Top Specific Features:
- Intel QSV-specific GPU monitoring
- Native terminal UI (no browser needed)
- Low resource overhead
- Fast refresh rates (1-2 second updates)
- Clean, modern ratatui design

## Technical Implementation

### Architecture
- **UI Framework**: ratatui (modern TUI library)
- **System Info**: sysinfo crate
- **Job Data**: JSON files on disk
- **Refresh Strategy**: Async updates every 1-2 seconds

### Performance
- **Memory**: ~5-10 MB
- **CPU**: < 1% when idle
- **Updates**: Non-blocking, smooth rendering

## Troubleshooting

### GPU Stats Show 0%
- **Linux**: Check `/sys/class/drm/card0/` exists
- **Permissions**: May need to be in `render` or `video` group
- **macOS**: GPU stats not available on macOS

### I/O Stats Show 0.0 MB/s
- **Linux**: Should work automatically via `/proc/diskstats`
- **macOS**: Not implemented (requires different API)

### Jobs Not Updating
- Check daemon is running and writing job files
- Verify jobs directory: `~/.local/share/av1janitor/jobs/`
- Look for JSON files in that directory

## Future Enhancements

Potential additions (not yet implemented):
- [ ] Historical graphs (CPU/GPU over time)
- [ ] Per-disk I/O breakdown
- [ ] Network transfer rates
- [ ] Temperature monitoring
- [ ] Log viewer panel
- [ ] Interactive job control (pause/resume)
- [ ] Filter/search in job table
- [ ] Export job history to CSV

---

**The TUI is now comprehensive, modern, and provides all the information you need to monitor your AV1 transcoding operations!**

