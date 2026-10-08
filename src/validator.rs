use crate::errors::{HyperlapseError, Result};
use crate::config::{ProcConfig, StabilizationParams};
use std::process::Command;

/// Check if rawtherapee-cli is available
pub fn ensure_rawtherapee() -> Result<()> {
    let output = Command::new("rawtherapee-cli")
        .arg("--version")
        .output();
    
    if output.is_err() {
        return Err(HyperlapseError::RawTherapeeNotFound);
    }
    Ok(())
}

/// Check if ffmpeg is available
pub fn ensure_ffmpeg() -> Result<()> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output();
    
    if output.is_err() {
        return Err(HyperlapseError::FFmpegNotFound);
    }
    Ok(())
}

/// Check if vidstab filter is available in ffmpeg
pub fn check_vidstab_available() -> Result<()> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-filters"])
        .output();
    
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stdout.contains("vidstabdetect") && stdout.contains("vidstabtransform") {
                Ok(())
            } else {
                Err(HyperlapseError::VidstabNotAvailable)
            }
        }
        Err(_) => Err(HyperlapseError::FFmpegNotFound),
    }
}

/// Count frame files in a directory
pub fn count_frames(input_dir: &std::path::Path, extension: &str) -> usize {
    if let Ok(entries) = std::fs::read_dir(input_dir) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_lowercase() == extension)
                    .unwrap_or(false)
            })
            .count()
    } else {
        0
    }
}

/// Get video duration in seconds using ffprobe
pub fn get_video_duration(path: &std::path::Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "json", ":-"])
        .stdin(std::process::Stdio::piped())
        .output()?;
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| HyperlapseError::ConfigError(format!("JSON parse error: {}", e)))?;
    let duration = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    Ok(duration)
}

/// Verify a directory has sequential files
pub fn verify_sequential_files(
    input_dir: &std::path::Path,
    pattern: &str,
    count: usize,
) -> Result<()> {
    for i in 0..count {
        let filename = format!("{}_{:04}.jpg", pattern, i + 1);
        let path = input_dir.join(&filename);
        
        if !path.exists() {
            return Err(HyperlapseError::ConfigError(format!(
                "Expected file missing: {}",
                filename
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_frames_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        assert_eq!(count_frames(temp_dir.path(), "jpg"), 0);
    }

    #[test]
    fn test_count_frames_with_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        
        // Create test files
        for i in 1..=5 {
            std::fs::File::create(temp_dir.path().join(format!("test_{:04}.jpg", i)))
                .unwrap();
        }
        
        assert_eq!(count_frames(temp_dir.path(), "jpg"), 5);
    }
}