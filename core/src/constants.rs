// Constants used throughout the application
// Centralizes magic numbers for maintainability

/// Byte size constants
pub mod units {
    /// Kibibyte (1024 bytes)
    pub const KIB: u64 = 1024;
    
    /// Mebibyte (1024 KiB)
    pub const MIB: u64 = KIB * 1024;
    
    /// Gibibyte (1024 MiB)
    pub const GIB: u64 = MIB * 1024;
    
    /// Tebibyte (1024 GiB)
    pub const TIB: u64 = GIB * 1024;
    
    /// Disk sector size in bytes (standard)
    pub const SECTOR_SIZE_BYTES: u64 = 512;
}

/// Default configuration values
pub mod defaults {
    use super::units::GIB;
    
    /// Default minimum file size (2 GiB)
    pub const MIN_FILE_SIZE_BYTES: u64 = 2 * GIB;
    
    /// Default size gate factor (90%)
    pub const SIZE_GATE_FACTOR: f64 = 0.9;
    
    /// Default scan interval in seconds
    pub const SCAN_INTERVAL_SECONDS: u64 = 60;
    
    /// Default QSV quality for resolutions below 1080p
    pub const QSV_QUALITY_BELOW_1080P: u8 = 25;
    
    /// Default QSV quality for 1080p
    pub const QSV_QUALITY_AT_1080P: u8 = 24;
    
    /// Default QSV quality for 1440p and above
    pub const QSV_QUALITY_AT_1440P_PLUS: u8 = 23;
    
    /// Default media file extensions to scan
    pub const MEDIA_EXTENSIONS: &[&str] = &["mkv", "mp4", "avi"];
}

/// File stability checking constants
pub mod stability {
    /// Number of samples to take when checking file stability
    pub const SAMPLE_COUNT: usize = 3;
    
    /// Delay between samples in milliseconds
    pub const SAMPLE_DELAY_MS: u64 = 500;
}

/// Progress reporting constants
pub mod progress {
    /// Print progress every N frames
    pub const PRINT_EVERY_N_FRAMES: u64 = 100;
    
    /// Update job state every N frames
    pub const SAVE_JOB_EVERY_N_FRAMES: u64 = 500;
}

/// GPU monitoring constants
pub mod gpu {
    /// GPU frequency threshold for considering encoder "active" (MHz)
    pub const ACTIVE_FREQ_THRESHOLD_MHZ: u32 = 500;
    
    /// Typical maximum GPU frequency for Intel GPUs (MHz)
    pub const TYPICAL_MAX_FREQ_MHZ: u32 = 2000;
}

/// FFmpeg execution constants
pub mod ffmpeg {
    /// Default timeout for FFmpeg operations in seconds (4 hours)
    pub const DEFAULT_TIMEOUT_SECONDS: u64 = 3600 * 4;
    
    /// Maximum stderr lines to store (prevent memory exhaustion)
    pub const MAX_STDERR_LINES: usize = 1000;
    
    /// FFmpeg version string offset in version output
    pub const VERSION_STRING_OFFSET: usize = 8;
}

/// Resolution thresholds for quality selection
pub mod resolution {
    /// 1080p height
    pub const HEIGHT_1080P: u32 = 1080;
    
    /// 1440p height
    pub const HEIGHT_1440P: u32 = 1440;
    
    /// 4K height
    pub const HEIGHT_4K: u32 = 2160;
}

/// Bit depth thresholds
pub mod bitdepth {
    /// Threshold for using 10-bit pixel formats
    pub const USE_10BIT_THRESHOLD: u8 = 10;
}

