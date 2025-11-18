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
	gpuUsage    float64
	gpuMemoryMB uint64

	// Jobs
	jobs        []*persistence.Job
	jobsDir     string
	lastReload  time.Time

	// Console logs
	consoleLogs []string
	lastLogFetch time.Time

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
		jobsDir:     jobsDir,
		lastReload:  time.Now(),
		lastLogFetch: time.Now(),
		ticker:      time.NewTicker(time.Second),
	}
}

func (m Model) Init() tea.Cmd {
	return tea.Batch(
		m.updateSystemMetrics(),
		m.reloadJobs(),
		m.fetchLogs(),
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
		
		// Fetch logs every 2 seconds
		if time.Since(m.lastLogFetch) > 2*time.Second {
			cmds = append(cmds, m.fetchLogs())
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
		return m, nil

	case jobsMsg:
		m.jobs = msg.jobs
		m.lastReload = time.Now()
		return m, nil

	case logsMsg:
		m.consoleLogs = msg.logs
		m.lastLogFetch = time.Now()
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

	// Console logs
	logs := m.renderLogs()

	// Footer
	footer := m.renderFooter()

	// Combine everything
	return lipgloss.JoinVertical(
		lipgloss.Left,
		header,
		stats,
		currentJob,
		jobsTable,
		logs,
		footer,
	)
}

func (m Model) renderHeader() string {
	stats := m.getQueueStats()
	
	titleStyle := lipgloss.NewStyle().
		Bold(true).
		Foreground(lipgloss.Color("86"))

	queueStyle := lipgloss.NewStyle().
		Foreground(lipgloss.Color("226"))

	runningStyle := lipgloss.NewStyle().
		Foreground(lipgloss.Color("46"))

	successStyle := lipgloss.NewStyle().
		Foreground(lipgloss.Color("46"))

	failedStyle := lipgloss.NewStyle().
		Foreground(lipgloss.Color("196"))

	skippedStyle := lipgloss.NewStyle().
		Foreground(lipgloss.Color("240"))

	header := fmt.Sprintf("%s │ Queue: %s │ Running: %s │ ✓ %s │ ✗ %s │ ⊘ %s",
		titleStyle.Render("AV1 Janitor"),
		queueStyle.Render(fmt.Sprintf("%d", stats.pending)),
		runningStyle.Render(fmt.Sprintf("%d", stats.running)),
		successStyle.Render(fmt.Sprintf("%d", stats.completed)),
		failedStyle.Render(fmt.Sprintf("%d", stats.failed)),
		skippedStyle.Render(fmt.Sprintf("%d", stats.skipped)),
	)

	border := strings.Repeat("─", m.width-2)
	return lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Render(fmt.Sprintf("%s\n%s", header, border))
}

func (m Model) renderSystemStats() string {
	// CPU
	cpuBar := m.renderBar("CPU", m.cpuUsage, 80.0)
	cpuInfo := fmt.Sprintf("Usage: %.1f%%\nCores: %d", m.cpuUsage, getCPUCores())

	// GPU
	gpuBar := m.renderBar("GPU (Intel QSV)", m.gpuUsage, 80.0)
	gpuInfo := fmt.Sprintf("Usage: %.1f%%\nVRAM: %d MB\nEncoder: Active", m.gpuUsage, m.gpuMemoryMB)

	// Memory
	memBar := m.renderBar("Memory", m.memUsage, 90.0)
	memInfo := fmt.Sprintf("Used: %s / %s\n%.1f%%", 
		formatBytes(m.memUsed), 
		formatBytes(m.memTotal),
		m.memUsage)

	// I/O
	ioInfo := fmt.Sprintf("Read:  %.1f MB/s\nWrite: %.1f MB/s", m.ioReadMB, m.ioWriteMB)

	cpuPanel := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Width(m.width/4 - 2).
		Render(fmt.Sprintf("%s\n%s", cpuBar, cpuInfo))

	gpuPanel := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Width(m.width/4 - 2).
		Render(fmt.Sprintf("%s\n%s", gpuBar, gpuInfo))

	memPanel := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Width(m.width/4 - 2).
		Render(fmt.Sprintf("%s\n%s", memBar, memInfo))

	ioPanel := lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Width(m.width/4 - 2).
		Render(fmt.Sprintf("I/O\n%s", ioInfo))

	return lipgloss.JoinHorizontal(lipgloss.Top, cpuPanel, gpuPanel, memPanel, ioPanel)
}

func (m Model) renderBar(title string, value, warnThreshold float64) string {
	barWidth := 20
	filled := int((value / 100.0) * float64(barWidth))
	if filled > barWidth {
		filled = barWidth
	}

	color := "46" // Green
	if value > warnThreshold {
		color = "196" // Red
	}

	bar := strings.Repeat("█", filled) + strings.Repeat("░", barWidth-filled)
	return fmt.Sprintf("%s\n%s", title, lipgloss.NewStyle().Foreground(lipgloss.Color(color)).Render(bar))
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

	if runningJob == nil {
		return lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("86")).
			Padding(0, 1).
			Render("Current Transcode\nNo active transcoding job")
	}

	info := fmt.Sprintf("File: %s\nStatus: %s\nStarted: %s",
		filepath.Base(runningJob.FilePath),
		runningJob.Status,
		runningJob.CreatedAt)

	return lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Render(fmt.Sprintf("Current Transcode\n%s", info))
}

func (m Model) renderJobsTable() string {
	if len(m.jobs) == 0 {
		return lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("86")).
			Padding(0, 1).
			Render("Transcode History\nNo jobs found")
	}

	var rows []string
	rows = append(rows, "Status │ File │ Created")
	rows = append(rows, strings.Repeat("─", m.width-4))

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

	for i := 0; i < maxShow; i++ {
		job := sortedJobs[i]
		statusColor := "240" // Gray
		switch job.Status {
		case persistence.StatusComplete:
			statusColor = "46" // Green
		case persistence.StatusFailed:
			statusColor = "196" // Red
		case persistence.StatusRunning:
			statusColor = "226" // Yellow
		case persistence.StatusPending:
			statusColor = "33" // Blue
		}

		status := lipgloss.NewStyle().Foreground(lipgloss.Color(statusColor)).Render(string(job.Status))
		filename := job.FilePath
		if filename == "" {
			filename = "(unknown)"
		} else {
			filename = filepath.Base(filename)
		}
		if len(filename) > 40 {
			filename = filename[:37] + "..."
		}

		rows = append(rows, fmt.Sprintf("%s │ %s │ %s", status, filename, job.CreatedAt))
	}

	return lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Render(strings.Join(rows, "\n"))
}

