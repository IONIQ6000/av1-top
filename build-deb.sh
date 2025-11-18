#!/bin/bash
#
# Build Debian package for AV1 Janitor
# Creates a .deb file that can be installed with: sudo dpkg -i av1janitor_*.deb
#
# Usage: ./build-deb.sh
#

set -e

VERSION="0.1.0"
ARCH="amd64"
PACKAGE_NAME="av1janitor"
DEB_NAME="${PACKAGE_NAME}_${VERSION}_${ARCH}"

echo "Building Debian package: $DEB_NAME"
echo ""

# Build release binaries
echo "[1/5] Building release binaries..."

# Detect if we're on macOS and need cross-compilation
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "WARNING: Building on macOS - binaries will be macOS binaries, not Linux!"
    echo "For Linux .deb packages, build on a Linux system or use cross-compilation."
    echo ""
    echo "To build on Linux:"
    echo "  1. Transfer source to Linux system"
    echo "  2. Run: cargo build --release --workspace"
    echo "  3. Run: ./build-deb.sh"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted. Build on Linux for proper Linux binaries."
        exit 1
    fi
fi

cargo build --release --workspace

# Create package directory structure
echo "[2/5] Creating package structure..."
rm -rf "debian-package"
mkdir -p "debian-package/$DEB_NAME"

# Create DEBIAN control directory
mkdir -p "debian-package/$DEB_NAME/DEBIAN"

# Create installation directories
# Note: Debian packages should use /usr/bin, not /usr/local/bin
# /usr/local is for manual installations, /usr is for packages
mkdir -p "debian-package/$DEB_NAME/usr/bin"
mkdir -p "debian-package/$DEB_NAME/etc/av1janitor"
mkdir -p "debian-package/$DEB_NAME/etc/systemd/system"
mkdir -p "debian-package/$DEB_NAME/usr/share/doc/av1janitor"

# Copy binaries
echo "[3/5] Copying files..."
cp target/release/av1d "debian-package/$DEB_NAME/usr/bin/"
cp target/release/av1top "debian-package/$DEB_NAME/usr/bin/"
chmod +x "debian-package/$DEB_NAME/usr/bin/"*

# Copy config example
cp config.example.toml "debian-package/$DEB_NAME/etc/av1janitor/config.toml.example"

# Copy systemd service
cp av1janitor.service "debian-package/$DEB_NAME/etc/systemd/system/"

# Copy documentation (only if files exist)
if [ -f README.md ]; then
    cp README.md "debian-package/$DEB_NAME/usr/share/doc/av1janitor/"
fi
if [ -f FFMPEG_SETUP.md ]; then
    cp FFMPEG_SETUP.md "debian-package/$DEB_NAME/usr/share/doc/av1janitor/"
fi
if [ -f DEPLOYMENT.md ]; then
    cp DEPLOYMENT.md "debian-package/$DEB_NAME/usr/share/doc/av1janitor/"
fi

# Ensure directory exists even if no docs (dpkg requires it)
touch "debian-package/$DEB_NAME/usr/share/doc/av1janitor/.keep" 2>/dev/null || true

# Create control file
echo "[4/5] Creating control file..."
cat > "debian-package/$DEB_NAME/DEBIAN/control" << EOF
Package: av1janitor
Version: $VERSION
Section: video
Priority: optional
Architecture: $ARCH
Depends: libc6, libgcc-s1, intel-media-va-driver-non-free | intel-media-va-driver
Recommends: ffmpeg (>= 8.0), vainfo
Maintainer: AV1 Janitor Team <av1janitor@example.com>
Description: Automated AV1 transcoding with Intel QSV
 AV1 Janitor automatically transcodes your media library to AV1 using
 Intel Quick Sync Video hardware acceleration.
 .
 Features:
  - Automatic FFmpeg 8.0+ detection and validation
  - Intel QSV hardware acceleration
  - Concurrent file processing
  - Real-time filesystem watching
  - Comprehensive TUI monitor
  - TOML configuration files
  - Graceful shutdown handling
  - Size gate verification
  - Atomic file operations
Homepage: https://github.com/example/av1janitor
EOF

# Create preinst script (runs before package files are extracted)
cat > "debian-package/$DEB_NAME/DEBIAN/preinst" << 'EOF'
#!/bin/bash
set -e

# Ensure doc directory exists before dpkg extracts files
mkdir -p /usr/share/doc/av1janitor

exit 0
EOF

# Create postinst script
cat > "debian-package/$DEB_NAME/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# Create service user if doesn't exist
if ! id av1janitor &>/dev/null; then
    useradd -r -s /bin/false -d /opt/av1janitor -c "AV1 Janitor Service" av1janitor
fi

# Add to GPU groups
usermod -a -G render,video av1janitor 2>/dev/null || true

# Create directories
mkdir -p /opt/av1janitor
mkdir -p /var/log/av1janitor
mkdir -p /var/lib/av1janitor/jobs

# Ensure doc directory exists (dpkg may not create it automatically)
mkdir -p /usr/share/doc/av1janitor

# Set permissions
chown -R av1janitor:av1janitor /opt/av1janitor
chown -R av1janitor:av1janitor /var/log/av1janitor
chown -R av1janitor:av1janitor /var/lib/av1janitor

