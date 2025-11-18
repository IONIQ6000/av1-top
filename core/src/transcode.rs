// FFmpeg transcoding command builder
// Constructs the exact ffmpeg command line for AV1 QSV transcoding

use crate::heuristics::{choose_quality, choose_surface};
use crate::metadata::FileMetadata;
use std::path::{Path, PathBuf};

/// Parameters for a transcode operation
/// Contains all the information needed to build an ffmpeg command
#[derive(Debug, Clone)]
pub struct TranscodeParams {
    /// Path to the input file
    pub input_path: PathBuf,
    
    /// Path to the output file (temporary, will be renamed on success)
    pub output_path: PathBuf,
    
    /// Index of the video stream to use (0-based)
    pub video_stream_index: usize,
    
    /// Quality value for QSV (23, 24, or 25)
    pub quality: u8,
    
    /// Surface format for QSV (nv12 or p010)
    pub surface: String,
    
    /// Whether this is a WebRip-like file (needs special handling)
    pub is_webrip: bool,
}

impl TranscodeParams {
    /// Create transcode parameters from file metadata
    ///
    /// Analyzes the metadata and determines all encoding parameters.
    ///
    /// # Arguments
    /// * `input_path` - Path to the input file
    /// * `metadata` - Metadata from ffprobe
    /// * `is_webrip` - Whether this is detected as WebRip-like
    ///
    /// # Returns
    /// TranscodeParams ready for command building, or None if no suitable video stream
    pub fn from_metadata(
        input_path: PathBuf,
        metadata: &FileMetadata,
        is_webrip: bool,
    ) -> Option<Self> {
        // Get the default video stream
        let video_stream = metadata.default_video_stream()?;
        let video_stream_index = metadata.default_video_stream_index()?;
        
        // Determine encoding parameters
        let quality = choose_quality(video_stream.height);
        let surface = choose_surface(video_stream.bit_depth).to_string();
        
        // Generate output path (same directory, different name during encoding)
        let output_path = input_path
            .with_file_name(format!(
                "{}.av1-tmp.mkv",
                input_path.file_stem()?.to_string_lossy()
            ));
        
        Some(Self {
            input_path,
            output_path,
            video_stream_index,
            quality,
            surface,
            is_webrip,
        })
    }
}

