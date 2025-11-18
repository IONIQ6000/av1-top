// AV1 Top - Comprehensive TUI monitor for system metrics and transcode jobs
// Displays detailed CPU, GPU, I/O, memory, disk, and transcode job information

use anyhow::Result;
use core::{JobStatus, PathsConfig, TranscodeJob};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run the app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// Application state with comprehensive metrics
struct App {
    /// System info provider
    sys: System,
    
    /// Last time we updated system info
    last_update: Instant,
    
    /// Last time we reloaded jobs
    last_job_reload: Instant,
    
    /// Transcode jobs loaded from disk
    jobs: Vec<TranscodeJob>,
    
    /// Configuration for paths
    paths_config: PathsConfig,
    
    /// Error loading jobs (if any)
    job_load_error: Option<String>,
    
    /// I/O statistics
    io_stats: IoStats,
    
    /// GPU statistics (Intel)
    gpu_stats: GpuStats,
    
    /// Network statistics (for remote access info)
    network_stats: NetworkStats,
}

/// I/O statistics
#[derive(Default)]
struct IoStats {
    read_bytes_per_sec: u64,
    write_bytes_per_sec: u64,
    #[allow(dead_code)]
    last_read_bytes: u64,
    #[allow(dead_code)]
    last_write_bytes: u64,
}

/// GPU statistics (Intel specific)
#[derive(Default)]
struct GpuStats {
    usage_percent: f32,
    memory_used_mb: u64,
    #[allow(dead_code)]
    memory_total_mb: u64,
    #[allow(dead_code)]
    temperature_c: f32,
    encoder_active: bool,
}

/// Network statistics
#[derive(Default)]
struct NetworkStats {
    #[allow(dead_code)]
    upload_kbps: f64,
    #[allow(dead_code)]
    download_kbps: f64,
}

impl App {
    fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_all();

        let paths_config = PathsConfig::default();

        // Try to load real jobs from disk
        let (jobs, job_load_error) = match core::load_all_jobs(&paths_config.jobs_dir) {
            Ok(loaded_jobs) => {
                if loaded_jobs.is_empty() {
                    (create_dummy_jobs(), Some("No job files found, showing demo data".to_string()))
                } else {
                    (loaded_jobs, None)
                }
            }
            Err(e) => {
                (create_dummy_jobs(), Some(format!("Error loading jobs: {}", e)))
            }
        };

