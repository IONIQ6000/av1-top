// Configuration types for the AV1 transcoder
// These types define how the daemon should operate

use crate::constants::defaults;
use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for transcoding operations
/// Defines ffmpeg settings, watched directories, and encoding parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeConfig {
    /// Path to the ffmpeg binary to use for transcoding
    /// Default: auto-detected from system PATH
    /// If not found, will be detected automatically at runtime
    pub ffmpeg_path: Option<PathBuf>,

    /// List of directories to watch for media files
    /// The daemon will recursively scan these for transcodable media
    pub watched_directories: Vec<PathBuf>,

    /// Minimum file size in bytes to consider for transcoding
    /// Files smaller than this will be skipped
    /// Default: 2 GiB (2147483648 bytes)
    pub min_file_size_bytes: u64,

    /// Size gate factor: output must be <= this fraction of original
    /// If the transcoded file is larger than (original * factor), reject it
    /// Default: 0.9 (output must be ≤ 90% of original)
    pub size_gate_factor: f64,

    /// QSV quality settings per resolution
    /// Lower values = higher quality, higher bitrate
    pub qsv_quality: QsvQualitySettings,

    /// File extensions to consider for transcoding
    /// Default: ["mkv", "mp4", "avi"]
    pub media_extensions: Vec<String>,

    /// Seconds to wait between directory scans
    /// Default: 60
    pub scan_interval_seconds: u64,
}

/// Quality settings for Intel QSV AV1 encoding based on resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QsvQualitySettings {
    /// Quality for resolutions below 1080p
    /// Default: 25
    pub below_1080p: u8,

    /// Quality for 1080p (height == 1080)
    /// Default: 24
    pub at_1080p: u8,

    /// Quality for 1440p and above
    /// Default: 23
    pub at_1440p_and_above: u8,
}

impl Default for QsvQualitySettings {
    fn default() -> Self {
        Self {
            below_1080p: defaults::QSV_QUALITY_BELOW_1080P,
            at_1080p: defaults::QSV_QUALITY_AT_1080P,
            at_1440p_and_above: defaults::QSV_QUALITY_AT_1440P_PLUS,
        }
    }
}

impl Default for TranscodeConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: None, // Will be auto-detected
            watched_directories: vec![],
            min_file_size_bytes: defaults::MIN_FILE_SIZE_BYTES,
            size_gate_factor: defaults::SIZE_GATE_FACTOR,
            qsv_quality: QsvQualitySettings::default(),
            media_extensions: defaults::MEDIA_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            scan_interval_seconds: defaults::SCAN_INTERVAL_SECONDS,
        }
    }
}

impl TranscodeConfig {
    /// Validate configuration values
    ///
    /// Ensures all configuration values are within valid ranges.
    ///
    /// # Returns
    /// Ok(()) if config is valid, Err with description if invalid
    pub fn validate(&self) -> Result<()> {
        // Validate size gate factor
        if self.size_gate_factor <= 0.0 || self.size_gate_factor > 1.0 {
            return Err(CoreError::ConfigError(format!(
                "size_gate_factor must be between 0 and 1, got {}",
                self.size_gate_factor
            )));
        }

        // Validate minimum file size
        if self.min_file_size_bytes == 0 {
            return Err(CoreError::ConfigError(
                "min_file_size_bytes cannot be 0".into(),
            ));
        }

        // Validate watched directories (at least one required)
        if self.watched_directories.is_empty() {
            return Err(CoreError::ConfigError(
                "watched_directories cannot be empty".into(),
            ));
        }

        // Validate all watched directories exist
        for dir in &self.watched_directories {
            if !dir.exists() {
                return Err(CoreError::ConfigError(format!(
                    "Watched directory does not exist: {}",
                    dir.display()
                )));
            }
        }

        // Validate media extensions
        if self.media_extensions.is_empty() {
            return Err(CoreError::ConfigError(
                "media_extensions cannot be empty".into(),
            ));
        }

        // Validate QSV quality values (should be reasonable)
        if self.qsv_quality.below_1080p > 51 || self.qsv_quality.below_1080p == 0 {
            return Err(CoreError::ConfigError(format!(
                "QSV quality must be 1-51, got {}",
                self.qsv_quality.below_1080p
            )));
        }

        Ok(())
    }
}

/// Configuration for paths used by the daemon and TUI
/// Defines where logs and job state are stored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Directory for log files
    /// Default: ~/.local/share/av1janitor/logs
    pub logs_dir: PathBuf,

    /// Directory for job state JSON files
    /// Each job gets a <job_id>.json file here
    /// Default: ~/.local/share/av1janitor/jobs
    pub jobs_dir: PathBuf,
}

impl Default for PathsConfig {
    fn default() -> Self {
        // TODO: Use proper XDG base directory specification
        // For now, use a placeholder that will be replaced with actual home dir
        let base = PathBuf::from(
            std::env::var("HOME")
                .ok()
                .unwrap_or_else(|| "/tmp".to_string()),
        )
        .join(".local/share/av1janitor");

        Self {
            logs_dir: base.join("logs"),
            jobs_dir: base.join("jobs"),
        }
    }
}

impl TranscodeConfig {
    /// Load configuration from a TOML file
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    /// Loaded and validated configuration
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            CoreError::ConfigError(format!("Failed to read config file: {}", e))
        })?;
        
        let config: TranscodeConfig = toml::from_str(&contents).map_err(|e| {
            CoreError::ConfigError(format!("Failed to parse config file: {}", e))
        })?;
        
        // Validate the loaded config
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load configuration from default location or use defaults
    ///
    /// Tries to load from: ~/.config/av1janitor/config.toml
    /// Falls back to defaults if file doesn't exist
    ///
    /// # Returns
    /// Configuration (either loaded or default)
    pub fn load_or_default() -> Self {
        let config_path = PathBuf::from(
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        )
        .join(".config/av1janitor/config.toml");
        
        if config_path.exists() {
            match Self::load_from_file(&config_path) {
                Ok(config) => {
                    eprintln!("Loaded config from: {}", config_path.display());
                    config
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load config: {}", e);
                    eprintln!("Using default configuration");
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }
    
    /// Save configuration to a TOML file
    ///
    /// # Arguments
    /// * `path` - Path where to save the configuration
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let toml_string = toml::to_string_pretty(self).map_err(|e| {
            CoreError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, toml_string)?;
        Ok(())
    }
}

