// Metadata types for media files
// Contains information extracted from ffprobe

use serde::{Deserialize, Serialize};

/// Information about a video stream from ffprobe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    /// Codec name (e.g., "h264", "av1", "hevc")
    pub codec: String,

    /// Video width in pixels
    pub width: u32,

    /// Video height in pixels
    pub height: u32,

    /// Bit depth (8, 10, 12, etc.)
    pub bit_depth: u8,

    /// Whether this stream is marked as default
    pub is_default: bool,

    /// Average frame rate as reported by ffprobe
    /// Format: "24/1" or "30000/1001"
    pub avg_frame_rate: String,

    /// Real (raw) frame rate as reported by ffprobe
    /// Format: "24/1" or "30000/1001"
    pub r_frame_rate: String,
}

impl VideoStreamInfo {
    /// Check if this stream is Variable Frame Rate (VFR)
    /// VFR is detected when avg_frame_rate != r_frame_rate
    pub fn is_vfr(&self) -> bool {
        self.avg_frame_rate != self.r_frame_rate
    }

    /// Check if dimensions are odd (not divisible by 2)
    /// This is a sign of WebRip-like content that may need padding
    pub fn has_odd_dimensions(&self) -> bool {
        self.width % 2 != 0 || self.height % 2 != 0
    }

    /// Get resolution string for display
    /// Example: "1920x1080"
    pub fn resolution_string(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// Get a simplified resolution label
    /// Examples: "1080p", "720p", "4K"
    pub fn resolution_label(&self) -> String {
        match self.height {
            2160.. => "4K".to_string(),
            1440..=2159 => "1440p".to_string(),
            1080..=1439 => "1080p".to_string(),
            720..=1079 => "720p".to_string(),
            480..=719 => "480p".to_string(),
            _ => format!("{}p", self.height),
        }
    }
}

/// Complete metadata for a media file
/// Extracted via ffprobe and used for decision-making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// All video streams in the file
    /// Usually there's only one, but some files may have multiple
    pub video_streams: Vec<VideoStreamInfo>,

    /// Container format name from ffprobe
    /// Examples: "matroska,webm", "mov,mp4,m4a,3gp,3g2,mj2"
    pub format_name: String,

    /// Muxing application tag (if present)
    /// Can help identify WebRips
    pub tags_muxing_app: Option<String>,

    /// Major brand tag (if present, typically from MP4/MOV)
    pub tags_major_brand: Option<String>,

    /// Compatible brands tag (if present)
    pub tags_compatible_brands: Option<String>,

    /// Total file size in bytes
    pub size: Option<u64>,
}

impl FileMetadata {
    /// Get the default (primary) video stream
    /// Returns the first stream marked as default, or the first stream if none are default
    pub fn default_video_stream(&self) -> Option<&VideoStreamInfo> {
        self.video_streams
            .iter()
            .find(|s| s.is_default)
            .or_else(|| self.video_streams.first())
    }

    /// Get the index of the default video stream
    /// Used for selecting the correct stream in ffmpeg commands
    pub fn default_video_stream_index(&self) -> Option<usize> {
        self.video_streams
            .iter()
            .position(|s| s.is_default)
            .or_else(|| {
                if self.video_streams.is_empty() {
                    None
                } else {
                    Some(0)
                }
            })
    }

    /// Check if the file has any video streams
    pub fn has_video(&self) -> bool {
        !self.video_streams.is_empty()
    }
}

