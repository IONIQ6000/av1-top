#!/bin/bash
# Build script for Go version

set -e

echo "Building Go binaries..."

# Build TUI
echo "Building av1top (TUI)..."
go build -o av1top-go ./cmd/av1top
echo "✓ Built: av1top-go"

# Build Daemon
echo "Building av1d (daemon)..."
go build -o av1d-go ./cmd/av1d
echo "✓ Built: av1d-go"

echo ""
echo "Binaries built:"
echo "  - av1top-go (TUI)"
echo "  - av1d-go (daemon)"
echo ""
echo "Run with:"
echo "  ./av1top-go"
echo "  ./av1d-go --config /etc/av1janitor/config.toml"

