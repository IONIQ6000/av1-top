#!/bin/bash
#
# AV1 Janitor - ONE-CLICK INSTALLER FOR DEBIAN/UBUNTU
# This script installs EVERYTHING needed for AV1 transcoding with Intel QSV
#
# Usage: curl -sSL https://example.com/install.sh | sudo bash
#        Or: sudo bash install.sh
#
# What it does:
# - Installs FFmpeg 8.0+ with Intel QSV support
# - Installs Intel GPU drivers
# - Installs Rust (if needed)
# - Builds AV1 Janitor from source
# - Installs binaries to /usr/local/bin
# - Creates config files
# - Sets up systemd service
# - Configures user permissions
#
# Author: AV1 Janitor Team
# License: MIT

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/opt/av1janitor"
BIN_DIR="/usr/local/bin"
CONFIG_DIR="/etc/av1janitor"
SERVICE_USER="av1janitor"

echo -e "${BLUE}"
cat << "EOF"
   ___   _   ____    __                 _ __            
  / _ | | | / <  /   / /__ ____  (_)__  (_) /_____  ____
 / __ | | |/ // /   / / _ `/ _ \/ / _ \/ / __/ __ \/ __/
/_/ |_| |___//_/   /_/\_,_/_//_/_/_//_/_/\__/\___/_/   
                                                        
EOF
echo -e "${NC}"
echo -e "${GREEN}ONE-CLICK INSTALLER FOR DEBIAN/UBUNTU${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}ERROR: This script must be run as root${NC}"
    echo "Usage: sudo bash install.sh"
    exit 1
fi

# Detect actual user (not root)
if [ -n "$SUDO_USER" ]; then
    ACTUAL_USER="$SUDO_USER"
else
    ACTUAL_USER="$USER"
fi

echo -e "${YELLOW}Installing AV1 Janitor...${NC}"
echo ""

# ============================================================================
# STEP 1: System Update
# ============================================================================
echo -e "${BLUE}[1/10] Updating system packages...${NC}"
apt-get update -qq

# ============================================================================
# STEP 2: Install Intel GPU Drivers
# ============================================================================
echo -e "${BLUE}[2/10] Installing Intel GPU drivers and media libraries...${NC}"

# Check Ubuntu/Debian version
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS_VERSION=$VERSION_CODENAME
else
    echo -e "${YELLOW}Warning: Cannot detect OS version, using defaults${NC}"
    OS_VERSION="noble"  # Ubuntu 24.04
fi

# Install Intel media drivers
apt-get install -y -qq \
    intel-media-va-driver-non-free \
    intel-gpu-tools \
    vainfo \
    i965-va-driver \
    mesa-va-drivers \
    libva2 \
    libva-drm2 \
    libmfx1 \
    libvpl2 \
    || echo -e "${YELLOW}Warning: Some Intel drivers may not be available${NC}"

echo -e "${GREEN}✓ Intel GPU drivers installed${NC}"

# ============================================================================
# STEP 3: Install FFmpeg 8.0+
# ============================================================================
echo -e "${BLUE}[3/10] Installing FFmpeg 8.0+ with Intel QSV support...${NC}"

# Check if FFmpeg is already installed and version
if command -v ffmpeg &> /dev/null; then
    FFMPEG_VERSION=$(ffmpeg -version | head -n1 | awk '{print $3}')
    echo "Found FFmpeg $FFMPEG_VERSION"
    
    # Check if it's version 8.0+
    MAJOR_VERSION=$(echo "$FFMPEG_VERSION" | cut -d. -f1 | tr -d 'n')
    if [ "$MAJOR_VERSION" -ge 8 ] 2>/dev/null; then
        echo -e "${GREEN}✓ FFmpeg 8.0+ already installed${NC}"
    else
        echo -e "${YELLOW}FFmpeg version is too old, installing latest...${NC}"
        apt-get install -y -qq ffmpeg || {
            echo -e "${YELLOW}System FFmpeg may be old, downloading static build...${NC}"
            install_static_ffmpeg
        }
    fi
