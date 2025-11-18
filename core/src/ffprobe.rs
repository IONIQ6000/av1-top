// FFprobe integration for extracting media file metadata
// This module provides functions to run ffprobe and parse its output

use crate::error::{CoreError, Result};
use crate::metadata::{FileMetadata, VideoStreamInfo};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// Run ffprobe on a media file and extract metadata
///
/// This function executes ffprobe (from the ffmpeg binary path) and parses
/// the JSON output to extract video stream information and format details.
///
/// # Arguments
/// * `ffmpeg_path` - Path to the ffmpeg binary (ffprobe is assumed to be in the same directory)
/// * `file` - Path to the media file to probe
///
/// # Returns
/// `FileMetadata` containing video streams and format information
///
/// # Errors
/// Returns `CoreError::FFmpegError` if ffprobe cannot be executed or returns an error
/// Returns `CoreError::ParseError` if the ffprobe output cannot be parsed
pub fn run_ffprobe(ffmpeg_path: &Path, file: &Path) -> Result<FileMetadata> {
    // 1. Determine ffprobe path
    // Try to find ffprobe in the same directory as ffmpeg
    let ffprobe_path = if let Some(parent) = ffmpeg_path.parent() {
        parent.join("ffprobe")
    } else {
        // If no parent directory, just try "ffprobe" from PATH
        std::path::PathBuf::from("ffprobe")
    };

    // 2. Execute ffprobe with JSON output
    let file_str = file
        .to_str()
        .ok_or_else(|| CoreError::FFmpegError("Invalid file path".into()))?;

    let output = Command::new(&ffprobe_path)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            file_str,
        ])
        .output()
        .map_err(|e| {
            CoreError::FFmpegError(format!(
                "Failed to run ffprobe at {:?}: {}",
                ffprobe_path, e
            ))
        })?;

    // 3. Check if ffprobe succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::FFmpegError(format!(
            "ffprobe failed for {}: {}",
            file.display(),
            stderr
        )));
    }

    // 4. Parse JSON output
    let json_str = String::from_utf8_lossy(&output.stdout);
    let parsed: FFprobeOutput = serde_json::from_str(&json_str).map_err(|e| {
        CoreError::ParseError(format!("Failed to parse ffprobe JSON: {}", e))
    })?;

    // 5. Convert to FileMetadata
    convert_ffprobe_output(parsed, file)
}

/// Helper struct for deserializing ffprobe JSON output
#[derive(Deserialize, Debug)]
struct FFprobeOutput {
    streams: Vec<FFprobeStream>,
    format: FFprobeFormat,
}

/// Stream information from ffprobe
#[derive(Deserialize, Debug)]
struct FFprobeStream {
    /// Type of stream: "video", "audio", "subtitle", etc.
    codec_type: String,
    /// Codec name: "h264", "hevc", "av1", etc.
    codec_name: String,
    /// Video width in pixels (video streams only)
    width: Option<u32>,
    /// Video height in pixels (video streams only)
    height: Option<u32>,
    /// Pixel format: "yuv420p", "yuv420p10le", etc.
    pix_fmt: Option<String>,
    /// Bits per raw sample (may be missing or "N/A")
    bits_per_raw_sample: Option<String>,
    /// Average frame rate as fraction string: "24/1", "30000/1001", etc.
    avg_frame_rate: String,
    /// Real (container) frame rate as fraction string
    r_frame_rate: String,
    /// Stream disposition flags
    disposition: Option<FFprobeDisposition>,
}

/// Disposition flags for a stream
#[derive(Deserialize, Debug)]
struct FFprobeDisposition {
    /// 1 if this is the default stream for its type, 0 otherwise
    default: i32,
}

/// Format (container) information from ffprobe
#[derive(Deserialize, Debug)]
struct FFprobeFormat {
    /// Format name: "matroska,webm", "mov,mp4,m4a,3gp,3g2,mj2", etc.
    format_name: String,
    /// File size as string (may be missing)
    size: Option<String>,
    /// Format tags/metadata
    tags: Option<FFprobeTags>,
}

