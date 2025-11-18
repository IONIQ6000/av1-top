# Deployment Guide

## Production Deployment of AV1 Janitor

This guide covers deploying the AV1 daemon as a system service for 24/7 operation.

---

## Prerequisites

1. **FFmpeg 8.0+** installed and in PATH
2. **Intel GPU drivers** installed
3. **Rust** for building (or use pre-built binaries)
4. **User account** with GPU access (render/video group)

---

## Build for Production

```bash
# Build optimized binaries
cargo build --release --workspace

# Binaries are in target/release/
ls -lh target/release/av1d
ls -lh target/release/av1top
```

---

## Installation

### 1. Install Binaries

```bash
# Copy to system binaries
sudo cp target/release/av1d /usr/local/bin/
sudo cp target/release/av1top /usr/local/bin/

# Make executable
sudo chmod +x /usr/local/bin/av1d
sudo chmod +x /usr/local/bin/av1top

# Verify
av1d --version
av1top --version
```

### 2. Create User & Directories

```bash
# Create dedicated user (optional but recommended)
sudo useradd -r -s /bin/false -d /opt/av1janitor av1janitor

# Add to GPU groups
sudo usermod -a -G render,video av1janitor

# Create directories
sudo mkdir -p /opt/av1janitor
sudo mkdir -p /var/log/av1janitor
sudo chown av1janitor:av1janitor /opt/av1janitor
sudo chown av1janitor:av1janitor /var/log/av1janitor
```

### 3. Create Configuration

```bash
# Create config directory
sudo mkdir -p /opt/av1janitor/.config/av1janitor

# Copy and edit config
sudo cp config.example.toml /opt/av1janitor/.config/av1janitor/config.toml
sudo nano /opt/av1janitor/.config/av1janitor/config.toml

# Set ownership
sudo chown -R av1janitor:av1janitor /opt/av1janitor/.config
```

Example `/opt/av1janitor/.config/av1janitor/config.toml`:
```toml
watched_directories = [
    "/media/movies",
    "/media/tv",
    "/media/downloads"
]

min_file_size_bytes = 2147483648  # 2 GiB
size_gate_factor = 0.9

media_extensions = ["mkv", "mp4", "avi", "m4v"]

[qsv_quality]
below_1080p = 25
at_1080p = 24
at_1440p_and_above = 23
```

---

## Systemd Service

### 1. Install Service File

```bash
# Copy service file
sudo cp av1janitor.service /etc/systemd/system/

# Edit if needed
sudo nano /etc/systemd/system/av1janitor.service

# Reload systemd
sudo systemctl daemon-reload
```

### 2. Start Service

```bash
# Enable on boot
sudo systemctl enable av1janitor

# Start service
sudo systemctl start av1janitor

# Check status
sudo systemctl status av1janitor

# View logs
sudo journalctl -u av1janitor -f
```

### 3. Manage Service

```bash
# Stop (graceful - finishes current job)
sudo systemctl stop av1janitor

# Restart
sudo systemctl restart av1janitor

# View logs (last 100 lines)
sudo journalctl -u av1janitor -n 100

# Follow logs in real-time
sudo journalctl -u av1janitor -f
```

---

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libva-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
WORKDIR /build
COPY . .

# Build release
RUN cargo build --release --workspace

FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg \
    intel-media-va-driver-non-free \
    libva2 \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /build/target/release/av1d /usr/local/bin/
COPY --from=builder /build/target/release/av1top /usr/local/bin/

# Create user
RUN useradd -r -s /bin/false av1janitor

# Set up directories
RUN mkdir -p /config /jobs /media && \
    chown av1janitor:av1janitor /config /jobs

USER av1janitor
WORKDIR /media