else
    # Try to install from apt first
    apt-get install -y -qq ffmpeg || {
        echo -e "${YELLOW}FFmpeg not in apt, downloading static build...${NC}"
        install_static_ffmpeg
    }
fi

# Verify FFmpeg has QSV support
if ffmpeg -encoders 2>&1 | grep -q av1_qsv; then
    echo -e "${GREEN}✓ FFmpeg with av1_qsv encoder installed${NC}"
else
    echo -e "${YELLOW}Warning: FFmpeg may not have QSV support${NC}"
    echo -e "${YELLOW}Attempting to install static build with QSV...${NC}"
    install_static_ffmpeg
fi

# ============================================================================
# STEP 4: Install Rust (if needed)
# ============================================================================
echo -e "${BLUE}[4/10] Checking Rust installation...${NC}"

if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    echo -e "${GREEN}✓ Rust already installed (version $RUST_VERSION)${NC}"
else
    echo "Installing Rust..."
    # Install Rust as the actual user (not root)
    sudo -u "$ACTUAL_USER" bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
    
    # Source cargo env for this script
    export PATH="/home/$ACTUAL_USER/.cargo/bin:$PATH"
    
    echo -e "${GREEN}✓ Rust installed${NC}"
fi

# ============================================================================
# STEP 5: Build AV1 Janitor
# ============================================================================
echo -e "${BLUE}[5/10] Building AV1 Janitor from source...${NC}"

# Determine source directory
if [ -d "$(pwd)/core" ] && [ -f "$(pwd)/Cargo.toml" ]; then
    SOURCE_DIR="$(pwd)"
    echo "Building from current directory: $SOURCE_DIR"
else
    echo -e "${RED}ERROR: Cannot find AV1 Janitor source code${NC}"
    echo "Please run this script from the rust-av1 directory"
    exit 1
fi

# Build as the actual user
cd "$SOURCE_DIR"
sudo -u "$ACTUAL_USER" bash -c "source ~/.cargo/env && cargo build --release --workspace"

if [ ! -f "target/release/av1d" ] || [ ! -f "target/release/av1top" ]; then
    echo -e "${RED}ERROR: Build failed - binaries not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Built successfully${NC}"

# ============================================================================
# STEP 6: Install Binaries
# ============================================================================
echo -e "${BLUE}[6/10] Installing binaries to $BIN_DIR...${NC}"

cp target/release/av1d "$BIN_DIR/av1d"
cp target/release/av1top "$BIN_DIR/av1top"
chmod +x "$BIN_DIR/av1d"
chmod +x "$BIN_DIR/av1top"

echo -e "${GREEN}✓ Binaries installed:${NC}"
echo "  - $BIN_DIR/av1d ($(du -h $BIN_DIR/av1d | cut -f1))"
echo "  - $BIN_DIR/av1top ($(du -h $BIN_DIR/av1top | cut -f1))"

# ============================================================================
# STEP 7: Create Service User
# ============================================================================
echo -e "${BLUE}[7/10] Setting up service user and permissions...${NC}"

# Create user if doesn't exist
if ! id "$SERVICE_USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$INSTALL_DIR" -c "AV1 Janitor Service" "$SERVICE_USER"
    echo -e "${GREEN}✓ Created user: $SERVICE_USER${NC}"
else
    echo -e "${GREEN}✓ User already exists: $SERVICE_USER${NC}"
fi

# Add to GPU groups
usermod -a -G render,video "$SERVICE_USER" 2>/dev/null || {
    echo -e "${YELLOW}Warning: Could not add user to render/video groups${NC}"
    echo "You may need to manually add the user to GPU groups"
}

# Create directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p /var/log/av1janitor
mkdir -p /var/lib/av1janitor/jobs

chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
chown -R "$SERVICE_USER:$SERVICE_USER" /var/log/av1janitor
chown -R "$SERVICE_USER:$SERVICE_USER" /var/lib/av1janitor

echo -e "${GREEN}✓ Directories created and permissions set${NC}"

