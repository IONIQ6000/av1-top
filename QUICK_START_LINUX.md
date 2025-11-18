# 🚀 Quick Start on Linux

## ONE-TIME SETUP

### Step 1: Build on Linux

```bash
# You're in ~/av1-top on your Linux system

# Pull latest
git pull

# Build (creates Linux binaries!)
cargo build --release --workspace

# Create package
./build-deb.sh

# Install
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f
```

### Step 2: Configure

```bash
sudo nano /etc/av1janitor/config.toml
```

Change this line to YOUR media directories:
```toml
watched_directories = ["/media/movies", "/media/tv"]
```

Save and exit (Ctrl+O, Enter, Ctrl+X)

### Step 3: Start Daemon

```bash
sudo systemctl start av1janitor
sudo systemctl enable av1janitor
```

### Step 4: Wait 30 seconds

The daemon needs time to:
- Scan your directories
- Find files that need transcoding
- Create job files
- Start transcoding

### Step 5: Monitor

```bash
av1top
```

**Now you'll see REAL data, not demo data!**

---

## Check Everything Works

```bash
# 1. Check binary
file /usr/bin/av1top
# Should show: ELF 64-bit LSB executable

# 2. Check daemon is running
sudo systemctl status av1janitor
# Should show: active (running)

# 3. Check for jobs
ls -la /var/lib/av1janitor/jobs/
# Should show job JSON files

# 4. View daemon logs
sudo journalctl -u av1janitor -f
# Should show scanning/transcoding activity

# 5. Run TUI
av1top
# Should show real system stats and jobs
```

---

## If You Still See Demo Data

### Cause 1: Daemon not running
```bash
sudo systemctl status av1janitor
# If not running:
sudo systemctl start av1janitor
```

### Cause 2: No files need transcoding
Check daemon logs:
```bash
sudo journalctl -u av1janitor -n 50
```

Look for:
- "Scanning..." messages
- "Found X files" messages
- Any errors

### Cause 3: Config is wrong
```bash
# Check config
cat /etc/av1janitor/config.toml

# Verify directories exist
ls -la /media/movies
```

### Cause 4: Files don't meet criteria
Files are skipped if:
- Already AV1 codec
- Smaller than 2 GB (default threshold)
- In wrong format (only .mkv, .mp4, .avi by default)

---

## Test With A Real File

```bash
# 1. Find a large video file
find /media -name "*.mkv" -size +2G | head -5

# 2. Check what codec it uses
ffprobe -v error -select_streams v:0 -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 /path/to/file.mkv

# If not av1, daemon should pick it up
```

---

## Manual Test

Force create a job to test:
```bash
sudo -u av1janitor av1d --once --dry-run -vvv
```

This runs the daemon once in dry-run mode with verbose logging.

---

## What You Should See in av1top

### Real Data:
- **CPU**: Your actual CPU usage (changing every second)
- **GPU**: Real GPU usage when transcoding
- **Memory**: Your actual RAM usage
- **I/O**: Real disk read/write speeds
- **Current Transcode**: Actual file being processed
- **Jobs Table**: Real files from your system

### Demo Data:
- **Static values** that don't change
- **Fake filenames** like "example_movie.mkv"
- **Message at bottom**: "No job files found, showing demo data"

---

## ONE-LINER

```bash
cd ~/av1-top && git pull && cargo build --release --workspace && ./build-deb.sh && sudo dpkg -i av1janitor_0.1.0_amd64.deb && sudo systemctl restart av1janitor && sleep 5 && av1top
```

This does everything:
1. Pull latest code
2. Build on Linux
3. Create package
4. Install
5. Restart daemon
6. Wait 5 seconds
7. Show TUI

---

**Build on Linux. Start daemon. Wait. See real data!** ✅

