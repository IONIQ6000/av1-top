# Quick FFmpeg 8.0+ Installation

## One-Line Install

```bash
# Download and run the script
curl -sSL https://raw.githubusercontent.com/IONIQ6000/av1-top/main/install_ffmpeg8.sh | sudo bash

# Or if you have the repo:
cd av1-top
sudo bash install_ffmpeg8.sh
```

## What It Does

1. **Checks** if FFmpeg 8.0+ is already installed
2. **Downloads** static build from johnvansickle.com (fastest)
3. **Extracts** and installs to `/usr/local/bin`
4. **Verifies** installation and QSV encoder support
5. **Falls back** to building from source if download fails

## After Installation

```bash
# Verify
ffmpeg -version

# Check for QSV encoder
ffmpeg -encoders | grep av1_qsv

# Restart daemon
sudo systemctl restart av1janitor

# Check status
sudo systemctl status av1janitor
```

## Manual Installation (Alternative)

If the script doesn't work:

```bash
# Download static build manually
cd /tmp
wget https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz
tar xf ffmpeg-git-amd64-static.tar.xz
cd ffmpeg-git-*-static

# Install
sudo cp ffmpeg /usr/local/bin/
sudo cp ffprobe /usr/local/bin/
sudo chmod +x /usr/local/bin/ffmpeg
sudo chmod +x /usr/local/bin/ffprobe

# Verify
ffmpeg -version
```

## Troubleshooting

**"Command not found" after installation:**
```bash
# Add to PATH (if needed)
export PATH="/usr/local/bin:$PATH"

# Or create symlink
sudo ln -s /usr/local/bin/ffmpeg /usr/bin/ffmpeg
sudo ln -s /usr/local/bin/ffprobe /usr/bin/ffprobe
```

**"av1_qsv encoder not found":**
- Static builds may not include QSV support
- Build from source with Intel GPU drivers installed
- Or use system FFmpeg if it has QSV support

**"Permission denied":**
```bash
# Make sure binaries are executable
sudo chmod +x /usr/local/bin/ffmpeg
sudo chmod +x /usr/local/bin/ffprobe

# Check av1janitor user can access
sudo -u av1janitor /usr/local/bin/ffmpeg -version
```

## Build Time

- **Static build download**: ~2-5 minutes
- **Build from source**: ~10-30 minutes (depending on CPU)

## Requirements

- Root access
- Internet connection
- ~500 MB disk space (static build)
- ~2 GB disk space (source build)

