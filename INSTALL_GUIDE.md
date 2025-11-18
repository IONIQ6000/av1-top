# Installation Guide - Dead Simple

## 🚀 ONE-LINE INSTALL (Debian/Ubuntu)

### Method 1: From Source (Recommended)

```bash
# Clone and run installer
git clone https://github.com/yourusername/rust-av1.git
cd rust-av1
sudo ./install.sh
```

That's it! The script does EVERYTHING:
- ✅ Installs FFmpeg 8.0+ with QSV
- ✅ Installs Intel GPU drivers
- ✅ Installs Rust (if needed)
- ✅ Builds AV1 Janitor
- ✅ Installs binaries
- ✅ Creates service user
- ✅ Sets up systemd
- ✅ Configures permissions

### Method 2: Debian Package

```bash
# Build package
./build-deb.sh

# Install
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f  # Install dependencies
```

---

## 📋 What The Installer Does

### Step 1: System Update
Updates apt package lists

### Step 2: Intel GPU Drivers
Installs:
- intel-media-va-driver-non-free
- vainfo
- libva2
- All required GPU libraries

### Step 3: FFmpeg 8.0+
- Tries system FFmpeg first
- Falls back to static build if needed
- Verifies av1_qsv encoder exists

### Step 4: Rust
- Checks if Rust is installed
- Installs via rustup if needed
- Sets up PATH correctly

### Step 5: Build
- Builds in release mode
- Optimized binaries

### Step 6: Install Binaries
- Copies to /usr/local/bin
- av1d (3.5 MB)
- av1top (1.3 MB)

### Step 7: Create User
- Creates av1janitor service user
- Adds to render/video groups for GPU access

### Step 8: Configuration
- Creates /etc/av1janitor/config.toml
- Prompts for media directory
- Sets sensible defaults

### Step 9: Systemd Service
- Installs /etc/systemd/system/av1janitor.service
- Configures auto-restart
- Sets resource limits

### Step 10: Verification
- Tests all components
- Checks FFmpeg
- Checks GPU
- Reports status

---

## ⚡ Quick Start After Install

```bash
# 1. Edit config (set your media directories)
sudo nano /etc/av1janitor/config.toml

# 2. Start service
sudo systemctl start av1janitor

# 3. Monitor
av1top

# 4. Check logs
sudo journalctl -u av1janitor -f
```

---

## 🎯 Requirements

### Minimum
- **OS**: Debian 11+ or Ubuntu 22.04+
- **CPU**: Any (for control)
- **GPU**: Intel 11th gen+ or Arc A-series (for QSV)
- **RAM**: 2 GB minimum
- **Disk**: 100 MB for binaries + space for transcoding

### Recommended
- **OS**: Ubuntu 24.04 LTS
- **GPU**: Intel Arc A380 or better
- **RAM**: 8 GB
- **Disk**: Fast SSD

---

## 🐛 Troubleshooting

### Install Script Fails

**"FFmpeg not found"**
```bash
# Install manually
sudo apt install ffmpeg
# Or download static build
wget https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz
```

**"GPU not detected"**
```bash
# Check GPU
lspci | grep VGA
# Install drivers
sudo apt install intel-media-va-driver-non-free
# Verify
vainfo
```

**"Build failed"**
```bash
# Check Rust
rustc --version
# Rebuild manually
cargo build --release --workspace
```

### Service Won't Start

```bash
# Check status
sudo systemctl status av1janitor

# Check logs
sudo journalctl -u av1janitor -n 50

# Verify config
cat /etc/av1janitor/config.toml

# Test manually
sudo -u av1janitor av1d --once --dry-run -vv
```

### Permission Issues

```bash
# Verify user groups
groups av1janitor

# Add to groups manually
sudo usermod -a -G render,video av1janitor

# Check GPU access
sudo -u av1janitor vainfo
```

---

## 🔧 Manual Installation (Advanced)

If the script doesn't work for your system:

### 1. Install Dependencies
```bash
sudo apt update
sudo apt install -y \
    ffmpeg \
    intel-media-va-driver-non-free \
    vainfo \
    build-essential \
    pkg-config
```

### 2. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 3. Build
```bash
cd rust-av1
cargo build --release --workspace
```

### 4. Install
```bash
sudo cp target/release/av1d /usr/local/bin/
sudo cp target/release/av1top /usr/local/bin/
sudo cp av1janitor.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### 5. Configure
```bash
sudo mkdir -p /etc/av1janitor
sudo cp config.example.toml /etc/av1janitor/config.toml
sudo nano /etc/av1janitor/config.toml
```

---

## 📦 Debian Package Details

### Package Contents
```
/usr/local/bin/av1d              # Daemon
/usr/local/bin/av1top            # TUI monitor
/etc/av1janitor/config.toml      # Configuration
/etc/systemd/system/av1janitor.service  # Service file
/usr/share/doc/av1janitor/       # Documentation
```

### Package Info
```bash
# View package info
dpkg -l | grep av1janitor

# List files
dpkg -L av1janitor

# Remove
sudo apt remove av1janitor

# Purge (remove config too)
sudo apt purge av1janitor
```

---

## 🎊 Success Checklist

After installation, verify:

- [ ] FFmpeg 8.0+ installed: `ffmpeg -version`
- [ ] QSV encoder available: `ffmpeg -encoders | grep av1_qsv`
- [ ] GPU accessible: `vainfo`
- [ ] Binaries installed: `which av1d av1top`
- [ ] Service user exists: `id av1janitor`
- [ ] Config file exists: `ls /etc/av1janitor/config.toml`
- [ ] Service file exists: `ls /etc/systemd/system/av1janitor.service`

If all checked: **You're ready to go!** 🎉

---

## 📞 Support

If you encounter issues:

1. Check the logs: `sudo journalctl -u av1janitor -n 100`
2. Test manually: `av1d --once --dry-run -vvv`
3. Verify GPU: `vainfo`
4. Check permissions: `groups av1janitor`
5. Review config: `cat /etc/av1janitor/config.toml`

---

**Install time: ~5-10 minutes**  
**One script does it all!** ✅

