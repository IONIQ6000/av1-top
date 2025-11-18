// Post-processing after transcoding
// Handles size gate verification, file replacement, and marker file generation

use crate::error::{CoreError, Result};
use std::fs;
use std::path::Path;

/// Result of size gate check
#[derive(Debug, Clone, PartialEq)]
pub enum SizeGateResult {
    /// File passed size gate (new file is smaller than threshold)
    Passed {
        original_bytes: u64,
        new_bytes: u64,
        savings_ratio: f64,
    },
    
    /// File failed size gate (new file is too large)
    Failed {
        original_bytes: u64,
        new_bytes: u64,
        ratio: f64,
        threshold: f64,
    },
}

/// Check if transcoded file passes the size gate
///
/// Compares the original and new file sizes. The new file must be smaller than
/// (original_size * size_gate_factor) to pass.
///
/// # Arguments
/// * `original_path` - Path to the original file
/// * `new_path` - Path to the transcoded file
/// * `size_gate_factor` - Maximum allowed ratio (e.g., 0.9 = 90%)
///
/// # Returns
/// Result indicating whether the file passed or failed, with size information
pub fn check_size_gate(
    original_path: &Path,
    new_path: &Path,
    size_gate_factor: f64,
) -> Result<SizeGateResult> {
    // Get file sizes
    let original_bytes = fs::metadata(original_path)
        .map_err(|e| CoreError::IoError(e))?
        .len();
    
    let new_bytes = fs::metadata(new_path)
        .map_err(|e| CoreError::IoError(e))?
        .len();
    
    // Calculate ratio (new / original)
    let ratio = new_bytes as f64 / original_bytes as f64;
    
    // Check if it passes the gate
    if ratio <= size_gate_factor {
        let savings_ratio = 1.0 - ratio;
        Ok(SizeGateResult::Passed {
            original_bytes,
            new_bytes,
            savings_ratio,
        })
    } else {
        Ok(SizeGateResult::Failed {
            original_bytes,
            new_bytes,
            ratio,
            threshold: size_gate_factor,
        })
    }
}

/// Write a .why.txt file explaining why a file was rejected
///
/// # Arguments
/// * `original_path` - Path to the original file
/// * `reason` - Explanation text
pub fn write_why_file(original_path: &Path, reason: &str) -> Result<()> {
    let why_path = original_path.with_extension("why.txt");
    fs::write(&why_path, reason)?;
    Ok(())
}

/// Write a .av1skip marker file
///
/// This file signals that the original file should not be transcoded again.
///
/// # Arguments
/// * `original_path` - Path to the original file
pub fn write_skip_marker(original_path: &Path) -> Result<()> {
    let skip_path = original_path.with_extension("av1skip");
    // Write empty file or a timestamp
    fs::write(&skip_path, format!("Created: {}", chrono::Utc::now()))?;
    Ok(())
}

/// Atomically replace the original file with the transcoded file
///
/// This performs a safe replacement:
/// 1. Rename original to .bak-<UUID>
/// 2. Rename transcoded to original name
/// 3. Delete the backup
///
/// If anything fails, attempts to restore the original.
/// Uses UUID for backup name to prevent collisions.
///
/// # Arguments
/// * `original_path` - Path to the original file
/// * `transcoded_path` - Path to the transcoded file (temporary name)
///
/// # Returns
/// Ok(()) if replacement succeeded
pub fn replace_file_atomic(original_path: &Path, transcoded_path: &Path) -> Result<()> {
    use uuid::Uuid;
    
    // Create unique backup path using UUID to prevent collisions
    let backup_name = format!(
        "{}.bak-{}",
        original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file"),
        Uuid::new_v4()
    );
    let backup_path = original_path.with_file_name(backup_name);
    
    // Step 1: Rename original to backup
    fs::rename(original_path, &backup_path).map_err(|e| {
        CoreError::IoError(e)
    })?;
    
    // Step 2: Rename transcoded to original name
    match fs::rename(transcoded_path, original_path) {
        Ok(_) => {
            // Success! Now delete the backup
            if let Err(e) = fs::remove_file(&backup_path) {
                eprintln!(
                    "Warning: Failed to delete backup at {:?}: {}",
                    backup_path, e
                );
            }
            Ok(())
        }
        Err(e) => {
            // Failed to rename transcoded file, restore original
            eprintln!("Failed to rename transcoded file, attempting restore...");
            if let Err(restore_err) = fs::rename(&backup_path, original_path) {
                eprintln!(
                    "CRITICAL: Failed to restore original from backup at {:?}: {}",
                    backup_path, restore_err
                );
                eprintln!("Your original file is at: {:?}", backup_path);
            }
            Err(CoreError::IoError(e))
        }
    }
}

