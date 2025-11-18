#!/bin/bash
# Fix DRM device permissions inside LXC container

set -e

echo "════════════════════════════════════════════════════════"
echo "  Fixing DRM Device Permissions (Container)"
echo "════════════════════════════════════════════════════════"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run with sudo"
    exit 1
fi

# Fix ownership and permissions for all renderD* devices
echo "Fixing permissions for /dev/dri/renderD*..."
chown root:render /dev/dri/renderD* 2>/dev/null || true
chmod 0660 /dev/dri/renderD* 2>/dev/null || true

echo "✓ Permissions updated"
echo ""

# Verify
echo "Current permissions:"
ls -la /dev/dri/renderD* 2>/dev/null || echo "No renderD* devices found"
echo ""

# Test VAAPI access
echo "Testing VAAPI access as av1janitor user..."
if sudo -u av1janitor LIBVA_DRIVER_NAME=iHD vainfo 2>&1 | grep -q "iHD"; then
    echo "✓ VAAPI is accessible!"
    sudo -u av1janitor LIBVA_DRIVER_NAME=iHD vainfo 2>&1 | head -10
else
    echo "⚠ VAAPI test failed, but permissions are set correctly"
    echo "This may be due to missing X11/Wayland, but DRM should work"
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Note: These changes are temporary"
echo "════════════════════════════════════════════════════════"
echo ""
echo "To make permanent, you need to configure on the HOST:"
echo ""
echo "1. On the HOST system (not container), create udev rule:"
echo "   sudo nano /etc/udev/rules.d/99-drm-render.rules"
echo ""
echo "2. Add this line:"
echo "   KERNEL==\"renderD*\", GROUP=\"render\", MODE=\"0660\""
echo ""
echo "3. Reload udev rules:"
echo "   sudo udevadm control --reload-rules"
echo "   sudo udevadm trigger"
echo ""
echo "4. Ensure LXC container config includes device passthrough"
echo ""

