# Why Am I Seeing "Demo Data"?

## This is NORMAL!

The TUI shows "demo data" when **there are no real transcode jobs yet**. This is by design, not a bug!

---

## How The TUI Works

```rust
// From av1top/src/main.rs lines 124-136:
match core::load_all_jobs(&paths_config.jobs_dir) {
    Ok(loaded_jobs) => {
        if loaded_jobs.is_empty() {
            // NO JOBS FOUND = SHOW DEMO DATA
            (create_dummy_jobs(), Some("No job files found, showing demo data"))
        } else {
            // JOBS FOUND = SHOW REAL DATA
            (loaded_jobs, None)
        }
    }
    Err(e) => {
        (create_dummy_jobs(), Some(format!("Error loading jobs: {}", e)))
    }
}
```

**The TUI shows demo data when the jobs directory is empty!**

---

## Why Is It Empty?

Jobs are stored in: `/var/lib/av1janitor/jobs/`

The daemon (`av1d`) creates job files when it:
1. Scans your configured directories
2. Finds media files that need transcoding
3. Creates a job JSON file for each

**If you don't have any jobs yet, you see demo data.**

---

## How To See Real Data

### Step 1: Make sure you're on Linux

```bash
# On your Linux system (not macOS)
cargo build --release --workspace
./build-deb.sh
sudo dpkg -i av1janitor_0.1.0_amd64.deb
```

### Step 2: Configure your media directories

```bash
sudo nano /etc/av1janitor/config.toml
```

Set your actual media directories:
```toml
watched_directories = [
    "/media/movies",
    "/media/tv",
    "/path/to/your/media"
]
```

### Step 3: Start the daemon

```bash
sudo systemctl start av1janitor
sudo systemctl status av1janitor
```

### Step 4: Wait for jobs to be created

The daemon will:
1. Scan your configured directories (takes a few seconds)
2. Find files that need transcoding
3. Create job files in `/var/lib/av1janitor/jobs/`

Check for jobs:
```bash
ls -la /var/lib/av1janitor/jobs/
```

### Step 5: Run av1top

```bash
av1top
```

**Now you'll see real data!**

- Real CPU/GPU/Memory/I/O stats from your system
- Real files being transcoded
- Real progress bars
- Real compression ratios

---

## Quick Test: Create a Test Job

If you want to test immediately without waiting for the daemon:

```bash
# Create a test job file
sudo mkdir -p /var/lib/av1janitor/jobs
sudo bash -c 'cat > /var/lib/av1janitor/jobs/test-job.json << EOF
{
  "source_path": "/media/test-movie.mkv",
  "status": "Pending",
  "created_at": "2025-11-17T21:00:00Z",
  "original_bytes": 5000000000,
  "reason": "Test"
}
EOF'

# Now run av1top
av1top
```

You should see the test job in the table!

---

## Check Daemon Status

```bash
# Is the daemon running?
sudo systemctl status av1janitor

# View daemon logs
sudo journalctl -u av1janitor -f

# Check for errors
sudo journalctl -u av1janitor -n 50
```

---

## Common Issues

### 1. "Exec format error"
**Cause**: Binaries were built on macOS, not Linux  
**Solution**: Build on your Linux system

### 2. "No job files found"
**Cause**: Daemon hasn't scanned yet OR no files need transcoding  
**Solution**: Wait a minute, check logs, verify config

### 3. Service won't start
**Cause**: Binary missing, config error, permissions  
**Solution**: Check logs with `journalctl -u av1janitor -n 50`

### 4. No files being transcoded
**Cause**: 
- Files are already AV1
- Files are below size threshold (2 GB default)
- Config directories are wrong
**Solution**: Check daemon logs, verify config

---

## Expected Behavior

### When daemon starts:
```
[av1d] Starting AV1 Janitor daemon...
[av1d] Scanning /media/movies...
[av1d] Found 3 files needing transcode
[av1d] Created job: /media/movies/big-movie.mkv
[av1d] Starting transcode...
```

### When av1top runs:
```
┌─ AV1 Transcoding Monitor ─┐
│ Queue: 2 │ Running: 1      │
│ CPU: 45% │ GPU: 78%        │
│ Current: big-movie.mkv     │
│ [████████░░] 82%           │
└────────────────────────────┘
```

---

## TLDR

**"Demo data" = No real jobs exist yet**

To see real data:
1. Build on Linux (not macOS)
2. Configure media directories
3. Start the daemon
4. Wait for scan
5. Run av1top

**Demo data is a FEATURE, not a bug!** It shows you what the TUI looks like before real jobs exist.

