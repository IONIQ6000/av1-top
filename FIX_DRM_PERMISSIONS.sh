#!/bin/bash
# Fix DRM render node permissions for QSV hardware acceleration

set -e

echo "Fixing DRM render node permissions..."

# Check if render nodes exist
if [ ! -e /dev/dri/renderD128 ]; then
    echo "⚠ No DRM render nodes found at /dev/dri/renderD128"
    echo "Checking for other render nodes..."
    ls -la /dev/dri/renderD* || echo "No render nodes found"
    exit 1
fi

# Fix ownership to root:render
echo "Setting ownership to root:render..."
sudo chown root:render /dev/dri/renderD*

# Fix permissions to allow group read/write
echo "Setting permissions to 0660 (rw-rw----)..."
sudo chmod 0660 /dev/dri/renderD*

# Verify
echo ""
echo "Verifying permissions:"
ls -la /dev/dri/renderD*

echo ""
echo "✓ DRM permissions fixed!"
echo ""
echo "Note: These changes may be reset on reboot."
echo "To make permanent, add a udev rule or systemd-tmpfiles rule."