# ============================================================================
# STEP 8: Install Configuration
# ============================================================================
echo -e "${BLUE}[8/10] Setting up configuration...${NC}"

# Copy example config if it doesn't exist
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    if [ -f "$SOURCE_DIR/config.example.toml" ]; then
        cp "$SOURCE_DIR/config.example.toml" "$CONFIG_DIR/config.toml"
        
        # Update with sensible defaults (ask user for media directory)
        echo ""
        echo -e "${YELLOW}Where are your media files located?${NC}"
        read -p "Enter path (e.g., /media/movies): " MEDIA_DIR
        
        if [ -n "$MEDIA_DIR" ]; then
            # Update config with user's directory
            cat > "$CONFIG_DIR/config.toml" << EOL
# AV1 Janitor Configuration
# Edit this file to customize settings

watched_directories = [
    "$MEDIA_DIR"
]

min_file_size_bytes = 2147483648  # 2 GiB
size_gate_factor = 0.9             # 90%

media_extensions = ["mkv", "mp4", "avi", "m4v", "mov"]
scan_interval_seconds = 60

[qsv_quality]
below_1080p = 25
at_1080p = 24
at_1440p_and_above = 23
EOL
        fi
        
        echo -e "${GREEN}✓ Configuration created: $CONFIG_DIR/config.toml${NC}"
    else
        echo -e "${YELLOW}Warning: config.example.toml not found, creating minimal config${NC}"
        cat > "$CONFIG_DIR/config.toml" << EOL
watched_directories = ["/media"]
min_file_size_bytes = 2147483648
size_gate_factor = 0.9
media_extensions = ["mkv", "mp4", "avi"]
EOL
    fi
else
    echo -e "${GREEN}✓ Config already exists: $CONFIG_DIR/config.toml${NC}"
fi

chown "$SERVICE_USER:$SERVICE_USER" "$CONFIG_DIR/config.toml"
chmod 644 "$CONFIG_DIR/config.toml"

# ============================================================================
# STEP 9: Install Systemd Service
# ============================================================================
echo -e "${BLUE}[9/10] Installing systemd service...${NC}"

cat > /etc/systemd/system/av1janitor.service << 'EOL'
[Unit]
Description=AV1 Janitor - Automated AV1 Transcoding Daemon
After=network.target

[Service]
Type=simple
User=av1janitor
Group=av1janitor
WorkingDirectory=/opt/av1janitor

ExecStart=/usr/local/bin/av1d --config /etc/av1janitor/config.toml --concurrent 2

Restart=on-failure
RestartSec=10s
TimeoutStopSec=14400

Environment="RUST_LOG=info"

LimitNOFILE=4096
MemoryMax=4G

NoNewPrivileges=true
PrivateTmp=true

SupplementaryGroups=render video

[Install]
WantedBy=multi-user.target
EOL

systemctl daemon-reload

echo -e "${GREEN}✓ Systemd service installed${NC}"

# ============================================================================
# STEP 10: Verify Installation
# ============================================================================
echo -e "${BLUE}[10/10] Verifying installation...${NC}"

# Check binaries
if [ -f "$BIN_DIR/av1d" ] && [ -x "$BIN_DIR/av1d" ]; then
    echo -e "${GREEN}✓ av1d binary installed${NC}"
else
    echo -e "${RED}✗ av1d binary missing${NC}"
    exit 1
fi

if [ -f "$BIN_DIR/av1top" ] && [ -x "$BIN_DIR/av1top" ]; then
    echo -e "${GREEN}✓ av1top binary installed${NC}"
else
    echo -e "${RED}✗ av1top binary missing${NC}"
    exit 1
fi

# Check FFmpeg
if command -v ffmpeg &> /dev/null; then
    FFMPEG_VERSION=$(ffmpeg -version 2>&1 | head -n1 | awk '{print $3}')
    echo -e "${GREEN}✓ FFmpeg $FFMPEG_VERSION installed${NC}"
    
    if ffmpeg -encoders 2>&1 | grep -q av1_qsv; then
        echo -e "${GREEN}✓ av1_qsv encoder available${NC}"
    else
        echo -e "${YELLOW}⚠ Warning: av1_qsv encoder not found${NC}"
    fi
