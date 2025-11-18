# Installing the .deb Package

## ⚠️ IMPORTANT: .deb files are NOT executable!

A `.deb` file is a **Debian package archive**, not an executable script. You cannot run it with `./filename.deb`.

## ✅ Correct Installation Method

### Step 1: Install the Package

```bash
sudo dpkg -i av1janitor_0.1.0_amd64.deb
```

This will:
- Extract and install all files
- Run post-install scripts
- Create the service user
- Set up directories and permissions
- Configure systemd service

### Step 2: Install Dependencies (if needed)

If you see dependency errors, run:

```bash
sudo apt-get install -f
```

This will install any missing dependencies (like `libc6`, `libgcc-s1`, etc.).

### Step 3: Configure

Edit the configuration file:

```bash
sudo nano /etc/av1janitor/config.toml
```

Set your media directories:
```toml
watched_directories = ["/media/movies", "/media/tv"]
```

### Step 4: Start the Service

```bash
sudo systemctl start av1janitor
sudo systemctl enable av1janitor  # Enable on boot (optional)
```

### Step 5: Monitor

```bash
av1top
```

---

## 🔍 What Happens During Installation

When you run `dpkg -i`, the package:

1. **Extracts files** to their locations:
   - `/usr/local/bin/av1d` - Daemon binary
   - `/usr/local/bin/av1top` - TUI monitor
   - `/etc/av1janitor/config.toml.example` - Example config
   - `/etc/systemd/system/av1janitor.service` - Service file

2. **Runs post-install script** (`postinst`):
   - Creates `av1janitor` service user
   - Adds user to `render` and `video` groups (for GPU access)
   - Creates directories (`/opt/av1janitor`, `/var/lib/av1janitor`, etc.)
   - Sets permissions
   - Copies example config to active config (if it doesn't exist)
   - Reloads systemd

3. **Reports success** with next steps

---

## ❌ Common Mistakes

### ❌ WRONG: Trying to execute the .deb file
```bash
./av1janitor_0.1.0_amd64.deb  # ERROR!
chmod +x av1janitor_0.1.0_amd64.deb  # WRONG!
```

### ✅ CORRECT: Use dpkg
```bash
sudo dpkg -i av1janitor_0.1.0_amd64.deb
```

---

## 🐛 Troubleshooting

### "dpkg: error processing package"
```bash
# Fix broken dependencies
sudo apt-get install -f
```

### "Permission denied"
```bash
# Must run as root
sudo dpkg -i av1janitor_0.1.0_amd64.deb
```

### "Dependency problems"
```bash
# Install missing dependencies
sudo apt-get update
sudo apt-get install -f
```

### "Package already installed"
```bash
# Remove old version first
sudo dpkg -r av1janitor
# Or upgrade
sudo dpkg -i av1janitor_0.1.0_amd64.deb
```

### Service won't start
```bash
# Check status
sudo systemctl status av1janitor

# Check logs
sudo journalctl -u av1janitor -n 50

# Verify config exists
ls -la /etc/av1janitor/config.toml

# Test manually
sudo -u av1janitor av1d --once --dry-run -vv
```

---

## 📦 Package Information

### View Package Contents
```bash
dpkg -L av1janitor
```

### Check Package Status
```bash
dpkg -l | grep av1janitor
```

### View Package Info
```bash
dpkg -I av1janitor_0.1.0_amd64.deb
```

### Extract Package (without installing)
```bash
dpkg -x av1janitor_0.1.0_amd64.deb /tmp/extracted
```

---

## 🗑️ Uninstalling

### Remove Package (keeps config)
```bash
sudo dpkg -r av1janitor
```

### Purge Package (removes config too)
```bash
sudo dpkg -P av1janitor
```

### Or use apt
```bash
sudo apt remove av1janitor      # Remove package
sudo apt purge av1janitor       # Remove package + config
```

---

## ✅ Verification Checklist

After installation, verify:

- [ ] Package installed: `dpkg -l | grep av1janitor`
- [ ] Binaries exist: `which av1d av1top`
- [ ] Config exists: `ls /etc/av1janitor/config.toml`
- [ ] Service file exists: `ls /etc/systemd/system/av1janitor.service`
- [ ] User created: `id av1janitor`
- [ ] Service can start: `sudo systemctl start av1janitor`

---

## 📝 Summary

**To install:**
```bash
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f
```

**To configure:**
```bash
sudo nano /etc/av1janitor/config.toml
```

**To start:**
```bash
sudo systemctl start av1janitor
```

**To monitor:**
```bash
av1top
```

**That's it!** 🎉