# Copy example config if user config doesn't exist
if [ ! -f /etc/av1janitor/config.toml ]; then
    cp /etc/av1janitor/config.toml.example /etc/av1janitor/config.toml
    chown av1janitor:av1janitor /etc/av1janitor/config.toml
    chmod 644 /etc/av1janitor/config.toml
    echo "Created default config: /etc/av1janitor/config.toml"
    echo "Please edit it to set your media directories"
fi

# Reload systemd
systemctl daemon-reload

echo ""
echo "AV1 Janitor installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Edit config: sudo nano /etc/av1janitor/config.toml"
echo "  2. Start service: sudo systemctl start av1janitor"
echo "  3. Enable on boot: sudo systemctl enable av1janitor"
echo "  4. Monitor: av1top"
echo ""

exit 0
EOF

# Create postrm script
cat > "debian-package/$DEB_NAME/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    purge)
        # Remove user
        if id av1janitor &>/dev/null; then
            userdel av1janitor 2>/dev/null || true
        fi
        
        # Remove data directories (optional - commented out for safety)
        # rm -rf /var/lib/av1janitor
        # rm -rf /var/log/av1janitor
        # rm -rf /opt/av1janitor
        
        echo "AV1 Janitor purged (data directories preserved)"
        ;;
    remove)
        echo "AV1 Janitor removed"
        ;;
esac

exit 0
EOF

chmod 755 "debian-package/$DEB_NAME/DEBIAN/preinst"
chmod 755 "debian-package/$DEB_NAME/DEBIAN/postinst"
chmod 755 "debian-package/$DEB_NAME/DEBIAN/postrm"

# Build the package
echo "[5/5] Building .deb package..."

# Check if dpkg-deb is available
if command -v dpkg-deb &> /dev/null; then
    # Use dpkg-deb if available (Linux)
    dpkg-deb --build "debian-package/$DEB_NAME"
    mv "debian-package/${DEB_NAME}.deb" .
else
    # Manual .deb creation for macOS/other systems
    echo "dpkg-deb not found, creating .deb manually..."
    
    DEB_FILE="${DEB_NAME}.deb"
    PACKAGE_DIR="debian-package/$DEB_NAME"
    
    # Create temporary directory for .deb components
    TMP_DEB=$(mktemp -d)
    
    # Create debian-binary file
    echo "2.0" > "$TMP_DEB/debian-binary"
    
    # Create control.tar.gz (Linux-compatible, no macOS xattrs)
    cd "$PACKAGE_DIR/DEBIAN"
    # Disable macOS extended attributes and use ustar format for Linux compatibility
    export COPYFILE_DISABLE=1  # macOS: don't copy extended attributes
    # Try ustar format first (Linux-compatible), fallback to default
    tar --format=ustar -czf "$TMP_DEB/control.tar.gz" . 2>/dev/null || \
    tar -czf "$TMP_DEB/control.tar.gz" . --format=ustar 2>/dev/null || \
    COPYFILE_DISABLE=1 tar -czf "$TMP_DEB/control.tar.gz" .
    unset COPYFILE_DISABLE
    cd - > /dev/null
    
    # Create data.tar.gz (all files except DEBIAN, Linux-compatible)
    cd "$PACKAGE_DIR"
    # Disable macOS extended attributes and use ustar format
    export COPYFILE_DISABLE=1
    # Try ustar format first, fallback to default with COPYFILE_DISABLE
    find . -type f ! -path "./DEBIAN/*" -print0 | \
        tar --format=ustar --null -czf "$TMP_DEB/data.tar.gz" -T - 2>/dev/null || \
    find . -type f ! -path "./DEBIAN/*" -print0 | \
        tar --null -czf "$TMP_DEB/data.tar.gz" -T - --format=ustar 2>/dev/null || \
    find . -type f ! -path "./DEBIAN/*" -print0 | \
        COPYFILE_DISABLE=1 tar --null -czf "$TMP_DEB/data.tar.gz" -T -
    unset COPYFILE_DISABLE
    cd - > /dev/null
    
    # Create the .deb file using ar
    cd "$TMP_DEB"
    ar r "$DEB_FILE" debian-binary control.tar.gz data.tar.gz 2>/dev/null || {
        # Fallback: create as tar.gz if ar doesn't work
        echo "ar failed, creating tar.gz package instead..."
        cd "$PACKAGE_DIR"
        tar czf "../${DEB_NAME}.tar.gz" .
        cd - > /dev/null
        mv "$TMP_DEB/${DEB_NAME}.tar.gz" "../$DEB_FILE.tar.gz"
        rm -rf "$TMP_DEB"
        cd ..
        echo ""
        echo "✓ Package created as: ${DEB_NAME}.tar.gz"
        echo "  (Install by extracting and copying files manually)"
        exit 0
    }
    cd - > /dev/null
    
    # Move .deb to current directory
    mv "$TMP_DEB/$DEB_FILE" .
    rm -rf "$TMP_DEB"
fi

# Cleanup
rm -rf debian-package

echo ""
echo "✓ Debian package created: ${DEB_NAME}.deb"
echo ""
echo "Install with:"
echo "  sudo dpkg -i ${DEB_NAME}.deb"
echo "  sudo apt-get install -f  # Install dependencies"
echo ""
echo "Package size: $(du -h ${DEB_NAME}.deb 2>/dev/null | cut -f1 || echo 'unknown')"