else
    echo -e "${RED}✗ FFmpeg not found${NC}"
    exit 1
fi

# Check GPU
if command -v vainfo &> /dev/null; then
    echo -e "${GREEN}✓ vainfo available (checking GPU...)${NC}"
    if vainfo 2>&1 | grep -q "VAProfileAV1"; then
        echo -e "${GREEN}✓ Intel GPU with AV1 encoding detected${NC}"
    else
        echo -e "${YELLOW}⚠ Warning: GPU may not support AV1 encoding${NC}"
    fi
else
    echo -e "${YELLOW}⚠ vainfo not available${NC}"
fi

# ============================================================================
# Installation Complete
# ============================================================================

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     Installation Complete! 🎉${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo ""

echo "📁 Configuration file: $CONFIG_DIR/config.toml"
echo "📁 Job directory: /var/lib/av1janitor/jobs"
echo "📁 Log directory: /var/log/av1janitor"
echo ""

echo -e "${YELLOW}Next Steps:${NC}"
echo ""
echo "1. Edit configuration (set your media directories):"
echo -e "   ${BLUE}sudo nano $CONFIG_DIR/config.toml${NC}"
echo ""
echo "2. Enable and start the service:"
echo -e "   ${BLUE}sudo systemctl enable av1janitor${NC}"
echo -e "   ${BLUE}sudo systemctl start av1janitor${NC}"
echo ""
echo "3. Monitor with the TUI:"
echo -e "   ${BLUE}av1top${NC}"
echo ""
echo "4. Check status:"
echo -e "   ${BLUE}sudo systemctl status av1janitor${NC}"
echo ""
echo "5. View logs:"
echo -e "   ${BLUE}sudo journalctl -u av1janitor -f${NC}"
echo ""

echo -e "${GREEN}For help, see: /opt/av1janitor/README.md${NC}"
echo ""

# Offer to start service now
echo -e "${YELLOW}Would you like to start the service now? (y/N)${NC}"
read -p "> " START_NOW

if [[ "$START_NOW" =~ ^[Yy]$ ]]; then
    systemctl enable av1janitor
    systemctl start av1janitor
    sleep 2
    echo ""
    systemctl status av1janitor --no-pager
    echo ""
    echo -e "${GREEN}Service started! Monitor with: av1top${NC}"
else
    echo ""
    echo -e "${YELLOW}Service not started. Start manually with:${NC}"
    echo -e "   ${BLUE}sudo systemctl start av1janitor${NC}"
fi

echo ""
echo -e "${GREEN}🎬 Ready to transcode! Happy space-saving! 🎬${NC}"
echo ""

exit 0

# ============================================================================
# Helper Functions
# ============================================================================

install_static_ffmpeg() {
    echo "Downloading FFmpeg static build with QSV support..."
    
    cd /tmp
    
    # Download latest static build
    wget -q --show-progress \
        https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz \
        || {
            echo -e "${RED}Failed to download FFmpeg${NC}"
            exit 1
        }
    
    # Extract
    tar xf ffmpeg-git-amd64-static.tar.xz
    
    # Find the extracted directory
    FFMPEG_DIR=$(find /tmp -maxdepth 1 -type d -name "ffmpeg-git-*-static" | head -n1)
    
    if [ -z "$FFMPEG_DIR" ]; then
        echo -e "${RED}Failed to extract FFmpeg${NC}"
        exit 1
    fi
    
    # Install binaries
    cp "$FFMPEG_DIR/ffmpeg" /usr/local/bin/
    cp "$FFMPEG_DIR/ffprobe" /usr/local/bin/
    chmod +x /usr/local/bin/ffmpeg
    chmod +x /usr/local/bin/ffprobe
    
    # Cleanup
    rm -rf "$FFMPEG_DIR" ffmpeg-git-amd64-static.tar.xz
    
    echo -e "${GREEN}✓ FFmpeg static build installed${NC}"
}

