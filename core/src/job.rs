// Job model for transcode operations
// Tracks the state of each file's conversion attempt

use crate::utils::format_bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Status of a transcode job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job has been created but not yet started
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Success,
    /// Job failed due to an error
    Failed,
    /// Job was skipped (size too small, already AV1, etc.)
    Skipped,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "PENDING"),
            JobStatus::Running => write!(f, "RUNNING"),
            JobStatus::Success => write!(f, "SUCCESS"),
            JobStatus::Failed => write!(f, "FAILED"),
            JobStatus::Skipped => write!(f, "SKIPPED"),
        }
    }
}

/// Textual explanation for why a job was skipped or failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobReason(pub String);

impl JobReason {
    pub fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

impl std::fmt::Display for JobReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a single file's transcode operation
/// Contains all metadata needed to track the job through its lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeJob {
    /// Unique identifier for this job (UUID v4)
    pub id: String,

    /// Path to the original source file
    pub source_path: PathBuf,

    /// Path to the output file (if transcoding started)
    pub output_path: Option<PathBuf>,

    /// When this job was created
    pub created_at: DateTime<Utc>,

    /// When transcoding started (if it has started)
    pub started_at: Option<DateTime<Utc>>,

    /// When the job finished (success, failure, or skip)
    pub finished_at: Option<DateTime<Utc>>,

    /// Current status of the job
    pub status: JobStatus,

    /// Optional reason for failure or skip
    pub reason: Option<JobReason>,

    /// Size of the original file in bytes
    pub original_bytes: Option<u64>,

    /// Size of the transcoded file in bytes (if completed)
    pub new_bytes: Option<u64>,

    /// Whether this file was detected as WebRip-like
    /// Used to apply special handling (VFR, odd dimensions, etc.)
    pub is_webrip_like: bool,
}

impl TranscodeJob {
    /// Create a new pending job for the given source file
    pub fn new(source_path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source_path,
            output_path: None,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            status: JobStatus::Pending,
            reason: None,
            original_bytes: None,
            new_bytes: None,
            is_webrip_like: false,
        }
    }

    /// Calculate the duration of the job
    /// Returns None if the job hasn't started yet
    pub fn duration(&self) -> Option<chrono::Duration> {
        let start = self.started_at?;
        let end = self.finished_at.unwrap_or_else(Utc::now);
        Some(end - start)
    }

    /// Calculate the size savings ratio
    /// Returns None if size information is not available
    /// Returns a value between 0.0 and 1.0 representing the fraction saved
    /// Example: 0.3 means the new file is 30% smaller (saved 30%)
    pub fn size_savings_ratio(&self) -> Option<f64> {
        let original = self.original_bytes? as f64;
        let new = self.new_bytes? as f64;

        if original == 0.0 {
            return None;
        }

        Some((original - new) / original)
    }

    /// Get the absolute size savings in bytes
    /// Returns None if size information is not available
    pub fn size_savings_bytes(&self) -> Option<i64> {
        let original = self.original_bytes?;
        let new = self.new_bytes?;
        Some(original as i64 - new as i64)
    }

    /// Format duration as a human-readable string
    /// Example: "1h 23m 45s"
    pub fn duration_string(&self) -> String {
        match self.duration() {
            Some(d) => {
                let total_seconds = d.num_seconds();
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                if hours > 0 {
                    format!("{}h {}m {}s", hours, minutes, seconds)
                } else if minutes > 0 {
                    format!("{}m {}s", minutes, seconds)
                } else {
                    format!("{}s", seconds)
                }
            }
            None => "N/A".to_string(),
        }
    }

    /// Format size savings as a human-readable string
    /// Example: "1.2 GiB (35%)"
    pub fn size_savings_string(&self) -> String {
        match (self.size_savings_bytes(), self.size_savings_ratio()) {
            (Some(bytes), Some(ratio)) => {
                let formatted = format_bytes(bytes.abs() as u64);
                let percentage = (ratio * 100.0).round();
                if bytes >= 0 {
                    format!("{} ({:.0}%)", formatted, percentage)
                } else {
                    format!("-{} ({:.0}%)", formatted, percentage)
                }
            }
            _ => "N/A".to_string(),
        }
    }
}