func (m Model) renderLogs() string {
	if len(m.consoleLogs) == 0 {
		return lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("86")).
			Padding(0, 1).
			Render("Console Logs\nNo logs available")
	}

	// Show last 5 log lines
	start := len(m.consoleLogs) - 5
	if start < 0 {
		start = 0
	}

	logs := m.consoleLogs[start:]
	return lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("86")).
		Padding(0, 1).
		Render(fmt.Sprintf("Console Logs\n%s", strings.Join(logs, "\n")))
}

func (m Model) renderFooter() string {
	stats := m.getQueueStats()
	message := "Press 'q' to quit, 'r' to refresh"
	if stats.pending == 0 && stats.running == 0 {
		message = "Waiting for jobs... | " + message
	}

	return lipgloss.NewStyle().
		Foreground(lipgloss.Color("241")).
		Padding(0, 1).
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
	cpuUsage    float64
	memUsage    float64
	memTotal    uint64
	memUsed     uint64
	ioReadMB    float64
	ioWriteMB   float64
	gpuUsage    float64
	gpuMemoryMB uint64
}
type jobsMsg struct{ jobs []*persistence.Job }
type logsMsg struct{ logs []string }

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

		// GPU metrics using intel_gpu_top
		gpuUsage := 0.0
		gpuMemoryMB := uint64(0)
		
		// Try to get GPU metrics from intel_gpu_top
		cmd := exec.Command("timeout", "1", "intel_gpu_top", "-l", "-n", "1")
		output, err := cmd.Output()
		if err == nil {
			// Parse intel_gpu_top output
			lines := strings.Split(string(output), "\n")
			for _, line := range lines {
				// Look for render/3D usage percentage
				if strings.Contains(line, "Render/3D") {
					// Parse percentage from line like "Render/3D:    5%"
					parts := strings.Fields(line)
					for i, part := range parts {
						if strings.HasSuffix(part, "%") {
							if usage, err := strconv.ParseFloat(strings.TrimSuffix(part, "%"), 64); err == nil {
								gpuUsage = usage
								break
							}
						}
						// Also check for memory usage
						if part == "VRAM" && i+1 < len(parts) {
							if memStr := parts[i+1]; strings.HasSuffix(memStr, "MB") {
								if mem, err := strconv.ParseUint(strings.TrimSuffix(memStr, "MB"), 10, 64); err == nil {
									gpuMemoryMB = mem
								}
							}
						}
					}
				}
			}
		}
		
		// Fallback: try reading from /sys/class/drm
		if gpuUsage == 0 {
			// Try to read GPU frequency as proxy for usage
			freqFile := "/sys/class/drm/card0/gt/cur_freq_mhz"
			if data, err := os.ReadFile(freqFile); err == nil {
				if freq, err := strconv.ParseFloat(strings.TrimSpace(string(data)), 64); err == nil {
					// Normalize to percentage (assuming max ~1200 MHz)
					gpuUsage = (freq / 1200.0) * 100.0
					if gpuUsage > 100 {
						gpuUsage = 100
					}
				}
			}
		}

		return systemMetricsMsg{
			cpuUsage:    cpuUsage,
			memUsage:    memUsage,
			memTotal:    memTotal,
			memUsed:     memUsed,
			ioReadMB:    ioReadMB,
			ioWriteMB:   ioWriteMB,
			gpuUsage:    gpuUsage,
			gpuMemoryMB: gpuMemoryMB,
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

func (m Model) fetchLogs() tea.Cmd {
	return func() tea.Msg {
		// Try journalctl first
		cmd := exec.Command("journalctl", "-u", "av1janitor", "-n", "10", "--no-pager")
		output, err := cmd.Output()
		if err == nil {
			lines := strings.Split(strings.TrimSpace(string(output)), "\n")
			// Keep only last 5 lines
			if len(lines) > 5 {
				lines = lines[len(lines)-5:]
			}
			return logsMsg{logs: lines}
		}

		// Fallback to log file
		logFile := "/var/log/av1janitor/av1d.log"
		data, err := os.ReadFile(logFile)
		if err != nil {
			return logsMsg{logs: []string{}}
		}

		lines := strings.Split(strings.TrimSpace(string(data)), "\n")
		if len(lines) > 5 {
			lines = lines[len(lines)-5:]
		}
		return logsMsg{logs: lines}
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
