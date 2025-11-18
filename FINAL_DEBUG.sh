#!/bin/bash
# Final comprehensive debug script

echo "════════════════════════════════════════════════════════"
echo "  COMPREHENSIVE DEBUG"
echo "════════════════════════════════════════════════════════"
echo ""

echo "─── 1. BINARY CHECK ───"
echo "Checking if binaries are Linux format:"
file /usr/bin/av1d /usr/bin/av1top
echo ""

echo "─── 2. DAEMON STATUS ───"
systemctl is-active av1janitor && echo "✓ Running" || echo "✗ Not running"
systemctl status av1janitor --no-pager | head -10
echo ""

echo "─── 3. DAEMON LOGS - JOBS DIRECTORY ───"
echo "Looking for 'Jobs directory' in logs:"
journalctl -u av1janitor --no-pager | grep -i "jobs directory" | tail -3
echo ""

echo "─── 4. DAEMON LOGS - SAVING JOBS ───"
echo "Looking for 'Saving job' or 'saved' in logs:"
journalctl -u av1janitor --no-pager | grep -i "saving\|saved" | tail -10
echo ""

echo "─── 5. JOB DIRECTORY CHECK ───"
JOBS_DIR="/var/lib/av1janitor/jobs"
echo "Checking: $JOBS_DIR"
if [ -d "$JOBS_DIR" ]; then
    echo "✓ Directory exists"
    ls -ld "$JOBS_DIR"
    echo ""
    echo "Permissions:"
    stat -c "%a %U:%G %n" "$JOBS_DIR"
    echo ""
    echo "Job files:"
    ls -lah "$JOBS_DIR" 2>/dev/null || echo "Empty"
    JOB_COUNT=$(ls -1 "$JOBS_DIR"/*.json 2>/dev/null | wc -l)
    echo "Total: $JOB_COUNT job files"
    
    if [ $JOB_COUNT -gt 0 ]; then
        echo ""
        echo "Sample job file:"
        cat "$(ls "$JOBS_DIR"/*.json | head -1)" | head -30
    fi
else
    echo "✗ Directory doesn't exist!"
fi
echo ""

echo "─── 6. WRITE PERMISSION TEST ───"
echo "Testing if av1janitor user can write:"
sudo -u av1janitor touch "$JOBS_DIR/test_write.txt" 2>&1
if [ -f "$JOBS_DIR/test_write.txt" ]; then
    echo "✓ Can write to directory"
    sudo -u av1janitor rm -f "$JOBS_DIR/test_write.txt"
else
    echo "✗ Cannot write to directory!"
    echo "Fixing permissions..."
    sudo chown -R av1janitor:av1janitor "$JOBS_DIR"
    sudo chmod 755 "$JOBS_DIR"
fi
echo ""

echo "─── 7. DAEMON PROCESS CHECK ───"
if pgrep -f av1d > /dev/null; then
    echo "✓ Daemon process running"
    ps aux | grep av1d | grep -v grep
    echo ""
    echo "Process user:"
    ps aux | grep av1d | grep -v grep | awk '{print $1}'
else
    echo "✗ Daemon process not found"
fi
echo ""

echo "─── 8. RECENT DAEMON ACTIVITY ───"
echo "Last 20 log lines:"
journalctl -u av1janitor -n 20 --no-pager
echo ""

echo "─── 9. TUI TEST ───"
echo "Testing if TUI can read jobs directory:"
if [ -d "$JOBS_DIR" ]; then
    ls -la "$JOBS_DIR"/*.json 2>/dev/null | head -3 || echo "No JSON files found"
else
    echo "Directory doesn't exist for TUI to read"
fi
echo ""

echo "════════════════════════════════════════════════════════"
echo "  DIAGNOSIS"
echo "════════════════════════════════════════════════════════"
echo ""

# Check if binaries are Linux
if ! file /usr/bin/av1d | grep -q "ELF"; then
    echo "⚠️  ISSUE: Binaries are NOT Linux format!"
    echo "   FIX: Rebuild on Linux:"
    echo "        cargo build --release --workspace"
    echo "        sudo cp target/release/av1d /usr/bin/av1d"
    echo "        sudo cp target/release/av1top /usr/bin/av1top"
    echo ""
fi

# Check if daemon is running
if ! systemctl is-active av1janitor >/dev/null 2>&1; then
    echo "⚠️  ISSUE: Daemon is not running!"
    echo "   FIX: sudo systemctl start av1janitor"
    echo ""
fi

# Check if jobs directory exists
if [ ! -d "$JOBS_DIR" ]; then
    echo "⚠️  ISSUE: Jobs directory doesn't exist!"
    echo "   FIX: sudo mkdir -p $JOBS_DIR"
    echo "        sudo chown av1janitor:av1janitor $JOBS_DIR"
    echo ""
fi

# Check if job files exist
JOB_COUNT=$(ls -1 "$JOBS_DIR"/*.json 2>/dev/null | wc -l)
if [ $JOB_COUNT -eq 0 ]; then
    echo "⚠️  ISSUE: No job files found!"
    echo ""
    echo "   Possible causes:"
    echo "   1. Daemon not saving jobs (check logs for errors)"
    echo "   2. Daemon using different directory"
    echo "   3. Permission issues"
    echo ""
    echo "   Check daemon logs:"
    echo "   sudo journalctl -u av1janitor -f"
    echo ""
else
    echo "✓ Job files exist ($JOB_COUNT files)"
    echo ""
    echo "If TUI still doesn't show them:"
    echo "  1. Rebuild TUI: cargo build --release --bin av1top"
    echo "  2. Copy: sudo cp target/release/av1top /usr/bin/av1top"
    echo "  3. Run: av1top"
fi

echo ""
echo "════════════════════════════════════════════════════════"

