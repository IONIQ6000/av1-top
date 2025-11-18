# Quick Config Guide - Change Watched Directory

## Option 1: Create Config File (Recommended)

```bash
# Create config directory
sudo mkdir -p /etc/av1janitor

# Create/edit config file (replace /path/to/your/media with your actual directory)
sudo nano /etc/av1janitor/config.toml
```

Add this content (replace `/path/to/your/media` with your actual directory):

```toml
# Directories to watch for media files
watched_directories = [
    "/path/to/your/media"
]

# Minimum file size in bytes (default: 2 GiB)
min_file_size_bytes = 2147483648

# Size gate factor (default: 0.9 = 90%)
size_gate_factor = 0.9

# File extensions to consider
media_extensions = ["mkv", "mp4", "avi", "m4v", "mov"]

# QSV quality settings
[qsv_quality]
below_1080p = 25
at_1080p = 24
at_1440p_and_above = 23
```

Then update systemd service to use the config:

```bash
sudo nano /etc/systemd/system/av1janitor.service
```

Change the `ExecStart` line to:
```
ExecStart=/usr/bin/av1d --config /etc/av1janitor/config.toml --concurrent 2
```

Reload and restart:
```bash
sudo systemctl daemon-reload
sudo systemctl restart av1janitor
```

## Option 2: Pass Directory via CLI (Quick Fix)

Edit the systemd service file:

```bash
sudo nano /etc/systemd/system/av1janitor.service
```

Change the `ExecStart` line to include `--directory`:
```
ExecStart=/usr/bin/av1d --directory /path/to/your/media --concurrent 2
```

Reload and restart:
```bash
sudo systemctl daemon-reload
sudo systemctl restart av1janitor
```

## Option 3: Multiple Directories

If you want to watch multiple directories, use the config file:

```toml
watched_directories = [
    "/media/movies",
    "/media/tv",
    "/media/downloads"
]
```

## Verify It's Working

Check the logs to see which directories are being watched:

```bash
sudo journalctl -u av1janitor -f
```

You should see lines like:
```
Scanning directory: /path/to/your/media
Found X media files in /path/to/your/media
```

