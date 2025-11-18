// Heuristics for deciding whether and how to transcode files
// These pure functions implement the business logic for the transcoder

use crate::constants::{bitdepth, resolution};
use crate::metadata::FileMetadata;

/// Determine if a file is WebRip-like based on heuristics
///
/// A file is considered WebRip-like if any of these conditions are true:
/// - Format name includes "mp4", "mov", or "webm"
/// - Any video stream is Variable Frame Rate (VFR)
/// - Any video stream has odd dimensions (width or height not divisible by 2)
///
/// WebRip-like files need special handling during transcoding:
/// - Add fflags +genpts -copyts -start_at_zero
/// - Add -vsync 0 -avoid_negative_ts make_zero
pub fn is_webrip_like(meta: &FileMetadata) -> bool {
    // Check format name for WebRip indicators
    let format_lower = meta.format_name.to_lowercase();
    if format_lower.contains("mp4")
        || format_lower.contains("mov")
        || format_lower.contains("webm")
    {
        return true;
    }

    // Check video streams for VFR or odd dimensions
    for stream in &meta.video_streams {
        if stream.is_vfr() || stream.has_odd_dimensions() {
            return true;
        }
    }

    false
}

/// Check if a file should be skipped due to being too small
///
/// Files below the minimum size threshold are not worth transcoding
/// as the size savings would be minimal.
///
/// # Arguments
/// * `bytes` - Size of the file in bytes
/// * `min_bytes` - Minimum size threshold in bytes
///
/// # Returns
/// `true` if the file should be skipped (too small), `false` otherwise
pub fn should_skip_for_size(bytes: u64, min_bytes: u64) -> bool {
    bytes < min_bytes
}

/// Check if a file is already encoded in AV1
///
/// Scans all video streams and returns true if any of them use the AV1 codec.
/// Files already in AV1 should be skipped.
///
/// # Arguments
/// * `meta` - File metadata from ffprobe
///
/// # Returns
/// `true` if any video stream is AV1, `false` otherwise
pub fn is_already_av1(meta: &FileMetadata) -> bool {
    meta.video_streams
        .iter()
        .any(|s| s.codec.to_lowercase() == "av1")
}

/// Choose the quality parameter for QSV encoding based on video height
///
/// Quality values are used with the -global_quality parameter.
/// Lower values = higher quality, higher bitrate
///
/// # Resolution buckets:
/// - Below 1080p (height < 1080): quality 25
/// - At 1080p (height == 1080): quality 24
/// - At 1440p and above (height >= 1440): quality 23
///
/// # Arguments
/// * `height` - Video height in pixels
///
/// # Returns
/// Quality parameter value (23, 24, or 25)
pub fn choose_quality(height: u32) -> u8 {
    use crate::constants::defaults;
    
    if height < resolution::HEIGHT_1080P {
        defaults::QSV_QUALITY_BELOW_1080P
    } else if height == resolution::HEIGHT_1080P {
        defaults::QSV_QUALITY_AT_1080P
    } else {
        // height >= 1440
        defaults::QSV_QUALITY_AT_1440P_PLUS
    }
}

/// Choose the pixel format (surface) for QSV encoding based on bit depth
///
/// Intel QSV requires specific pixel formats for hardware upload:
/// - 10-bit or higher content: use p010 (10-bit 4:2:0 format)
/// - 8-bit content: use nv12 (8-bit 4:2:0 format)
///
/// # Arguments
/// * `bit_depth` - Bit depth of the video (8, 10, 12, etc.)
///
/// # Returns
/// Pixel format string for ffmpeg ("p010" or "nv12")
pub fn choose_surface(bit_depth: u8) -> &'static str {
    if bit_depth >= bitdepth::USE_10BIT_THRESHOLD {
        "p010"
    } else {
        "nv12"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{FileMetadata, VideoStreamInfo};
    use crate::constants::units::GIB;

    #[test]
    fn test_choose_quality() {
        assert_eq!(choose_quality(720), 25);
        assert_eq!(choose_quality(1079), 25);
        assert_eq!(choose_quality(1080), 24);
        assert_eq!(choose_quality(1440), 23);
        assert_eq!(choose_quality(2160), 23);
    }

    #[test]
    fn test_choose_surface() {
        assert_eq!(choose_surface(8), "nv12");
        assert_eq!(choose_surface(9), "nv12");
        assert_eq!(choose_surface(10), "p010");
        assert_eq!(choose_surface(12), "p010");
    }

    #[test]
    fn test_should_skip_for_size() {
        let two_gib = 2 * GIB;
        assert!(should_skip_for_size(1000, two_gib));
        assert!(should_skip_for_size(two_gib - 1, two_gib));
        assert!(!should_skip_for_size(two_gib, two_gib));
        assert!(!should_skip_for_size(two_gib + 1, two_gib));
    }

    #[test]
    fn test_is_already_av1() {
        let mut meta = FileMetadata {
            video_streams: vec![],
            format_name: "matroska".to_string(),
            tags_muxing_app: None,
            tags_major_brand: None,
            tags_compatible_brands: None,
            size: None,
        };

        // No streams
        assert!(!is_already_av1(&meta));

        // H264 stream
        meta.video_streams.push(VideoStreamInfo {
            codec: "h264".to_string(),
            width: 1920,
            height: 1080,
            bit_depth: 8,
            is_default: true,
            avg_frame_rate: "24/1".to_string(),
            r_frame_rate: "24/1".to_string(),
        });
        assert!(!is_already_av1(&meta));

        // AV1 stream (lowercase)
        meta.video_streams[0].codec = "av1".to_string();
        assert!(is_already_av1(&meta));

        // AV1 stream (uppercase)
        meta.video_streams[0].codec = "AV1".to_string();
        assert!(is_already_av1(&meta));
    }

    #[test]
    fn test_is_webrip_like_format() {
        let meta = FileMetadata {
            video_streams: vec![VideoStreamInfo {
                codec: "h264".to_string(),
                width: 1920,
                height: 1080,
                bit_depth: 8,
                is_default: true,
                avg_frame_rate: "24/1".to_string(),
                r_frame_rate: "24/1".to_string(),
            }],
            format_name: "mov,mp4,m4a,3gp,3g2,mj2".to_string(),
            tags_muxing_app: None,
            tags_major_brand: None,
            tags_compatible_brands: None,
            size: None,
        };

        assert!(is_webrip_like(&meta));
    }

    #[test]
    fn test_is_webrip_like_vfr() {
        let meta = FileMetadata {
            video_streams: vec![VideoStreamInfo {
                codec: "h264".to_string(),
                width: 1920,
                height: 1080,
                bit_depth: 8,
                is_default: true,
                avg_frame_rate: "23/1".to_string(),
                r_frame_rate: "24/1".to_string(), // Different = VFR
            }],
            format_name: "matroska".to_string(),
            tags_muxing_app: None,
            tags_major_brand: None,
            tags_compatible_brands: None,
            size: None,
        };

        assert!(is_webrip_like(&meta));
    }

    #[test]
    fn test_is_webrip_like_odd_dimensions() {
        let meta = FileMetadata {
            video_streams: vec![VideoStreamInfo {
                codec: "h264".to_string(),
                width: 1921, // Odd width
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
            size: None,
        };

        assert!(is_webrip_like(&meta));
    }

    #[test]
    fn test_is_not_webrip_like() {
        let meta = FileMetadata {
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
            size: None,
        };

        assert!(!is_webrip_like(&meta));
    }
}

