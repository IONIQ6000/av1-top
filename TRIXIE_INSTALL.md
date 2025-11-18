# Fresh Installation on Debian Trixie

This is a **one-command installation** that sets up everything from scratch.

## What It Installs

1. **Non-free repositories** (for Intel drivers)
2. **Intel GPU drivers** (`intel-media-va-driver-non-free`, `libvpl2`, VAAPI)
3. **FFmpeg 8 static build** from GitHub (BtbN/FFmpeg-Builds)
4. **Rust toolchain** (if not already installed)
5. **AV1 Janitor** (builds from source)
6. **Systemd service** (configured and ready)

## Installation

Run this **one command** as root or with sudo:

```bash
curl -fsSL https://raw.githubusercontent.com/IONIQ6000/av1-top/main/FRESH_INSTALL_TRIXIE.sh | sudo bash
```

Or download and run:

```bash
wget https://raw.githubusercontent.com/IONIQ6000/av1-top/main/FRESH_INSTALL_TRIXIE.sh
chmod +x FRESH_INSTALL_TRIXIE.sh
sudo ./FRESH_INSTALL_TRIXIE.sh
```

Or if you already have the repo:

```bash
cd ~/av1-top
git pull
sudo bash FRESH_INSTALL_TRIXIE.sh
```

## After Installation

The script will prompt you to:

1. **Edit the config** to set your media directories:
   ```bash
   sudo nano /etc/av1janitor/config.toml
   ```
   
   Change:
   ```toml
   watched_directories = ["/main-library-2/Media/Movies"]
   ```

2. **Start the service**:
   ```bash
   sudo systemctl start av1janitor
   sudo systemctl enable av1janitor  # Auto-start on boot
   ```

3. **Monitor the daemon**:
   ```bash
   # Watch logs
   sudo journalctl -u av1janitor -f
   
   # Or use the TUI
   av1top
   ```

## Troubleshooting

If you're in an **LXC container**, ensure:

1. GPU passthrough is configured on the host
2. `/dev/dri` devices are accessible in the container
3. The `av1janitor` user is in the `video` and `render` groups (script does this)

Check GPU access:
```bash
ls -la /dev/dri/
sudo -u av1janitor vainfo
```

Check service status:
```bash
sudo systemctl status av1janitor
sudo journalctl -u av1janitor -n 50
```

## What Gets Installed

- **Binaries**: `/usr/bin/av1d`, `/usr/bin/av1top`
- **Config**: `/etc/av1janitor/config.toml`
- **Job data**: `/var/lib/av1janitor/jobs/`
- **Service**: `/etc/systemd/system/av1janitor.service`
- **FFmpeg**: `/usr/local/bin/ffmpeg` (symlinked to `/usr/bin/ffmpeg`)

## Clean Reinstall

If you need to completely reinstall:

```bash
# Stop and remove service
sudo systemctl stop av1janitor
sudo systemctl disable av1janitor
sudo rm /etc/systemd/system/av1janitor.service

# Remove binaries and data
sudo rm /usr/bin/av1d /usr/bin/av1top
sudo rm -rf /var/lib/av1janitor
sudo rm -rf /etc/av1janitor

# Remove user
sudo userdel av1janitor

# Then run the install script again
sudo bash FRESH_INSTALL_TRIXIE.sh
```