/// Tags from format metadata
/// Note: Tag names can vary by container format
#[derive(Deserialize, Debug)]
struct FFprobeTags {
    /// Muxing application (Matroska: MUXING_APP, MP4: may not exist)
    #[serde(rename = "MUXING_APP")]
    muxing_app_upper: Option<String>,
    #[serde(rename = "muxing_app")]
    muxing_app_lower: Option<String>,
    
    /// Major brand (MP4 containers)
    #[serde(rename = "major_brand")]
    major_brand: Option<String>,
    
    /// Compatible brands (MP4 containers)
    #[serde(rename = "compatible_brands")]
    compatible_brands: Option<String>,
}

/// Convert ffprobe output to our FileMetadata structure
///
/// Extracts video streams, format information, and tags from the raw ffprobe JSON.
fn convert_ffprobe_output(output: FFprobeOutput, file: &Path) -> Result<FileMetadata> {
    // Extract video streams only
    let video_streams: Vec<VideoStreamInfo> = output
        .streams
        .into_iter()
        .filter(|s| s.codec_type == "video")
        .map(|s| {
            // Parse bit depth from pix_fmt or bits_per_raw_sample
            let bit_depth = parse_bit_depth(&s.pix_fmt, &s.bits_per_raw_sample);

            VideoStreamInfo {
                codec: s.codec_name,
                width: s.width.unwrap_or(0),
                height: s.height.unwrap_or(0),
                bit_depth,
                is_default: s
                    .disposition
                    .as_ref()
                    .map(|d| d.default == 1)
                    .unwrap_or(false),
                avg_frame_rate: s.avg_frame_rate,
                r_frame_rate: s.r_frame_rate,
            }
        })
        .collect();

    // Parse file size from format or fall back to filesystem
    let size = output
        .format
        .size
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| {
            // Try to get size from filesystem if ffprobe didn't provide it
            std::fs::metadata(file).ok().map(|m| m.len())
        });

    // Extract tags, handling case variations
    let tags = output.format.tags;
    let muxing_app = tags
        .as_ref()
        .and_then(|t| t.muxing_app_upper.clone().or_else(|| t.muxing_app_lower.clone()));
    let major_brand = tags.as_ref().and_then(|t| t.major_brand.clone());
    let compatible_brands = tags.as_ref().and_then(|t| t.compatible_brands.clone());

    Ok(FileMetadata {
        video_streams,
        format_name: output.format.format_name,
        tags_muxing_app: muxing_app,
        tags_major_brand: major_brand,
        tags_compatible_brands: compatible_brands,
        size,
    })
}

/// Parse bit depth from pixel format and bits_per_raw_sample
///
/// Tries multiple methods to determine bit depth:
/// 1. Parse bits_per_raw_sample if available
/// 2. Infer from pixel format name (e.g., "yuv420p10le" -> 10-bit)
/// 3. Default to 8-bit if unable to determine
fn parse_bit_depth(pix_fmt: &Option<String>, bits_str: &Option<String>) -> u8 {
    // Try bits_per_raw_sample first
    if let Some(bits) = bits_str {
        if let Ok(depth) = bits.parse::<u8>() {
            return depth;
        }
    }

    // Try to infer from pixel format name
    if let Some(fmt) = pix_fmt {
        let fmt_lower = fmt.to_lowercase();
        
        // Look for "10le", "10be", "p10", etc. in the format name
        if fmt_lower.contains("10le") || fmt_lower.contains("10be") || fmt_lower.contains("p10") {
            return 10;
        }
        
        // Look for 12-bit indicators
        if fmt_lower.contains("12le") || fmt_lower.contains("12be") || fmt_lower.contains("p12") {
            return 12;
        }
        
        // Look for 16-bit indicators
        if fmt_lower.contains("16le") || fmt_lower.contains("16be") || fmt_lower.contains("p16") {
            return 16;
        }
    }

    // Default to 8-bit
    8
}

