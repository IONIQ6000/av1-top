package tui

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/IONIQ6000/av1-top/internal/persistence"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/disk"
	"github.com/shirou/gopsutil/v3/mem"
)

type Model struct {
	width  int
	height int

	// System metrics
	cpuUsage    float64
	memUsage    float64
	memTotal    uint64
	memUsed     uint64
	ioReadMB    float64
	ioWriteMB   float64
	gpuUsage        float64
	gpuMemoryMB     uint64
	gpuMemoryUsedMB uint64

	// Jobs
	jobs        []*persistence.Job
	jobsDir     string
	lastReload  time.Time

	// Update ticker
	ticker *time.Ticker
	
	// Commands
	cmds []tea.Cmd
}

func NewModel() Model {
	jobsDir := "/var/lib/av1janitor/jobs"
	if _, err := os.Stat(jobsDir); os.IsNotExist(err) {
		// Fallback to home directory
		home, _ := os.UserHomeDir()
		jobsDir = filepath.Join(home, ".local/share/av1janitor/jobs")
	}

	return Model{
		jobsDir:    jobsDir,
		lastReload: time.Now(),
		ticker:     time.NewTicker(time.Second),
	}
}

func (m Model) Init() tea.Cmd {
	return tea.Batch(
		m.updateSystemMetrics(),
		m.reloadJobs(),
		m.tick(),
	)
}

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		return m, nil

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			m.ticker.Stop()
			return m, tea.Quit
		case "r":
			return m, m.reloadJobs()
		}

	case tickMsg:
		var cmds []tea.Cmd
		
		// Update system metrics every second
		cmds = append(cmds, m.updateSystemMetrics())
		
		// Reload jobs every 2 seconds
		if time.Since(m.lastReload) > 2*time.Second {
			cmds = append(cmds, m.reloadJobs())
		}
		
		cmds = append(cmds, m.tick())
		return m, tea.Batch(cmds...)

	case systemMetricsMsg:
		m.cpuUsage = msg.cpuUsage
		m.memUsage = msg.memUsage
		m.memTotal = msg.memTotal
		m.memUsed = msg.memUsed
		m.ioReadMB = msg.ioReadMB
		m.ioWriteMB = msg.ioWriteMB
		m.gpuUsage = msg.gpuUsage
		m.gpuMemoryMB = msg.gpuMemoryMB
		m.gpuMemoryUsedMB = msg.gpuMemoryUsedMB
		return m, nil

	case jobsMsg:
		m.jobs = msg.jobs
		m.lastReload = time.Now()
		return m, nil

	}

	return m, nil
}

func (m Model) View() string {
	if m.width == 0 {
		return "Loading..."
	}

	// Header
	header := m.renderHeader()

	// System stats
	stats := m.renderSystemStats()

	// Current job
	currentJob := m.renderCurrentJob()

	// Jobs table
	jobsTable := m.renderJobsTable()

	// Footer
	footer := m.renderFooter()

	// Combine everything
	return lipgloss.JoinVertical(
		lipgloss.Left,
		header,
		stats,
		currentJob,
		jobsTable,
		footer,
	)
}

func (m Model) renderHeader() string {
	stats := m.getQueueStats()
	
	// Use adaptive colors that work across all terminals
	// Lipgloss automatically adapts to terminal color support
	titleStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.AdaptiveColor{Light: "#00d4ff", Dark: "#00ffff"}). // Cyan
		Background(lipgloss.AdaptiveColor{Light: "#ffffff", Dark: "#262626"})

	// Functional colors - adaptive for all terminals
	queueStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#0055ff", Dark: "#00aaff"}). // Blue
		Bold(true)

	runningStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#ff8800", Dark: "#ffaa00"}). // Orange
		Bold(true)

	successStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#00aa00", Dark: "#00ff00"}). // Green
		Bold(true)

	failedStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#cc0000", Dark: "#ff0000"}). // Red
		Bold(true)

	skippedStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#666666", Dark: "#888888"}) // Gray

	separator := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#999999", Dark: "#555555"}).
		Render(" • ")

	header := fmt.Sprintf("%s%sQueue: %s%sRunning: %s%s✓ %s%s✗ %s%s⊘ %s",
		titleStyle.Render(" AV1 Janitor "),
		separator,
		queueStyle.Render(fmt.Sprintf("%d", stats.pending)),
		separator,
		runningStyle.Render(fmt.Sprintf("%d", stats.running)),
		separator,
		successStyle.Render(fmt.Sprintf("%d", stats.completed)),
		separator,
		failedStyle.Render(fmt.Sprintf("%d", stats.failed)),
		separator,
		skippedStyle.Render(fmt.Sprintf("%d", stats.skipped)),
	)

	return lipgloss.NewStyle().
		Width(m.width).
		Padding(0, 1).
		Background(lipgloss.AdaptiveColor{Light: "#ffffff", Dark: "#262626"}).
		Foreground(lipgloss.AdaptiveColor{Light: "#000000", Dark: "#ffffff"}).
		Render(header)
}

