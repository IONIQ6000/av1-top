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

echo -e "${YELLOW}[1/4] Installing required tools (wget, xz-utils)...${NC}"
apt-get update -qq

# Install wget if needed
if ! command -v wget &> /dev/null; then
    apt-get install -y -qq wget || {
        echo -e "${YELLOW}⚠ wget not available, trying curl...${NC}"
        if ! command -v curl &> /dev/null; then
            apt-get install -y -qq curl || {
                echo -e "${RED}✗ Cannot install wget or curl${NC}"
                exit 1
            }
        fi
    }
fi

# Install xz-utils for extracting .tar.xz files
if ! command -v xz &> /dev/null; then
    apt-get install -y -qq xz-utils || {
        echo -e "${RED}✗ Cannot install xz-utils (needed for extraction)${NC}"
        exit 1
    }
fi

echo -e "${YELLOW}[2/4] Downloading FFmpeg 8.0+ static build...${NC}"

# Try to download static build from johnvansickle.com
STATIC_URL="https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz"

echo "Downloading from: $STATIC_URL"
DOWNLOAD_CMD="wget"
if ! command -v wget &> /dev/null; then
    DOWNLOAD_CMD="curl -L -o"
fi

if $DOWNLOAD_CMD --progress=bar:force:noscroll -q --show-progress "$STATIC_URL" -O ffmpeg-static.tar.xz 2>&1 || \
   curl -L --progress-bar "$STATIC_URL" -o ffmpeg-static.tar.xz 2>&1; then
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
    
    # Core dependencies (required)
    apt-get install -y -qq \
        build-essential \
        yasm \
        cmake \
        libtool \
        pkg-config \
        wget \
        curl \
        git \
        || {
            echo -e "${RED}✗ Failed to install core dependencies${NC}"
            exit 1
        }
    
    # Optional codec libraries (install what's available)
    echo -e "${YELLOW}Installing optional codec libraries...${NC}"
    apt-get install -y -qq \
        libx264-dev \
        libx265-dev \
        libnuma-dev \
        libvpx-dev \
        libmp3lame-dev \
        libopus-dev \
        libvorbis-dev \
        libtheora-dev \
        libxvidcore-dev \
        2>/dev/null || echo -e "${YELLOW}⚠ Some optional libraries not available, continuing...${NC}"
    
    # Try to install libfdk-aac-dev (may not be available)
    apt-get install -y -qq libfdk-aac-dev 2>/dev/null || \
        echo -e "${YELLOW}⚠ libfdk-aac-dev not available, skipping...${NC}"
    
    # Download FFmpeg source
    echo -e "${YELLOW}[2/5] Downloading FFmpeg source...${NC}"
    FFMPEG_VERSION="8.0"
    
    # Try wget first, fall back to curl
    if command -v wget &> /dev/null; then
        wget -q --show-progress "https://ffmpeg.org/releases/ffmpeg-${FFMPEG_VERSION}.tar.xz" 2>/dev/null || \
        wget -q --show-progress "https://github.com/FFmpeg/FFmpeg/archive/refs/tags/n${FFMPEG_VERSION}.tar.gz" -O ffmpeg-${FFMPEG_VERSION}.tar.gz 2>/dev/null || {
            echo -e "${YELLOW}Trying git clone instead...${NC}"
            git clone --depth 1 --branch release/8.0 https://git.ffmpeg.org/ffmpeg.git ffmpeg-source 2>/dev/null || \
            git clone --depth 1 https://git.ffmpeg.org/ffmpeg.git ffmpeg-source 2>/dev/null || {
                echo -e "${RED}✗ Failed to download FFmpeg source${NC}"
                exit 1
            }
        }
    else
        curl -L --progress-bar "https://ffmpeg.org/releases/ffmpeg-${FFMPEG_VERSION}.tar.xz" -o "ffmpeg-${FFMPEG_VERSION}.tar.xz" 2>/dev/null || \
        curl -L --progress-bar "https://github.com/FFmpeg/FFmpeg/archive/refs/tags/n${FFMPEG_VERSION}.tar.gz" -o "ffmpeg-${FFMPEG_VERSION}.tar.gz" 2>/dev/null || {
            echo -e "${YELLOW}Trying git clone instead...${NC}"
            git clone --depth 1 --branch release/8.0 https://git.ffmpeg.org/ffmpeg.git ffmpeg-source 2>/dev/null || \
            git clone --depth 1 https://git.ffmpeg.org/ffmpeg.git ffmpeg-source 2>/dev/null || {
                echo -e "${RED}✗ Failed to download FFmpeg source${NC}"
                exit 1
            }
        }
    fi
    
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
    
    # Build configure command with available libraries
    CONFIGURE_ARGS="--prefix=/usr/local --enable-gpl --enable-version3 --enable-shared --disable-debug --disable-doc"
    
    # Add optional libraries if available
    pkg-config --exists libx264 && CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libx264" || echo "⚠ libx264 not found"
    pkg-config --exists x265 && CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libx265" || echo "⚠ libx265 not found"
    pkg-config --exists vpx && CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libvpx" || echo "⚠ libvpx not found"
    pkg-config --exists fdk-aac && CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libfdk-aac" || echo "⚠ libfdk-aac not found"
    pkg-config --exists libmp3lame && CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libmp3lame" || echo "⚠ libmp3lame not found"
    pkg-config --exists opus && CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libopus" || echo "⚠ libopus not found"
    
    echo "Configure args: $CONFIGURE_ARGS"
    ./configure $CONFIGURE_ARGS || {
        echo -e "${YELLOW}⚠ Configuration failed, trying minimal build...${NC}"
        ./configure \
            --prefix=/usr/local \
            --enable-gpl \
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

