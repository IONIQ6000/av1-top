#!/bin/bash
#
# Install FFmpeg 8.0 Static Build
# Downloads and installs pre-built FFmpeg 8.0 static binary
#
# Usage: sudo bash install_ffmpeg8_static.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  FFmpeg 8.0 Static Build Installation${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}ERROR: This script must be run as root${NC}"
    echo "Usage: sudo bash install_ffmpeg8_static.sh"
    exit 1
fi

INSTALL_DIR="/usr/local/bin"
STATIC_URL="https://johnvansickle.com/ffmpeg/releases/ffmpeg-8.0-amd64-static.tar.xz"
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Function to check FFmpeg version
check_ffmpeg_version() {
    if command -v ffmpeg &> /dev/null; then
        VERSION=$(ffmpeg -version 2>&1 | head -n1 | grep -oP '\d+\.\d+' | head -1)
        MAJOR=$(echo "$VERSION" | cut -d. -f1)
        if [ "$MAJOR" -ge 8 ] 2>/dev/null; then
            return 0
        fi
    fi
    return 1
}

# Check if FFmpeg 8.0+ already installed
if check_ffmpeg_version; then
    echo -e "${GREEN}✓ FFmpeg 8.0+ already installed${NC}"
    ffmpeg -version | head -1
    
    # Check for QSV encoder
    if ffmpeg -encoders 2>&1 | grep -q av1_qsv; then
        echo -e "${GREEN}✓ av1_qsv encoder available${NC}"
        exit 0
    else
        echo -e "${YELLOW}⚠ FFmpeg 8.0+ installed but av1_qsv encoder not found${NC}"
        echo -e "${YELLOW}Note: Static builds may not include QSV support${NC}"
    fi
fi

# Install required tools
echo -e "${YELLOW}[1/4] Installing required tools...${NC}"
apt-get update -qq

if ! command -v wget &> /dev/null; then
    apt-get install -y -qq wget || {
        if ! command -v curl &> /dev/null; then
            apt-get install -y -qq curl || {
                echo -e "${RED}✗ Cannot install wget or curl${NC}"
                exit 1
            }
        fi
    }
fi

if ! command -v xz &> /dev/null; then
    apt-get install -y -qq xz-utils || {
        echo -e "${RED}✗ Cannot install xz-utils${NC}"
        exit 1
    }
fi

# Download static build
echo -e "${YELLOW}[2/4] Downloading FFmpeg 8.0 static build...${NC}"
echo "URL: $STATIC_URL"

if command -v wget &> /dev/null; then
    wget --progress=bar:force:noscroll -q --show-progress "$STATIC_URL" -O ffmpeg-static.tar.xz || {
        echo -e "${RED}✗ Download failed${NC}"
        exit 1
    }
else
    curl -L --progress-bar "$STATIC_URL" -o ffmpeg-static.tar.xz || {
        echo -e "${RED}✗ Download failed${NC}"
        exit 1
    }
fi

echo -e "${GREEN}✓ Download complete${NC}"

# Extract
echo -e "${YELLOW}[3/4] Extracting...${NC}"
tar xf ffmpeg-static.tar.xz

# Find extracted directory
FFMPEG_DIR=$(find . -maxdepth 1 -type d -name "ffmpeg-*-static" | head -n1)

if [ -z "$FFMPEG_DIR" ]; then
    echo -e "${RED}✗ Failed to find extracted directory${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Extracted to: $FFMPEG_DIR${NC}"

# Install binaries
echo -e "${YELLOW}[4/4] Installing binaries...${NC}"

# Ensure install directory exists
mkdir -p "$INSTALL_DIR"

# Install binaries
cp "$FFMPEG_DIR/ffmpeg" "$INSTALL_DIR/ffmpeg"
cp "$FFMPEG_DIR/ffprobe" "$INSTALL_DIR/ffprobe"
chmod +x "$INSTALL_DIR/ffmpeg"
chmod +x "$INSTALL_DIR/ffprobe"

echo -e "${GREEN}✓ Installed to $INSTALL_DIR${NC}"

# Create symlinks in /usr/bin for system-wide access
echo -e "${YELLOW}Creating symlinks...${NC}"
ln -sf "$INSTALL_DIR/ffmpeg" /usr/bin/ffmpeg
ln -sf "$INSTALL_DIR/ffprobe" /usr/bin/ffprobe

# Verify installation
echo -e "${YELLOW}Verifying installation...${NC}"

if [ -x "$INSTALL_DIR/ffmpeg" ]; then
    "$INSTALL_DIR/ffmpeg" -version | head -1
    echo -e "${GREEN}✓ FFmpeg installed at $INSTALL_DIR/ffmpeg${NC}"
fi

if check_ffmpeg_version; then
    echo -e "${GREEN}✓ FFmpeg 8.0+ installed successfully!${NC}"
    echo ""
    ffmpeg -version | head -1
    echo ""
    
    # Check for QSV encoder (may not be available in static build)
    if ffmpeg -encoders 2>&1 | grep -q av1_qsv; then
        echo -e "${GREEN}✓ av1_qsv encoder available${NC}"
    else
        echo -e "${YELLOW}⚠ av1_qsv encoder not found${NC}"
        echo "Note: Static builds typically don't include QSV support"
        echo "For QSV support, you need to build from source with Intel GPU libraries"
    fi
    
    # Cleanup
    cd /
    rm -rf "$TMP_DIR"
    
    echo ""
    echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Installation Complete!${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "FFmpeg location: $INSTALL_DIR/ffmpeg"
    echo "FFprobe location: $INSTALL_DIR/ffprobe"
    echo ""
    echo "Test with: ffmpeg -version"
    echo ""
    
    exit 0
else
    echo -e "${RED}✗ Installation verification failed${NC}"
    exit 1
fi

