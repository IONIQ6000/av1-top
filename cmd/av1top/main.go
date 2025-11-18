package main

import (
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/IONIQ6000/av1-top/pkg/tui"
)

func main() {
	// Ensure TERM supports colors if not already set to a color-capable terminal
	if term := os.Getenv("TERM"); term != "" {
		// If TERM doesn't indicate color support, try to upgrade it
		if term == "xterm" || term == "vt100" || term == "dumb" {
			// Try xterm-256color which is widely supported
			if os.Getenv("COLORTERM") == "" {
				os.Setenv("TERM", "xterm-256color")
			}
		}
	} else {
		// TERM not set, default to xterm-256color
		os.Setenv("TERM", "xterm-256color")
	}

	model := tui.NewModel()
	p := tea.NewProgram(model, tea.WithAltScreen())

	if _, err := p.Run(); err != nil {
		fmt.Printf("Error running TUI: %v\n", err)
		os.Exit(1)
	}
}

