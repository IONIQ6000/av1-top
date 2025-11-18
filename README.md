# AV1 Transcoding Daemon + TUI Monitor

> **Pure Go Implementation** - Automated AV1 video transcoding with Intel QSV hardware acceleration and real-time TUI monitoring built with Bubble Tea.

## Overview

A complete system for automated AV1 transcoding with hardware acceleration:

1. **av1d** - Daemon that watches directories and transcodes media files to AV1
2. **av1top** - Real-time TUI monitor showing transcoding progress and system metrics

**Features:**
- 🚀 Intel QSV/VAAPI hardware-accelerated AV1 encoding
- 📊 Beautiful TUI with Bubble Tea showing live progress
- ⚡ Concurrent transcoding (configurable workers)
- 📁 Automatic directory scanning and file discovery
- 🎯 Smart quality presets based on resolution
- 💾 Size gate validation (rejects if output > 90% of original)
- 🔄 Atomic file replacement on success
- 📝 Comprehensive job tracking and history

## Quick Start

### Prerequisites

- Go 1.21 or higher
- FFmpeg 8.0+ with VAAPI/QSV support
- Intel GPU with AV1 encoding support

### Installation

```bash
# Clone the repository
git clone https://github.com/IONIQ6000/av1-top.git
cd av1-top

# Build binaries
go build -o av1d ./cmd/av1d
go build -o av1top ./cmd/av1top

# Install system-wide
sudo cp av1d /usr/local/bin/
sudo cp av1top /usr/local/bin/

# Create config directory
sudo mkdir -p /etc/av1janitor

# Create configuration (see Configuration section)
sudo nano /etc/av1janitor/config.toml
```

### Configuration

Create `/etc/av1janitor/config.toml`:

```toml
# Directories to watch for media files
watched_directories = ["/path/to/your/media"]

# Minimum file size in bytes (2 GiB default)
min_file_size_bytes = 2147483648

# Size gate factor (0.9 = output must be <= 90% of original)
size_gate_factor = 0.9

# File extensions to consider
media_extensions = ["mkv", "mp4", "avi"]

# Scan interval in seconds
scan_interval_seconds = 60

# Maximum scan depth (-1 = unlimited, 0 = current dir only, 1 = one level deep)
max_scan_depth = 1

# Quality settings per resolution (lower = better quality, higher bitrate)
[qsv_quality]
below_1080p = 25        # For 720p and below
at_1080p = 24           # For 1080p
at_1440p_and_above = 23 # For 1440p, 4K, and above
```

### Running

**Start the daemon:**
```bash
# Run directly
./av1d --config /etc/av1janitor/config.toml --concurrent 2

# Or as systemd service (see Installation Guide)
sudo systemctl start av1janitor
```

**Monitor with TUI:**
```bash
# Launch the TUI
./av1top

# Or if installed system-wide
av1top
```

**TUI Controls:**
- `q` or `Ctrl+C` - Quit
- `r` - Force refresh

## Project Structure

```
av1-top/
├── cmd/
│   ├── av1d/          # Daemon binary
│   └── av1top/        # TUI binary
├── internal/
│   ├── config/        # Configuration management
│   ├── ffmpeg/        # FFmpeg detection and validation
│   ├── persistence/   # Job state management
│   └── scanner/       # Directory scanning logic
├── pkg/
│   └── tui/          # Bubble Tea TUI implementation
├── go.mod            # Go module definition
└── README.md         # This file
```

## Features

### Daemon (av1d)

- **Automatic Discovery:** Scans configured directories for media files
- **Smart Filtering:** Respects minimum file size and extension filters
- **Hardware Acceleration:** Uses Intel QSV/VAAPI for fast encoding
- **Quality Presets:** Automatic quality selection based on resolution
- **Size Gate:** Validates output file size before replacing original
- **Concurrent Processing:** Process multiple files simultaneously
- **Job Tracking:** Creates JSON job files for TUI monitoring
- **Graceful Shutdown:** Finishes current job on Ctrl+C

### TUI Monitor (av1top)

- **System Metrics:** Real-time CPU, GPU, Memory, I/O monitoring
- **Job Queue:** Shows pending, running, completed, failed, and skipped jobs
- **Current Transcode:** Live progress of active transcoding job
- **Job History:** Table view of recent transcode jobs
- **Console Logs:** Recent daemon log output
- **Auto-refresh:** Updates every 2 seconds

## FFmpeg Setup

### Debian/Ubuntu (Trixie)

```bash
# Install Intel drivers
sudo apt-get install -y intel-media-va-driver-non-free libvpl2 vainfo

# Install FFmpeg 8.0+
# See FFMPEG_SETUP.md for detailed instructions

# Verify VAAPI
vainfo

# Test encoding
ffmpeg -hide_banner -init_hw_device vaapi=va:/dev/dri/renderD128 \
  -f lavfi -i testsrc=duration=1:size=1920x1080:rate=30 \
  -c:v av1_vaapi -b:v 5M -f null -
```

### Permissions

Ensure the daemon user has access to GPU devices:

```bash
# Add user to video and render groups
sudo usermod -aG video,render av1janitor

# Set DRM device permissions
sudo chmod 0660 /dev/dri/renderD*
sudo chown root:render /dev/dri/renderD*
```

## Systemd Service

Create `/etc/systemd/system/av1janitor.service`:

```ini
[Unit]
Description=AV1 Janitor - Automated AV1 Transcoding Daemon
After=network.target

[Service]
Type=simple
User=av1janitor
Group=av1janitor
WorkingDirectory=/opt/av1janitor

ExecStart=/usr/local/bin/av1d --config /etc/av1janitor/config.toml --concurrent 2

Restart=on-failure
RestartSec=10s
TimeoutStopSec=14400

Environment="LIBVA_DRIVER_NAME=iHD"
SupplementaryGroups=render video

ReadWritePaths=/main-library-2 /var/lib/av1janitor /opt/av1janitor

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable av1janitor
sudo systemctl start av1janitor
```

## Development Status

**✅ Completed:**
- Go project structure and module setup
- Configuration loading and validation
- FFmpeg detection and validation
- Directory scanning with depth control
- Bubble Tea TUI with live metrics
- Job persistence and state management
- System metrics monitoring (CPU, GPU, Memory, I/O)

**🚧 In Progress:**
- Full transcoding implementation in Go daemon
- FFmpeg command builder and executor
- Progress monitoring and job updates
- File watching and automatic processing

**📋 Planned:**
- Size gate validation
- Atomic file replacement
- Error handling and recovery
- Comprehensive testing

## Dependencies

```go
require (
    github.com/charmbracelet/bubbletea v0.25.0  // TUI framework
    github.com/charmbracelet/lipgloss v0.9.1     // Terminal styling
    github.com/pelletier/go-toml/v2 v2.2.0       // TOML configuration
    github.com/shirou/gopsutil/v3 v3.23.12       // System metrics
)
```

## Contributing

This project is under active development. Contributions are welcome!

## License

MIT

## Related Documentation

- `config.example.toml` - Example configuration file
- `FFMPEG_SETUP.md` - Detailed FFmpeg installation guide
- `av1janitor.service` - Systemd service file template

## Support

For issues, questions, or contributions, please open an issue on GitHub.
