#!/bin/bash
# Diagnostic script to check oneVPL/QSV setup

set -e

echo "════════════════════════════════════════════════════════"
echo "  oneVPL / Intel QSV Diagnostic"
echo "════════════════════════════════════════════════════════"
echo ""

# 1. Check if oneVPL packages are installed
echo "[1/6] Checking oneVPL packages..."
if dpkg -l | grep -q "libvpl"; then
    dpkg -l | grep libvpl
else
    echo "⚠ No libvpl packages found"
    echo ""
    echo "Install with:"
    echo "  sudo apt-get install -y libvpl2 libvpl-dev intel-media-va-driver-non-free"
fi
echo ""

# 2. Check for oneVPL GPU runtime
echo "[2/6] Checking for oneVPL GPU runtime..."
if [ -f "/usr/lib/x86_64-linux-gnu/libmfx-gen.so.1.2" ] || [ -f "/usr/lib/x86_64-linux-gnu/libvpl.so.2" ]; then
    echo "✓ oneVPL libraries found"
    ls -la /usr/lib/x86_64-linux-gnu/libmfx* 2>/dev/null || true
    ls -la /usr/lib/x86_64-linux-gnu/libvpl* 2>/dev/null || true
else
    echo "⚠ oneVPL GPU runtime not found"
    echo ""
    echo "Install with:"
    echo "  sudo apt-get install -y intel-media-va-driver-non-free"
fi
echo ""

# 3. Check VAAPI
echo "[3/6] Checking VAAPI setup..."
if command -v vainfo &> /dev/null; then
    LIBVA_DRIVER_NAME=iHD vainfo 2>&1 | head -20
else
    echo "⚠ vainfo not installed"
    echo "  sudo apt-get install -y vainfo"
fi
echo ""

# 4. Check DRM devices
echo "[4/6] Checking DRM devices..."
ls -la /dev/dri/
echo ""

# 5. Check FFmpeg build
echo "[5/6] Checking FFmpeg QSV support..."
ffmpeg -hide_banner -encoders 2>/dev/null | grep qsv | head -10
echo ""

# 6. Test VAAPI encoding (not QSV)
echo "[6/6] Testing VAAPI encoding (fallback option)..."
if ffmpeg -hide_banner -encoders 2>/dev/null | grep -q "av1_vaapi"; then
    echo "✓ av1_vaapi encoder available (can use as fallback)"
    echo ""
    echo "Test command:"
    echo "  ffmpeg -hide_banner -init_hw_device vaapi=va:/dev/dri/renderD128 \\"
    echo "    -f lavfi -i testsrc2=s=64x64:d=0.1 \\"
    echo "    -vf 'format=nv12,hwupload' \\"
    echo "    -c:v av1_vaapi -frames:v 1 -f null -"
else
    echo "⚠ av1_vaapi encoder not available"
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "  Diagnosis Complete"
echo "════════════════════════════════════════════════════════"
echo ""
echo "If oneVPL is missing, you have two options:"
echo ""
echo "Option 1: Install oneVPL GPU runtime"
echo "  sudo apt-get update"
echo "  sudo apt-get install -y libvpl2 intel-media-va-driver-non-free"
echo ""
echo "Option 2: Use VAAPI encoder instead of QSV"
echo "  (Modify code to use av1_vaapi instead of av1_qsv)"
echo ""