func (m Model) renderSystemStats() string {
	// CPU
	cpuBar := m.renderBar("CPU", m.cpuUsage, 80.0)
	cpuInfo := fmt.Sprintf("Usage: %.1f%%\nCores: %d", m.cpuUsage, getCPUCores())

	// GPU
	gpuBar := m.renderBar("GPU (Intel QSV)", m.gpuUsage, 80.0)
	gpuInfo := fmt.Sprintf("Usage: %.1f%%\nEncoder: Active", m.gpuUsage)

	// Memory
	memBar := m.renderBar("Memory", m.memUsage, 90.0)
	memInfo := fmt.Sprintf("Used: %s / %s\n%.1f%%", 
		formatBytes(m.memUsed), 
		formatBytes(m.memTotal),
		m.memUsage)

	// I/O
	ioInfo := fmt.Sprintf("Read:  %.1f MB/s\nWrite: %.1f MB/s", m.ioReadMB, m.ioWriteMB)

	// Modern panel styling with subtle borders - adaptive colors
	panelStyle := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.AdaptiveColor{Light: "#cccccc", Dark: "#555555"}).
		Padding(1, 2).
		Width(m.width/4 - 3).
		Background(lipgloss.AdaptiveColor{Light: "#f5f5f5", Dark: "#1e1e1e"})

	infoStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#333333", Dark: "#e0e0e0"})

	cpuPanel := panelStyle.Render(fmt.Sprintf("%s\n%s", cpuBar, infoStyle.Render(cpuInfo)))
	gpuPanel := panelStyle.Render(fmt.Sprintf("%s\n%s", gpuBar, infoStyle.Render(gpuInfo)))
	memPanel := panelStyle.Render(fmt.Sprintf("%s\n%s", memBar, infoStyle.Render(memInfo)))
	ioPanel := panelStyle.Render(fmt.Sprintf("%s\n%s", 
		lipgloss.NewStyle().Bold(true).Foreground(lipgloss.AdaptiveColor{Light: "#000000", Dark: "#ffffff"}).Render("I/O"),
		infoStyle.Render(ioInfo)))

	return lipgloss.JoinHorizontal(lipgloss.Top, cpuPanel, gpuPanel, memPanel, ioPanel)
}

func (m Model) renderBar(title string, value, warnThreshold float64) string {
	barWidth := 24
	filled := int((value / 100.0) * float64(barWidth))
	if filled > barWidth {
		filled = barWidth
	}

	// Functional color coding: green -> yellow -> red based on thresholds
	var color lipgloss.AdaptiveColor
	if value < warnThreshold*0.6 {
		color = lipgloss.AdaptiveColor{Light: "#00aa00", Dark: "#00ff00"} // Green - healthy
	} else if value < warnThreshold {
		color = lipgloss.AdaptiveColor{Light: "#ffaa00", Dark: "#ffaa00"} // Yellow - getting high
	} else {
		color = lipgloss.AdaptiveColor{Light: "#cc0000", Dark: "#ff0000"} // Red - critical
	}

	// Modern bar with gradient effect
	filledBar := strings.Repeat("█", filled)
	emptyBar := strings.Repeat("░", barWidth-filled)
	bar := lipgloss.NewStyle().Foreground(color).Render(filledBar) +
		lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#cccccc", Dark: "#555555"}).Render(emptyBar)
	
	titleStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.AdaptiveColor{Light: "#000000", Dark: "#ffffff"})
	
	return fmt.Sprintf("%s\n%s", titleStyle.Render(title), bar)
}