        Self {
            sys,
            last_update: Instant::now(),
            last_job_reload: Instant::now(),
            jobs,
            paths_config,
            job_load_error,
            io_stats: IoStats::default(),
            gpu_stats: GpuStats::default(),
            network_stats: NetworkStats::default(),
        }
    }

    /// Update all metrics
    fn update(&mut self) {
        let now = Instant::now();

        // Update system info every second
        if now.duration_since(self.last_update) >= Duration::from_secs(1) {
            self.sys.refresh_cpu_all();
            self.sys.refresh_memory();
            self.update_io_stats();
            self.update_gpu_stats();
            self.update_network_stats();
            self.last_update = now;
        }

        // Reload jobs every 2 seconds
        if now.duration_since(self.last_job_reload) >= Duration::from_secs(2) {
            self.reload_jobs();
            self.last_job_reload = now;
        }
    }

    /// Reload jobs from disk
    fn reload_jobs(&mut self) {
        match core::load_all_jobs(&self.paths_config.jobs_dir) {
            Ok(loaded_jobs) => {
                if loaded_jobs.is_empty() {
                    if self.jobs.is_empty() {
                        self.jobs = create_dummy_jobs();
                        self.job_load_error = Some("No job files found, showing demo data".to_string());
                    }
                } else {
                    self.jobs = loaded_jobs;
                    self.job_load_error = None;
                }
            }
            Err(e) => {
                self.job_load_error = Some(format!("Error reloading jobs: {}", e));
            }
        }
    }

    /// Update I/O statistics
    fn update_io_stats(&mut self) {
        // Try to read /proc/diskstats on Linux for I/O metrics
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/proc/diskstats") {
                let mut total_read = 0u64;
                let mut total_write = 0u64;

                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 14 {
                        // Sectors read (5th field) and written (9th field)
                        if let (Ok(read), Ok(write)) = (parts[5].parse::<u64>(), parts[9].parse::<u64>()) {
                            total_read += read * 512; // sectors to bytes
                            total_write += write * 512;
                        }
                    }
                }

                if self.io_stats.last_read_bytes > 0 {
                    self.io_stats.read_bytes_per_sec = total_read.saturating_sub(self.io_stats.last_read_bytes);
                    self.io_stats.write_bytes_per_sec = total_write.saturating_sub(self.io_stats.last_write_bytes);
                }

                self.io_stats.last_read_bytes = total_read;
                self.io_stats.last_write_bytes = total_write;
            }
        }
    }

    /// Update GPU statistics (Intel specific)
    fn update_gpu_stats(&mut self) {
        // Try to read Intel GPU stats from sysfs
        #[cfg(target_os = "linux")]
        {
            // Intel GPU render usage
            if let Ok(content) = fs::read_to_string("/sys/class/drm/card0/gt_cur_freq_mhz") {
                if let Ok(freq) = content.trim().parse::<u32>() {
                    // Rough estimate: assume usage based on frequency
                    let max_freq = 2000; // Typical Intel GPU max freq
                    self.gpu_stats.usage_percent = (freq as f32 / max_freq as f32) * 100.0;
                    self.gpu_stats.encoder_active = freq > 500; // Active if freq > 500 MHz
                }
            }

            // Try reading GPU memory (if available)
            if let Ok(content) = fs::read_to_string("/sys/class/drm/card0/mem_info_vram_used") {
                if let Ok(used) = content.trim().parse::<u64>() {
                    self.gpu_stats.memory_used_mb = used / (1024 * 1024);
                }
            }
        }

        // macOS: No direct Intel GPU stats available
        #[cfg(target_os = "macos")]
        {
            // Set placeholder values
            self.gpu_stats.usage_percent = 0.0;
            self.gpu_stats.encoder_active = false;
        }
    }

    /// Update network statistics
    fn update_network_stats(&mut self) {
        // Placeholder for network stats
        // In a real implementation, would read from /proc/net/dev or use network crate
        self.network_stats.upload_kbps = 0.0;
        self.network_stats.download_kbps = 0.0;
    }

    /// Get currently running job
    fn get_running_job(&self) -> Option<&TranscodeJob> {
        self.jobs.iter().find(|j| j.status == JobStatus::Running)
    }

    /// Get queue statistics
    fn get_queue_stats(&self) -> QueueStats {
        let pending = self.jobs.iter().filter(|j| j.status == JobStatus::Pending).count();
        let running = self.jobs.iter().filter(|j| j.status == JobStatus::Running).count();
        let completed = self.jobs.iter().filter(|j| j.status == JobStatus::Success).count();
        let failed = self.jobs.iter().filter(|j| j.status == JobStatus::Failed).count();
        let skipped = self.jobs.iter().filter(|j| j.status == JobStatus::Skipped).count();

        QueueStats {
            pending,
            running,
            completed,
            failed,
            skipped,
            total: self.jobs.len(),
        }
    }
}

struct QueueStats {
    pending: usize,
    running: usize,
    completed: usize,
    failed: usize,
    skipped: usize,
    total: usize,
}

/// Main application loop
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        app.update();
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('r') => {
                            app.sys.refresh_all();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Draw the comprehensive UI
fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create main layout: Header, Stats, Jobs, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // System stats
            Constraint::Length(8),  // Current job details
            Constraint::Min(10),    // Jobs table
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Header
    draw_header(f, chunks[0], app);

    // System stats (CPU, GPU, Memory, I/O)
    draw_system_stats(f, chunks[1], app);

    // Current job details
    draw_current_job(f, chunks[2], app);

    // Jobs table
    draw_jobs_table(f, chunks[3], app);

    // Footer
    draw_footer(f, chunks[4], app);
}

/// Draw header with title and queue summary
fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let queue_stats = app.get_queue_stats();
    
    let title = vec![
        Line::from(vec![
            Span::styled("AV1 Janitor ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("│ "),
            Span::styled(format!("Queue: {}", queue_stats.pending), Style::default().fg(Color::Yellow)),
            Span::raw(" │ "),
            Span::styled(format!("Running: {}", queue_stats.running), Style::default().fg(Color::Green)),
            Span::raw(" │ "),
            Span::styled(format!("✓ {}", queue_stats.completed), Style::default().fg(Color::Green)),
            Span::raw(" │ "),
            Span::styled(format!("✗ {}", queue_stats.failed), Style::default().fg(Color::Red)),
            Span::raw(" │ "),
            Span::styled(format!("⊘ {}", queue_stats.skipped), Style::default().fg(Color::Gray)),
        ]),
    ];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL).title("AV1 Transcoding Monitor"));
    f.render_widget(header, area);
}

