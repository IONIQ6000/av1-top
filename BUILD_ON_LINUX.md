# ⚠️ IMPORTANT: Build the .deb Package on Linux!

## The Problem

The `.deb` package was built on macOS, which creates **macOS binaries**, not Linux binaries. This causes:
```
bash: /usr/bin/av1top: cannot execute binary file: Exec format error
```

## ✅ Solution: Build on Your Linux System

**You already have the source code on your Linux system!** Just build it there:

```bash
# On your Linux system (you're already in ~/av1-top)

# 1. Make sure Rust is installed
rustc --version
# If not installed:
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# source ~/.cargo/env

# 2. Build release binaries (this creates Linux binaries!)
cargo build --release --workspace

# 3. Build the .deb package
./build-deb.sh

# 4. Install the package
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f

# 5. Test it
av1top
```

**That's it!** The binaries will be proper Linux binaries that work on your system.

---

## Why This Happens

- **macOS** uses Mach-O binary format
- **Linux** uses ELF binary format
- Binaries compiled on macOS won't run on Linux (and vice versa)

---

## Quick Commands

```bash
# Build everything
cargo build --release --workspace && ./build-deb.sh

# Install
sudo dpkg -i av1janitor_0.1.0_amd64.deb && sudo apt-get install -f

# Test
av1top
```

---

## Verify It Works

After building on Linux:

```bash
# Check binary type
file /usr/bin/av1top
# Should show: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked...

# Run it
av1top
# Should show the TUI!
```

---

**Build on Linux = Linux binaries = Works perfectly!** ✅

