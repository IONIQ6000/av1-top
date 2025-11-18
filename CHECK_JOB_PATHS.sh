#!/bin/bash
# Check where jobs are being saved vs where TUI is looking

echo "════════════════════════════════════════════════════════"
echo "  JOB PATHS DEBUG"
echo "════════════════════════════════════════════════════════"
echo ""

echo "─── 1. WHERE DAEMON SAVES JOBS ───"
echo "Expected: /var/lib/av1janitor/jobs/"
echo "Checking..."
ls -la /var/lib/av1janitor/jobs/ 2>/dev/null || echo "✗ Directory doesn't exist"
echo ""

echo "─── 2. JOB FILES COUNT ───"
JOB_COUNT=$(ls -1 /var/lib/av1janitor/jobs/*.json 2>/dev/null | wc -l)
echo "Job files found: $JOB_COUNT"
if [ $JOB_COUNT -gt 0 ]; then
    echo "Sample files:"
    ls -lh /var/lib/av1janitor/jobs/*.json | head -5
    echo ""
    echo "Sample job content:"
    cat "$(ls /var/lib/av1janitor/jobs/*.json | head -1)" | head -20
else
    echo "✗ No job files found!"
fi
echo ""

echo "─── 3. WHERE TUI LOOKS FOR JOBS ───"
echo "TUI uses PathsConfig::default() which uses:"
echo "  ~/.local/share/av1janitor/jobs (for current user)"
echo ""
echo "But daemon might be using:"
echo "  /var/lib/av1janitor/jobs"
echo ""
echo "Checking TUI default path:"
TUI_USER=$(whoami)
TUI_HOME=$(eval echo ~$TUI_USER)
TUI_JOBS_DIR="$TUI_HOME/.local/share/av1janitor/jobs"
echo "TUI would look in: $TUI_JOBS_DIR"
if [ -d "$TUI_JOBS_DIR" ]; then
    echo "✓ Directory exists"
    ls -la "$TUI_JOBS_DIR" 2>/dev/null || echo "Empty"
else
    echo "✗ Directory doesn't exist"
fi
echo ""

echo "─── 4. DAEMON PROCESS CHECK ───"
if pgrep -f av1d > /dev/null; then
    echo "✓ Daemon is running"
    ps aux | grep av1d | grep -v grep
else
    echo "✗ Daemon is not running"
fi
echo ""

echo "─── 5. DAEMON LOGS (job saving) ───"
echo "Looking for 'Saving job' or 'saved to' in logs:"
journalctl -u av1janitor -n 50 --no-pager | grep -i "saving\|saved\|job" | tail -10 || echo "No job-related log entries"
echo ""

echo "─── 6. CONFIG FILE CHECK ───"
if [ -f /etc/av1janitor/config.toml ]; then
    echo "Config exists:"
    cat /etc/av1janitor/config.toml
else
    echo "✗ Config file not found"
fi
echo ""

echo "════════════════════════════════════════════════════════"
echo "  DIAGNOSIS"
echo "════════════════════════════════════════════════════════"
echo ""

if [ $JOB_COUNT -eq 0 ]; then
    echo "⚠️  PROBLEM: No job files found!"
    echo ""
    echo "Possible causes:"
    echo "  1. Daemon not saving jobs (check logs)"
    echo "  2. Daemon using different directory"
    echo "  3. Permission issues"
    echo ""
    echo "Check daemon logs:"
    echo "  sudo journalctl -u av1janitor -f | grep -i job"
    echo ""
    echo "Test daemon manually:"
    echo "  sudo -u av1janitor /usr/bin/av1d --once --dry-run -vv"
else
    echo "✓ Job files exist in /var/lib/av1janitor/jobs/"
    echo ""
    echo "If TUI still doesn't show them:"
    echo "  - TUI might be looking in wrong directory"
    echo "  - TUI binary might be wrong format"
    echo "  - Rebuild TUI on Linux: cargo build --release --bin av1top"
fi

echo ""
echo "─── QUICK FIX ───"
echo "If jobs are in /var/lib/av1janitor/jobs/ but TUI can't see them:"
echo "  Run TUI with explicit path:"
echo "  AV1JANITOR_JOBS_DIR=/var/lib/av1janitor/jobs av1top"
echo ""
echo "Or create symlink:"
echo "  mkdir -p ~/.local/share/av1janitor"
echo "  ln -s /var/lib/av1janitor/jobs ~/.local/share/av1janitor/jobs"

