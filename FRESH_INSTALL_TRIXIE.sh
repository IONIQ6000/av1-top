#!/bin/bash
# Fresh installation script for AV1 Janitor on Debian Trixie
# This installs everything from scratch: drivers, FFmpeg 8, Rust, and the daemon

set -e

echo "════════════════════════════════════════════════════════"
echo "  AV1 Janitor - Fresh Installation for Debian Trixie"
echo "════════════════════════════════════════════════════════"
echo ""

# Ensure we're running as root or with sudo
if [ "$EUID" -ne 0 ]; then 
    echo "Please run with sudo or as root"
    exit 1
fi

# Get the actual user (not root) for later
ACTUAL_USER="${SUDO_USER:-$USER}"
ACTUAL_HOME=$(eval echo "~$ACTUAL_USER")

echo "Installing as user: $ACTUAL_USER"
echo "Home directory: $ACTUAL_HOME"
echo ""

# ============================================================
# STEP 1: Enable non-free repositories and update system
# ============================================================
echo "[1/8] Enabling non-free repositories..."

# Check if using new DEB822 format
if [ -f /etc/apt/sources.list.d/debian.sources ]; then
    echo "Detected DEB822 format sources"
    
    # Backup
    cp /etc/apt/sources.list.d/debian.sources /etc/apt/sources.list.d/debian.sources.backup
    
    # Add non-free components if not present
    if ! grep -q "non-free-firmware" /etc/apt/sources.list.d/debian.sources; then
        sed -i 's/Components: main/Components: main contrib non-free non-free-firmware/' /etc/apt/sources.list.d/debian.sources
        echo "✓ Added non-free repositories"
    else
        echo "✓ Non-free repositories already enabled"
    fi
elif [ -f /etc/apt/sources.list ]; then
    echo "Detected traditional sources.list format"
    
    # Backup
    cp /etc/apt/sources.list /etc/apt/sources.list.backup
    
    # Add non-free components if not present
    if ! grep -q "non-free-firmware" /etc/apt/sources.list; then
        sed -i 's/main$/main contrib non-free non-free-firmware/' /etc/apt/sources.list
        echo "✓ Added non-free repositories"
    else
        echo "✓ Non-free repositories already enabled"
    fi
else
    echo "⚠ Could not find apt sources configuration"
fi

echo "Updating package lists..."
apt-get update
echo ""

# ============================================================
# STEP 2: Install Intel GPU drivers and dependencies
# ============================================================
echo "[2/8] Installing Intel GPU drivers and hardware acceleration..."

apt-get install -y \
    intel-media-va-driver-non-free \
    intel-gpu-tools \
    vainfo \
    libva2 \
    libva-drm2 \
    libvpl2 \
    libvpl-dev \
    i965-va-driver \
    mesa-va-drivers

echo "✓ GPU drivers installed"
echo ""

# ============================================================
# STEP 3: Install build dependencies
# ============================================================
echo "[3/8] Installing build dependencies..."

apt-get install -y \
    build-essential \
    pkg-config \
    curl \
    wget \
    git \
    xz-utils \
    ca-certificates

echo "✓ Build dependencies installed"
echo ""

# ============================================================
# STEP 4: Install Rust toolchain
# ============================================================
echo "[4/8] Installing Rust toolchain..."

if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✓ Rust already installed: $RUST_VERSION"
else
    echo "Installing Rust via rustup..."
    
    # Install as the actual user, not root
    sudo -u "$ACTUAL_USER" bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
    
    # Source the cargo environment
    export PATH="$ACTUAL_HOME/.cargo/bin:$PATH"
    
    echo "✓ Rust installed"
fi

echo ""

# ============================================================
# STEP 5: Install FFmpeg 8 static build
# ============================================================
echo "[5/8] Installing FFmpeg 8 static build..."

