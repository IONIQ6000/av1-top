#!/bin/bash
# Simple installation script for AV1 Janitor
# Assumes Intel drivers are already installed

set -e

echo "════════════════════════════════════════════════════════"
echo "  AV1 Janitor - Simple Installation"
echo "════════════════════════════════════════════════════════"
echo ""

# Ensure we're running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run with sudo"
    exit 1
fi

ACTUAL_USER="${SUDO_USER:-$USER}"
ACTUAL_HOME=$(eval echo "~$ACTUAL_USER")

# ============================================================
# STEP 1: Install build tools
# ============================================================
echo "[1/6] Installing build tools..."
apt-get update
apt-get install -y git curl build-essential pkg-config wget xz-utils
echo "✓ Build tools installed"
echo ""

# ============================================================
# STEP 2: Clone repository
# ============================================================
echo "[2/6] Cloning repository..."
PROJECT_DIR="$ACTUAL_HOME/av1-top"

if [ -d "$PROJECT_DIR" ]; then
    echo "Project exists, updating..."
    cd "$PROJECT_DIR"
    sudo -u "$ACTUAL_USER" git pull
else
    cd "$ACTUAL_HOME"
    sudo -u "$ACTUAL_USER" git clone https://github.com/IONIQ6000/av1-top.git
    cd "$PROJECT_DIR"
fi
echo "✓ Repository ready"
echo ""

# ============================================================
# STEP 3: Install Rust
# ============================================================
echo "[3/6] Installing Rust..."
if command -v rustc &> /dev/null; then
    echo "✓ Rust already installed: $(rustc --version)"
else
    sudo -u "$ACTUAL_USER" bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
    echo "✓ Rust installed"
fi
echo ""

# ============================================================
# STEP 4: Install FFmpeg 8
# ============================================================
echo "[4/6] Installing FFmpeg 8..."

# Remove old installations
rm -f /usr/local/bin/ffmpeg /usr/local/bin/ffprobe
rm -f /usr/bin/ffmpeg /usr/bin/ffprobe 2>/dev/null || true

# Download and install
FFMPEG_URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n8.0-latest-linux64-gpl-8.0.tar.xz"
DOWNLOAD_DIR="/tmp/ffmpeg-$$"
mkdir -p "$DOWNLOAD_DIR"
cd "$DOWNLOAD_DIR"

echo "Downloading FFmpeg 8..."
wget -q --show-progress -O ffmpeg.tar.xz "$FFMPEG_URL"
tar xf ffmpeg.tar.xz
FFMPEG_DIR=$(find . -maxdepth 1 -type d -name "ffmpeg-n8.0*" | head -1)

mkdir -p /usr/local/bin
cp "$FFMPEG_DIR/bin/ffmpeg" /usr/local/bin/
cp "$FFMPEG_DIR/bin/ffprobe" /usr/local/bin/
chmod +x /usr/local/bin/ffmpeg /usr/local/bin/ffprobe
ln -sf /usr/local/bin/ffmpeg /usr/bin/ffmpeg
ln -sf /usr/local/bin/ffprobe /usr/bin/ffprobe

cd /
rm -rf "$DOWNLOAD_DIR"

echo "✓ FFmpeg installed: $(ffmpeg -version 2>&1 | head -1)"
echo ""

# ============================================================
# STEP 5: Build project
# ============================================================
echo "[5/6] Building AV1 Janitor..."
cd "$PROJECT_DIR"
sudo -u "$ACTUAL_USER" bash -c "source $ACTUAL_HOME/.cargo/env && cargo build --release --workspace"

if [ ! -f "target/release/av1d" ]; then
    echo "✗ Build failed"
    exit 1
fi

echo "✓ Build complete"
echo ""

# ============================================================
# STEP 6: Install and configure
# ============================================================
echo "[6/6] Installing and configuring..."

# Install binaries
cp target/release/av1d /usr/bin/
cp target/release/av1top /usr/bin/
chmod +x /usr/bin/av1d /usr/bin/av1top

# Create user
if ! id -u av1janitor &> /dev/null; then
    useradd -r -s /bin/false -d /opt/av1janitor -c "AV1 Janitor Service" av1janitor
fi
usermod -a -G video,render av1janitor

# Create directories
mkdir -p /etc/av1janitor
mkdir -p /var/lib/av1janitor/jobs
mkdir -p /var/log/av1janitor
mkdir -p /opt/av1janitor

chown -R av1janitor:av1janitor /var/lib/av1janitor
chown -R av1janitor:av1janitor /var/log/av1janitor
chown -R av1janitor:av1janitor /opt/av1janitor

# Create config if not exists
if [ ! -f /etc/av1janitor/config.toml ]; then
    cat > /etc/av1janitor/config.toml <<'EOF'
# AV1 Janitor Configuration
# Edit this file to set your media directories

watched_directories = ["/media"]

# Uncomment and modify as needed:
# max_file_size = 53687091200  # 50GB
# quality_1080p = 24
# quality_2160p = 23
# quality_default = 25
# min_file_size = 524288000  # 500MB
EOF
    echo "✓ Created config at /etc/av1janitor/config.toml"
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

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/av1janitor /opt/av1janitor /var/log/av1janitor /media /main-library-2
ProtectKernelTunables=true
ProtectControlGroups=true

MemoryMax=8G
CPUQuota=400%

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo "✓ Service installed"
echo ""

# Test GPU access
echo "Testing GPU access..."
ls -la /dev/dri/
echo ""

if sudo -u av1janitor LIBVA_DRIVER_NAME=iHD vainfo 2>&1 | grep -q "iHD"; then
    echo "✓ GPU accessible by av1janitor user"
else
    echo "⚠ GPU test had issues (may still work)"
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Installation Complete!"
echo "════════════════════════════════════════════════════════"
echo ""
echo "IMPORTANT: Edit your media directories:"
echo "  sudo nano /etc/av1janitor/config.toml"
echo ""
echo "Example config:"
echo '  watched_directories = ["/main-library-2/Media/Movies"]'
echo ""
echo "Then start the service:"
echo "  sudo systemctl start av1janitor"
echo "  sudo systemctl enable av1janitor"
echo ""
echo "Monitor:"
echo "  sudo journalctl -u av1janitor -f"
echo "  av1top"
echo ""

