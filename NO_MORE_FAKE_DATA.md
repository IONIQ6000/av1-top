# ✅ FAKE DATA REMOVED!

## What Changed

The TUI has been updated to **NEVER show fake/demo data**.

### Before:
- If no jobs existed → showed fake example jobs
- Confusing "demo data" message

### After:
- If no jobs exist → shows empty table with "Waiting for jobs..." message
- Always shows REAL system stats (CPU, GPU, Memory, I/O)
- No more confusion!

---

## What You'll See Now

### System Stats (Always Real)
- ✅ **CPU usage**: Real-time from your system
- ✅ **GPU usage**: Real Intel GPU stats
- ✅ **Memory**: Actual RAM usage
- ✅ **I/O**: Real disk read/write speeds

### Jobs Section
- **If jobs exist**: Shows real jobs with real data
- **If no jobs**: Shows empty table with "Waiting for jobs... (check daemon logs)"
- **Never**: Shows fake example data

---

## If You See "Waiting for jobs..."

This means the daemon hasn't created any job files yet. Check:

```bash
# 1. Is daemon running?
sudo systemctl status av1janitor

# 2. What's it doing?
sudo journalctl -u av1janitor -f

# 3. Are there job files?
ls -la /var/lib/av1janitor/jobs/

# 4. Run diagnostic
sudo bash QUICK_DEBUG.sh
```

---

## If GPU is Active But TUI Shows Nothing

If `intel_gpu_top` shows GPU usage but TUI shows no jobs:

**Problem**: The binaries are still macOS binaries, or the daemon isn't creating job files properly.

**Solution**:
```bash
# On your Linux system:
git pull
cargo build --release --workspace
./build-deb.sh
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo systemctl restart av1janitor

# Wait a few seconds, then:
av1top
```

---

## Expected Behavior

1. **Daemon starts** → Scans directories
2. **Finds files** → Creates job JSON files in `/var/lib/av1janitor/jobs/`
3. **TUI reads jobs** → Displays real job data
4. **GPU activates** → Shown in GPU panel and current transcode section

If the GPU is active but TUI shows no jobs, the daemon is running but the TUI isn't reading the job files properly.

**Most likely cause**: macOS binaries instead of Linux binaries.

---

## Rebuild on Linux

```bash
# This creates proper Linux binaries:
cargo build --release --workspace

# This packages them:
./build-deb.sh

# This installs them:
sudo dpkg -i av1janitor_0.1.0_amd64.deb

# This runs the TUI:
av1top
```

---

## Verify

After rebuild, check:

```bash
# Binary should be ELF format
file /usr/bin/av1top
# Should show: ELF 64-bit LSB executable...

# TUI should show real stats
av1top
# Should show real CPU/GPU/Memory/I/O
# Jobs table either has real jobs or says "Waiting for jobs..."
```

---

**No more fake data! Everything is real or empty!** ✅

