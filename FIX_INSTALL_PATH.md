# Fixed: Installation Path Issue

## Problem

The error `unable to create '/usr/local/bin/av1d.dpkg-new': No such file or directory` was caused by the package trying to install to `/usr/local/bin`, which may not exist or may have incorrect permissions on some systems.

## Root Cause

- Debian packages should install to `/usr/bin` (system-managed)
- `/usr/local/bin` is for manual installations by system administrators
- The package was incorrectly using `/usr/local/bin`

## Solution

Updated `build-deb.sh` and `av1janitor.service` to use `/usr/bin` instead of `/usr/local/bin`.

### Changes Made:

1. **build-deb.sh**: Changed binary installation path
   ```bash
   # OLD: debian-package/.../usr/local/bin/
   # NEW: debian-package/.../usr/bin/
   ```

2. **av1janitor.service**: Updated ExecStart path
   ```ini
   # OLD: ExecStart=/usr/local/bin/av1d
   # NEW: ExecStart=/usr/bin/av1d
   ```

## Installation Now Works

The rebuilt package installs binaries to:
- `/usr/bin/av1d` (daemon)
- `/usr/bin/av1top` (TUI monitor)

These paths are standard for Debian packages and will work on all systems.

## Reinstall Instructions

On your Linux system:

```bash
# 1. Clean up any failed installation
sudo dpkg -r av1janitor 2>/dev/null || true

# 2. Pull latest package
git pull

# 3. Install the fixed package
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f

# 4. Verify installation
which av1d av1top
# Should show: /usr/bin/av1d and /usr/bin/av1top

# 5. Check service
sudo systemctl daemon-reload
sudo systemctl status av1janitor

# 6. Configure and start
sudo nano /etc/av1janitor/config.toml
sudo systemctl start av1janitor

# 7. Monitor
av1top
```

## Status

✅ **FIXED** - Package now uses standard Debian paths (`/usr/bin`)

