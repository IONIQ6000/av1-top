# ⚠️ IMPORTANT: Reinstall with Latest Package

## The TUI IS Fully Implemented!

If you're seeing what looks like a "demo GUI," you may have installed an older version of the package. The TUI has been fully implemented with all features, but you need to reinstall to get the latest version.

---

## 🔄 Reinstall Steps

### On Your Linux System:

```bash
# 1. Stop the service (if running)
sudo systemctl stop av1janitor

# 2. Remove old package
sudo dpkg -r av1janitor

# 3. Re-download the latest .deb file from your Mac
# (Transfer the file from your Mac to your Linux system)

# 4. Install the latest package
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f

# 5. Verify the new binary
ls -lh /usr/local/bin/av1top
# Should show: 1.3M (the new version)

# 6. Test the TUI
av1top
# You should see:
# - Header with queue summary
# - 4 stat panels (CPU, GPU, Memory, I/O)
# - Current transcode section
# - Jobs history table
# - Footer with controls

# 7. Start the daemon
sudo systemctl start av1janitor

# 8. Monitor in real-time
av1top
```

---

## 🎯 What You Should See

After reinstalling, `av1top` will show:

### **5 Main Sections:**

1. **Header**
   - "AV1 Janitor Transcoding Monitor"
   - Queue: X | Running: X | ✓ X | ✗ X | ⊘ X

2. **System Stats Grid** (4 panels)
   - **CPU**: Usage % with gauge bar
   - **GPU**: Usage %, VRAM, encoder status
   - **Memory**: Used/Total with gauge
   - **I/O**: Read/Write MB/s

3. **Current Transcode**
   - Filename
   - Original size | Current size | Ratio
   - Progress bar
   - Duration | Status: TRANSCODING

4. **Jobs History Table**
   - Columns: STATUS | FILE | ORIGINAL | OUTPUT | SAVINGS | RATIO | DURATION | REASON
   - Color-coded rows
   - Scrollable

5. **Footer**
   - Controls: [q] Quit [r] Refresh
   - Job directory path
   - Status messages

---

## 📝 "Demo Data" vs "Demo GUI"

### ❌ You DON'T have a "demo GUI"
The TUI is **fully implemented** with all features!

### ✅ You DO see "demo data" initially
This is **normal** when:
- The daemon hasn't started yet
- No files have been transcoded yet
- No job files exist in `/var/lib/av1janitor/jobs/`

**Solution**: Just start the daemon!
```bash
sudo systemctl start av1janitor
```

Within seconds, you'll see:
- Real system stats (CPU, GPU, Memory, I/O)
- Real files being processed
- Live transcode progress
- Actual compression ratios

---

## 🔍 Verify Installation

### Check Binary Size:
```bash
ls -lh /usr/local/bin/av1top
```
**Expected**: ~1.3 MB

If it's much smaller, you have an old version.

### Check Binary Date:
```bash
ls -l /usr/local/bin/av1top
```
**Expected**: Today's date

### Run It:
```bash
av1top
```
**Expected**: Full TUI with 5 sections (not just a simple screen)

---

## 💡 Quick Test

To quickly verify the TUI is fully implemented:

```bash
# Run av1top
av1top

# Look for these keywords on screen:
# - "Queue:"
# - "CPU" panel
# - "GPU" panel
# - "Memory" panel
# - "I/O" panel
# - "Current Transcode"
# - Table with columns: STATUS | FILE | ORIGINAL | OUTPUT | SAVINGS
# - "[q] Quit  [r] Refresh"
```

If you see all of these, **the TUI is fully implemented!** 🎉

---

## 🚀 To See It In Action

```bash
# 1. Configure your media directories
sudo nano /etc/av1janitor/config.toml
# Set: watched_directories = ["/media/movies", "/media/tv"]

# 2. Start the daemon
sudo systemctl start av1janitor
sudo systemctl enable av1janitor  # Auto-start on boot

# 3. Monitor in real-time
av1top

# 4. Watch as:
# - Daemon scans your directories
# - Jobs appear in the queue
# - GPU starts working
# - I/O shows read/write activity
# - Files get transcoded
# - Compression ratios are displayed
# - Savings accumulate
```

---

## 📦 Latest Package Info

- **File**: `av1janitor_0.1.0_amd64.deb`
- **Size**: 2.0 MB
- **av1d**: 3.5 MB (daemon)
- **av1top**: 1.3 MB (TUI with **FULL FEATURES**)
- **Status**: ✅ Ready to install

---

## ❓ Still Seeing Issues?

If after reinstalling you still see a simple/demo GUI:

1. **Check binary version:**
   ```bash
   /usr/local/bin/av1top --version 2>&1 || echo "No version flag"
   md5sum /usr/local/bin/av1top
   ```

2. **Check if old binary is cached:**
   ```bash
   which av1top
   # Should be: /usr/local/bin/av1top
   ```

3. **Force clean install:**
   ```bash
   sudo dpkg -P av1janitor  # Purge
   sudo rm -f /usr/local/bin/av1top /usr/local/bin/av1d
   sudo dpkg -i av1janitor_0.1.0_amd64.deb
   ```

---

**The TUI is fully implemented! Reinstall to get the latest version!** ✅

