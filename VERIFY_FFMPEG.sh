#!/bin/bash
# Quick script to verify FFmpeg installation

echo "Checking FFmpeg installation..."
echo ""

# Check common locations
for path in /usr/local/bin/ffmpeg /usr/bin/ffmpeg /bin/ffmpeg; do
    if [ -x "$path" ]; then
        echo "✓ Found: $path"
        "$path" -version | head -1
        echo ""
        
        # Check for QSV encoder
        if "$path" -encoders 2>&1 | grep -q av1_qsv; then
            echo "✓ av1_qsv encoder available"
        else
            echo "⚠ av1_qsv encoder not found (may be normal for static builds)"
        fi
        echo ""
    fi
done

# Check PATH
if command -v ffmpeg &> /dev/null; then
    echo "✓ FFmpeg found in PATH: $(which ffmpeg)"
    ffmpeg -version | head -1
else
    echo "⚠ FFmpeg not in PATH"
    echo "  Add to PATH: export PATH=\"/usr/local/bin:\$PATH\""
    echo "  Or create symlink: sudo ln -s /usr/local/bin/ffmpeg /usr/bin/ffmpeg"
fi

