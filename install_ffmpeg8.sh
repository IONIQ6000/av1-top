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

echo -e "${YELLOW}[2/4] Checking for FFmpeg 8.0 source...${NC}"

# Check if user has provided FFmpeg 8.0 source locally
FFMPEG_SOURCE=""
if [ -f "ffmpeg-8.0.tar.xz" ]; then
    echo -e "${GREEN}✓ Found local ffmpeg-8.0.tar.xz${NC}"
    FFMPEG_SOURCE="ffmpeg-8.0.tar.xz"
elif [ -f "../ffmpeg-8.0.tar.xz" ]; then
    echo -e "${GREEN}✓ Found ffmpeg-8.0.tar.xz in parent directory${NC}"
    cp ../ffmpeg-8.0.tar.xz .
    FFMPEG_SOURCE="ffmpeg-8.0.tar.xz"
fi

# If no local source, download FFmpeg 8.0 from official source
if [ -z "$FFMPEG_SOURCE" ]; then
    echo -e "${YELLOW}Downloading FFmpeg 8.0 from official source...${NC}"
    FFMPEG_VERSION="8.0"
    FFMPEG_URL="https://ffmpeg.org/releases/ffmpeg-${FFMPEG_VERSION}.tar.xz"
    
    if command -v wget &> /dev/null; then
        wget --progress=bar:force:noscroll -q --show-progress "$FFMPEG_URL" -O "ffmpeg-${FFMPEG_VERSION}.tar.xz" 2>&1 && \
        FFMPEG_SOURCE="ffmpeg-${FFMPEG_VERSION}.tar.xz"
    elif command -v curl &> /dev/null; then
        curl -L --progress-bar "$FFMPEG_URL" -o "ffmpeg-${FFMPEG_VERSION}.tar.xz" && \
        FFMPEG_SOURCE="ffmpeg-${FFMPEG_VERSION}.tar.xz"
    fi
fi

# If we have FFmpeg 8.0 source, build from source (required for QSV support)
if [ -n "$FFMPEG_SOURCE" ]; then
    echo -e "${GREEN}✓ Will build FFmpeg 8.0 from source with Intel QSV support${NC}"
    # Skip static build, go straight to source build
    SKIP_STATIC=true
else
    echo -e "${YELLOW}⚠ FFmpeg 8.0 source not found, trying static build (may be older version)...${NC}"
    SKIP_STATIC=false
fi

# Try static build only if we don't have source
if [ "$SKIP_STATIC" = "false" ]; then
    STATIC_URL="https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz"
    echo "Downloading static build from: $STATIC_URL"
    
    if command -v wget &> /dev/null; then
        DOWNLOAD_SUCCESS=$(wget --progress=bar:force:noscroll -q --show-progress "$STATIC_URL" -O ffmpeg-static.tar.xz 2>&1 && echo "yes" || echo "no")
    else
        DOWNLOAD_SUCCESS=$(curl -L --progress-bar "$STATIC_URL" -o ffmpeg-static.tar.xz 2>&1 && echo "yes" || echo "no")
    fi
    
    if [ "$DOWNLOAD_SUCCESS" = "yes" ] && [ -f "ffmpeg-static.tar.xz" ]; then
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
    
    # Ensure install directory exists
    mkdir -p "$INSTALL_DIR"
    
    # Install binaries
    cp "$FFMPEG_DIR/ffmpeg" "$INSTALL_DIR/ffmpeg"
    cp "$FFMPEG_DIR/ffprobe" "$INSTALL_DIR/ffprobe"
    chmod +x "$INSTALL_DIR/ffmpeg"
    chmod +x "$INSTALL_DIR/ffprobe"
    
    echo -e "${GREEN}✓ Installed to $INSTALL_DIR${NC}"
    
    # Verify installation
    echo -e "${YELLOW}[4/4] Verifying installation...${NC}"
    
    # Check using full path first
    if [ -x "$INSTALL_DIR/ffmpeg" ]; then
        "$INSTALL_DIR/ffmpeg" -version | head -1
        echo -e "${GREEN}✓ FFmpeg installed at $INSTALL_DIR/ffmpeg${NC}"
    fi
    
    # Also create symlinks in /usr/bin for system-wide access
    if [ "$INSTALL_DIR" != "/usr/bin" ]; then
        echo -e "${YELLOW}Creating symlinks in /usr/bin for system-wide access...${NC}"
        ln -sf "$INSTALL_DIR/ffmpeg" /usr/bin/ffmpeg
        ln -sf "$INSTALL_DIR/ffprobe" /usr/bin/ffprobe
    fi
    
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
    fi
