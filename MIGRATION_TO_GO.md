# Migration from Rust to Go

This document outlines the migration of the AV1 Janitor project from Rust to Go, using Bubble Tea for the TUI.

## Overview

The project is being rewritten in Go to:
- Use Bubble Tea for a more modern TUI framework
- Leverage Go's simplicity and ecosystem
- Maintain the same functionality and features

## Architecture

### Project Structure

```
av1-top/
├── cmd/
│   ├── av1d/          # Daemon binary
│   └── av1top/         # TUI binary
├── internal/
│   ├── config/         # Configuration management
│   ├── transcode/      # Transcoding logic
│   ├── heuristics/     # File analysis and decisions
│   ├── persistence/    # Job state management
│   └── ffmpeg/         # FFmpeg integration
├── pkg/
│   └── tui/            # TUI components (Bubble Tea)
└── go.mod

```

## Key Changes

### TUI Framework: Ratatui → Bubble Tea

- **Ratatui (Rust)**: Low-level terminal UI library
- **Bubble Tea (Go)**: High-level TUI framework with Elm architecture

Bubble Tea provides:
- Component-based architecture
- Built-in state management
- Better event handling
- More modern API

### Language: Rust → Go

- Simpler syntax and faster compilation
- Excellent concurrency with goroutines
- Strong standard library
- Good FFmpeg integration options

## Migration Status

This is a work in progress. The Go version will maintain feature parity with the Rust version.

