# FFmpeg 8.0 Setup Guide

This project **requires FFmpeg 8.0 or later** (released August 2025) with Intel QSV support.

## Why FFmpeg 8.0?

FFmpeg 8.0 (August 2025) includes critical improvements for AV1 transcoding:

- **Improved AV1_QSV encoder stability** - Better hardware acceleration handling
- **Enhanced VFR (Variable Frame Rate) support** - Proper handling of web rips and streaming content
- **Better odd-dimension video padding** - Automatic handling of non-even dimensions
- **Intel Arc GPU optimizations** - Improved support for Arc A-series GPUs
- **Fixed QSV initialization bugs** - More reliable hardware encoder startup

## Checking Your Current Version

```bash
ffmpeg -version
```

Look for the version number in the first line. You need version **8.0** or higher (or **n8.0** for git builds).

Example acceptable versions:
- `ffmpeg version 8.0`
- `ffmpeg version n8.0`
- `ffmpeg version 8.1`
- `ffmpeg version n8.1-dev`

## Installation Methods

### Option 1: Pre-built Static Binaries (Recommended)

Download the latest FFmpeg 8.0+ static build with QSV support:

**Linux:**
```bash
# Create installation directory
sudo mkdir -p /external-ffmpeg
cd /tmp

# Download FFmpeg 8.0+ static build with Intel QSV
# Option A: From official sources (when available)
wget https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz
tar xf ffmpeg-git-amd64-static.tar.xz
sudo cp ffmpeg-git-*-static/ffmpeg /external-ffmpeg/
sudo cp ffmpeg-git-*-static/ffprobe /external-ffmpeg/

# Option B: Build from source (see below)
```

**Verify QSV support:**
```bash
/external-ffmpeg/ffmpeg -encoders | grep av1_qsv
```

You should see:
```
V..... av1_qsv              AV1 (Intel Quick Sync Video acceleration) (codec av1)
```

### Option 2: Build from Source

Building from source ensures you have the latest version with QSV support.

#### Prerequisites

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    git \
    pkg-config \
    yasm \
    nasm \
    libva-dev \
    libmfx-dev \
    libvpl-dev \
    libdrm-dev
```

**Arch Linux:**
```bash
sudo pacman -S base-devel git yasm nasm libva intel-media-driver onevpl
```

#### Build FFmpeg 8.0+

```bash
# Clone FFmpeg (ensure you're on release/8.0 or later)
cd /tmp
git clone https://git.ffmpeg.org/ffmpeg.git
cd ffmpeg
git checkout release/8.0  # or later

# Configure with QSV support
./configure \
    --prefix=/external-ffmpeg \
    --enable-gpl \
    --enable-version3 \
    --enable-nonfree \
    --enable-libvpl \
    --enable-vaapi \
    --enable-opencl \
    --disable-debug \
    --disable-doc \
    --disable-shared \
    --enable-static

# Build (this takes 10-30 minutes)
make -j$(nproc)

# Install
sudo make install
```

#### Verify Installation

```bash
/external-ffmpeg/ffmpeg -version
/external-ffmpeg/ffmpeg -encoders | grep av1_qsv
```

### Option 3: Docker Container

If you're running the daemon in Docker, use this base image approach:

**Dockerfile:**
```dockerfile
FROM ubuntu:24.04

# Install Intel Media drivers
RUN apt-get update && apt-get install -y \
    wget \
    xz-utils \
    intel-media-va-driver-non-free \
    libmfx1 \
    libva2 \
    libva-drm2 \
    vainfo

# Download and install FFmpeg 8.0 static build
RUN mkdir -p /external-ffmpeg && \
    cd /tmp && \
    wget https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz && \
    tar xf ffmpeg-git-amd64-static.tar.xz && \
    cp ffmpeg-git-*/ffmpeg /external-ffmpeg/ && \
    cp ffmpeg-git-*/ffprobe /external-ffmpeg/ && \
    chmod +x /external-ffmpeg/ffmpeg /external-ffmpeg/ffprobe

# Install Rust and build the project
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY . /app
WORKDIR /app
RUN cargo build --release

# Expose devices for QSV
# Run with: docker run --device=/dev/dri ...
```

## Intel GPU Setup

### Required Hardware

- Intel CPU with integrated graphics (8th gen or later recommended)
- Intel Arc discrete GPU (A310, A380, A750, A770)

### Driver Installation

**Ubuntu 24.04:**
```bash
# Add Intel graphics repository
wget -qO - https://repositories.intel.com/gpu/intel-graphics.key | \
    sudo gpg --dearmor --output /usr/share/keyrings/intel-graphics.gpg

echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-graphics.gpg] https://repositories.intel.com/gpu/ubuntu noble client" | \
    sudo tee /etc/apt/sources.list.d/intel-gpu.list

# Install drivers
sudo apt update
sudo apt install -y \
    intel-media-va-driver-non-free \
    intel-opencl-icd \
    intel-level-zero-gpu \
    level-zero

# Add user to render group
sudo usermod -a -G render $USER
```

**Verify GPU access:**
```bash
vainfo
# Should show your Intel GPU and supported profiles including AV1 encoding
```

### Troubleshooting

**"No QSV device found":**
- Check GPU drivers: `vainfo`
- Ensure `/dev/dri/renderD128` exists and is accessible
- Verify user is in `render` or `video` group

**"av1_qsv encoder not found":**
- Your FFmpeg build doesn't have QSV support
- Rebuild with `--enable-libvpl` or `--enable-libmfx`

**"Cannot initialize QSV session":**
- GPU may not support AV1 encoding (need 11th gen Intel or Arc)
- Check with: `vainfo | grep VAProfileAV1`

## Testing Your Setup

Once installed, test with the daemon:

```bash
cd /path/to/rust-av1
cargo run --bin av1d
```

You should see:
```
Verifying ffmpeg/ffprobe and Intel QSV support...
✓ ffmpeg version: 8.0 (or later)
✓ ffprobe is available
✓ av1_qsv encoder is available
✓ QSV hardware test passed
```

If the QSV hardware test fails, your FFmpeg is installed but can't access the GPU. Check:
1. GPU drivers are installed (`vainfo`)
2. User has GPU access (`groups | grep render`)
3. `/dev/dri/renderD128` exists

## Docker-Specific Setup

When running in Docker, you must:

1. **Pass through the GPU device:**
   ```bash
   docker run --device=/dev/dri:/dev/dri ...
   ```

2. **Use privileged mode (if needed):**
   ```bash
   docker run --privileged ...
   ```

3. **Mount the render group:**
   ```bash
   docker run --group-add=$(getent group render | cut -d: -f3) ...
   ```

## References

- [FFmpeg Official Site](https://ffmpeg.org/)
- [Intel Media SDK](https://github.com/Intel-Media-SDK/MediaSDK)
- [Intel oneVPL](https://github.com/oneapi-src/oneVPL)
- [FFmpeg QSV Documentation](https://trac.ffmpeg.org/wiki/Hardware/QuickSync)

## Support

If you encounter issues:

1. Verify FFmpeg version: `/external-ffmpeg/ffmpeg -version`
2. Check QSV encoder: `/external-ffmpeg/ffmpeg -encoders | grep av1_qsv`
3. Test GPU access: `vainfo`
4. Run daemon validation: `cargo run --bin av1d`

For more help, check the project's GitHub issues or documentation.

