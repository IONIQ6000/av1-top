#!/bin/bash
# Install FFmpeg 8.0 static build from BtbN/FFmpeg-Builds

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Please run as root (use sudo)${NC}"
    exit 1
fi

echo "════════════════════════════════════════════════════════"
echo "  FFmpeg 8.0 Static Build Installation"
echo "════════════════════════════════════════════════════════"
echo ""

# Function to check FFmpeg version
check_ffmpeg_version() {
    if command -v ffmpeg &> /dev/null; then
        VERSION_LINE=$(ffmpeg -version 2>&1 | head -1)
        
        # Check for semantic version (e.g., "ffmpeg version 8.0")
        VERSION=$(echo "$VERSION_LINE" | grep -oP 'ffmpeg version \K[0-9]+\.[0-9]+' || echo "")
        if [ -n "$VERSION" ]; then
            MAJOR=$(echo "$VERSION" | cut -d. -f1)
            MINOR=$(echo "$VERSION" | cut -d. -f2)
            if [ "$MAJOR" -gt 8 ] || ([ "$MAJOR" -eq 8 ] && [ "$MINOR" -ge 0 ]); then
                return 0
            fi
        fi
        
        # Check for git build version (e.g., "N-71064-gd5e603ddc0" or "n8.0")
        # If it contains "n8.0" or "8.0" in the version string, it's FFmpeg 8.0+
        if echo "$VERSION_LINE" | grep -qiE '(n8\.0|8\.0|ffmpeg-n8\.0)'; then
            return 0
        fi
        
        # Check if it's from BtbN FFmpeg-Builds (which are FFmpeg 8.0+)
        if echo "$VERSION_LINE" | grep -qi 'BtbN\|FFmpeg-Builds'; then
            return 0
        fi
    fi
    return 1
}

# Check if FFmpeg 8.0+ already installed
if check_ffmpeg_version; then
    echo -e "${GREEN}✓ FFmpeg 8.0+ already installed${NC}"
    ffmpeg -version | head -1
    
    # Check if it has QSV support
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

# Install wget/curl and xz-utils if needed
if ! command -v wget &> /dev/null && ! command -v curl &> /dev/null; then
    apt-get install -y -qq wget || apt-get install -y -qq curl || {
        echo -e "${RED}✗ Cannot install wget or curl${NC}"
        exit 1
    }
fi

if ! command -v xz &> /dev/null; then
    apt-get install -y -qq xz-utils || {
        echo -e "${RED}✗ Cannot install xz-utils (needed for extraction)${NC}"
        exit 1
    }
fi

# Download FFmpeg 8.0 static build
echo -e "${YELLOW}[2/4] Downloading FFmpeg 8.0 static build...${NC}"
STATIC_URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n8.0-latest-linux64-gpl-8.0.tar.xz"
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "Downloading from: $STATIC_URL"
if command -v wget &> /dev/null; then
    wget --progress=bar:force:noscroll -q --show-progress "$STATIC_URL" -O ffmpeg-static.tar.xz 2>&1 || {
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
tar xf ffmpeg-static.tar.xz || {
    echo -e "${RED}✗ Extraction failed${NC}"
    exit 1
}

# Find the extracted directory
EXTRACTED_DIR=$(find . -maxdepth 1 -type d -name "ffmpeg-n8.0-*" | head -1)
if [ -z "$EXTRACTED_DIR" ]; then
    echo -e "${RED}✗ Could not find extracted directory${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Extracted to: $EXTRACTED_DIR${NC}"

# Install binaries
echo -e "${YELLOW}[4/4] Installing binaries...${NC}"
INSTALL_DIR="/usr/local/bin"

# Copy binaries
cp "$EXTRACTED_DIR/bin/ffmpeg" "$INSTALL_DIR/ffmpeg" || {
    echo -e "${RED}✗ Failed to copy ffmpeg${NC}"
    exit 1
}

cp "$EXTRACTED_DIR/bin/ffprobe" "$INSTALL_DIR/ffprobe" || {
    echo -e "${RED}✗ Failed to copy ffprobe${NC}"
    exit 1
}

# Copy ffplay if it exists
[ -f "$EXTRACTED_DIR/bin/ffplay" ] && cp "$EXTRACTED_DIR/bin/ffplay" "$INSTALL_DIR/ffplay" || true

# Make executable
chmod +x "$INSTALL_DIR/ffmpeg"
chmod +x "$INSTALL_DIR/ffprobe"
[ -f "$INSTALL_DIR/ffplay" ] && chmod +x "$INSTALL_DIR/ffplay" || true

# Create symlinks in /usr/bin for system-wide access
if [ "$INSTALL_DIR" != "/usr/bin" ]; then
    ln -sf "$INSTALL_DIR/ffmpeg" /usr/bin/ffmpeg
    ln -sf "$INSTALL_DIR/ffprobe" /usr/bin/ffprobe
    [ -f "$INSTALL_DIR/ffplay" ] && ln -sf "$INSTALL_DIR/ffplay" /usr/bin/ffplay || true
fi

# Cleanup
cd /
rm -rf "$TMP_DIR"

# Verify installation
echo ""
echo -e "${YELLOW}Verifying installation...${NC}"

# Update PATH for current session
export PATH="/usr/local/bin:/usr/bin:$PATH"

# Check if binaries exist
if [ ! -f "$INSTALL_DIR/ffmpeg" ]; then
    echo -e "${RED}✗ ffmpeg binary not found at $INSTALL_DIR/ffmpeg${NC}"
    exit 1
fi

if [ ! -f "$INSTALL_DIR/ffprobe" ]; then
    echo -e "${RED}✗ ffprobe binary not found at $INSTALL_DIR/ffprobe${NC}"
    exit 1
fi

# Test binaries directly
if "$INSTALL_DIR/ffmpeg" -version &>/dev/null; then
    echo -e "${GREEN}✓ FFmpeg binary works${NC}"
else
    echo -e "${RED}✗ FFmpeg binary failed to execute${NC}"
    exit 1
fi

if check_ffmpeg_version; then
    echo -e "${GREEN}✓ FFmpeg 8.0+ installed successfully!${NC}"
    echo ""
    ffmpeg -version | head -1
    echo ""
    
    # Check for encoders
    echo "Available AV1 encoders:"
    ffmpeg -encoders 2>&1 | grep -i av1 || echo "  (none found)"
    echo ""
    
    # Check for QSV encoder (may not be available in static build)
    if ffmpeg -encoders 2>&1 | grep -q av1_qsv; then
        echo -e "${GREEN}✓ av1_qsv encoder available${NC}"
    else
        echo -e "${YELLOW}⚠ av1_qsv encoder not found${NC}"
        echo "Note: Static builds may not include QSV support."
        echo "For QSV support, you may need to build from source with Intel GPU libraries."
    fi
    
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

