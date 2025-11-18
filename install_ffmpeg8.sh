#!/bin/bash
#
# Install FFmpeg 8.0+ with Intel QSV support
# Downloads static build or builds from source
#
# Usage: sudo bash install_ffmpeg8.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  FFmpeg 8.0+ Installation Script${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}ERROR: This script must be run as root${NC}"
    echo "Usage: sudo bash install_ffmpeg8.sh"
    exit 1
fi

INSTALL_DIR="/usr/local/bin"
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
    exit 0
fi

echo -e "${YELLOW}[1/4] Downloading FFmpeg 8.0+ static build...${NC}"

# Try to download static build from johnvansickle.com
STATIC_URL="https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz"

echo "Downloading from: $STATIC_URL"
if wget --progress=bar:force:noscroll -q --show-progress "$STATIC_URL" -O ffmpeg-static.tar.xz 2>&1; then
    echo -e "${GREEN}✓ Download complete${NC}"
    
    echo -e "${YELLOW}[2/4] Extracting...${NC}"
    tar xf ffmpeg-static.tar.xz
    
    # Find extracted directory
    FFMPEG_DIR=$(find . -maxdepth 1 -type d -name "ffmpeg-git-*-static" | head -n1)
    
    if [ -z "$FFMPEG_DIR" ]; then
        echo -e "${RED}✗ Failed to find extracted directory${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Extracted to: $FFMPEG_DIR${NC}"
    
    echo -e "${YELLOW}[3/4] Installing binaries...${NC}"
    
    # Install binaries
    cp "$FFMPEG_DIR/ffmpeg" "$INSTALL_DIR/"
    cp "$FFMPEG_DIR/ffprobe" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/ffmpeg"
    chmod +x "$INSTALL_DIR/ffprobe"
    
    echo -e "${GREEN}✓ Installed to $INSTALL_DIR${NC}"
    
    # Verify installation
    echo -e "${YELLOW}[4/4] Verifying installation...${NC}"
    
    if check_ffmpeg_version; then
        echo -e "${GREEN}✓ FFmpeg 8.0+ installed successfully!${NC}"
        echo ""
        ffmpeg -version | head -1
        echo ""
        
        # Check for QSV encoder
        if ffmpeg -encoders 2>&1 | grep -q av1_qsv; then
            echo -e "${GREEN}✓ av1_qsv encoder available${NC}"
        else
            echo -e "${YELLOW}⚠ Warning: av1_qsv encoder not found${NC}"
            echo "This may be normal if using a static build without QSV support"
            echo "For QSV support, you may need to build from source"
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
else
    echo -e "${YELLOW}⚠ Static build download failed, trying alternative...${NC}"
    
    # Try alternative: GitHub releases or build from source
    echo -e "${YELLOW}Attempting to build from source (this will take 10-30 minutes)...${NC}"
    
    # Install build dependencies
    echo -e "${YELLOW}[1/5] Installing build dependencies...${NC}"
    apt-get update -qq
    apt-get install -y -qq \
        build-essential \
        yasm \
        cmake \
        libtool \
        libc6-dev \
        libx264-dev \
        libx265-dev \
        libnuma-dev \
        libvpx-dev \
        libfdk-aac-dev \
        libmp3lame-dev \
        libopus-dev \
        libvorbis-dev \
        libtheora-dev \
        libxvidcore-dev \
        libx264-dev \
        libx265-dev \
        pkg-config \
        wget \
        git \
        || {
            echo -e "${RED}✗ Failed to install dependencies${NC}"
            exit 1
        }
    
    # Download FFmpeg source
    echo -e "${YELLOW}[2/5] Downloading FFmpeg source...${NC}"
    FFMPEG_VERSION="8.0"
    wget -q --show-progress "https://ffmpeg.org/releases/ffmpeg-${FFMPEG_VERSION}.tar.xz" || \
    wget -q --show-progress "https://github.com/FFmpeg/FFmpeg/archive/refs/tags/n${FFMPEG_VERSION}.tar.gz" -O ffmpeg-${FFMPEG_VERSION}.tar.gz || {
        echo -e "${YELLOW}Trying git clone instead...${NC}"
        git clone --depth 1 --branch release/8.0 https://git.ffmpeg.org/ffmpeg.git ffmpeg-source || \
        git clone --depth 1 https://git.ffmpeg.org/ffmpeg.git ffmpeg-source
    }
    
    if [ -f "ffmpeg-${FFMPEG_VERSION}.tar.xz" ]; then
        tar xf "ffmpeg-${FFMPEG_VERSION}.tar.xz"
        cd ffmpeg-${FFMPEG_VERSION}
    elif [ -f "ffmpeg-${FFMPEG_VERSION}.tar.gz" ]; then
        tar xf "ffmpeg-${FFMPEG_VERSION}.tar.gz"
        cd FFmpeg-${FFMPEG_VERSION}
    elif [ -d "ffmpeg-source" ]; then
        cd ffmpeg-source
    else
        echo -e "${RED}✗ Failed to download FFmpeg source${NC}"
        exit 1
    fi
    
    # Configure and build
    echo -e "${YELLOW}[3/5] Configuring FFmpeg (this may take a few minutes)...${NC}"
    ./configure \
        --prefix=/usr/local \
        --enable-gpl \
        --enable-version3 \
        --enable-nonfree \
        --enable-libx264 \
        --enable-libx265 \
        --enable-libvpx \
        --enable-libfdk-aac \
        --enable-libmp3lame \
        --enable-libopus \
        --enable-libvorbis \
        --enable-libtheora \
        --enable-libxvid \
        --enable-libv4l2 \
        --enable-libdrm \
        --enable-vaapi \
        --enable-libmfx \
        --enable-libvpl \
        --enable-libvmaf \
        --enable-shared \
        --enable-pic \
        --disable-debug \
        --disable-doc \
        || {
            echo -e "${YELLOW}⚠ Some optional libraries not found, continuing with basic build...${NC}"
            ./configure \
                --prefix=/usr/local \
                --enable-gpl \
                --enable-version3 \
                --enable-shared \
                --disable-debug \
                --disable-doc \
                || {
                    echo -e "${RED}✗ Configuration failed${NC}"
                    exit 1
                }
        }
    
    echo -e "${YELLOW}[4/5] Building FFmpeg (this will take 10-30 minutes)...${NC}"
    make -j$(nproc) || {
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    }
    
    echo -e "${YELLOW}[5/5] Installing...${NC}"
    make install || {
        echo -e "${RED}✗ Installation failed${NC}"
        exit 1
    }
    
    # Update library cache
    ldconfig
    
    # Verify
    if check_ffmpeg_version; then
        echo -e "${GREEN}✓ FFmpeg 8.0+ built and installed successfully!${NC}"
        ffmpeg -version | head -1
        
        # Cleanup
        cd /
        rm -rf "$TMP_DIR"
        
        exit 0
    else
        echo -e "${RED}✗ Build verification failed${NC}"
        exit 1
    fi
fi