fi

# Build from source (either user-provided or downloaded FFmpeg 8.0)
if [ -n "$FFMPEG_SOURCE" ] || [ "$SKIP_STATIC" = "true" ]; then
    echo -e "${YELLOW}Building FFmpeg 8.0 from source with Intel QSV support...${NC}"
    echo -e "${YELLOW}(This will take 15-30 minutes depending on CPU)${NC}"
    
    # Install build dependencies
    echo -e "${YELLOW}[1/5] Installing build dependencies...${NC}"
    apt-get update -qq
    
    # Core dependencies (required)
    apt-get install -y -qq \
        build-essential \
        yasm \
        nasm \
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
    
    # Extract FFmpeg source
    echo -e "${YELLOW}[2/5] Extracting FFmpeg 8.0 source...${NC}"
    
    if [ -n "$FFMPEG_SOURCE" ] && [ -f "$FFMPEG_SOURCE" ]; then
        echo "Using: $FFMPEG_SOURCE"
        tar xf "$FFMPEG_SOURCE"
        cd ffmpeg-8.0
    elif [ -f "ffmpeg-8.0.tar.xz" ]; then
        tar xf "ffmpeg-8.0.tar.xz"
        cd ffmpeg-8.0
    elif [ -f "ffmpeg-8.0.tar.gz" ]; then
        tar xf "ffmpeg-8.0.tar.gz"
        cd ffmpeg-8.0
    else
        echo -e "${RED}✗ FFmpeg 8.0 source not found${NC}"
        exit 1
    fi
    
    # Configure and build
    echo -e "${YELLOW}[3/5] Configuring FFmpeg (this may take a few minutes)...${NC}"
    
    # Build configure command with Intel QSV support
    CONFIGURE_ARGS="--prefix=/usr/local --enable-gpl --enable-version3 --enable-shared --disable-debug --disable-doc"
    
    # Intel QSV support (REQUIRED for AV1 transcoding)
    CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libmfx --enable-libvpl --enable-vaapi"
    
    # Add optional libraries if available (check multiple ways)
    if pkg-config --exists x264 2>/dev/null || pkg-config --exists libx264 2>/dev/null || [ -f /usr/lib/pkgconfig/x264.pc ]; then
        CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libx264"
    else
        echo "⚠ libx264 not found"
    fi
    
    if pkg-config --exists x265 2>/dev/null || [ -f /usr/lib/pkgconfig/x265.pc ]; then
        CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libx265"
    else
        echo "⚠ libx265 not found"
    fi
    
    if pkg-config --exists vpx 2>/dev/null || [ -f /usr/lib/pkgconfig/vpx.pc ]; then
        CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libvpx"
    else
        echo "⚠ libvpx not found"
    fi
    
    if pkg-config --exists fdk-aac 2>/dev/null || [ -f /usr/lib/pkgconfig/fdk-aac.pc ]; then
        CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libfdk-aac"
    else
        echo "⚠ libfdk-aac not found"
    fi
    
    if pkg-config --exists libmp3lame 2>/dev/null || [ -f /usr/lib/pkgconfig/mp3lame.pc ]; then
        CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libmp3lame"
    else
        echo "⚠ libmp3lame not found"
    fi
    
    if pkg-config --exists opus 2>/dev/null || [ -f /usr/lib/pkgconfig/opus.pc ]; then
        CONFIGURE_ARGS="$CONFIGURE_ARGS --enable-libopus"
    else
        echo "⚠ libopus not found"
    fi
    
    echo "Configure args: $CONFIGURE_ARGS"
    
    # Check if nasm is available
    if ! command -v nasm &> /dev/null || ! nasm -v 2>&1 | grep -q "version 2"; then
        echo -e "${YELLOW}⚠ nasm not found or too old, disabling x86asm (build will be slower)${NC}"
        CONFIGURE_ARGS="$CONFIGURE_ARGS --disable-x86asm"
    fi
    
    ./configure $CONFIGURE_ARGS || {
        echo -e "${YELLOW}⚠ Configuration failed, trying minimal build with x86asm disabled...${NC}"
        ./configure \
            --prefix=/usr/local \
            --enable-gpl \
            --enable-version3 \
            --enable-shared \
            --disable-debug \
            --disable-doc \
            --enable-libmfx \
            --enable-libvpl \
            --enable-vaapi \
            --disable-x86asm \
            || {
                echo -e "${RED}✗ Configuration failed${NC}"
                echo "Check config.log for details:"
                [ -f ffbuild/config.log ] && tail -50 ffbuild/config.log
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