/// Build the complete ffmpeg command line for transcoding
///
/// Constructs the exact command from the spec, following all rules:
/// - Hardware acceleration setup (QSV)
/// - Video stream selection and mapping
/// - Audio/subtitle copying with Russian language filtering
/// - WebRip-specific flags when needed
/// - Proper filter chain for padding and format conversion
///
/// # Arguments
/// * `ffmpeg_path` - Path to the ffmpeg binary
/// * `params` - Transcode parameters
///
/// # Returns
/// Vector of command-line arguments for ffmpeg
pub fn build_ffmpeg_command(_ffmpeg_path: &Path, params: &TranscodeParams) -> Vec<String> {
    let mut args = Vec::new();
    
    // Output control flags
    args.push("-y".to_string()); // Overwrite output
    args.push("-v".to_string());
    args.push("verbose".to_string());
    args.push("-stats".to_string());
    args.push("-benchmark".to_string());
    args.push("-benchmark_all".to_string());
    
    // Hardware acceleration setup
    args.push("-hwaccel".to_string());
    args.push("none".to_string()); // Don't use hwaccel for input
    
    // Initialize QSV hardware device
    // For LXC containers, we need to use VAAPI as the child device since
    // direct DRM access may not work properly with oneVPL
    // Try VAAPI first, then fall back to direct QSV
    let qsv_device = if let Ok(entries) = std::fs::read_dir("/dev/dri") {
        // Check if we have renderD* devices
        let has_render = entries
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with("renderD"));
        
        if has_render {
            // Use VAAPI as child device - this works better in containers
            // VAAPI will handle DRM access, and QSV will use VAAPI
            "qsv=hw,child_device_type=vaapi".to_string()
        } else {
            "qsv=hw".to_string()
        }
    } else {
        "qsv=hw".to_string()
    };
    
    args.push("-init_hw_device".to_string());
    args.push(qsv_device);
    args.push("-filter_hw_device".to_string());
    args.push("hw".to_string());
    
    // Input file analysis settings
    args.push("-analyzeduration".to_string());
    args.push("50M".to_string());
    args.push("-probesize".to_string());
    args.push("50M".to_string());
    
    // WebRip-specific input flags
    if params.is_webrip {
        args.push("-fflags".to_string());
        args.push("+genpts".to_string());
        args.push("-copyts".to_string());
        args.push("-start_at_zero".to_string());
    }
    
    // Input file
    args.push("-i".to_string());
    args.push(params.input_path.to_string_lossy().to_string());
    
    // Stream mapping - complex but follows the spec exactly
    // Map all streams first
    args.push("-map".to_string());
    args.push("0".to_string());
    
    // Remove all video streams
    args.push("-map".to_string());
    args.push("-0:v".to_string());
    
    // Remove attachment streams (fonts, etc.)
    args.push("-map".to_string());
    args.push("-0:t".to_string());
    
    // Add back only the selected video stream
    args.push("-map".to_string());
    args.push(format!("0:v:{}", params.video_stream_index));
    
    // Map audio streams (optional - may not exist)
    args.push("-map".to_string());
    args.push("0:a?".to_string());
    
    // Remove Russian audio streams (both "rus" and "ru")
    args.push("-map".to_string());
    args.push("-0:a:m:language:rus".to_string());
    args.push("-map".to_string());
    args.push("-0:a:m:language:ru".to_string());
    
    // Map subtitle streams (optional)
    args.push("-map".to_string());
    args.push("0:s?".to_string());
    
    // Remove Russian subtitle streams
    args.push("-map".to_string());
    args.push("-0:s:m:language:rus".to_string());
    args.push("-map".to_string());
    args.push("-0:s:m:language:ru".to_string());
    
    // Map chapters
    args.push("-map_chapters".to_string());
    args.push("0".to_string());
    
    // WebRip-specific sync flags
    if params.is_webrip {
        args.push("-vsync".to_string());
        args.push("0".to_string());
        args.push("-avoid_negative_ts".to_string());
        args.push("make_zero".to_string());
    }
    
    // Video filter chain
    // 1. Pad to even dimensions (ceil to nearest even number)
    // 2. Set SAR to 1:1
    // 3. Convert to target surface format
    // 4. Upload to hardware with extra frames for B-frames
    let video_filter = format!(
        "pad=ceil(iw/2)*2:ceil(ih/2)*2,setsar=1,format={},hwupload=extra_hw_frames=64",
        params.surface
    );
    args.push("-vf:v:0".to_string());
    args.push(video_filter);
    
    // Video encoding settings
    args.push("-c:v:0".to_string());
    args.push("av1_qsv".to_string());
    args.push("-global_quality:v:0".to_string());
    args.push(params.quality.to_string());
    args.push("-preset:v:0".to_string());
    args.push("medium".to_string());
    args.push("-look_ahead".to_string());
    args.push("1".to_string());
    
    // Audio and subtitle copying (no re-encoding)
    args.push("-c:a".to_string());
    args.push("copy".to_string());
    args.push("-c:s".to_string());
    args.push("copy".to_string());
    
    // Muxing settings
    args.push("-max_muxing_queue_size".to_string());
    args.push("2048".to_string());
    
    // Copy metadata
    args.push("-map_metadata".to_string());
    args.push("0".to_string());
    
    // Output format
    args.push("-f".to_string());
    args.push("matroska".to_string());
    args.push("-movflags".to_string());
    args.push("+faststart".to_string());
    
    // Output file
    args.push(params.output_path.to_string_lossy().to_string());
    
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{FileMetadata, VideoStreamInfo};
    use std::path::PathBuf;
    
    #[test]
    fn test_build_command_basic() {
        let params = TranscodeParams {
            input_path: PathBuf::from("/media/test.mkv"),
            output_path: PathBuf::from("/media/test.av1-tmp.mkv"),
            video_stream_index: 0,
            quality: 24,
            surface: "nv12".to_string(),
            is_webrip: false,
        };
        
        let args = build_ffmpeg_command(Path::new("/usr/bin/ffmpeg"), &params);
        
        // Check key arguments are present
        assert!(args.contains(&"-c:v:0".to_string()));
        assert!(args.contains(&"av1_qsv".to_string()));
        assert!(args.contains(&"-global_quality:v:0".to_string()));
        assert!(args.contains(&"24".to_string()));
        assert!(args.contains(&"-init_hw_device".to_string()));
        assert!(args.contains(&"qsv=hw".to_string()));
    }
    
    #[test]
    fn test_build_command_webrip() {
        let params = TranscodeParams {
            input_path: PathBuf::from("/media/webrip.mp4"),
            output_path: PathBuf::from("/media/webrip.av1-tmp.mkv"),
            video_stream_index: 0,
            quality: 25,
            surface: "nv12".to_string(),
            is_webrip: true,
        };
        
        let args = build_ffmpeg_command(Path::new("/usr/bin/ffmpeg"), &params);
        
        // Check WebRip-specific flags
        assert!(args.contains(&"-fflags".to_string()));
        assert!(args.contains(&"+genpts".to_string()));
        assert!(args.contains(&"-copyts".to_string()));
        assert!(args.contains(&"-vsync".to_string()));
        assert!(args.contains(&"0".to_string()));
        assert!(args.contains(&"-avoid_negative_ts".to_string()));
        assert!(args.contains(&"make_zero".to_string()));
    }
    
    #[test]
    fn test_transcode_params_from_metadata() {
        let metadata = FileMetadata {
            video_streams: vec![VideoStreamInfo {
                codec: "h264".to_string(),
                width: 1920,
                height: 1080,
                bit_depth: 8,
                is_default: true,
                avg_frame_rate: "24/1".to_string(),
                r_frame_rate: "24/1".to_string(),
            }],
            format_name: "matroska".to_string(),
            tags_muxing_app: None,
            tags_major_brand: None,
            tags_compatible_brands: None,
            size: Some(5000000000),
        };
        
        let params = TranscodeParams::from_metadata(
            PathBuf::from("/media/test.mkv"),
            &metadata,
            false,
        ).unwrap();
        
        assert_eq!(params.quality, 24); // 1080p
        assert_eq!(params.surface, "nv12"); // 8-bit
        assert_eq!(params.video_stream_index, 0);
        assert!(!params.is_webrip);
    }
}

