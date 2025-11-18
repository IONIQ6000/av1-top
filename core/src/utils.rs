// Utility functions used across the codebase
// Centralized to avoid duplication

use crate::constants::units::{GIB, KIB, MIB, TIB};

/// Format bytes as human-readable string with appropriate unit
///
/// Automatically selects the best unit (B, KiB, MiB, GiB, TiB) based on size.
///
/// # Arguments
/// * `bytes` - Number of bytes to format
///
/// # Returns
/// Formatted string like "5.23 GiB" or "123 MiB"
///
/// # Examples
/// ```
/// # fn format_bytes(bytes: u64) -> String { String::new() }
/// // In actual usage:
/// // use core::utils::format_bytes;
/// // assert_eq!(format_bytes(1024), "1 KiB");
/// ```
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.0} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Parse bytes value with unit suffix (used for parsing ffmpeg output)
///
/// # Arguments
/// * `size_str` - String like "12345kB", "123MB", "2GB"
///
/// # Returns
/// Size in bytes
pub fn parse_size_with_unit(size_str: &str) -> u64 {
    if let Some(kb_str) = size_str.strip_suffix("kB") {
        kb_str.parse::<u64>().unwrap_or(0) * KIB
    } else if let Some(mb_str) = size_str.strip_suffix("MB") {
        mb_str.parse::<u64>().unwrap_or(0) * MIB
    } else if let Some(gb_str) = size_str.strip_suffix("GB") {
        gb_str.parse::<u64>().unwrap_or(0) * GIB
    } else {
        size_str.parse().unwrap_or(0)
    }
}

/// Parse time string in HH:MM:SS.MS format to seconds
///
/// # Arguments
/// * `time_str` - Time string like "00:01:23.45"
///
/// # Returns
/// Time in seconds as f64
pub fn parse_time_to_seconds(time_str: &str) -> f64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let hours: f64 = parts[0].parse().unwrap_or(0.0);
        let minutes: f64 = parts[1].parse().unwrap_or(0.0);
        let seconds: f64 = parts[2].parse().unwrap_or(0.0);
        hours * 3600.0 + minutes * 60.0 + seconds
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1 KiB");
        assert_eq!(format_bytes(1536), "2 KiB"); // Rounds to 2
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(5 * 1024 * 1024 * 1024), "5.00 GiB");
    }

    #[test]
    fn test_parse_size_with_unit() {
        assert_eq!(parse_size_with_unit("1024kB"), 1024 * 1024);
        assert_eq!(parse_size_with_unit("5MB"), 5 * 1024 * 1024);
        assert_eq!(parse_size_with_unit("2GB"), 2 * 1024 * 1024 * 1024);
        assert_eq!(parse_size_with_unit("100"), 100);
    }

    #[test]
    fn test_parse_time_to_seconds() {
        assert_eq!(parse_time_to_seconds("00:01:23.45"), 83.45);
        assert_eq!(parse_time_to_seconds("01:00:00.00"), 3600.0);
        assert_eq!(parse_time_to_seconds("00:00:30.50"), 30.5);
    }
}

