# Installing Rust for macOS

To build this project, you need Rust installed. Here's how:

## Quick Install (Recommended)

```bash
# Install Rust using rustup (official installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then restart your terminal or run:
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## After Installation

Once Rust is installed, you can build the project:

```bash
cd /Users/obsidian/rust-av1
cargo build --workspace
```

## Using Homebrew (Alternative)

```bash
brew install rust
```

## Verify Installation

```bash
rustc --version  # Should show rustc 1.70+ or later
cargo --version  # Should show cargo 1.70+ or later
```

## Build the Project

```bash
# Build in debug mode (faster, includes debug info)
cargo build --workspace

# Build in release mode (optimized, for production)
cargo build --release --workspace

# Run tests
cargo test --workspace

# Check for errors without building
cargo check --workspace
```

## Troubleshooting

If you get "command not found" after installation:
1. Restart your terminal
2. Or run: `source ~/.cargo/env`
3. Or add to your `~/.zshrc`: `source ~/.cargo/env`

