// Transcode execution and job management
// Handles running ffmpeg and tracking job state

use crate::ffmpeg::{
    build_transcode_command, get_final_output_path, get_skip_marker_path, get_temp_output_path,
    get_why_file_path,
};
use anyhow::{Context, Result};
use core::{
    run_ffprobe, save_job_state, JobReason, JobStatus, PathsConfig, TranscodeConfig, TranscodeJob,
};
use std::fs;
use std::path::Path;
use std::process::Stdio;

/// Execute a transcode job from start to finish
///
/// This function:
/// 1. Updates job status to Running
/// 2. Runs ffmpeg to transcode the file
/// 3. Checks the size gate
/// 4. On success: replaces the original file
/// 5. On failure: creates .av1skip and .why.txt
/// 6. Updates job status accordingly
///
/// # Arguments
/// * `job` - The transcode job to execute
/// * `config` - Transcode configuration
/// * `paths_config` - Path configuration for job state files
///
/// # Returns
/// `Ok(())` if the job completes (success or controlled failure)
/// `Err(_)` if an unexpected error occurs
pub fn execute_transcode(
    job: &mut TranscodeJob,
    config: &TranscodeConfig,
    paths_config: &PathsConfig,
) -> Result<()> {
    eprintln!("Starting transcode job: {}", job.source_path.display());

    // Update job to running state
    job.status = JobStatus::Running;
    job.started_at = Some(chrono::Utc::now());
    save_job_state(job, &paths_config.jobs_dir)?;

    // Get metadata for the file
    let metadata = match run_ffprobe(&config.ffmpeg_path, &job.source_path) {
        Ok(meta) => meta,
        Err(e) => {
            job.status = JobStatus::Failed;
            job.reason = Some(JobReason::new(format!("FFprobe failed: {}", e)));
            job.finished_at = Some(chrono::Utc::now());
            save_job_state(job, &paths_config.jobs_dir)?;
            return Err(e.into());
        }
    };

    // Store original file size
    job.original_bytes = metadata.size.or_else(|| {
        fs::metadata(&job.source_path).ok().map(|m| m.len())
    });

    // Set WebRip flag
    job.is_webrip_like = core::is_webrip_like(&metadata);

    // Determine output path
    let temp_output = get_temp_output_path(&job.source_path);
    job.output_path = Some(temp_output.clone());

    // Build ffmpeg command
    let mut cmd = build_transcode_command(config, &job.source_path, &temp_output, &metadata);

    // Configure command to inherit stderr for logging
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::inherit());

    eprintln!("Running ffmpeg...");
    eprintln!("  Input: {}", job.source_path.display());
    eprintln!("  Output: {}", temp_output.display());
    eprintln!("  WebRip-like: {}", job.is_webrip_like);

    // Execute ffmpeg
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            job.status = JobStatus::Failed;
            job.reason = Some(JobReason::new(format!("Failed to execute ffmpeg: {}", e)));
            job.finished_at = Some(chrono::Utc::now());
            save_job_state(job, &paths_config.jobs_dir)?;
            return Err(e.into());
        }
    };

    // Check if ffmpeg succeeded
    if !status.success() {
        job.status = JobStatus::Failed;
        job.reason = Some(JobReason::new(format!(
            "FFmpeg exited with status: {}",
            status
        )));
        job.finished_at = Some(chrono::Utc::now());
        save_job_state(job, &paths_config.jobs_dir)?;

        // Clean up temporary file
        let _ = fs::remove_file(&temp_output);

        return Ok(()); // Controlled failure
    }

    eprintln!("FFmpeg completed successfully");

    // Get the size of the transcoded file
    let new_size = fs::metadata(&temp_output)
        .context("Failed to get transcoded file size")?
        .len();
    job.new_bytes = Some(new_size);

    // Apply size gate
    let original_size = job
        .original_bytes
        .context("Original file size not available")?;
    let size_ratio = new_size as f64 / original_size as f64;

    if size_ratio > config.size_gate_factor {
        // Size gate failed: transcoded file is too large
        let reason = format!(
            "Size gate failed: new file is {:.1}% of original (limit: {:.1}%)",
            size_ratio * 100.0,
            config.size_gate_factor * 100.0
        );
        
        eprintln!("Size gate FAILED: {}", reason);
        
        job.status = JobStatus::Skipped;
        job.reason = Some(JobReason::new(reason.clone()));
        job.finished_at = Some(chrono::Utc::now());
        save_job_state(job, &paths_config.jobs_dir)?;

        // Write .why.txt file
        let why_path = get_why_file_path(&job.source_path);
        fs::write(&why_path, &reason)?;

        // Write .av1skip marker
        let skip_path = get_skip_marker_path(&job.source_path);
        fs::write(&skip_path, "")?;

        // Remove transcoded file
        fs::remove_file(&temp_output)?;

        eprintln!("Created .av1skip and .why.txt markers");

        return Ok(());
    }

    // Size gate passed: replace original file
    eprintln!(
        "Size gate PASSED: new file is {:.1}% of original",
        size_ratio * 100.0
    );

    // Atomically replace the original file
    // Strategy: rename original to .bak, rename new to original, delete .bak
    let backup_path = job.source_path.with_extension("bak");
    
    fs::rename(&job.source_path, &backup_path)
        .context("Failed to backup original file")?;
    
    let final_path = get_final_output_path(&job.source_path);
    
    if let Err(e) = fs::rename(&temp_output, &final_path) {
        // Restore backup on failure
        fs::rename(&backup_path, &job.source_path)?;
        return Err(e).context("Failed to rename transcoded file");
    }

    // Delete backup
    fs::remove_file(&backup_path)?;

    eprintln!("File replaced successfully: {}", final_path.display());

    // Update job to success
    job.status = JobStatus::Success;
    job.finished_at = Some(chrono::Utc::now());
    save_job_state(job, &paths_config.jobs_dir)?;

    eprintln!("Job completed successfully");
    eprintln!("  Original size: {} bytes", original_size);
    eprintln!("  New size: {} bytes", new_size);
    eprintln!("  Savings: {}", job.size_savings_string());

    Ok(())
}

/// Check if a file is stable (not currently being written to)
///
/// Checks the file size multiple times with delays to ensure it's not changing.
/// This prevents starting transcodes on files that are still being copied.
///
/// # Arguments
/// * `path` - Path to the file to check
/// * `samples` - Number of samples to take (default: 3)
/// * `delay_ms` - Delay between samples in milliseconds (default: 2000)
///
/// # Returns
/// `true` if the file size is stable, `false` if it's changing
pub fn is_file_stable(path: &Path, samples: usize, delay_ms: u64) -> bool {
    let mut last_size: Option<u64> = None;

    for _ in 0..samples {
        let current_size = match fs::metadata(path) {
            Ok(meta) => meta.len(),
            Err(_) => return false, // File doesn't exist or can't be read
        };

        if let Some(last) = last_size {
            if current_size != last {
                return false; // Size changed, file is not stable
            }
        }

        last_size = Some(current_size);

        // Wait before next sample (except on last iteration)
        if samples > 1 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    }

    true
}

