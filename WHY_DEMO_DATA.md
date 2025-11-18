# Why the TUI Shows "Demo Data" - Quick Troubleshooting

## TL;DR

The TUI shows "demo data" when **no job files exist in `/var/lib/av1janitor/jobs/`**. This is NORMAL if:
- The daemon just started
- No files meet the transcoding criteria yet
- The daemon is scanning your directories

**The TUI is NOT a demo - it's the full production TUI!** It just needs real jobs from the daemon.

---

## Quick Checklist

Run these commands on your Linux system:

```bash
# 1. Check if daemon is running
sudo systemctl status av1janitor

# 2. Check for job files
ls -la /var/lib/av1janitor/jobs/

# 3. Check daemon logs
sudo journalctl -u av1janitor -n 50

# 4. Test TUI
av1top
```

---

## Scenario 1: Daemon Not Running

**Symptoms:**
- TUI shows "demo data"
- No job files exist
- `systemctl status av1janitor` shows "inactive (dead)"

**Solution:**
```bash
# Check config first
sudo nano /etc/av1janitor/config.toml
# Make sure watched_directories points to your media files

# Start daemon
sudo systemctl start av1janitor
sudo systemctl enable av1janitor

# Wait 10 seconds, then check
sleep 10
av1top
```

---

## Scenario 2: No Files Meet Criteria

**Symptoms:**
- Daemon is running
- No job files created
- Logs show "Scanning complete, 0 files found"

**Why:** Files must meet these criteria to be transcoded:
- Size ≥ 2 GB
- Extension: .mkv, .mp4, .avi
- Not already AV1
- Not WebRip/WEBRip (skipped by default)

**Solution:**
```bash
# Check your media files
find /media -type f -size +2G \( -name "*.mkv" -o -name "*.mp4" -o -name "*.avi" \) | head -5

# Check daemon logs to see what it found
sudo journalctl -u av1janitor -f
```

If you don't have files that meet the criteria, you can:
1. Lower the size threshold in config
2. Add more file extensions
3. Wait for the daemon to scan

---

## Scenario 3: Daemon Can't Access Media

**Symptoms:**
- Daemon is running
- Logs show permission errors

**Solution:**
```bash
# Check permissions
sudo -u av1janitor ls -la /media/movies

# If permission denied, add av1janitor to proper group
sudo usermod -a -G your-media-group av1janitor
sudo systemctl restart av1janitor
```

---

## Scenario 4: Wrong Binaries (macOS built)

**Symptoms:**
- `av1top` shows error: "cannot execute binary file"
- OR av1top runs but shows demo data forever

**Solution:**
Build on Linux:
```bash
# On your Linux system
cargo build --release --workspace
./build-deb.sh
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo systemctl restart av1janitor
av1top
```

---

## Expected Behavior

### When First Started (Normal)
1. Daemon starts
2. Scans directories
3. Finds files that meet criteria
4. Creates job JSON files in `/var/lib/av1janitor/jobs/`
5. TUI reads job files and displays them
6. "demo data" message disappears

### Timeline:
- **0-5 seconds:** TUI shows "demo data" (no jobs yet)
- **5-30 seconds:** Daemon finishes scanning, creates jobs
- **30+ seconds:** TUI shows real jobs, real system stats

---

## Verify Real Data is Showing

When the TUI has real data, you'll see:
- ✅ Real CPU/GPU/Memory/I/O stats (not 0%)
- ✅ Actual file paths in the jobs table
- ✅ Real file sizes
- ✅ No "showing demo data" message in footer
- ✅ Queue counts change as jobs run

---

## Debug Commands

```bash
# 1. Check daemon status
sudo systemctl status av1janitor

# 2. View real-time logs
sudo journalctl -u av1janitor -f

# 3. Check job directory
ls -la /var/lib/av1janitor/jobs/
cat /var/lib/av1janitor/jobs/*.json 2>/dev/null | head -20

# 4. Test daemon manually
sudo -u av1janitor av1d --once --dry-run -vv

# 5. Check config
cat /etc/av1janitor/config.toml
```

---

## If Still Showing Demo Data After 1 Minute

Run this diagnostic:

```bash
echo "=== AV1 Janitor Diagnostic ==="
echo ""
echo "1. Daemon status:"
sudo systemctl status av1janitor --no-pager | head -10
echo ""
echo "2. Recent logs:"
sudo journalctl -u av1janitor -n 20 --no-pager
echo ""
echo "3. Job files:"
ls -la /var/lib/av1janitor/jobs/ 2>&1 || echo "Directory doesn't exist"
echo ""
echo "4. Config:"
cat /etc/av1janitor/config.toml 2>&1 || echo "Config not found"
echo ""
echo "5. Binary check:"
file /usr/bin/av1d /usr/bin/av1top
echo ""
echo "6. Media files check:"
find /media -type f -size +2G \( -name "*.mkv" -o -name "*.mp4" -o -name "*.avi" \) 2>/dev/null | head -3 || echo "No large media files found"
```

---

## Summary

**The TUI is NOT a demo!** It shows:
- ✅ Real system stats (CPU, GPU, Memory, I/O)
- ✅ Real jobs when they exist
- ✅ Demo placeholder data ONLY when no jobs exist yet

**To get real data:**
1. Make sure daemon is running
2. Make sure config points to your media
3. Wait for daemon to scan and create jobs
4. TUI will automatically show real data

**Check:** `sudo journalctl -u av1janitor -f` to see what the daemon is doing!
