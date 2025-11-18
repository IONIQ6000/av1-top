// FFmpeg installation and management
// Auto-detects, validates, and optionally installs FFmpeg 8.0+

use crate::error::{CoreError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// FFmpeg installation information
#[derive(Debug, Clone)]
pub struct FFmpegInstallation {
    /// Path to ffmpeg binary
    pub ffmpeg_path: PathBuf,
    
    /// Path to ffprobe binary
    pub ffprobe_path: PathBuf,
    
    /// Version string (e.g., "8.0" or "n8.0")
    pub version: String,
    
    /// Whether av1_qsv encoder is available
    pub has_av1_qsv: bool,
    
    /// Whether QSV hardware test passed
    pub qsv_hardware_works: bool,
}

/// Find and validate FFmpeg 8.0+ installation
///
/// Searches for ffmpeg in the following order:
/// 1. System PATH (ffmpeg/ffprobe commands)
/// 2. Common installation locations
/// 3. If none found or version < 8.0, provides installation guidance
///
/// # Returns
/// FFmpegInstallation with paths and validation results, or error if not found/invalid
pub fn find_and_validate_ffmpeg() -> Result<FFmpegInstallation> {
    // Try to find ffmpeg in PATH first
    if let Ok(installation) = check_ffmpeg_at_path("ffmpeg") {
        return Ok(installation);
    }
    
    // Try common installation locations
    let common_locations = vec![
        "/usr/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/ffmpeg/bin/ffmpeg",
        "/snap/bin/ffmpeg",
    ];
    
    for location in common_locations {
        if Path::new(location).exists() {
            if let Ok(installation) = check_ffmpeg_at_path(location) {
                return Ok(installation);
            }
        }
    }
    
    // No valid FFmpeg found
    Err(CoreError::FFmpegError(
        "FFmpeg 8.0+ not found. Please install it using your package manager or see FFMPEG_SETUP.md".into()
    ))
}

/// Check if ffmpeg at given path is valid (version 8.0+, has QSV)
fn check_ffmpeg_at_path(ffmpeg_path: &str) -> Result<FFmpegInstallation> {
    let ffmpeg = PathBuf::from(ffmpeg_path);
    
    // Check if the binary exists and is executable
    if !ffmpeg.exists() {
        return Err(CoreError::FFmpegError(format!("FFmpeg not found at {}", ffmpeg_path)));
    }
    
    // Get version
    let version_output = Command::new(&ffmpeg)
        .arg("-version")
        .output()
        .map_err(|e| CoreError::FFmpegError(format!("Failed to run ffmpeg: {}", e)))?;
    
    if !version_output.status.success() {
        return Err(CoreError::FFmpegError("FFmpeg execution failed".into()));
    }
    
    let version_text = String::from_utf8_lossy(&version_output.stdout);
    let version = extract_version(&version_text)?;
    
    // Validate version is 8.0 or later
    if !is_version_valid(&version) {
        return Err(CoreError::FFmpegError(format!(
            "FFmpeg version {} is too old. Need version 8.0+ (August 2025). See FFMPEG_SETUP.md",
            version
        )));
    }
    
    // Find ffprobe (should be in same directory or PATH)
    let ffprobe = find_ffprobe(&ffmpeg)?;
    
    // Check for av1_qsv encoder
    let encoders_output = Command::new(&ffmpeg)
        .arg("-encoders")
        .output()
        .map_err(|e| CoreError::FFmpegError(format!("Failed to list encoders: {}", e)))?;
    
    let encoders = String::from_utf8_lossy(&encoders_output.stdout);
    let has_av1_qsv = encoders.contains("av1_qsv");
    
    if !has_av1_qsv {
        return Err(CoreError::FFmpegError(
            "FFmpeg does not have av1_qsv encoder. You need a build with Intel QSV support. See FFMPEG_SETUP.md".into()
        ));
    }
    
    // Test QSV hardware (optional, doesn't fail validation)
    let qsv_hardware_works = test_qsv_hardware(&ffmpeg);
    
    Ok(FFmpegInstallation {
        ffmpeg_path: ffmpeg,
        ffprobe_path: ffprobe,
        version,
        has_av1_qsv,
        qsv_hardware_works,
    })
}

/// Extract version from ffmpeg -version output
fn extract_version(output: &str) -> Result<String> {
    // First line: "ffmpeg version n8.0 Copyright..."
    let first_line = output
        .lines()
        .next()
        .ok_or_else(|| CoreError::FFmpegError("Empty version output".into()))?;
    
    if let Some(start) = first_line.find("version ") {
        let version_part = &first_line[start + 8..];
        let version = version_part
            .split_whitespace()
            .next()
            .ok_or_else(|| CoreError::FFmpegError("Could not parse version".into()))?;
        Ok(version.to_string())
    } else {
        Err(CoreError::FFmpegError("Version not found in output".into()))
    }
}

/// Check if version is 8.0 or later
fn is_version_valid(version: &str) -> bool {
    // Check for 8.x or n8.x format
    if version.starts_with("8.") || version.starts_with("n8.") {
        return true;
    }
    
    // Check for higher versions (9.x, 10.x, etc.)
    if let Some(first_char) = version.chars().next() {
        if let Some(major) = first_char.to_digit(10) {
            return major >= 8;
        }
    }
    
    // Also check n-prefix versions
    if version.starts_with('n') {
        if let Some(first_digit) = version[1..].chars().next() {
            if let Some(major) = first_digit.to_digit(10) {
                return major >= 8;
            }
        }
    }
    
    false
}

/// Find ffprobe binary (should be alongside ffmpeg)
fn find_ffprobe(ffmpeg_path: &Path) -> Result<PathBuf> {
    // Try in same directory as ffmpeg
    if let Some(parent) = ffmpeg_path.parent() {
        let ffprobe = parent.join("ffprobe");
        if ffprobe.exists() {
            return Ok(ffprobe);
        }
    }
    
    // Try in PATH
    let ffprobe = PathBuf::from("ffprobe");
    let test = Command::new(&ffprobe).arg("-version").output();
    if test.is_ok() {
        return Ok(ffprobe);
    }
    
    Err(CoreError::FFmpegError(
        "ffprobe not found. It should be installed alongside ffmpeg".into()
    ))
}

/// Test if QSV hardware is accessible
fn test_qsv_hardware(ffmpeg_path: &Path) -> bool {
    let output = Command::new(ffmpeg_path)
        .args([
            "-hide_banner",
            "-init_hw_device",
            "qsv=hw",
            "-filter_hw_device",
            "hw",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=64x64:d=0.1",
            "-vf",
            "format=nv12,hwupload=extra_hw_frames=64",
            "-c:v",
            "av1_qsv",
            "-frames:v",
            "1",
            "-f",
            "null",
            "-",
        ])
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Get installation instructions based on platform
pub fn get_installation_instructions() -> String {
    let os = std::env::consts::OS;
    
    match os {
        "linux" => {
            r#"FFmpeg 8.0+ Installation Required

To install FFmpeg 8.0 (August 2025) with Intel QSV support:

Ubuntu/Debian:
  sudo apt update
  sudo apt install -y ffmpeg

  # If version is too old, build from source:
  # See FFMPEG_SETUP.md for detailed instructions

Arch Linux:
  sudo pacman -S ffmpeg

Fedora:
  sudo dnf install ffmpeg

Or download static build:
  wget https://johnvansickle.com/ffmpeg/builds/ffmpeg-git-amd64-static.tar.xz
  tar xf ffmpeg-git-amd64-static.tar.xz
  sudo cp ffmpeg-git-*/ffmpeg /usr/local/bin/
  sudo cp ffmpeg-git-*/ffprobe /usr/local/bin/

For complete setup including Intel GPU drivers, see FFMPEG_SETUP.md
"#.to_string()
        }
        "macos" => {
            r#"FFmpeg 8.0+ Installation Required

To install FFmpeg 8.0 on macOS:

Using Homebrew:
  brew install ffmpeg

Note: Intel QSV is only available on Linux. On macOS, use:
  - VideoToolbox hardware acceleration (not AV1)
  - Or software encoding (very slow)

This project is designed for Linux with Intel GPUs.
"#.to_string()
        }
        _ => {
            "FFmpeg 8.0+ required. See FFMPEG_SETUP.md for installation instructions.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_validation() {
        assert!(is_version_valid("8.0"));
        assert!(is_version_valid("n8.0"));
        assert!(is_version_valid("8.1"));
        assert!(is_version_valid("9.0"));
        assert!(is_version_valid("n9.0"));
        
        assert!(!is_version_valid("7.0"));
        assert!(!is_version_valid("n7.0"));
        assert!(!is_version_valid("6.1"));
    }
    
    #[test]
    fn test_extract_version() {
        let output = "ffmpeg version n8.0 Copyright (c) 2000-2025";
        let version = extract_version(output).unwrap();
        assert_eq!(version, "n8.0");
        
        let output2 = "ffmpeg version 8.0.1-static https://johnvansickle.com/ffmpeg/";
        let version2 = extract_version(output2).unwrap();
        assert!(version2.starts_with("8."));
    }
}