/// Draw comprehensive system statistics
fn draw_system_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // CPU
            Constraint::Percentage(25), // GPU
            Constraint::Percentage(25), // Memory
            Constraint::Percentage(25), // I/O
        ])
        .split(area);

    // CPU Panel
    draw_cpu_panel(f, chunks[0], app);

    // GPU Panel
    draw_gpu_panel(f, chunks[1], app);

    // Memory Panel
    draw_memory_panel(f, chunks[2], app);

    // I/O Panel
    draw_io_panel(f, chunks[3], app);
}

/// Draw CPU panel
fn draw_cpu_panel(f: &mut Frame, area: Rect, app: &App) {
    let cpu_usage = app.sys.global_cpu_usage() * 100.0;
    
    let lines = vec![
        Line::from(vec![
            Span::raw("Usage: "),
            Span::styled(
                format!("{:.1}%", cpu_usage),
                Style::default().fg(if cpu_usage > 80.0 { Color::Red } else { Color::Green })
            ),
        ]),
        Line::from(vec![
            Span::raw("Cores: "),
            Span::raw(format!("{}", app.sys.cpus().len())),
        ]),
        Line::from(""),
        Line::from(""),
    ];

    let gauge = Gauge::default()
        .block(Block::default().title("CPU").borders(Borders::ALL))
        .gauge_style(Style::default().fg(if cpu_usage > 80.0 { Color::Red } else { Color::Cyan }))
        .percent(cpu_usage.min(100.0) as u16);

    let inner = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(4)])
        .split(inner);

    f.render_widget(Block::default().borders(Borders::ALL).title("CPU"), area);
    f.render_widget(gauge, gauge_chunks[0]);
    f.render_widget(Paragraph::new(lines), gauge_chunks[1]);
}

/// Draw GPU panel
fn draw_gpu_panel(f: &mut Frame, area: Rect, app: &App) {
    let gpu_usage = app.gpu_stats.usage_percent;
    
    let lines = vec![
        Line::from(vec![
            Span::raw("Usage: "),
            Span::styled(
                format!("{:.1}%", gpu_usage),
                Style::default().fg(if gpu_usage > 80.0 { Color::Red } else { Color::Green })
            ),
        ]),
        Line::from(vec![
            Span::raw("VRAM: "),
            Span::raw(format!("{} MB", app.gpu_stats.memory_used_mb)),
        ]),
        Line::from(vec![
            Span::raw("Encoder: "),
            Span::styled(
                if app.gpu_stats.encoder_active { "Active" } else { "Idle" },
                Style::default().fg(if app.gpu_stats.encoder_active { Color::Green } else { Color::Gray })
            ),
        ]),
    ];

    let gauge = Gauge::default()
        .block(Block::default().title("GPU (Intel QSV)").borders(Borders::ALL))
        .gauge_style(Style::default().fg(if gpu_usage > 80.0 { Color::Red } else { Color::Magenta }))
        .percent(gpu_usage.min(100.0) as u16);

    let inner = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(4)])
        .split(inner);

    f.render_widget(Block::default().borders(Borders::ALL).title("GPU (Intel QSV)"), area);
    f.render_widget(gauge, gauge_chunks[0]);
    f.render_widget(Paragraph::new(lines), gauge_chunks[1]);
}