func (m Model) renderCurrentJob() string {
	// Find running job
	var runningJob *persistence.Job
	for _, job := range m.jobs {
		if job.Status == persistence.StatusRunning {
			runningJob = job
			break
		}
	}

	panelStyle := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.AdaptiveColor{Light: "#cccccc", Dark: "#555555"}).
		Padding(1, 2).
		Background(lipgloss.AdaptiveColor{Light: "#f5f5f5", Dark: "#1e1e1e"})

	titleStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.AdaptiveColor{Light: "#ff8800", Dark: "#ffaa00"}) // Orange

	if runningJob == nil {
		return panelStyle.Render(fmt.Sprintf("%s\n%s",
			titleStyle.Render("Current Transcode"),
			lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#666666", Dark: "#888888"}).Render("No active transcoding job")))
	}

	filename := runningJob.FilePath
	if filename == "" {
		filename = "(unknown)"
	} else {
		filename = filepath.Base(filename)
	}

	statusStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#ff8800", Dark: "#ffaa00"}).
		Bold(true)

	infoStyle := lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#333333", Dark: "#e0e0e0"})
	info := fmt.Sprintf("File: %s\nStatus: %s\nStarted: %s",
		infoStyle.Render(filename),
		statusStyle.Render(string(runningJob.Status)),
		infoStyle.Render(runningJob.CreatedAt))

	return panelStyle.Render(fmt.Sprintf("%s\n%s", titleStyle.Render("Current Transcode"), info))
}

func (m Model) renderJobsTable() string {
	panelStyle := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.AdaptiveColor{Light: "#cccccc", Dark: "#555555"}).
		Padding(1, 2).
		Background(lipgloss.AdaptiveColor{Light: "#f5f5f5", Dark: "#1e1e1e"})

	titleStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.AdaptiveColor{Light: "#0055ff", Dark: "#00ffff"}) // Cyan

	if len(m.jobs) == 0 {
		return panelStyle.Render(fmt.Sprintf("%s\n%s",
			titleStyle.Render("Transcode History"),
			lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#666666", Dark: "#888888"}).Render("No jobs found")))
	}

	headerStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.AdaptiveColor{Light: "#000000", Dark: "#ffffff"}).
		Underline(true)

	var rows []string
	rows = append(rows, headerStyle.Render("Status │ File │ Created"))
	rows = append(rows, lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#cccccc", Dark: "#555555"}).Render(strings.Repeat("─", m.width-8)))

	// Sort jobs: running first, then by CreatedAt (newest first)
	sortedJobs := make([]*persistence.Job, len(m.jobs))
	copy(sortedJobs, m.jobs)
	
	// Simple sort: running jobs first, then by CreatedAt descending
	for i := 0; i < len(sortedJobs)-1; i++ {
		for j := i + 1; j < len(sortedJobs); j++ {
			// Running jobs come first
			if sortedJobs[j].Status == persistence.StatusRunning && sortedJobs[i].Status != persistence.StatusRunning {
				sortedJobs[i], sortedJobs[j] = sortedJobs[j], sortedJobs[i]
			} else if sortedJobs[i].Status == sortedJobs[j].Status {
				// Same status: newer first
				if sortedJobs[j].CreatedAt > sortedJobs[i].CreatedAt {
					sortedJobs[i], sortedJobs[j] = sortedJobs[j], sortedJobs[i]
				}
			}
		}
	}

	// Show first 10 jobs (running jobs will be first)
	maxShow := 10
	if len(sortedJobs) < maxShow {
		maxShow = len(sortedJobs)
	}

	rowStyle := lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#333333", Dark: "#e0e0e0"})
	separator := lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#cccccc", Dark: "#555555"}).Render(" │ ")

	for i := 0; i < maxShow; i++ {
		job := sortedJobs[i]
		var statusColor lipgloss.AdaptiveColor
		var statusSymbol string
		switch job.Status {
		case persistence.StatusComplete:
			statusColor = lipgloss.AdaptiveColor{Light: "#00aa00", Dark: "#00ff00"} // Green
			statusSymbol = "✓"
		case persistence.StatusFailed:
			statusColor = lipgloss.AdaptiveColor{Light: "#cc0000", Dark: "#ff0000"} // Red
			statusSymbol = "✗"
		case persistence.StatusRunning:
			statusColor = lipgloss.AdaptiveColor{Light: "#ff8800", Dark: "#ffaa00"} // Orange
			statusSymbol = "▶"
		case persistence.StatusPending:
			statusColor = lipgloss.AdaptiveColor{Light: "#0055ff", Dark: "#00aaff"} // Blue
			statusSymbol = "○"
		case persistence.StatusSkipped:
			statusColor = lipgloss.AdaptiveColor{Light: "#666666", Dark: "#888888"} // Gray
			statusSymbol = "⊘"
		default:
			statusColor = lipgloss.AdaptiveColor{Light: "#666666", Dark: "#888888"}
			statusSymbol = "•"
		}

		status := lipgloss.NewStyle().
			Foreground(statusColor).
			Bold(true).
			Render(fmt.Sprintf("%s %s", statusSymbol, string(job.Status)))

		filename := job.FilePath
		if filename == "" {
			filename = "(unknown)"
		} else {
			filename = filepath.Base(filename)
		}
		if len(filename) > 40 {
			filename = filename[:37] + "..."
		}

		rows = append(rows, fmt.Sprintf("%s%s%s%s%s",
			status,
			separator,
			rowStyle.Render(filename),
			separator,
			rowStyle.Render(job.CreatedAt)))
	}

	return panelStyle.Render(fmt.Sprintf("%s\n%s",
		titleStyle.Render("Transcode History"),
		strings.Join(rows, "\n")))
}

func (m Model) renderFooter() string {
	stats := m.getQueueStats()
	
	keyStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#ff8800", Dark: "#ffaa00"}).
		Bold(true)
	
	textStyle := lipgloss.NewStyle().
		Foreground(lipgloss.AdaptiveColor{Light: "#666666", Dark: "#888888"})
	
	message := fmt.Sprintf("%s %s %s %s",
		keyStyle.Render("q"),
		textStyle.Render("quit"),
		keyStyle.Render("r"),
		textStyle.Render("refresh"))
	
	if stats.pending == 0 && stats.running == 0 {
		waitStyle := lipgloss.NewStyle().
			Foreground(lipgloss.AdaptiveColor{Light: "#0055ff", Dark: "#00aaff"}).
			Italic(true)
		separator := lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "#999999", Dark: "#555555"}).Render(" • ")
		message = waitStyle.Render("Waiting for jobs...") + separator + message
	}

	return lipgloss.NewStyle().
		Width(m.width).
		Padding(0, 1).
		Background(lipgloss.AdaptiveColor{Light: "#ffffff", Dark: "#262626"}).
		Render(message)
}

