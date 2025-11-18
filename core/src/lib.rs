// Core library for AV1 transcoding daemon and TUI monitor
// Contains shared types, configuration, job models, and utilities

pub mod config;
pub mod constants;
pub mod error;
pub mod executor;
pub mod ffmpeg_manager;
pub mod ffprobe;
pub mod heuristics;
pub mod job;
pub mod metadata;
pub mod persistence;
pub mod postprocess;
pub mod transcode;
pub mod utils;

// Re-export commonly used types
pub use config::{PathsConfig, TranscodeConfig};
pub use error::CoreError;
pub use executor::{execute_transcode, ExecuteOptions, TranscodeProgress, TranscodeResult};
pub use ffmpeg_manager::{find_and_validate_ffmpeg, get_installation_instructions, FFmpegInstallation};
pub use ffprobe::run_ffprobe;
pub use heuristics::{
    choose_quality, choose_surface, is_already_av1, is_webrip_like, should_skip_for_size,
};
pub use job::{JobReason, JobStatus, TranscodeJob};
pub use metadata::{FileMetadata, VideoStreamInfo};
pub use persistence::{load_all_jobs, save_job_state};
pub use postprocess::{
    check_size_gate, cleanup_failed_transcode, replace_file_atomic, write_skip_marker,
    write_why_file, SizeGateResult,
};
pub use transcode::{build_ffmpeg_command, TranscodeParams};
pub use utils::{format_bytes, parse_size_with_unit, parse_time_to_seconds};