# Run daemon
CMD ["av1d", "--config", "/config/config.toml", "--concurrent", "2"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  av1janitor:
    build: .
    container_name: av1janitor
    devices:
      - /dev/dri:/dev/dri
    group_add:
      - render
      - video
    volumes:
      - ./config:/config
      - ./jobs:/home/av1janitor/.local/share/av1janitor/jobs
      - /path/to/media:/media
    environment:
      - RUST_LOG=info
    restart: unless-stopped
```

### Run Docker

```bash
# Build image
docker build -t av1janitor .

# Run container
docker run -d \
  --name av1janitor \
  --device /dev/dri:/dev/dri \
  --group-add $(getent group render | cut -d: -f3) \
  --group-add $(getent group video | cut -d: -f3) \
  -v ./config.toml:/config/config.toml:ro \
  -v /media:/media \
  av1janitor
```

---

## Monitoring

### View Logs

```bash
# Systemd
sudo journalctl -u av1janitor -f

# Docker
docker logs -f av1janitor

# File logs (if configured)
tail -f /var/log/av1janitor/av1d.log
```

### TUI Monitoring

```bash
# SSH into server and run
av1top

# Or use tmux/screen
tmux new -s av1monitor
av1top
# Ctrl+B, D to detach
```

---

## Configuration Tuning

### Concurrent Jobs

Balance based on your hardware:

```toml
# In config.toml - not directly configurable yet
# Use CLI: --concurrent N
```

**Recommendations:**
- **Arc A310**: 1-2 concurrent jobs
- **Arc A380**: 2-3 concurrent jobs
- **Arc A750/A770**: 3-4 concurrent jobs
- **Integrated GPU**: 1 job

### Timeout

Default is 4 hours. Adjust if needed:

```rust
// Edit core/src/constants.rs
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 3600 * 6; // 6 hours
```

### Size Gate

Adjust threshold based on your quality preferences:

```toml
# In config.toml
size_gate_factor = 0.85  # Stricter (85%)
size_gate_factor = 0.95  # More lenient (95%)
```

---

## Troubleshooting

### Daemon Won't Start

```bash
# Check FFmpeg
ffmpeg -version
ffmpeg -encoders | grep av1_qsv

# Check GPU access
vainfo
groups | grep -E 'render|video'

# Check logs
journalctl -u av1janitor -n 50
```

### No Files Being Processed

```bash
# Check config
cat ~/.config/av1janitor/config.toml

# Verify directories exist
ls -la /media/movies

# Check permissions
sudo -u av1janitor ls /media/movies

# Test manually
sudo -u av1janitor av1d --once --directory /media/test -vv
```

### High Resource Usage

```bash
# Reduce concurrency
av1d --concurrent 1

# Check system resources
htop
nvidia-smi  # or intel_gpu_top
```

---

## Backup & Recovery

### Backup Configuration

```bash
# Backup config and jobs
tar -czf av1janitor-backup.tar.gz \
  ~/.config/av1janitor \
  ~/.local/share/av1janitor/jobs
```

### Restore

```bash
# Restore from backup
tar -xzf av1janitor-backup.tar.gz -C ~
```

---

## Updating

```bash
# Stop daemon
sudo systemctl stop av1janitor

# Pull latest code
git pull

# Rebuild
cargo build --release --workspace

# Update binaries
sudo cp target/release/av1d /usr/local/bin/
sudo cp target/release/av1top /usr/local/bin/

# Restart
sudo systemctl start av1janitor
```

---

## Performance Optimization

### For Large Libraries

```bash
# Use higher concurrency
av1d --concurrent 4

# Use dry-run first to estimate time
av1d --dry-run --directory /media/large-library
```

### For 24/7 Operation

1. **Use systemd** for auto-restart
2. **Monitor logs** for errors
3. **Set up log rotation**
4. **Monitor disk space**
5. **Use TUI** for real-time monitoring

---

## Security Considerations

1. **Run as dedicated user** (not root)
2. **Limit file access** (only media directories)
3. **Set resource limits** (memory, CPU)
4. **Enable systemd hardening** (see service file)
5. **Validate config** (automatic)
6. **Monitor logs** for suspicious activity

---

## Success Metrics

Monitor these to ensure smooth operation:

- **Job success rate**: Should be > 90%
- **Size gate pass rate**: Depends on content (typically 70-90%)
- **CPU usage**: Should be moderate (QSV offloads to GPU)
- **Memory usage**: Should stay under 500 MB
- **Disk I/O**: Should be reasonable (depends on file sizes)

---

## Support

If you encounter issues:

1. Check logs: `journalctl -u av1janitor -n 100`
2. Run manual test: `av1d --once --dry-run --directory /test -vvv`
3. Verify FFmpeg: `ffmpeg -encoders | grep av1_qsv`
4. Check GPU: `vainfo`
5. Review config: `cat ~/.config/av1janitor/config.toml`

---

## Next Steps

After deployment:

1. Monitor for first 24 hours
2. Tune concurrency based on performance
3. Adjust quality settings if needed
4. Set up log rotation
5. Configure backups

**Your AV1 transcoding system is ready for production!** 🎉