/// Draw memory panel
fn draw_memory_panel(f: &mut Frame, area: Rect, app: &App) {
    let mem_total = app.sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let mem_used = app.sys.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let mem_percent = if mem_total > 0.0 { (mem_used / mem_total * 100.0) } else { 0.0 };

    let swap_total = app.sys.total_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
    let swap_used = app.sys.used_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
    
    let lines = vec![
        Line::from(vec![
            Span::raw("Used: "),
            Span::styled(
                format!("{:.1} / {:.1} GiB", mem_used, mem_total),
                Style::default().fg(if mem_percent > 80.0 { Color::Red } else { Color::Green })
            ),
        ]),
        Line::from(vec![
            Span::raw("Swap: "),
            Span::raw(format!("{:.1} / {:.1} GiB", swap_used, swap_total)),
        ]),
    ];

    let gauge = Gauge::default()
        .block(Block::default().title("Memory").borders(Borders::ALL))
        .gauge_style(Style::default().fg(if mem_percent > 80.0 { Color::Red } else { Color::Green }))
        .percent(mem_percent.min(100.0) as u16);

    let inner = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(4)])
        .split(inner);

    f.render_widget(Block::default().borders(Borders::ALL).title("Memory"), area);
    f.render_widget(gauge, gauge_chunks[0]);
    f.render_widget(Paragraph::new(lines), gauge_chunks[1]);
}

