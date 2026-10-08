use crate::errors::{HyperlapseError, Result};
use crate::config::{ProcConfig, StabilizationParams};
use std::path::Path;

/// Find a .pp3 profile that matches the input directory
pub fn find_matching_profile(input_dir: &Path) -> Option<String> {
    // Look for first .pp3 file in directory
    if let Ok(entries) = std::fs::read_dir(input_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("pp3") {
                    return Some(path.display().to_string());
                }
            }
        }
    }
    None
}

/// Check if jpegs directory exists and has content
pub fn has_developed_jpegs(jpeg_dir: &Path) -> bool {
    if jpeg_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(jpeg_dir) {
            return entries.count() > 0;
        }
    }
    false
}

/// Count DNG files in a directory
pub fn count_dng_files(input_dir: &Path) -> usize {
    if let Ok(entries) = std::fs::read_dir(input_dir) {
        entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext.to_ascii_uppercase() == "DNG")
                    .unwrap_or(false)
            })
            .count()
    } else {
        0
    }
}

/// Validate the sequential file numbering
pub fn validate_sequence(
    input_dir: &Path,
    start: u32,
    count: usize,
) -> Result<Vec<String>> {
    let mut files = Vec::new();
    
    for i in 0..count {
        let frame_num = start + i as u32;
        let pattern = format!("{:04}", frame_num);
        
        // Try to find the file
        let found = if let Ok(entries) = std::fs::read_dir(input_dir) {
            entries
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .contains(&pattern)
                        && e.path()
                            .extension()
                            .map(|ext| ext.to_ascii_uppercase() == "DNG")
                            .unwrap_or(false)
                })
        } else {
            false
        };

        if found {
            files.push(format!("HYPERLAPSE_{}.DNG", pattern));
        } else {
            return Err(HyperlapseError::SequenceGap { frame: frame_num });
        }
    }
    
    Ok(files)
}

/// Build rawtherapee-cli command for development
pub fn build_rawtherapee_cmd(
    input_dir: &Path,
    jpeg_dir: &Path,
    profile: Option<&str>,
    dry_run: bool,
) -> Vec<String> {
    let mut cmd = vec![
        "rawtherapee-cli".to_string(),
        "-j".to_string(),           // JPEG output
        "-Y".to_string(),           // Fast pipeline
        "-f".to_string(),           // Force output
    ];

    if let Some(p) = profile {
        cmd.push(format!("-p {}", p));
    }

    // Ensure output directory exists
    if !dry_run {
        let _ = std::fs::create_dir_all(jpeg_dir);
    }

    cmd.push(format!("-o {}", jpeg_dir.display()));
    cmd.push(format!("-c {}", input_dir.display()));

    cmd
}

/// Build ffmpeg command for hyperlapse assembly
pub fn build_hyperlapse_cmd(
    config: &ProcConfig,
) -> Vec<String> {
    let basename = config.basename_pattern();
    let ext = config.extension_string().to_lowercase();
    let pattern = format!("{}_{}.{}", basename, "%04d", ext);
    
    let mut cmd = vec![
        "ffmpeg".to_string(),
        "-y".to_string(),                          // Overwrite
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        if config.verbose { "info" } else { "error" }.to_string(),
        "-framerate".to_string(),
        config.framerate.to_string(),
        "-start_number".to_string(),
        config.start_number.to_string(),
        "-i".to_string(),
        format!("{}/{}", config.input_dir.display(), pattern),
    ];

    // Build filter chain
    let vf = format!(
        "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{},setsar=1,fps={}",
        config.width, config.height, config.width, config.height, config.framerate
    );
    cmd.push(format!("-vf {}", vf));

    cmd.extend(vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-crf".to_string(),
        config.crf.to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-r".to_string(),
        config.framerate.to_string(),
        config.output_file.display().to_string(),
    ]);

    cmd
}

/// Build vidstab detection command
pub fn build_stabilize_detect_cmd(
    config: &ProcConfig,
    params: &StabilizationParams,
) -> Vec<String> {
    let basename = config.basename_pattern();
    let ext = "jpg";
    
    let mut cmd = vec![
        "ffmpeg".to_string(),
        "-y".to_string(),
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-framerate".to_string(),
        config.framerate.to_string(),
        "-start_number".to_string(),
        config.start_number.to_string(),
    ];

    // Input: developed JPEGs
    cmd.push("-i".to_string());
    cmd.push(format!(
        "{}/{}",
        config.jpeg_dir.display(),
        format!("{}_{}.{}", basename, "%04d", ext)
    ));

    // vidstab detection filter
    let vf = format!(
        "vidstabdetect=shakiness={}:accuracy={}:result={}",
        params.shakiness,
        params.accuracy,
        config.trf_file.display()
    );
    cmd.push(format!("-vf {}", vf));

    // Null output (we only want the transform file)
    cmd.extend(vec!["-f".to_string(), "null".to_string(), "-".to_string()]);

    cmd
}

/// Build vidstab transform command
pub fn build_stabilize_transform_cmd(
    config: &ProcConfig,
    params: &StabilizationParams,
) -> Vec<String> {
    let basename = config.basename_pattern();
    
    let mut cmd = vec![
        "ffmpeg".to_string(),
        "-y".to_string(),
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-framerate".to_string(),
        config.framerate.to_string(),
        "-start_number".to_string(),
        config.start_number.to_string(),
    ];

    // Input: developed JPEGs
    cmd.push("-i".to_string());
    cmd.push(format!(
        "{}/{}",
        config.jpeg_dir.display(),
        format!("{}_{}.jpg", basename, "%04d")
    ));

    // vidstab transform filter
    let vf = format!(
        "vidstabtransform=input={}:smoothing={}:optzoom={}:zoom={}",
        config.trf_file.display(),
        params.smoothing,
        params.optzoom,
        params.zoom
    );
    cmd.push(format!("-vf {}", vf));

    // Build scale/crop
    let scalevf = format!(
        "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}",
        config.width, config.height, config.width, config.height
    );

    cmd.extend(vec![
        "-vf".to_string(),
        format!("{},{}", vf, scalevf), // Chain the filters
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-crf".to_string(),
        config.crf.to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-r".to_string(),
        config.framerate.to_string(),
        config.output_file.display().to_string(),
    ]);

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_profile_none() {
        let empty_dir = tempfile::tempdir().unwrap();
        assert!(find_matching_profile(empty_dir.path()).is_none());
    }

    #[test]
    fn test_has_developed_jpegs_false() {
        let empty_dir = tempfile::tempdir().unwrap();
        assert!(!has_developed_jpegs(empty_dir.path()));
    }
}