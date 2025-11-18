// Error types for the core library
// Uses thiserror for ergonomic error handling

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for core library operations
#[derive(Error, Debug)]
pub enum CoreError {
    /// Error running ffprobe or ffmpeg
    #[error("FFmpeg process error: {0}")]
    FFmpegError(String),

    /// Error parsing ffprobe output
    #[error("Failed to parse ffprobe output: {0}")]
    ParseError(String),

    /// IO error (file operations, etc.)
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Job file not found
    #[error("Job file not found: {0}")]
    JobNotFound(PathBuf),

    /// Invalid job data
    #[error("Invalid job data: {0}")]
    InvalidJob(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// File metadata error
    #[error("Metadata error: {0}")]
    MetadataError(String),
}

/// Convenience type alias for Results using CoreError
pub type Result<T> = std::result::Result<T, CoreError>;

