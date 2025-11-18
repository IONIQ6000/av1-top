// AV1 Daemon - Production-ready transcoding with all improvements
// Features: Logging, CLI, Signal handling, Concurrent processing, Filesystem watching

mod cli;
mod shutdown;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Args;
use core::{
    build_ffmpeg_command, check_size_gate, cleanup_failed_transcode, execute_transcode,
    find_and_validate_ffmpeg, get_installation_instructions, is_already_av1, is_webrip_like,
    replace_file_atomic, run_ffprobe, save_job_state, should_skip_for_size, write_skip_marker,
    write_why_file, ExecuteOptions, FFmpegInstallation, JobReason, JobStatus, PathsConfig,
    SizeGateResult, TranscodeConfig, TranscodeJob, TranscodeParams, TranscodeProgress,
};
use core::constants::stability;
use log::{debug, error, info, warn};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();
    
    // Initialize logging based on verbosity
    let log_level = match args.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();
    
    info!("=== AV1 Daemon ===");
    info!("Automated AV1 transcoding with Intel QSV");
    
    // Install signal handlers
    shutdown::install_signal_handlers();
    info!("Signal handlers installed (Ctrl+C for graceful shutdown)");
    
    // Auto-detect and validate FFmpeg 8.0+
    info!("Detecting FFmpeg installation...");
    let ffmpeg = match find_and_validate_ffmpeg() {
        Ok(installation) => {
            info!("✓ Found FFmpeg {}", installation.version);
            info!("  Path: {}", installation.ffmpeg_path.display());
            info!("  FFprobe: {}", installation.ffprobe_path.display());
            info!("✓ av1_qsv encoder available");
            
            if installation.qsv_hardware_works {
                info!("✓ Intel QSV hardware test passed");
            } else {
                warn!("QSV hardware test failed - transcoding may not work");
                warn!("Check GPU drivers and permissions: vainfo");
            }
            installation
        }
        Err(e) => {
            error!("FFmpeg validation failed: {}", e);
            error!("{}", get_installation_instructions());
            anyhow::bail!("Cannot proceed without FFmpeg 8.0+");
        }
    };
    
    // Load configuration (from file or defaults)
    let config_path = args.get_config_path();
    let mut config = if config_path.exists() {
        info!("Loading configuration from: {}", config_path.display());
        TranscodeConfig::load_from_file(&config_path)?
    } else {
        info!("No config file found, using defaults");
        TranscodeConfig::default()
    };
    
    let paths_config = PathsConfig::default();
    
    // Override with CLI arguments
    let cli_directory = args.directory.clone();
    let dry_run = args.dry_run;
    let concurrent = args.concurrent;
    let once_mode = args.once;
    
    if let Some(dir) = cli_directory {
        info!("Using directory from CLI: {}", dir.display());
        config.watched_directories = vec![dir];
    }
    
    // Set ffmpeg path from detection
    config.ffmpeg_path = Some(ffmpeg.ffmpeg_path.clone());
    
    // Validate configuration
    config.validate()?;
    
    // Ensure job directory exists
    fs::create_dir_all(&paths_config.jobs_dir)?;
    
    // Print configuration
    print_configuration(&config, &paths_config, dry_run, concurrent);
    
    // Choose mode: one-shot or continuous daemon
    if once_mode {
        info!("Running in one-shot mode");
        process_all_files(&config, &paths_config, &ffmpeg, dry_run, concurrent).await?;
        info!("One-shot processing complete");
    } else {
        info!("Starting continuous daemon mode");
        info!("Concurrent jobs: {}", concurrent);
        run_daemon(config, paths_config, ffmpeg, dry_run, concurrent).await?;
    }
    
    Ok(())
}

