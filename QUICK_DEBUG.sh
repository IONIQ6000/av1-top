#!/bin/bash
# Quick diagnostic script for AV1 Janitor
# Run on your Linux system: sudo bash QUICK_DEBUG.sh

echo "╔════════════════════════════════════════════════════════════╗"
echo "║        AV1 JANITOR - QUICK DIAGNOSTIC                      ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

echo "━━━ 1. DAEMON STATUS ━━━"
systemctl is-active av1janitor && echo "✓ Daemon is running" || echo "✗ Daemon is NOT running"
systemctl status av1janitor --no-pager | head -5
echo ""

echo "━━━ 2. BINARIES CHECK ━━━"
if [ -f /usr/bin/av1d ]; then
    file /usr/bin/av1d | grep -q "ELF" && echo "✓ av1d is Linux binary" || echo "✗ av1d is NOT Linux binary"
else
    echo "✗ av1d not found"
fi

if [ -f /usr/bin/av1top ]; then
    file /usr/bin/av1top | grep -q "ELF" && echo "✓ av1top is Linux binary" || echo "✗ av1top is NOT Linux binary"
else
    echo "✗ av1top not found"
fi
echo ""

echo "━━━ 3. JOB FILES ━━━"
if [ -d /var/lib/av1janitor/jobs ]; then
    JOB_COUNT=$(ls -1 /var/lib/av1janitor/jobs/*.json 2>/dev/null | wc -l)
    echo "Job files found: $JOB_COUNT"
    if [ $JOB_COUNT -gt 0 ]; then
        echo "✓ Jobs exist - TUI should show real data"
        ls -lh /var/lib/av1janitor/jobs/*.json | head -3
    else
        echo "✗ No job files - TUI will show demo data"
    fi
else
    echo "✗ Job directory doesn't exist"
fi
echo ""

echo "━━━ 4. CONFIGURATION ━━━"
if [ -f /etc/av1janitor/config.toml ]; then
    echo "Config file exists:"
    grep "watched_directories" /etc/av1janitor/config.toml || echo "No watched_directories found"
else
    echo "✗ Config file not found"
fi
echo ""

echo "━━━ 5. RECENT LOGS (last 15 lines) ━━━"
journalctl -u av1janitor -n 15 --no-pager 2>/dev/null || echo "No logs available"
echo ""

echo "━━━ 6. MEDIA FILES CHECK ━━━"
WATCHED_DIR=$(grep "watched_directories" /etc/av1janitor/config.toml 2>/dev/null | head -1 | cut -d'"' -f2)
if [ -n "$WATCHED_DIR" ]; then
    echo "Checking: $WATCHED_DIR"
    MEDIA_COUNT=$(find "$WATCHED_DIR" -type f -size +2G \( -name "*.mkv" -o -name "*.mp4" -o -name "*.avi" \) 2>/dev/null | wc -l)
    echo "Large media files (>2GB) found: $MEDIA_COUNT"
    if [ $MEDIA_COUNT -gt 0 ]; then
        echo "Sample files:"
        find "$WATCHED_DIR" -type f -size +2G \( -name "*.mkv" -o -name "*.mp4" -o -name "*.avi" \) 2>/dev/null | head -3
    fi
else
    echo "Could not determine watched directory"
fi
echo ""

echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    QUICK FIXES                             ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Determine what to suggest
if ! systemctl is-active av1janitor >/dev/null 2>&1; then
    echo "⚠️  ISSUE: Daemon is not running"
    echo "   FIX: sudo systemctl start av1janitor"
    echo ""
fi

if [ ! -f /usr/bin/av1d ] || [ ! -f /usr/bin/av1top ]; then
    echo "⚠️  ISSUE: Binaries not installed"
    echo "   FIX: sudo dpkg -i av1janitor_0.1.0_amd64.deb"
    echo ""
fi

if [ -f /usr/bin/av1top ] && ! file /usr/bin/av1top | grep -q "ELF"; then
    echo "⚠️  ISSUE: Binaries are macOS format, not Linux"
    echo "   FIX: Build on Linux:"
    echo "        cargo build --release --workspace"
    echo "        ./build-deb.sh"
    echo "        sudo dpkg -i av1janitor_0.1.0_amd64.deb"
    echo ""
fi

if [ -d /var/lib/av1janitor/jobs ]; then
    JOB_COUNT=$(ls -1 /var/lib/av1janitor/jobs/*.json 2>/dev/null | wc -l)
    if [ $JOB_COUNT -eq 0 ]; then
        echo "⚠️  ISSUE: No job files created yet"
        echo "   This is normal if daemon just started"
        echo "   Wait 30 seconds, then run: av1top"
        echo "   OR check logs: sudo journalctl -u av1janitor -f"
        echo ""
    fi
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "To monitor in real-time: sudo journalctl -u av1janitor -f"
echo "To view TUI: av1top"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

