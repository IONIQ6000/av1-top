# Cross-Compilation Guide for Linux .deb Package

## Problem

The `.deb` package was built on macOS, creating macOS binaries instead of Linux binaries. This causes "Exec format error" on Linux systems.

## Solution: Build on Linux

**The easiest solution is to build the package directly on your Linux system.**

### On Your Linux System:

```bash
# 1. Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Clone the repository
git clone https://github.com/IONIQ6000/av1-top.git
cd av1-top

# 3. Build the package
./build-deb.sh

# 4. Install
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f
```

---

## Alternative: Cross-Compile from macOS

If you want to build Linux binaries from macOS, you need to cross-compile:

### 1. Install Cross-Compilation Target

```bash
rustup target add x86_64-unknown-linux-gnu
```

### 2. Install Cross-Compilation Tools

```bash
# Install via Homebrew
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu

# Or use cross crate
cargo install cross --git https://github.com/cross-rs/cross
```

### 3. Build for Linux

```bash
# Using cross
cross build --release --target x86_64-unknown-linux-gnu

# Or using cargo with linker
CC_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-gcc \
cargo build --release --target x86_64-unknown-linux-gnu
```

### 4. Update build-deb.sh

The build script would need to:
- Check if building on macOS
- Use cross-compilation if needed
- Copy binaries from `target/x86_64-unknown-linux-gnu/release/` instead of `target/release/`

---

## Recommended: Build on Linux

**The simplest and most reliable solution is to build directly on your Linux system.**

The build script (`build-deb.sh`) will work perfectly on Linux and create proper Linux binaries.

---

## Quick Fix for Your Current System

Since you already have the source on your Linux system:

```bash
# On your Linux system (av1-top)

# 1. Build release binaries
cargo build --release --workspace

# 2. Build the package
./build-deb.sh

# 3. Install
sudo dpkg -i av1janitor_0.1.0_amd64.deb
sudo apt-get install -f

# 4. Test
av1top
```

This will create proper Linux binaries that will work on your system!