/// Run the daemon continuously
async fn run_daemon(
    config: TranscodeConfig,
    paths_config: PathsConfig,
    ffmpeg: FFmpegInstallation,
    dry_run: bool,
    max_concurrent: usize,
) -> Result<()> {
    let config = Arc::new(config);
    let paths_config = Arc::new(paths_config);
    let ffmpeg = Arc::new(ffmpeg);
    
    // Set up filesystem watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    
    // Watch all configured directories
    for dir in &config.watched_directories {
        info!("Watching directory: {}", dir.display());
        watcher.watch(dir, RecursiveMode::Recursive)?;
    }
    
    // Track files being processed to avoid duplicates
    let processing = Arc::new(Mutex::new(HashSet::<PathBuf>::new()));
    
    // Initial scan
    info!("Performing initial directory scan...");
    process_all_files(&config, &paths_config, &ffmpeg, dry_run, max_concurrent).await?;
    
    info!("Watching for new files (Ctrl+C to stop)...");
    
    // Main daemon loop
    loop {
        if shutdown::should_shutdown() {
            info!("Shutdown requested, exiting gracefully...");
            break;
        }
        
        // Check for filesystem events (with timeout)
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                handle_filesystem_event(
                    event,
                    &config,
                    &paths_config,
                    &ffmpeg,
                    &processing,
                    dry_run,
                ).await;
            }
            Ok(Err(e)) => {
                warn!("Filesystem watcher error: {}", e);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Timeout is normal, just loop again
            }
            Err(e) => {
                error!("Channel error: {}", e);
                break;
            }
        }
    }
    
    info!("Daemon stopped");
    Ok(())
}

/// Handle a filesystem event
async fn handle_filesystem_event(
    event: Event,
    config: &Arc<TranscodeConfig>,
    paths_config: &Arc<PathsConfig>,
    ffmpeg: &Arc<FFmpegInstallation>,
    processing: &Arc<Mutex<HashSet<PathBuf>>>,
    dry_run: bool,
) {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                if is_media_file(&path, &config.media_extensions) {
                    // Check if already processing
                    let mut proc = processing.lock().unwrap();
                    if proc.contains(&path) {
                        continue;
                    }
                    proc.insert(path.clone());
                    drop(proc);
                    
                    info!("New file detected: {}", path.display());
                    
                    // Spawn task to process file
                    let config = Arc::clone(config);
                    let paths_config = Arc::clone(paths_config);
                    let ffmpeg = Arc::clone(ffmpeg);
                    let processing = Arc::clone(processing);
                    
                    tokio::spawn(async move {
                        match process_file(&path, &config, &paths_config, &ffmpeg, dry_run).await {
                            Ok(FileResult::Success) => {
                                info!("✓ Successfully transcoded: {}", path.display());
                            }
                            Ok(FileResult::Skipped(reason)) => {
                                debug!("⊘ Skipped {}: {}", path.display(), reason);
                            }
                            Err(e) => {
                                error!("✗ Failed {}: {}", path.display(), e);
                            }
                        }
                        
                        // Remove from processing set
                        processing.lock().unwrap().remove(&path);
                    });
                }
            }
        }
        _ => {}
    }
}

/// Check if a path is a media file
fn is_media_file(path: &Path, extensions: &[String]) -> bool {
    if !path.is_file() {
        return false;
    }
    
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return extensions.contains(&ext_str.to_lowercase());
        }
    }
    
    false
}

/// Process all files with concurrent support
async fn process_all_files(
    config: &TranscodeConfig,
    paths_config: &PathsConfig,
    ffmpeg: &FFmpegInstallation,
    dry_run: bool,
    max_concurrent: usize,
) -> Result<ProcessingResults> {
    let mut results = ProcessingResults {
        total: 0,
        success: 0,
        skipped: 0,
        failed: 0,
    };
    
    // Scan for files
    let files = scan_directories(config)?;
    results.total = files.len();
    
    info!("Found {} media files to process", files.len());
    
    if dry_run {
        warn!("DRY RUN MODE - No actual transcoding will occur");
    }
    
    // Process files with concurrency limit
    let mut set = JoinSet::new();
    
    for file_path in files {
        // Wait if we've hit the concurrency limit
        while set.len() >= max_concurrent {
            if let Some(result) = set.join_next().await {
                match result {
                    Ok(Ok(FileResult::Success)) => results.success += 1,
                    Ok(Ok(FileResult::Skipped(_))) => results.skipped += 1,
                    Ok(Err(_)) | Err(_) => results.failed += 1,
                }
            }
        }
        
        // Check for shutdown
        if shutdown::should_shutdown() {
            warn!("Shutdown requested, stopping new jobs");
            break;
        }
        
        // Spawn task
        let config = config.clone();
        let paths_config = paths_config.clone();
        let ffmpeg = ffmpeg.clone();
        
        set.spawn(async move {
            process_file(&file_path, &config, &paths_config, &ffmpeg, dry_run).await
        });
    }
    
    // Wait for remaining tasks
    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok(FileResult::Success)) => results.success += 1,
            Ok(Ok(FileResult::Skipped(_))) => results.skipped += 1,
            Ok(Err(_)) | Err(_) => results.failed += 1,
        }
    }
    
    info!("Processing summary:");
    info!("  Total: {}", results.total);
    info!("  Success: {}", results.success);
    info!("  Skipped: {}", results.skipped);
    info!("  Failed: {}", results.failed);
    
    Ok(results)
}

