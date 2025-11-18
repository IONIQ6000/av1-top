#!/bin/bash
# Comprehensive script to remove ALL FFmpeg installations

set -e

echo "════════════════════════════════════════════════════════"
echo "  Removing ALL FFmpeg Installations"
echo "════════════════════════════════════════════════════════"
echo ""

# Remove binaries from /usr/local/bin
echo "[1/6] Removing binaries from /usr/local/bin..."
[ -f /usr/local/bin/ffmpeg ] && sudo rm -f /usr/local/bin/ffmpeg && echo "✓ Removed /usr/local/bin/ffmpeg"
[ -f /usr/local/bin/ffprobe ] && sudo rm -f /usr/local/bin/ffprobe && echo "✓ Removed /usr/local/bin/ffprobe"
[ -f /usr/local/bin/ffplay ] && sudo rm -f /usr/local/bin/ffplay && echo "✓ Removed /usr/local/bin/ffplay"

# Remove symlinks from /usr/bin
echo "[2/6] Removing symlinks from /usr/bin..."
[ -L /usr/bin/ffmpeg ] && sudo rm -f /usr/bin/ffmpeg && echo "✓ Removed /usr/bin/ffmpeg symlink"
[ -L /usr/bin/ffprobe ] && sudo rm -f /usr/bin/ffprobe && echo "✓ Removed /usr/bin/ffprobe symlink"
[ -L /usr/bin/ffplay ] && sudo rm -f /usr/bin/ffplay && echo "✓ Removed /usr/bin/ffplay symlink"

# Remove static build directories
echo "[3/6] Removing static build directories..."
sudo rm -rf /usr/local/ffmpeg-git-*-static 2>/dev/null && echo "✓ Removed old static build directories"
sudo rm -rf /usr/local/ffmpeg-n8.0-* 2>/dev/null && echo "✓ Removed FFmpeg 8.0 static build directories"

# Remove source build directories (if any)
echo "[4/6] Removing source build directories..."
sudo rm -rf /tmp/ffmpeg-8.0 2>/dev/null && echo "✓ Removed /tmp/ffmpeg-8.0"
sudo rm -rf ~/ffmpeg-8.0 2>/dev/null && echo "✓ Removed ~/ffmpeg-8.0"
sudo rm -rf ~/av1-top/ffmpeg-8.0 2>/dev/null && echo "✓ Removed ~/av1-top/ffmpeg-8.0"

# Remove downloaded source archives
echo "[5/6] Removing downloaded source archives..."
sudo rm -f ~/av1-top/ffmpeg-8.0.tar.xz 2>/dev/null && echo "✓ Removed ffmpeg-8.0.tar.xz"
sudo rm -f ~/av1-top/ffmpeg-static.tar.xz 2>/dev/null && echo "✓ Removed ffmpeg-static.tar.xz"
sudo rm -f /tmp/ffmpeg-8.0.tar.xz 2>/dev/null && echo "✓ Removed /tmp/ffmpeg-8.0.tar.xz"

# Check for any remaining FFmpeg installations
echo "[6/6] Checking for remaining installations..."
REMAINING=$(which ffmpeg 2>/dev/null || echo "")
if [ -n "$REMAINING" ]; then
    echo "⚠ FFmpeg still found at: $REMAINING"
    echo "Version: $(ffmpeg -version 2>&1 | head -1)"
    echo ""
    echo "This may be from a system package. To remove:"
    echo "  sudo apt-get remove --purge ffmpeg"
else
    echo "✓ No FFmpeg found in PATH"
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Cleanup Complete!"
echo "════════════════════════════════════════════════════════"
