# GPU is Transcoding But TUI Shows Nothing

## Problem

You can see transcoding activity in `intel_gpu_top`, but the TUI shows:
- Empty "Current Transcode" section
- Empty "Transcode History" table
- Footer says "Waiting for jobs..."

## Root Cause

**The daemon is transcoding BUT not saving job state to disk**, OR the TUI can't read the job files.

---

## Quick Fix

### On Your Linux System:

```bash
# 1. Pull latest code
git pull

# 2. Run debug script
sudo bash DEBUG_JOBS.sh

# This will tell you:
# - If job directory exists
# - If job files exist
# - What the daemon is doing
# - Why TUI can't see jobs
```

---

## Common Causes & Fixes

### Cause 1: Job Directory Doesn't Exist

**Check:**
```bash
ls -la /var/lib/av1janitor/jobs/
```

**Fix:**
```bash
sudo mkdir -p /var/lib/av1janitor/jobs
sudo chown av1janitor:av1janitor /var/lib/av1janitor/jobs
sudo systemctl restart av1janitor
```

### Cause 2: Daemon Not Saving Job Files

The daemon might be transcoding without creating job state files.

**Check daemon logs:**
```bash
sudo journalctl -u av1janitor -f
```

Look for lines like:
- "Saving job state to..."
- "Job created: ..."
- "Saved job to /var/lib/av1janitor/jobs/..."

**If you DON'T see these**, the daemon isn't saving jobs.

**Fix:** Restart the daemon:
```bash
sudo systemctl restart av1janitor
sudo journalctl -u av1janitor -f
```

### Cause 3: Wrong Binary Format (macOS binary)

The TUI binary might be macOS format and can't properly read files on Linux.

**Check:**
```bash
file /usr/bin/av1top
```

Should show: `ELF 64-bit LSB executable...`

If it shows anything else (or error), **rebuild on Linux**:
```bash
cargo build --release --workspace
./build-deb.sh
sudo dpkg -i av1janitor_0.1.0_amd64.deb
```

### Cause 4: Permission Issues

The TUI might not have permission to read job files.

**Check:**
```bash
ls -la /var/lib/av1janitor/jobs/
sudo -u $USER cat /var/lib/av1janitor/jobs/*.json 2>&1 | head -5
```

**Fix:**
```bash
sudo chmod 755 /var/lib/av1janitor
sudo chmod 755 /var/lib/av1janitor/jobs
sudo chmod 644 /var/lib/av1janitor/jobs/*.json
```

### Cause 5: Daemon Using Different Directory

The daemon might be configured to save jobs elsewhere.

**Check config:**
```bash
cat /etc/av1janitor/config.toml
```

**Check daemon command:**
```bash
systemctl cat av1janitor | grep ExecStart
```

If the daemon uses a custom `--config` or different paths, that's the issue.

---

## Manual Test

Test if the daemon can create job files:

```bash
# Run daemon manually in dry-run mode
sudo -u av1janitor av1d --once --dry-run -vv 2>&1 | tee /tmp/daemon-test.log

# Check if it mentions job files
grep -i "job" /tmp/daemon-test.log
grep -i "saving" /tmp/daemon-test.log
```

---

## Expected Behavior

When working correctly:

1. **Daemon scans** → Finds files needing transcode
2. **Creates job** → Saves JSON to `/var/lib/av1janitor/jobs/<hash>.json`
3. **Starts transcoding** → Updates job state with "Running"
4. **During transcode** → Periodically updates job state (progress, size, etc.)
5. **TUI reads jobs** → Every 2 seconds, loads all JSON files from jobs directory
6. **TUI displays** → Shows jobs in "Current Transcode" and "History" table

If GPU is active but TUI shows nothing, step 2 or 5 is failing.

---

## Debug Commands

```bash
# 1. Check if job files exist
ls -la /var/lib/av1janitor/jobs/

# 2. Count job files
ls -1 /var/lib/av1janitor/jobs/*.json 2>/dev/null | wc -l

# 3. View a job file
cat /var/lib/av1janitor/jobs/*.json | head -50

# 4. Watch daemon logs in real-time
sudo journalctl -u av1janitor -f

# 5. Check daemon is actually running
ps aux | grep av1d

# 6. Test TUI can read directory
strace -e openat av1top 2>&1 | grep "jobs"
```

---

## Most Likely Issue

Based on your symptoms (GPU active, TUI shows nothing), the daemon is probably:

1. **Running FFmpeg directly** without creating job state files, OR
2. **Creating job files** but in the wrong location, OR
3. **The binaries are macOS binaries** and can't properly interact with the filesystem

**Solution**: Rebuild everything on Linux:

```bash
# Full rebuild
git pull
cargo clean
cargo build --release --workspace
./build-deb.sh
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo systemctl restart av1janitor

# Wait 10 seconds
sleep 10

# Check for jobs
ls -la /var/lib/av1janitor/jobs/

# Run TUI
av1top
```

---

## If Still Not Working

Run the diagnostic:
```bash
sudo bash DEBUG_JOBS.sh > debug-output.txt 2>&1
cat debug-output.txt
```

And check:
- Are job files being created?
- What do daemon logs say?
- Is the binary the right format?

**The issue is that job state isn't being persisted to disk or the TUI can't read it.**

