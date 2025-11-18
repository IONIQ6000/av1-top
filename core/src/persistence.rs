// Job state persistence functions
// Save and load TranscodeJob data to/from JSON files

use crate::error::Result;
use crate::job::TranscodeJob;
use std::fs;
use std::path::Path;

/// Save a job's state to a JSON file in the jobs directory
///
/// Each job is saved as `<job_id>.json` in the specified directory.
/// The directory will be created if it doesn't exist.
///
/// # Arguments
/// * `job` - The job to save
/// * `dir` - Directory where job files are stored
///
/// # Returns
/// `Ok(())` on success, or a `CoreError` on failure
///
/// # Example
/// ```no_run
/// # use core::{TranscodeJob, save_job_state};
/// # use std::path::{Path, PathBuf};
/// let job = TranscodeJob::new(PathBuf::from("/media/video.mkv"));
/// save_job_state(&job, Path::new("/var/lib/av1janitor/jobs")).unwrap();
/// ```
pub fn save_job_state(job: &TranscodeJob, dir: &Path) -> Result<()> {
    // Ensure the jobs directory exists
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    // Serialize job to JSON
    let json = serde_json::to_string_pretty(job)?;

    // Write to file named after job ID
    let file_path = dir.join(format!("{}.json", job.id));
    fs::write(&file_path, json)?;

    Ok(())
}

/// Load all job files from the jobs directory
///
/// Reads all `.json` files in the directory and deserializes them as `TranscodeJob` objects.
/// Invalid or corrupted files are skipped with a warning (printed to stderr).
///
/// # Arguments
/// * `dir` - Directory containing job JSON files
///
/// # Returns
/// A vector of successfully loaded jobs, or a `CoreError` if the directory cannot be read
///
/// # Example
/// ```no_run
/// # use core::load_all_jobs;
/// # use std::path::Path;
/// let jobs = load_all_jobs(Path::new("/var/lib/av1janitor/jobs")).unwrap();
/// println!("Loaded {} jobs", jobs.len());
/// ```
pub fn load_all_jobs(dir: &Path) -> Result<Vec<TranscodeJob>> {
    // If directory doesn't exist, return empty list
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut jobs = Vec::new();

    // Read all entries in the directory
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip non-JSON files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Try to load and parse the job file
        match load_job_file(&path) {
            Ok(job) => jobs.push(job),
            Err(e) => {
                // Log error but continue loading other jobs
                eprintln!(
                    "Warning: Failed to load job from {:?}: {}",
                    path, e
                );
            }
        }
    }

    Ok(jobs)
}

/// Load a single job from a JSON file
///
/// Helper function used by `load_all_jobs`.
///
/// # Arguments
/// * `path` - Path to the job JSON file
///
/// # Returns
/// The loaded `TranscodeJob`, or a `CoreError` on failure
fn load_job_file(path: &Path) -> Result<TranscodeJob> {
    let contents = fs::read_to_string(path)?;
    let job: TranscodeJob = serde_json::from_str(&contents)?;
    Ok(job)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::{JobStatus, TranscodeJob};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_job() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let jobs_dir = temp_dir.path();

        // Create a test job
        let mut job = TranscodeJob::new(PathBuf::from("/media/test.mkv"));
        job.status = JobStatus::Success;
        job.original_bytes = Some(1000000);
        job.new_bytes = Some(800000);

        // Save the job
        save_job_state(&job, jobs_dir).unwrap();

        // Load all jobs
        let loaded_jobs = load_all_jobs(jobs_dir).unwrap();

        // Verify we got our job back
        assert_eq!(loaded_jobs.len(), 1);
        let loaded = &loaded_jobs[0];
        assert_eq!(loaded.id, job.id);
        assert_eq!(loaded.source_path, job.source_path);
        assert_eq!(loaded.status, JobStatus::Success);
        assert_eq!(loaded.original_bytes, Some(1000000));
        assert_eq!(loaded.new_bytes, Some(800000));
    }

    #[test]
    fn test_load_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let jobs = load_all_jobs(temp_dir.path()).unwrap();
        assert_eq!(jobs.len(), 0);
    }

    #[test]
    fn test_load_nonexistent_directory() {
        let jobs = load_all_jobs(Path::new("/nonexistent/path/12345")).unwrap();
        assert_eq!(jobs.len(), 0);
    }
}