# Remove any old FFmpeg installations
echo "Cleaning up old FFmpeg installations..."
rm -f /usr/local/bin/ffmpeg /usr/local/bin/ffprobe /usr/local/bin/ffplay
rm -f /usr/bin/ffmpeg /usr/bin/ffprobe /usr/bin/ffplay 2>/dev/null || true
rm -rf /usr/local/ffmpeg-* 2>/dev/null || true

# Download FFmpeg 8 static build
FFMPEG_URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n8.0-latest-linux64-gpl-8.0.tar.xz"
DOWNLOAD_DIR="/tmp/ffmpeg-install-$$"
mkdir -p "$DOWNLOAD_DIR"
cd "$DOWNLOAD_DIR"

echo "Downloading FFmpeg 8 from: $FFMPEG_URL"
wget -O ffmpeg-static.tar.xz "$FFMPEG_URL"

echo "Extracting..."
tar xf ffmpeg-static.tar.xz

# Find the extracted directory
FFMPEG_DIR=$(find . -maxdepth 1 -type d -name "ffmpeg-n8.0*" | head -1)

if [ -z "$FFMPEG_DIR" ]; then
    echo "✗ Failed to find extracted FFmpeg directory"
    exit 1
fi

echo "Installing binaries to /usr/local/bin..."
mkdir -p /usr/local/bin
cp "$FFMPEG_DIR/bin/ffmpeg" /usr/local/bin/
cp "$FFMPEG_DIR/bin/ffprobe" /usr/local/bin/
chmod +x /usr/local/bin/ffmpeg /usr/local/bin/ffprobe

# Create symlinks in /usr/bin for system-wide access
ln -sf /usr/local/bin/ffmpeg /usr/bin/ffmpeg
ln -sf /usr/local/bin/ffprobe /usr/bin/ffprobe

# Verify installation
if ffmpeg -version 2>&1 | grep -q "ffmpeg version"; then
    FFMPEG_VERSION=$(ffmpeg -version 2>&1 | head -1)
    echo "✓ FFmpeg installed: $FFMPEG_VERSION"
else
    echo "✗ FFmpeg installation verification failed"
    exit 1
fi

# Cleanup
cd /
rm -rf "$DOWNLOAD_DIR"

echo ""

# ============================================================
# STEP 6: Clone/update repository and build
# ============================================================
echo "[6/8] Building AV1 Janitor..."

PROJECT_DIR="$ACTUAL_HOME/av1-top"

if [ -d "$PROJECT_DIR" ]; then
    echo "Project directory exists, pulling latest changes..."
    cd "$PROJECT_DIR"
    sudo -u "$ACTUAL_USER" git pull
else
    echo "Cloning repository..."
    cd "$ACTUAL_HOME"
    sudo -u "$ACTUAL_USER" git clone https://github.com/IONIQ6000/av1-top.git
    cd "$PROJECT_DIR"
fi

echo "Building release binaries..."
sudo -u "$ACTUAL_USER" bash -c "source $ACTUAL_HOME/.cargo/env && cd $PROJECT_DIR && cargo build --release --workspace"

if [ ! -f "target/release/av1d" ] || [ ! -f "target/release/av1top" ]; then
    echo "✗ Build failed - binaries not found"
    exit 1
fi

echo "✓ Build complete"
echo ""

# ============================================================
# STEP 7: Install binaries and configuration
# ============================================================
echo "[7/8] Installing binaries and configuration..."

# Install binaries
cp target/release/av1d /usr/bin/av1d
cp target/release/av1top /usr/bin/av1top
chmod +x /usr/bin/av1d /usr/bin/av1top

# Create av1janitor user if it doesn't exist
if ! id -u av1janitor &> /dev/null; then
    useradd -r -s /bin/false -d /opt/av1janitor -c "AV1 Janitor Service" av1janitor
    echo "✓ Created av1janitor user"
else
    echo "✓ av1janitor user already exists"
fi

# Add av1janitor to video and render groups
usermod -a -G video,render av1janitor

# Create directories
mkdir -p /etc/av1janitor
mkdir -p /var/lib/av1janitor/jobs
mkdir -p /var/log/av1janitor
mkdir -p /opt/av1janitor

