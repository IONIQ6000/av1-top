# 🚀 INSTALL IN 30 SECONDS

## The Absolute Simplest Way to Install

### On Debian/Ubuntu:

```bash
sudo ./install.sh
```

**That's it.** One fucking file. One fucking command. Everything gets installed.

---

## What You Get

After running that one command:

✅ FFmpeg 8.0+ with Intel QSV  
✅ Intel GPU drivers  
✅ Rust compiler  
✅ AV1 Janitor daemon  
✅ Monitoring TUI  
✅ Systemd service  
✅ Config files  
✅ Everything configured  

---

## What It Does (Automatically)

1. Updates your system
2. Installs Intel GPU drivers
3. Installs FFmpeg 8.0+ (with QSV)
4. Installs Rust if you don't have it
5. Builds AV1 Janitor
6. Installs to `/usr/local/bin`
7. Creates service user with GPU access
8. Sets up systemd service
9. Creates config files
10. Verifies everything works

**Time: ~5-10 minutes** (depending on your internet speed)

---

## After Installation

### 1. Edit Config (Set Your Media Directories)
```bash
sudo nano /etc/av1janitor/config.toml
```

Change this line:
```toml
watched_directories = ["/media/movies", "/media/tv"]
```

### 2. Start the Service
```bash
sudo systemctl start av1janitor
```

### 3. Monitor It
```bash
av1top
```

**Done!** Your files are being transcoded.

---

## Alternative: Debian Package

If you want a .deb package:

```bash
# Build package (if building from source)
./build-deb.sh

# Install the .deb file (NOT executable!)
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f  # Install dependencies if needed
```

**IMPORTANT:** `.deb` files are NOT executable scripts! Use `dpkg -i` to install them.

---

## Troubleshooting

**"Permission denied"**
```bash
chmod +x install.sh
sudo ./install.sh
```

**"FFmpeg not found"**
```bash
# The script auto-downloads it, but if it fails:
sudo apt install ffmpeg
```

**"GPU not detected"**
```bash
# Check your GPU
lspci | grep VGA
# Check drivers
vainfo
```

**Service not starting**
```bash
# Check logs
sudo journalctl -u av1janitor -n 50

# Test manually
av1d --once --dry-run -vv
```

---

## Uninstall

### If Installed via Script
```bash
sudo systemctl stop av1janitor
sudo systemctl disable av1janitor
sudo rm /usr/local/bin/av1d
sudo rm /usr/local/bin/av1top
sudo rm /etc/systemd/system/av1janitor.service
sudo userdel av1janitor
```

### If Installed via .deb
```bash
sudo apt remove av1janitor
# Or purge to remove config
sudo apt purge av1janitor
```

---

## TLDR

**To Install:**
```bash
sudo ./install.sh
```

**To Configure:**
```bash
sudo nano /etc/av1janitor/config.toml
```

**To Run:**
```bash
sudo systemctl start av1janitor
```

**To Monitor:**
```bash
av1top
```

**That's all you need to know!** 🎉

---

## Requirements

- Debian 11+ or Ubuntu 22.04+
- Intel GPU (11th gen+ or Arc A-series)
- Root access
- Internet connection

---

## Files Created

After installation:

```
/usr/local/bin/av1d              # Daemon binary
/usr/local/bin/av1top            # Monitor binary
/etc/av1janitor/config.toml      # Configuration
/etc/systemd/system/av1janitor.service  # Service
/var/lib/av1janitor/jobs/        # Job data
/var/log/av1janitor/             # Logs
```

---

**ONE SCRIPT. EVERYTHING INSTALLED. READY TO GO.** ✅

