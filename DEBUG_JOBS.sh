#!/bin/bash
# Debug script to check why TUI isn't showing jobs
# Run on Linux: sudo bash DEBUG_JOBS.sh

echo "════════════════════════════════════════════════════════"
echo "  AV1 JANITOR - JOB FILES DEBUG"
echo "════════════════════════════════════════════════════════"
echo ""

echo "─── 1. CHECK JOB DIRECTORY ───"
echo "Expected location: /var/lib/av1janitor/jobs/"
if [ -d /var/lib/av1janitor/jobs ]; then
    echo "✓ Directory exists"
    echo ""
    echo "Permissions:"
    ls -ld /var/lib/av1janitor/jobs/
    echo ""
    echo "Job files:"
    ls -lah /var/lib/av1janitor/jobs/
    echo ""
    JOB_COUNT=$(ls -1 /var/lib/av1janitor/jobs/*.json 2>/dev/null | wc -l)
    echo "Total job files: $JOB_COUNT"
    
    if [ $JOB_COUNT -gt 0 ]; then
        echo ""
        echo "─── SAMPLE JOB FILE CONTENT ───"
        ls /var/lib/av1janitor/jobs/*.json | head -1 | xargs cat | jq . 2>/dev/null || cat "$(ls /var/lib/av1janitor/jobs/*.json | head -1)"
    fi
else
    echo "✗ Directory doesn't exist!"
    echo "Creating it..."
    mkdir -p /var/lib/av1janitor/jobs
    chown av1janitor:av1janitor /var/lib/av1janitor/jobs
    echo "✓ Created"
fi
echo ""

echo "─── 2. CHECK DAEMON STATUS ───"
systemctl status av1janitor --no-pager | head -10
echo ""

echo "─── 3. CHECK DAEMON LOGS (last 20 lines) ───"
journalctl -u av1janitor -n 20 --no-pager | grep -E "(Saving job|Job created|Transcoding|saved to)"
echo ""

echo "─── 4. CHECK GPU ACTIVITY ───"
if command -v intel_gpu_top &> /dev/null; then
    echo "Running intel_gpu_top for 2 seconds..."
    timeout 2 intel_gpu_top -J 2>&1 | head -20 || echo "GPU monitoring not available"
else
    echo "intel_gpu_top not installed"
fi
echo ""

echo "─── 5. CHECK DAEMON CONFIG ───"
if [ -f /etc/av1janitor/config.toml ]; then
    echo "Config file exists:"
    cat /etc/av1janitor/config.toml
else
    echo "✗ Config file not found!"
fi
echo ""

echo "─── 6. MANUAL JOB CHECK ───"
echo "Testing if daemon can create job files..."
echo "Run manually: sudo -u av1janitor av1d --once --dry-run -vv"
echo ""

echo "════════════════════════════════════════════════════════"
echo "  DIAGNOSIS"
echo "════════════════════════════════════════════════════════"
echo ""

if [ ! -d /var/lib/av1janitor/jobs ]; then
    echo "⚠️  Job directory doesn't exist - daemon can't save jobs!"
    echo "   FIX: mkdir -p /var/lib/av1janitor/jobs && chown av1janitor:av1janitor /var/lib/av1janitor/jobs"
elif [ $JOB_COUNT -eq 0 ]; then
    echo "⚠️  Job directory exists but no job files!"
    echo "   Possible causes:"
    echo "   - Daemon not saving jobs to disk"
    echo "   - Daemon configured with different job directory"
    echo "   - Permission issues"
    echo ""
    echo "   Check daemon logs: sudo journalctl -u av1janitor -f"
    echo "   Test manually: sudo -u av1janitor av1d --once --dry-run -vv"
else
    echo "✓ Job files exist!"
    echo "   If TUI still doesn't show them, the TUI binary may be wrong format."
    echo "   Rebuild on Linux: cargo build --release --bin av1top"
fi
echo ""