# Set permissions
chown -R av1janitor:av1janitor /var/lib/av1janitor
chown -R av1janitor:av1janitor /var/log/av1janitor
chown -R av1janitor:av1janitor /opt/av1janitor
chmod 755 /var/lib/av1janitor
chmod 755 /var/log/av1janitor

# Install example config if not exists
if [ ! -f /etc/av1janitor/config.toml ]; then
    if [ -f config.example.toml ]; then
        cp config.example.toml /etc/av1janitor/config.toml
        echo "✓ Installed example config to /etc/av1janitor/config.toml"
    else
        # Create a basic config
        cat > /etc/av1janitor/config.toml <<'EOF'
# AV1 Janitor Configuration

# Directories to watch for media files (required)
watched_directories = ["/media", "/mnt/media"]

# Maximum file size to process (in bytes, default: 50GB)
# max_file_size = 53687091200

# Transcoding quality settings (23-25, lower = better quality)
# quality_1080p = 24
# quality_2160p = 23
# quality_default = 25

# Skip files smaller than this (in bytes, default: 500MB)
# Files smaller than this are assumed to be already optimized
# min_file_size = 524288000

# Concurrent transcoding jobs (default: 1)
# Be careful with high values - each job uses significant GPU/CPU
# concurrent_jobs = 1
EOF
        echo "✓ Created default config at /etc/av1janitor/config.toml"
    fi
    
    echo ""
    echo "⚠ IMPORTANT: Edit /etc/av1janitor/config.toml and set your media directories!"
    echo "   Example: watched_directories = [\"/main-library-2/Media/Movies\"]"
    echo ""
fi

# Install systemd service
cat > /etc/systemd/system/av1janitor.service <<'EOF'
[Unit]
Description=AV1 Janitor - Automated AV1 Transcoding Daemon
After=network.target

[Service]
Type=simple
User=av1janitor
Group=av1janitor
Environment="HOME=/opt/av1janitor"
Environment="LIBVA_DRIVER_NAME=iHD"
ExecStart=/usr/bin/av1d --config /etc/av1janitor/config.toml --concurrent 2
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/av1janitor /opt/av1janitor /var/log/av1janitor /media /main-library-2
ProtectKernelTunables=true
ProtectControlGroups=true

# Resource limits
MemoryMax=8G
CPUQuota=400%

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo "✓ Systemd service installed"
echo ""

# ============================================================
# STEP 8: Verify GPU access and setup
# ============================================================
echo "[8/8] Verifying GPU access..."

echo "DRM devices:"
ls -la /dev/dri/

echo ""
echo "Testing VAAPI as av1janitor user..."
if sudo -u av1janitor LIBVA_DRIVER_NAME=iHD vainfo 2>&1 | grep -q "iHD"; then
    echo "✓ VAAPI accessible by av1janitor user"
else
    echo "⚠ VAAPI test had issues, but may still work"
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Installation Complete!"
echo "════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo ""
echo "1. Edit configuration:"
echo "   sudo nano /etc/av1janitor/config.toml"
echo "   (Set your media directories!)"
echo ""
echo "2. Start the service:"
echo "   sudo systemctl start av1janitor"
echo ""
echo "3. Enable on boot:"
echo "   sudo systemctl enable av1janitor"
echo ""
echo "4. Check status:"
echo "   sudo systemctl status av1janitor"
echo ""
echo "5. View logs:"
echo "   sudo journalctl -u av1janitor -f"
echo ""
echo "6. Monitor with TUI:"
echo "   av1top"
echo ""
echo "Installed components:"
echo "  • FFmpeg: $(ffmpeg -version 2>&1 | head -1)"
echo "  • Binaries: /usr/bin/av1d, /usr/bin/av1top"
echo "  • Config: /etc/av1janitor/config.toml"
echo "  • Jobs: /var/lib/av1janitor/jobs"
echo "  • Logs: journalctl -u av1janitor"
echo ""

