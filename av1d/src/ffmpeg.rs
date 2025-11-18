// FFmpeg command builder for AV1 transcoding with Intel QSV
// Constructs the complex ffmpeg command line according to the project spec

use core::{choose_quality, choose_surface, is_webrip_like, FileMetadata, TranscodeConfig};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build an ffmpeg command for AV1 transcoding
///
/// Constructs the full ffmpeg command with all necessary parameters for
/// Intel QSV AV1 encoding, including WebRip-specific flags when needed.
///
/// # Arguments
/// * `config` - Transcode configuration with ffmpeg path and settings
/// * `input` - Path to the input file
/// * `output` - Path to the output file (will be created)
/// * `metadata` - File metadata from ffprobe
///
/// # Returns
/// A configured `Command` ready to be spawned
pub fn build_transcode_command(
    config: &TranscodeConfig,
    input: &Path,
    output: &Path,
    metadata: &FileMetadata,
) -> Command {
    let mut cmd = Command::new(&config.ffmpeg_path);

    // Get the default video stream for parameter selection
    let default_stream = metadata
        .default_video_stream()
        .expect("Should have validated video stream exists");
    
    let video_index = metadata
        .default_video_stream_index()
        .expect("Should have video stream index");

    // Determine encoding parameters based on metadata
    let quality = choose_quality(default_stream.height);
    let surface = choose_surface(default_stream.bit_depth);
    let is_webrip = is_webrip_like(metadata);

    // === Input flags ===
    cmd.arg("-y"); // Overwrite output file
    cmd.arg("-v").arg("verbose"); // Verbose logging
    cmd.arg("-stats"); // Show encoding stats
    cmd.arg("-benchmark"); // Show benchmark timing
    cmd.arg("-benchmark_all"); // Benchmark all stages

    // Hardware acceleration setup
    cmd.arg("-hwaccel").arg("none"); // Don't use hwaccel for input
    
    // Initialize VAAPI device with explicit DRM render node path
    // Find the first renderD* device
    let mut use_vaapi_device = false;
    if let Ok(entries) = std::fs::read_dir("/dev/dri") {
        if let Some(render_device) = entries
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().starts_with("renderD"))
            .map(|e| format!("/dev/dri/{}", e.file_name().to_string_lossy()))
        {
            // Use VAAPI with explicit device path
            // Format: vaapi=name:/path/to/device
            let device_arg = format!("vaapi=va:{}", render_device);
            eprintln!("[DEBUG] Initializing VAAPI device: {}", device_arg);
            cmd.arg("-init_hw_device").arg(device_arg);
            cmd.arg("-filter_hw_device").arg("va");
            use_vaapi_device = true;
        } else {
            eprintln!("[DEBUG] No renderD* device found, using QSV fallback");
            // Fallback to QSV if no render device found
            cmd.arg("-init_hw_device").arg("qsv=hw");
            cmd.arg("-filter_hw_device").arg("hw");
        }
    } else {
        eprintln!("[DEBUG] /dev/dri not accessible, using QSV fallback");
        // Fallback to QSV if /dev/dri doesn't exist
        cmd.arg("-init_hw_device").arg("qsv=hw");
        cmd.arg("-filter_hw_device").arg("hw");
    }

    // Input analysis parameters
    cmd.arg("-analyzeduration").arg("50M");
    cmd.arg("-probesize").arg("50M");

    // WebRip-specific input flags
    if is_webrip {
        cmd.arg("-fflags").arg("+genpts");
        cmd.arg("-copyts");
        cmd.arg("-start_at_zero");
    }

    // Input file
    cmd.arg("-i");
    cmd.arg(input);

    // === Stream mapping ===
    
    // Map all streams first
    cmd.arg("-map").arg("0");
    
    // Remove all video streams (we'll add back the one we want)
    cmd.arg("-map").arg("-0:v");
    
    // Remove text/subtitle tracks that might cause issues
    cmd.arg("-map").arg("-0:t");
    
    // Map the specific video stream we want
    cmd.arg("-map").arg(format!("0:v:{}", video_index));
    
    // Map audio streams (if any exist)
    cmd.arg("-map").arg("0:a?");
    
    // Exclude Russian audio tracks
    cmd.arg("-map").arg("-0:a:m:language:rus");
    cmd.arg("-map").arg("-0:a:m:language:ru");
    
    // Map subtitle streams (if any exist)
    cmd.arg("-map").arg("0:s?");
    
    // Exclude Russian subtitle tracks
    cmd.arg("-map").arg("-0:s:m:language:rus");
    cmd.arg("-map").arg("-0:s:m:language:ru");
    
    // Map chapters
    cmd.arg("-map_chapters").arg("0");

    // === WebRip sync flags ===
    if is_webrip {
        cmd.arg("-vsync").arg("0");
        cmd.arg("-avoid_negative_ts").arg("make_zero");
    }

    // === Video filter chain ===
    
    // Use the same VAAPI detection as hardware initialization
    // (re-check to be safe, but should match the initialization above)
    let use_vaapi = use_vaapi_device;
    
    // Build the video filter:
    // 1. Pad to even dimensions (required for encoding)
    // 2. Set SAR to 1:1
    // 3. Convert to appropriate pixel format
    // 4. Upload to hardware
    let filter = if use_vaapi {
        // VAAPI filter: pad, set SAR, convert to NV12, upload to VAAPI
        format!(
            "pad=ceil(iw/2)*2:ceil(ih/2)*2,setsar=1,format=nv12,hwupload"
        )
    } else {
        // QSV filter: pad, set SAR, convert to surface format, upload to QSV
        format!(
            "pad=ceil(iw/2)*2:ceil(ih/2)*2,setsar=1,format={},hwupload=extra_hw_frames=64",
            surface
        )
    };
    cmd.arg("-vf:v:0").arg(filter);

    // === Video encoding parameters ===
    
    if use_vaapi {
        // Use VAAPI AV1 encoder
        cmd.arg("-c:v:0").arg("av1_vaapi");
        // VAAPI uses compression_level (0-8, lower = better quality)
        // Map QSV quality (23-25) to VAAPI compression_level (roughly: 25->6, 24->5, 23->4)
        let compression_level = match quality {
            23 => "4",  // Best quality
            24 => "5",  // High quality
            25 => "6",  // Good quality
            _ => "5",
        };
        cmd.arg("-compression_level:v:0").arg(compression_level);
    } else {
        // Use QSV AV1 encoder (fallback)
        cmd.arg("-c:v:0").arg("av1_qsv");
        cmd.arg("-global_quality:v:0").arg(quality.to_string()); // Quality parameter
        cmd.arg("-preset:v:0").arg("medium"); // Encoding preset
        cmd.arg("-look_ahead").arg("1"); // Enable lookahead
    }

    // === Audio and subtitle encoding ===
    
    cmd.arg("-c:a").arg("copy"); // Copy audio streams
    cmd.arg("-c:s").arg("copy"); // Copy subtitle streams

    // === Muxing parameters ===
    
    cmd.arg("-max_muxing_queue_size").arg("2048"); // Large queue for complex files
    cmd.arg("-map_metadata").arg("0"); // Copy metadata from input

    // === Output format ===
    
    cmd.arg("-f").arg("matroska"); // Use Matroska container
    cmd.arg("-movflags").arg("+faststart"); // Optimize for streaming

    // Output file
    cmd.arg(output);

    cmd
}

