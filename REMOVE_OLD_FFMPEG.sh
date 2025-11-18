#!/bin/bash
# Quick script to remove old FFmpeg installations

echo "Removing old FFmpeg installations..."

# Remove binaries
[ -f /usr/local/bin/ffmpeg ] && sudo rm -f /usr/local/bin/ffmpeg && echo "✓ Removed /usr/local/bin/ffmpeg"
[ -f /usr/local/bin/ffprobe ] && sudo rm -f /usr/local/bin/ffprobe && echo "✓ Removed /usr/local/bin/ffprobe"

# Remove symlinks
[ -L /usr/bin/ffmpeg ] && sudo rm -f /usr/bin/ffmpeg && echo "✓ Removed /usr/bin/ffmpeg symlink"
[ -L /usr/bin/ffprobe ] && sudo rm -f /usr/bin/ffprobe && echo "✓ Removed /usr/bin/ffprobe symlink"

# Remove static build directories
sudo rm -rf /usr/local/ffmpeg-git-*-static 2>/dev/null && echo "✓ Removed old static build directories"

# Check if anything remains
if command -v ffmpeg &> /dev/null; then
    echo ""
    echo "⚠ FFmpeg still found at: $(which ffmpeg)"
    echo "Version: $(ffmpeg -version 2>&1 | head -1)"
else
    echo ""
    echo "✓ All old FFmpeg installations removed"
fi