/// Processing results
struct ProcessingResults {
    total: usize,
    success: usize,
    skipped: usize,
    failed: usize,
}

/// File processing result
enum FileResult {
    Success,
    Skipped(String),
}

/// Process a single file (async version)
async fn process_file(
    file_path: &Path,
    config: &TranscodeConfig,
    paths_config: &PathsConfig,
    ffmpeg: &FFmpegInstallation,
    dry_run: bool,
) -> Result<FileResult> {
    debug!("Processing: {}", file_path.display());
    
    // Pre-flight checks
    let skip_marker = file_path.with_extension("av1skip");
    if skip_marker.exists() {
        return Ok(FileResult::Skipped("Has .av1skip marker".into()));
    }
    
    let file_size = fs::metadata(file_path)?.len();
    
    if should_skip_for_size(file_size, config.min_file_size_bytes) {
        return Ok(FileResult::Skipped(format!(
            "Too small ({:.1} GiB)",
            file_size as f64 / (1024.0 * 1024.0 * 1024.0)
        )));
    }
    
    // Wait for file stability
    if !is_file_stable(file_path).await? {
        return Ok(FileResult::Skipped("File still being copied".into()));
    }
    
    // Run ffprobe
    let metadata = run_ffprobe(&ffmpeg.ffprobe_path, file_path)?;
    
    if !metadata.has_video() {
        return Ok(FileResult::Skipped("No video streams".into()));
    }
    
    if is_already_av1(&metadata) {
        return Ok(FileResult::Skipped("Already AV1".into()));
    }
    
    if dry_run {
        info!("DRY RUN: Would transcode {}", file_path.display());
        return Ok(FileResult::Skipped("Dry run mode".into()));
    }
    
    // Create job
    let mut job = TranscodeJob::new(file_path.to_path_buf());
    job.original_bytes = Some(file_size);
    job.is_webrip_like = is_webrip_like(&metadata);
    job.status = JobStatus::Pending;
    save_job_state(&job, &paths_config.jobs_dir)?;
    
    // Build params
    let params = TranscodeParams::from_metadata(
        file_path.to_path_buf(),
        &metadata,
        job.is_webrip_like,
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to build params"))?;
    
    job.output_path = Some(params.output_path.clone());
    
    // Build command
    let ffmpeg_args = build_ffmpeg_command(&ffmpeg.ffmpeg_path, &params);
    
    // Start transcoding
    job.status = JobStatus::Running;
    job.started_at = Some(chrono::Utc::now());
    save_job_state(&job, &paths_config.jobs_dir)?;
    
    info!(
        "Starting transcode: {} (Quality: {}, Surface: {}, WebRip: {})",
        file_path.display(),
        params.quality,
        params.surface,
        params.is_webrip
    );
    
    // Clone values needed for the blocking task
    let ffmpeg_path = ffmpeg.ffmpeg_path.clone();
    let output_path = params.output_path.clone();
    
    // Execute (blocking, but in async context we can await)
    let result = tokio::task::spawn_blocking(move || {
        execute_transcode(
            &ffmpeg_path,
            &params,
            ffmpeg_args,
            ExecuteOptions::default(),
            Some(|progress: TranscodeProgress| {
                if progress.frame % 100 == 0 {
                    debug!(
                        "Frame: {} | FPS: {:.1} | Speed: {:.2}x",
                        progress.frame, progress.fps, progress.speed
                    );
                }
            }),
        )
    })
    .await??;
    
    job.finished_at = Some(chrono::Utc::now());
    
    // Check result
    if result.timed_out {
        error!("Transcode timed out after {:.0}s", result.duration.as_secs_f64());
        job.status = JobStatus::Failed;
        job.reason = Some(JobReason::new("Timeout"));
        save_job_state(&job, &paths_config.jobs_dir)?;
        cleanup_failed_transcode(&output_path)?;
        return Err(anyhow::anyhow!("Timeout"));
    }
    
    if !result.success {
        error!("Transcode failed");
        job.status = JobStatus::Failed;
        job.reason = Some(JobReason::new(format!("FFmpeg failed: {:?}", result.exit_code)));
        save_job_state(&job, &paths_config.jobs_dir)?;
        cleanup_failed_transcode(&output_path)?;
        return Err(anyhow::anyhow!("FFmpeg failed"));
    }
    
    info!("Transcode completed in {:.1}s", result.duration.as_secs_f64());
    
    // Size gate
    let size_result = check_size_gate(file_path, &output_path, config.size_gate_factor)?;
    
    match size_result {
        SizeGateResult::Passed {
            new_bytes,
            savings_ratio,
            ..
        } => {
            job.new_bytes = Some(new_bytes);
            info!("✓ Size gate passed: {:.1}% savings", savings_ratio * 100.0);
            
            replace_file_atomic(file_path, &output_path)?;
            
            job.status = JobStatus::Success;
            save_job_state(&job, &paths_config.jobs_dir)?;
            
            Ok(FileResult::Success)
        }
        SizeGateResult::Failed { ratio, threshold, .. } => {
            warn!(
                "✗ Size gate failed: {:.1}% of original (max: {:.1}%)",
                ratio * 100.0,
                threshold * 100.0
            );
            
            let reason = format!(
                "Size gate failed: {:.1}% of original (max: {:.1}%)",
                ratio * 100.0,
                threshold * 100.0
            );
            
            write_why_file(file_path, &reason)?;
            write_skip_marker(file_path)?;
            cleanup_failed_transcode(&output_path)?;
            
            job.status = JobStatus::Skipped;
            job.reason = Some(JobReason::new("Size gate failed"));
            save_job_state(&job, &paths_config.jobs_dir)?;
            
            Ok(FileResult::Skipped("Size gate failed".into()))
        }
    }
}

/// Check file stability (async version)
async fn is_file_stable(file_path: &Path) -> Result<bool> {
    use core::constants::stability::{SAMPLE_COUNT, SAMPLE_DELAY_MS};
    
    let mut previous_size = fs::metadata(file_path)?.len();
    
    for _ in 0..SAMPLE_COUNT {
        tokio::time::sleep(Duration::from_millis(SAMPLE_DELAY_MS)).await;
        
        let current_size = fs::metadata(file_path)?.len();
        
        if current_size != previous_size {
            return Ok(false);
        }
        
        previous_size = current_size;
    }
    
    Ok(true)
}

/// Scan directories for media files
fn scan_directories(config: &TranscodeConfig) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    for root_dir in &config.watched_directories {
        if !root_dir.exists() {
            warn!("Directory does not exist: {}", root_dir.display());
            continue;
        }
        
        scan_directory_recursive(root_dir, &config.media_extensions, &mut files)?;
    }
    
    Ok(files)
}

/// Recursively scan directory
fn scan_directory_recursive(
    dir: &Path,
    extensions: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            scan_directory_recursive(&path, extensions, files)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if extensions.contains(&ext_str.to_lowercase()) {
                        files.push(path);
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Print configuration
fn print_configuration(config: &TranscodeConfig, paths_config: &PathsConfig, dry_run: bool, concurrent: usize) {
    info!("Configuration:");
    if let Some(ref ffmpeg_path) = config.ffmpeg_path {
        info!("  FFmpeg: {}", ffmpeg_path.display());
    }
    info!(
        "  Min file size: {:.1} GiB",
        config.min_file_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    info!("  Size gate: {:.0}%", config.size_gate_factor * 100.0);
    info!("  Extensions: {:?}", config.media_extensions);
    info!("  Jobs directory: {}", paths_config.jobs_dir.display());
    info!("  Concurrent jobs: {}", concurrent);
    info!("  Dry run: {}", dry_run);
    info!("  Watched directories:");
    for dir in &config.watched_directories {
        info!("    - {}", dir.display());
    }
}