/// Generate the output path for a transcode job
///
/// Creates a temporary output path in the same directory as the input,
/// with .av1-tmp.mkv extension. This will be renamed on success.
///
/// # Arguments
/// * `input` - Path to the input file
///
/// # Returns
/// Path for the temporary output file
pub fn get_temp_output_path(input: &Path) -> PathBuf {
    let parent = input.parent().expect("Input file must have a parent directory");
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Input file must have a valid filename");
    
    parent.join(format!("{}.av1-tmp.mkv", stem))
}

/// Generate the final output path after successful transcode
///
/// This is what the file will be renamed to after passing the size gate.
///
/// # Arguments
/// * `input` - Path to the input file
///
/// # Returns
/// Path for the final output file
pub fn get_final_output_path(input: &Path) -> PathBuf {
    let parent = input.parent().expect("Input file must have a parent directory");
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Input file must have a valid filename");
    
    // Use .mkv extension for the final file
    parent.join(format!("{}.mkv", stem))
}

/// Get the path for the .av1skip marker file
///
/// # Arguments
/// * `input` - Path to the input file
///
/// # Returns
/// Path for the .av1skip marker file
pub fn get_skip_marker_path(input: &Path) -> PathBuf {
    let parent = input.parent().expect("Input file must have a parent directory");
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Input file must have a valid filename");
    
    parent.join(format!("{}.av1skip", stem))
}

/// Get the path for the .why.txt explanation file
///
/// # Arguments
/// * `input` - Path to the input file
///
/// # Returns
/// Path for the .why.txt file
pub fn get_why_file_path(input: &Path) -> PathBuf {
    let parent = input.parent().expect("Input file must have a parent directory");
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Input file must have a valid filename");
    
    parent.join(format!("{}.why.txt", stem))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_temp_output_path() {
        let input = PathBuf::from("/media/movies/example.mkv");
        let output = get_temp_output_path(&input);
        assert_eq!(output, PathBuf::from("/media/movies/example.av1-tmp.mkv"));
    }

    #[test]
    fn test_final_output_path() {
        let input = PathBuf::from("/media/movies/example.mp4");
        let output = get_final_output_path(&input);
        assert_eq!(output, PathBuf::from("/media/movies/example.mkv"));
    }

    #[test]
    fn test_skip_marker_path() {
        let input = PathBuf::from("/media/movies/example.mkv");
        let marker = get_skip_marker_path(&input);
        assert_eq!(marker, PathBuf::from("/media/movies/example.av1skip"));
    }

    #[test]
    fn test_why_file_path() {
        let input = PathBuf::from("/media/movies/example.mkv");
        let why = get_why_file_path(&input);
        assert_eq!(why, PathBuf::from("/media/movies/example.why.txt"));
    }
}