/// Draw I/O panel
fn draw_io_panel(f: &mut Frame, area: Rect, app: &App) {
    let read_mbps = app.io_stats.read_bytes_per_sec as f64 / (1024.0 * 1024.0);
    let write_mbps = app.io_stats.write_bytes_per_sec as f64 / (1024.0 * 1024.0);
    
    let lines = vec![
        Line::from(vec![
            Span::raw("Read:  "),
            Span::styled(
                format!("{:.1} MB/s", read_mbps),
                Style::default().fg(Color::Cyan)
            ),
        ]),
        Line::from(vec![
            Span::raw("Write: "),
            Span::styled(
                format!("{:.1} MB/s", write_mbps),
                Style::default().fg(Color::Yellow)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Disks: "),
            Span::raw(format!("{}", count_disks())),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("I/O Stats").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn count_disks() -> usize {
    Disks::new_with_refreshed_list().list().len()
}

/// Draw current job details with progress and file size info
fn draw_current_job(f: &mut Frame, area: Rect, app: &App) {
    if let Some(job) = app.get_running_job() {
        let filename = job.source_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        let original_size = job.original_bytes.unwrap_or(0);
        let current_size = job.new_bytes.unwrap_or(0);
        
        // Estimate progress based on file size (rough estimate)
        let progress_percent = if original_size > 0 && current_size > 0 {
            ((current_size as f64 / original_size as f64) * 100.0).min(95.0)
        } else {
            0.0
        };

        // Calculate projected size (if we have current progress)
        let projected_size = if current_size > 0 {
            current_size // Will be updated with actual data
        } else {
            original_size
        };

        let ratio = if original_size > 0 {
            (projected_size as f64 / original_size as f64) * 100.0
        } else {
            100.0
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("FILE: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(filename),
            ]),
            Line::from(vec![
                Span::raw("Original:  "),
                Span::raw(format_bytes(original_size)),
                Span::raw("  │  Current: "),
                Span::styled(format_bytes(current_size), Style::default().fg(Color::Yellow)),
                Span::raw("  │  Ratio: "),
                Span::styled(
                    format!("{:.1}%", ratio),
                    Style::default().fg(if ratio <= 90.0 { Color::Green } else { Color::Red })
                ),
            ]),
            Line::from(vec![
                Span::raw("Duration: "),
                Span::raw(job.duration_string()),
                Span::raw("  │  Status: "),
                Span::styled("TRANSCODING", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
        ];

        let gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(Color::Green))
            .percent(progress_percent as u16);

        let inner = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(4)])
            .split(inner);

        f.render_widget(Block::default().borders(Borders::ALL).title("Current Transcode"), area);
        f.render_widget(gauge, chunks[0]);
        f.render_widget(Paragraph::new(lines), chunks[1]);
    } else {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No active transcodes",
                Style::default().fg(Color::Gray)
            )),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Current Transcode").borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }
}

/// Draw comprehensive jobs table
fn draw_jobs_table(f: &mut Frame, area: Rect, app: &App) {
    let headers = Row::new(vec![
        "STATUS",
        "FILE",
        "ORIGINAL",
        "OUTPUT",
        "SAVINGS",
        "RATIO",
        "DURATION",
        "REASON",
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let mut sorted_jobs: Vec<&TranscodeJob> = app.jobs.iter().collect();
    sorted_jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let rows: Vec<Row> = sorted_jobs
        .iter()
        .map(|job| {
            let status_style = match job.status {
                JobStatus::Success => Style::default().fg(Color::Green),
                JobStatus::Running => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                JobStatus::Failed => Style::default().fg(Color::Red),
                JobStatus::Skipped => Style::default().fg(Color::Yellow),
                JobStatus::Pending => Style::default().fg(Color::Gray),
            };

            let filename = job.source_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");

            let filename_display = if filename.len() > 20 {
                format!("{}...", &filename[..17])
            } else {
                filename.to_string()
            };

            let orig_size = job.original_bytes
                .map(|b| format_bytes(b))
                .unwrap_or_else(|| "N/A".to_string());

            let new_size = job.new_bytes
                .map(|b| format_bytes(b))
                .unwrap_or_else(|| "N/A".to_string());

            let savings = if let (Some(orig), Some(new)) = (job.original_bytes, job.new_bytes) {
                let saved = orig.saturating_sub(new) as f64 / (1024.0 * 1024.0 * 1024.0);
                format!("{:.2} GiB", saved)
            } else {
                "N/A".to_string()
            };

            let ratio = if let (Some(orig), Some(new)) = (job.original_bytes, job.new_bytes) {
                if orig > 0 {
                    format!("{:.1}%", (new as f64 / orig as f64) * 100.0)
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            let duration = job.duration_string();

            let reason = job.reason.as_ref()
                .map(|r| {
                    let s = r.0.as_str();
                    if s.len() > 15 {
                        format!("{}...", &s[..12])
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                Cell::from(format!("{}", job.status)),
                Cell::from(filename_display),
                Cell::from(orig_size),
                Cell::from(new_size),
                Cell::from(savings),
                Cell::from(ratio),
                Cell::from(duration),
                Cell::from(reason),
            ])
            .style(status_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),  // STATUS
            Constraint::Length(20),  // FILE
            Constraint::Length(10),  // ORIGINAL
            Constraint::Length(10),  // OUTPUT
            Constraint::Length(10),  // SAVINGS
            Constraint::Length(8),   // RATIO
            Constraint::Length(10),  // DURATION
            Constraint::Length(15),  // REASON
        ],
    )
    .header(headers)
    .block(Block::default().title("Transcode History").borders(Borders::ALL));

    f.render_widget(table, area);
}

/// Draw footer with controls and status
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let queue_stats = app.get_queue_stats();
    
    let mut spans = vec![
        Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" Quit  "),
        Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" Refresh  │  "),
    ];

    if let Some(err) = &app.job_load_error {
        spans.push(Span::styled(format!("⚠ {}", err), Style::default().fg(Color::Yellow)));
    } else {
        spans.push(Span::styled(
            format!("✓ {} jobs loaded", queue_stats.total),
            Style::default().fg(Color::Green)
        ));
    }

    let text = vec![Line::from(spans)];
    let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;

    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.0} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Create dummy jobs for display
fn create_dummy_jobs() -> Vec<TranscodeJob> {
    use core::JobReason;
    use std::path::PathBuf;

    let mut jobs = vec![];

    let mut job1 = TranscodeJob::new(PathBuf::from("/media/movies/example_movie.mkv"));
    job1.status = JobStatus::Success;
    job1.original_bytes = Some(5_000_000_000);
    job1.new_bytes = Some(3_500_000_000);
    jobs.push(job1);

    let mut job2 = TranscodeJob::new(PathBuf::from("/media/tv/show_s01e02.mkv"));
    job2.status = JobStatus::Running;
    job2.started_at = Some(chrono::Utc::now() - chrono::Duration::minutes(15));
    job2.original_bytes = Some(3_200_000_000);
    job2.new_bytes = Some(2_400_000_000);
    jobs.push(job2);

    let job3 = TranscodeJob::new(PathBuf::from("/media/movies/another_movie.mp4"));
    jobs.push(job3);

    let mut job4 = TranscodeJob::new(PathBuf::from("/media/movies/small_file.avi"));
    job4.status = JobStatus::Skipped;
    job4.reason = Some(JobReason::new("File too small"));
    job4.original_bytes = Some(500_000_000);
    jobs.push(job4);

    let mut job5 = TranscodeJob::new(PathBuf::from("/media/tv/corrupt_episode.mkv"));
    job5.status = JobStatus::Failed;
    job5.reason = Some(JobReason::new("FFmpeg error"));
    job5.original_bytes = Some(2_800_000_000);
    jobs.push(job5);

    jobs
}
