package ffmpeg

import (
	"fmt"
	"os/exec"
	"strings"
)

type Manager struct {
	ffmpegPath  string
	ffprobePath string
	version     string
}

func NewManager() (*Manager, error) {
	// Try to find ffmpeg in PATH
	ffmpegPath, err := exec.LookPath("ffmpeg")
	if err != nil {
		return nil, fmt.Errorf("ffmpeg not found in PATH: %w", err)
	}

	// Get version
	cmd := exec.Command(ffmpegPath, "-version")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("failed to run ffmpeg: %w", err)
	}

	version := extractVersion(string(output))

	// Find ffprobe
	ffprobePath, err := exec.LookPath("ffprobe")
	if err != nil {
		return nil, fmt.Errorf("ffprobe not found: %w", err)
	}

	return &Manager{
		ffmpegPath:  ffmpegPath,
		ffprobePath: ffprobePath,
		version:     version,
	}, nil
}

func (m *Manager) Path() string {
	return m.ffmpegPath
}

func (m *Manager) Version() string {
	return m.version
}

func extractVersion(output string) string {
	lines := strings.Split(output, "\n")
	if len(lines) > 0 {
		return strings.TrimSpace(lines[0])
	}
	return "unknown"
}

