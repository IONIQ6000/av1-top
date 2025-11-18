#!/bin/bash
#
# Remove ALL FFmpeg installations (static builds, source builds, packages)
# Use this to clean up before installing a fresh FFmpeg 8.0 static build
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Removing ALL FFmpeg installations...${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}ERROR: This script must be run as root${NC}"
    echo "Usage: sudo bash remove_all_ffmpeg.sh"
    exit 1
fi

REMOVED=0

# Remove binaries from /usr/local/bin
if [ -f /usr/local/bin/ffmpeg ]; then
    rm -f /usr/local/bin/ffmpeg
    echo -e "${GREEN}✓ Removed /usr/local/bin/ffmpeg${NC}"
    REMOVED=1
fi

if [ -f /usr/local/bin/ffprobe ]; then
    rm -f /usr/local/bin/ffprobe
    echo -e "${GREEN}✓ Removed /usr/local/bin/ffprobe${NC}"
    REMOVED=1
fi

# Remove symlinks from /usr/bin
if [ -L /usr/bin/ffmpeg ]; then
    rm -f /usr/bin/ffmpeg
    echo -e "${GREEN}✓ Removed /usr/bin/ffmpeg symlink${NC}"
    REMOVED=1
fi

if [ -L /usr/bin/ffprobe ]; then
    rm -f /usr/bin/ffprobe
    echo -e "${GREEN}✓ Removed /usr/bin/ffprobe symlink${NC}"
    REMOVED=1
fi

# Remove static build directories
for dir in /usr/local/ffmpeg-*-static /usr/local/ffmpeg-git-*-static; do
    if [ -d "$dir" ]; then
        rm -rf "$dir"
        echo -e "${GREEN}✓ Removed $dir${NC}"
        REMOVED=1
    fi
done

# Remove source build directories
if [ -d /usr/local/src/ffmpeg-8.0 ]; then
    rm -rf /usr/local/src/ffmpeg-8.0
    echo -e "${GREEN}✓ Removed /usr/local/src/ffmpeg-8.0${NC}"
    REMOVED=1
fi

# Remove package-installed FFmpeg (if any)
if dpkg -l | grep -q "^ii.*ffmpeg"; then
    echo -e "${YELLOW}Removing package-installed FFmpeg...${NC}"
    apt-get remove -y --purge ffmpeg 2>/dev/null || true
    echo -e "${GREEN}✓ Removed package FFmpeg${NC}"
    REMOVED=1
fi

# Clean up any temporary build directories
if [ -d /tmp/ffmpeg-8.0 ]; then
    rm -rf /tmp/ffmpeg-8.0
    echo -e "${GREEN}✓ Removed /tmp/ffmpeg-8.0${NC}"
    REMOVED=1
fi

# Clean up any downloaded source files in common locations
for file in /tmp/ffmpeg-8.0.tar.xz /tmp/ffmpeg-static.tar.xz ~/ffmpeg-8.0.tar.xz; do
    if [ -f "$file" ]; then
        rm -f "$file"
        echo -e "${GREEN}✓ Removed $file${NC}"
        REMOVED=1
    fi
done

if [ $REMOVED -eq 0 ]; then
    echo -e "${YELLOW}No FFmpeg installations found to remove${NC}"
else
    echo ""
    echo -e "${GREEN}✓ Cleanup complete!${NC}"
fi

# Verify removal
echo ""
echo "Verifying removal..."
if command -v ffmpeg &> /dev/null; then
    echo -e "${YELLOW}⚠ FFmpeg still found at: $(which ffmpeg)${NC}"
    echo "Version: $(ffmpeg -version 2>&1 | head -1)"
else
    echo -e "${GREEN}✓ FFmpeg completely removed${NC}"
fi

echo ""
echo "Ready to install FFmpeg 8.0 static build!"