type queueStats struct {
	pending   int
	running   int
	completed int
	failed    int
	skipped   int
}

func (m Model) getQueueStats() queueStats {
	var stats queueStats
	for _, job := range m.jobs {
		switch job.Status {
		case persistence.StatusPending:
			stats.pending++
		case persistence.StatusRunning:
			stats.running++
		case persistence.StatusComplete:
			stats.completed++
		case persistence.StatusFailed:
			stats.failed++
		case persistence.StatusSkipped:
			stats.skipped++
		}
	}
	return stats
}

// Messages
type tickMsg struct{}
type systemMetricsMsg struct {
	cpuUsage        float64
	memUsage        float64
	memTotal        uint64
	memUsed         uint64
	ioReadMB        float64
	ioWriteMB       float64
	gpuUsage        float64
	gpuMemoryMB     uint64
	gpuMemoryUsedMB uint64
}
type jobsMsg struct{ jobs []*persistence.Job }

func (m Model) tick() tea.Cmd {
	return func() tea.Msg {
		time.Sleep(time.Second)
		return tickMsg{}
	}
}

func (m Model) updateSystemMetrics() tea.Cmd {
	return func() tea.Msg {
		// CPU
		cpuPercent, _ := cpu.Percent(time.Second, false)
		cpuUsage := 0.0
		if len(cpuPercent) > 0 {
			cpuUsage = cpuPercent[0]
		}

		// Memory
		memInfo, _ := mem.VirtualMemory()
		memUsage := memInfo.UsedPercent
		memTotal := memInfo.Total
		memUsed := memInfo.Used

		// I/O (simplified - would need more sophisticated tracking)
		ioReadMB := 0.0
		ioWriteMB := 0.0
		diskIO, _ := disk.IOCounters()
		for _, io := range diskIO {
			ioReadMB += float64(io.ReadBytes) / 1024 / 1024
			ioWriteMB += float64(io.WriteBytes) / 1024 / 1024
		}

		// GPU metrics using intel_gpu_top or sysfs
		gpuUsage := 0.0
		gpuMemoryMB := uint64(0)      // Total VRAM capacity
		gpuMemoryUsedMB := uint64(0)   // Used VRAM
		
		// Method 1: Try intel_gpu_top (requires CAP_PERFMON, may not work)
		// intel_gpu_top doesn't support -n flag, use timeout wrapper
		cmd := exec.Command("sh", "-c", "timeout 1s intel_gpu_top -l -s 1000 2>/dev/null | head -20 || true")
		output, err := cmd.Output()
		if err == nil && len(output) > 0 && !strings.Contains(string(output), "Permission denied") {
			// Parse intel_gpu_top output
			// Output format varies, try multiple patterns
			lines := strings.Split(string(output), "\n")
			for _, line := range lines {
				line = strings.TrimSpace(line)
				// Look for various patterns
				if strings.Contains(line, "Render/3D") || strings.Contains(line, "render") {
					// Try to find percentage in the line
					parts := strings.Fields(line)
					for _, part := range parts {
						if strings.HasSuffix(part, "%") {
							if usage, err := strconv.ParseFloat(strings.TrimSuffix(part, "%"), 64); err == nil {
								if usage > gpuUsage {
									gpuUsage = usage
								}
							}
						}
					}
				}
				// Look for memory info
				if strings.Contains(strings.ToLower(line), "vram") || strings.Contains(strings.ToLower(line), "memory") {
					parts := strings.Fields(line)
					for i, part := range parts {
						if (strings.Contains(part, "MB") || strings.Contains(part, "MiB")) && i > 0 {
							// Extract number before MB/MiB
							numStr := strings.TrimSuffix(strings.TrimSuffix(part, "MB"), "MiB")
							if mem, err := strconv.ParseUint(numStr, 10, 64); err == nil {
								// Try to determine if this is used or total
								// Look for keywords like "used", "allocated", "free", "total"
								lineLower := strings.ToLower(line)
								if strings.Contains(lineLower, "used") || strings.Contains(lineLower, "allocated") {
									gpuMemoryUsedMB = mem
								} else if strings.Contains(lineLower, "total") || strings.Contains(lineLower, "capacity") {
									gpuMemoryMB = mem
								} else {
									// Default: assume it's total if we don't have one yet
									if gpuMemoryMB == 0 {
										gpuMemoryMB = mem
									}
								}
							}
						}
					}
				}
			}
		}
		
		// Method 2: Find Intel GPU card (i915 driver) and read frequency files
		// card0 might be ASpeed BMC, so we need to find the actual Intel GPU
		drmDir := "/sys/class/drm"
		entries, err := os.ReadDir(drmDir)
		if err == nil {
			for _, entry := range entries {
				if !strings.HasPrefix(entry.Name(), "card") {
					continue
				}
				
				// Check if this card uses i915 driver (Intel GPU)
				driverLink := filepath.Join(drmDir, entry.Name(), "device", "driver")
				if link, err := os.Readlink(driverLink); err == nil {
					driverName := filepath.Base(link)
					if driverName != "i915" {
						continue // Skip non-Intel GPUs
					}
					
					// Found Intel GPU, now find frequency files
					devicePath := filepath.Join(drmDir, entry.Name(), "device")
					if link, err := os.Readlink(devicePath); err == nil {
						// Resolve relative path
						absDevicePath := filepath.Join(drmDir, entry.Name(), link)
						absDevicePath = filepath.Clean(absDevicePath)
						
						// Try frequency files in various locations
						// Priority order: card-level files first, then device path, then gt subdirectories
						freqPaths := []string{
							// Card-level files (most direct)
							filepath.Join(drmDir, entry.Name(), "gt_min_freq_mhz"),
							filepath.Join(drmDir, entry.Name(), "gt_max_freq_mhz"),
							filepath.Join(drmDir, entry.Name(), "gt_RP0_freq_mhz"),
							filepath.Join(drmDir, entry.Name(), "gt_RPn_freq_mhz"),
							// In card's gt/gt0 subdirectory (RPS frequencies)
							filepath.Join(drmDir, entry.Name(), "gt", "gt0", "rps_min_freq_mhz"),
							filepath.Join(drmDir, entry.Name(), "gt", "gt0", "rps_max_freq_mhz"),
							filepath.Join(drmDir, entry.Name(), "gt", "gt0", "rps_RP0_freq_mhz"),
							filepath.Join(drmDir, entry.Name(), "gt", "gt0", "rps_RPn_freq_mhz"),
							// Device path files
							filepath.Join(absDevicePath, "gt_min_freq_mhz"),
							filepath.Join(absDevicePath, "gt_max_freq_mhz"),
							filepath.Join(absDevicePath, "gt_RP0_freq_mhz"),
							filepath.Join(absDevicePath, "gt_RPn_freq_mhz"),
							// In device's gt subdirectory
							filepath.Join(absDevicePath, "gt", "min_freq_mhz"),
							filepath.Join(absDevicePath, "gt", "max_freq_mhz"),
						}
						
						var minFreq, maxFreq float64
						for _, freqFile := range freqPaths {
							if data, err := os.ReadFile(freqFile); err == nil {
								if freq, err := strconv.ParseFloat(strings.TrimSpace(string(data)), 64); err == nil && freq > 0 {
									if strings.Contains(freqFile, "min") || strings.Contains(freqFile, "RPn") {
										if minFreq == 0 || freq < minFreq {
											minFreq = freq
										}
									} else if strings.Contains(freqFile, "max") || strings.Contains(freqFile, "RP0") {
										if freq > maxFreq {
											maxFreq = freq
										}
									}
								}
							}
						}
						
						// Try to find current frequency
						if maxFreq > 0 && minFreq > 0 {
							curFreqPaths := []string{
								// Card-level files (most direct)
								filepath.Join(drmDir, entry.Name(), "gt_cur_freq_mhz"),
								filepath.Join(drmDir, entry.Name(), "gt_act_freq_mhz"),
								// In card's gt/gt0 subdirectory (RPS current frequency)
								filepath.Join(drmDir, entry.Name(), "gt", "gt0", "rps_cur_freq_mhz"),
								filepath.Join(drmDir, entry.Name(), "gt", "gt0", "rps_act_freq_mhz"),
								// Device path files
								filepath.Join(absDevicePath, "gt_cur_freq_mhz"),
								filepath.Join(absDevicePath, "gt_act_freq_mhz"),
								filepath.Join(absDevicePath, "gt", "cur_freq_mhz"),
								filepath.Join(absDevicePath, "gt", "act_freq_mhz"),
							}
							curFreq := minFreq // Default to min
							for _, freqFile := range curFreqPaths {
								if data, err := os.ReadFile(freqFile); err == nil {
									if freq, err := strconv.ParseFloat(strings.TrimSpace(string(data)), 64); err == nil && freq > 0 {
										curFreq = freq
										break
									}
								}
							}
							
							// Calculate usage as percentage of frequency range
							if maxFreq > minFreq {
								usage := ((curFreq - minFreq) / (maxFreq - minFreq)) * 100.0
								if usage > gpuUsage {
									gpuUsage = usage
								}
								if gpuUsage > 100 {
									gpuUsage = 100
								}
							}
						}
						
						// Read GPU memory from sysfs (discrete Intel GPU)
						if gpuMemoryMB == 0 {
							// For discrete Intel GPUs, check memory regions
							// Memory info might be in gt/gt0/memory_regions or similar
							gt0Path := filepath.Join(drmDir, entry.Name(), "gt", "gt0")
							
							// Check for memory region directories
							gt0Entries, err := os.ReadDir(gt0Path)
							if err == nil {
								for _, gtEntry := range gt0Entries {
									if strings.HasPrefix(gtEntry.Name(), "memory_region") || strings.HasPrefix(gtEntry.Name(), "region") {
										regionPath := filepath.Join(gt0Path, gtEntry.Name())
										// Check for size file in region
										sizeFile := filepath.Join(regionPath, "size")
										if data, err := os.ReadFile(sizeFile); err == nil {
											if bytes, err := strconv.ParseUint(strings.TrimSpace(string(data)), 10, 64); err == nil && bytes > 0 {
												gpuMemoryMB += bytes / (1024 * 1024) // Convert bytes to MB
											}
										}
										// Also check for total_size or similar
										totalSizeFile := filepath.Join(regionPath, "total_size")
										if data, err := os.ReadFile(totalSizeFile); err == nil {
											if bytes, err := strconv.ParseUint(strings.TrimSpace(string(data)), 10, 64); err == nil && bytes > 0 {
												gpuMemoryMB += bytes / (1024 * 1024)
											}
										}
									}
								}
							}
							
							// Try card-level memory files
							if gpuMemoryMB == 0 {
								memPaths := []string{
									filepath.Join(drmDir, entry.Name(), "mem_info_vram_total"),
									filepath.Join(drmDir, entry.Name(), "gt", "gt0", "meminfo"),
									filepath.Join(drmDir, entry.Name(), "gt", "gt0", "memory"),
									filepath.Join(absDevicePath, "mem_info_vram_total"),
									filepath.Join(absDevicePath, "drm", entry.Name(), "gt", "gt0", "meminfo"),
								}
								
								for _, memFile := range memPaths {
									if data, err := os.ReadFile(memFile); err == nil {
										content := strings.TrimSpace(string(data))
										// Try parsing as bytes (convert to MB)
										if bytes, err := strconv.ParseUint(content, 10, 64); err == nil && bytes > 0 {
											gpuMemoryMB = bytes / (1024 * 1024)
											break
										}
										// Try parsing as MB directly
										if strings.HasSuffix(content, "MB") || strings.HasSuffix(content, "MiB") {
											numStr := strings.TrimSuffix(strings.TrimSuffix(content, "MB"), "MiB")
											if mem, err := strconv.ParseUint(strings.TrimSpace(numStr), 10, 64); err == nil && mem > 0 {
												gpuMemoryMB = mem
												break
											}
										}
									}
								}
							}
							
							// Try reading from PCI sysfs (no lspci needed)
							if gpuMemoryMB == 0 {
								// Read from PCI device resource file (text format)
								// /sys/bus/pci/devices/0000:XX:XX.X/resource
								devicePath := filepath.Join(drmDir, entry.Name(), "device")
								// Use EvalSymlinks to properly resolve all symlinks
								resolvedPath, err := filepath.EvalSymlinks(devicePath)
								if err == nil {
									// Read the text resource file (not binary resource0, resource1, etc.)
									resourceFile := filepath.Join(resolvedPath, "resource")
									if data, err := os.ReadFile(resourceFile); err == nil {
										// Format: one line per resource, "0xSTART 0xEND 0xFLAGS"
										lines := strings.Split(string(data), "\n")
										for _, line := range lines {
											line = strings.TrimSpace(line)
											if line == "" {
												continue
											}
											parts := strings.Fields(line)
											if len(parts) >= 2 {
												start, err1 := strconv.ParseUint(strings.TrimPrefix(parts[0], "0x"), 16, 64)
												end, err2 := strconv.ParseUint(strings.TrimPrefix(parts[1], "0x"), 16, 64)
												if err1 == nil && err2 == nil && end > start {
													// Check if this is a memory region (not I/O)
													if len(parts) >= 3 {
														flags, err3 := strconv.ParseUint(strings.TrimPrefix(parts[2], "0x"), 16, 64)
														if err3 == nil {
															// Bit 0 = I/O space (not memory), bit 1 = prefetchable
															// We want memory regions (bit 0 = 0)
															if flags&0x1 == 0 {
																size := end - start + 1
																// Only count large memory regions (likely VRAM)
																// Small regions are usually registers
																if size >= 1024*1024*1024 { // At least 1GB
																	gpuMemoryMB += size / (1024 * 1024)
																}
															}
														}
													}
												}
											}
										}
									}
								}
							}
							
							// Try reading from lspci output as final fallback (if available)
							if gpuMemoryMB == 0 {
								// Check if lspci exists
								if _, err := exec.LookPath("lspci"); err == nil {
									// Find GPU PCI device
									lspciCmd := exec.Command("sh", "-c", "lspci | grep -i 'vga\\|display\\|3d' | grep -i intel | head -1 | cut -d' ' -f1")
									if pciAddr, err := lspciCmd.Output(); err == nil {
										pciAddrStr := strings.TrimSpace(string(pciAddr))
										if pciAddrStr != "" {
											cmd := exec.Command("lspci", "-v", "-s", pciAddrStr)
											output, err := cmd.Output()
											if err == nil {
												lines := strings.Split(string(output), "\n")
												for _, line := range lines {
													if strings.Contains(strings.ToLower(line), "memory") && (strings.Contains(line, "MiB") || strings.Contains(line, "size=")) {
														// Parse "Memory at ... [size=8G]" or "Memory: ... [size=8192M]"
														parts := strings.Fields(line)
														for _, part := range parts {
															if strings.Contains(part, "size=") {
																sizeStr := strings.TrimPrefix(part, "size=")
																// Remove trailing ] if present
																sizeStr = strings.TrimSuffix(sizeStr, "]")
																// Check if it's in GB
																if strings.HasSuffix(sizeStr, "G") {
																	if gb, err := strconv.ParseUint(strings.TrimSuffix(sizeStr, "G"), 10, 64); err == nil {
																		gpuMemoryMB = gb * 1024
																		break
																	}
																} else if strings.HasSuffix(sizeStr, "M") {
																	if mb, err := strconv.ParseUint(strings.TrimSuffix(sizeStr, "M"), 10, 64); err == nil {
																		gpuMemoryMB = mb
																		break
																	}
																}
															}
														}
														if gpuMemoryMB > 0 {
															break
														}
													}
												}
											}
										}
									}
								}
							}
						}
						
						// Found Intel GPU, no need to check other cards
						break
					}
				}
			}
		}
		
		// Method 3: Try reading from /proc (if available)
		// Some systems expose GPU stats in /proc
		if gpuUsage == 0 {
			procPaths := []string{
				"/proc/driver/i915/gt/cur_freq_mhz",
				"/proc/driver/i915/gt_min_freq_mhz",
			}
			for _, procFile := range procPaths {
				if data, err := os.ReadFile(procFile); err == nil {
					if freq, err := strconv.ParseFloat(strings.TrimSpace(string(data)), 64); err == nil && freq > 0 {
						// Try to get max for normalization
						maxProcFile := strings.Replace(procFile, "cur_freq", "max_freq", 1)
						maxProcFile = strings.Replace(maxProcFile, "min_freq", "max_freq", 1)
						maxFreq := 1200.0
						if maxData, err := os.ReadFile(maxProcFile); err == nil {
							if mf, err := strconv.ParseFloat(strings.TrimSpace(string(maxData)), 64); err == nil && mf > 0 {
								maxFreq = mf
							}
						}
						usage := (freq / maxFreq) * 100.0
						if usage > gpuUsage {
							gpuUsage = usage
						}
						if gpuUsage > 100 {
							gpuUsage = 100
						}
					}
				}
			}
		}
		
		// GPU memory is already parsed from intel_gpu_top if available
		// If still 0, we can't easily get it from sysfs without parsing complex files

		return systemMetricsMsg{
			cpuUsage:        cpuUsage,
			memUsage:        memUsage,
			memTotal:        memTotal,
			memUsed:         memUsed,
			ioReadMB:        ioReadMB,
			ioWriteMB:        ioWriteMB,
			gpuUsage:        gpuUsage,
			gpuMemoryMB:     gpuMemoryMB,
			gpuMemoryUsedMB: gpuMemoryUsedMB,
		}
	}
}

func (m Model) reloadJobs() tea.Cmd {
	return func() tea.Msg {
		jobs, err := persistence.LoadJobs(m.jobsDir)
		if err != nil {
			// Return empty jobs on error
			return jobsMsg{jobs: []*persistence.Job{}}
		}
		return jobsMsg{jobs: jobs}
	}
}


// Helper functions
func formatBytes(bytes uint64) string {
	const unit = 1024
	if bytes < unit {
		return fmt.Sprintf("%d B", bytes)
	}
	div, exp := int64(unit), 0
	for n := bytes / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %cB", float64(bytes)/float64(div), "KMGTPE"[exp])
}

func getCPUCores() int {
	count, _ := cpu.Counts(true)
	return count
}
