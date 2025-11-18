# Go Migration Guide

This project is being migrated from Rust to Go, using [Bubble Tea](https://github.com/charmbracelet/bubbletea) for the TUI.

## Project Structure

```
av1-top/
├── cmd/
│   ├── av1d/          # Daemon binary (Go)
│   └── av1top/        # TUI binary (Go + Bubble Tea)
├── internal/
│   ├── config/        # Configuration management
│   ├── transcode/     # Transcoding logic
│   ├── heuristics/    # File analysis and decisions
│   ├── persistence/   # Job state management
│   └── ffmpeg/        # FFmpeg integration
├── pkg/
│   └── tui/           # TUI components (Bubble Tea)
├── go.mod             # Go module definition
└── Cargo.toml         # Rust workspace (legacy)
```

## Key Changes

### TUI Framework: Ratatui → Bubble Tea

**Ratatui (Rust)**:
- Low-level terminal UI library
- Manual state management
- Direct terminal control

**Bubble Tea (Go)**:
- High-level TUI framework
- Elm architecture (Model-Update-View)
- Built-in state management
- Component-based design
- Better event handling

### Language: Rust → Go

**Benefits**:
- Simpler syntax
- Faster compilation
- Excellent concurrency (goroutines)
- Strong standard library
- Good FFmpeg integration

## Getting Started

### Prerequisites

1. Install Go 1.21 or later:
   ```bash
   # macOS
   brew install go
   
   # Linux
   sudo apt-get install golang-go
   ```

2. Install dependencies:
   ```bash
   go mod download
   ```

### Building

```bash
# Build daemon
go build -o av1d ./cmd/av1d

# Build TUI
go build -o av1top ./cmd/av1top
```

### Running

```bash
# Run daemon
./av1d --config /etc/av1janitor/config.toml

# Run TUI
./av1top
```

## Migration Status

### ✅ Completed
- [x] Go module setup (`go.mod`)
- [x] Basic project structure
- [x] Configuration management (`internal/config`)
- [x] FFmpeg manager (`internal/ffmpeg`)
- [x] Job persistence (`internal/persistence`)
- [x] Basic TUI skeleton (`pkg/tui`)

### 🚧 In Progress
- [ ] Full TUI implementation with Bubble Tea
- [ ] Transcoding logic port
- [ ] Heuristics port
- [ ] File watching
- [ ] System metrics display

### 📋 TODO
- [ ] Port all Rust functionality to Go
- [ ] Implement full Bubble Tea TUI
- [ ] System metrics (CPU, GPU, I/O)
- [ ] Job queue display
- [ ] Real-time updates
- [ ] Error handling
- [ ] Testing

## Bubble Tea Resources

- [Bubble Tea Documentation](https://github.com/charmbracelet/bubbletea)
- [Bubble Tea Tutorial](https://github.com/charmbracelet/bubbletea/blob/master/tutorials/basics/README.md)
- [Examples](https://github.com/charmbracelet/bubbletea/tree/master/examples)

## Next Steps

1. Complete the TUI implementation using Bubble Tea components
2. Port transcoding logic from Rust
3. Implement file watching and job management
4. Add system metrics display
5. Test and validate against Rust version