/// Clean up failed transcode artifacts
///
/// Removes the temporary output file and any partial artifacts.
///
/// # Arguments
/// * `temp_output_path` - Path to the temporary output file
pub fn cleanup_failed_transcode(temp_output_path: &Path) -> Result<()> {
    if temp_output_path.exists() {
        fs::remove_file(temp_output_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_size_gate_pass() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("original.mkv");
        let transcoded = temp_dir.path().join("transcoded.mkv");
        
        // Create files with different sizes
        fs::write(&original, vec![0u8; 1000]).unwrap();
        fs::write(&transcoded, vec![0u8; 800]).unwrap(); // 80% of original
        
        let result = check_size_gate(&original, &transcoded, 0.9).unwrap();
        
        match result {
            SizeGateResult::Passed { original_bytes, new_bytes, savings_ratio } => {
                assert_eq!(original_bytes, 1000);
                assert_eq!(new_bytes, 800);
                assert!((savings_ratio - 0.2).abs() < 0.01); // 20% savings
            }
            _ => panic!("Expected passed"),
        }
    }
    
    #[test]
    fn test_size_gate_fail() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("original.mkv");
        let transcoded = temp_dir.path().join("transcoded.mkv");
        
        // Transcoded file is larger (shouldn't happen, but test the gate)
        fs::write(&original, vec![0u8; 1000]).unwrap();
        fs::write(&transcoded, vec![0u8; 950]).unwrap(); // 95% of original
        
        let result = check_size_gate(&original, &transcoded, 0.9).unwrap();
        
        match result {
            SizeGateResult::Failed { ratio, threshold, .. } => {
                assert!((ratio - 0.95).abs() < 0.01);
                assert_eq!(threshold, 0.9);
            }
            _ => panic!("Expected failed"),
        }
    }
    
    #[test]
    fn test_write_why_file() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("test.mkv");
        fs::write(&original, b"test").unwrap();
        
        write_why_file(&original, "Size gate failed").unwrap();
        
        let why_path = temp_dir.path().join("test.why.txt");
        assert!(why_path.exists());
        
        let content = fs::read_to_string(&why_path).unwrap();
        assert_eq!(content, "Size gate failed");
    }
    
    #[test]
    fn test_write_skip_marker() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("test.mkv");
        fs::write(&original, b"test").unwrap();
        
        write_skip_marker(&original).unwrap();
        
        let skip_path = temp_dir.path().join("test.av1skip");
        assert!(skip_path.exists());
    }
    
    #[test]
    fn test_atomic_replacement() {
        let temp_dir = TempDir::new().unwrap();
        let original = temp_dir.path().join("original.mkv");
        let transcoded = temp_dir.path().join("transcoded.mkv");
        
        fs::write(&original, b"original content").unwrap();
        fs::write(&transcoded, b"transcoded content").unwrap();
        
        replace_file_atomic(&original, &transcoded).unwrap();
        
        // Original should now have transcoded content
        let content = fs::read_to_string(&original).unwrap();
        assert_eq!(content, "transcoded content");
        
        // Transcoded temp file should be gone
        assert!(!transcoded.exists());
    }
}

