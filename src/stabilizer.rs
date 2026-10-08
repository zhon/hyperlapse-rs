use crate::config::{ProcConfig, StabilizationParams};
use crate::errors::{HyperlapseError, Result};

/// Stabilize a video using vidstab
pub fn stabilize(input: &str, output: &str, trf: &str, params: &StabilizationParams) -> Result<()> {
    // Build transform command
    let mut cmd = vec![
        "ffmpeg".to_string(),
        "-y".to_string(),
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-i".to_string(),
        input.to_string(),
    ];

    // vidstab transform filter
    let vf = format!(
        "vidstabtransform=input={}:smoothing={}:optzoom={}:zoom={}",
        trf, params.smoothing, params.optzoom, params.zoom
    );
    cmd.push(format!("-vf{}", vf));

    cmd.extend(vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-crf".to_string(),
        "18".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-r".to_string(),
        "24".to_string(),
        output.to_string(),
    ]);

    // Execute the command
    let output_result = std::process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map_err(|e| HyperlapseError::IoError(e))?;

    if !output_result.status.success() {
        return Err(HyperlapseError::StabilizationTransformError(
            String::from_utf8_lossy(&output_result.stderr).to_string(),
        ));
    }

    Ok(())
}

/// Verify the output video matches expected frame count
pub fn verify_output(output_path: &str, expected_frames: usize) -> Result<()> {
    use std::process::Command;

    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-count_packets",
            "-show_entries",
            "stream=nb_read_packets",
            "-of",
            "csv=p=0",
            output_path,
        ])
        .output()
        .map_err(|e| HyperlapseError::IoError(e))?;

    if !output.status.success() {
        return Err(HyperlapseError::VerificationFailed {
            expected: expected_frames,
            actual: 0,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = stdout
        .trim()
        .trim_end_matches(',')
        .parse::<usize>()
        .map_err(|_| HyperlapseError::VerificationFailed {
            expected: expected_frames,
            actual: 0,
        })?;

    if actual != expected_frames {
        return Err(HyperlapseError::VerificationFailed {
            expected: expected_frames,
            actual,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_output_success() {
        // This test requires a valid video file
        // Skipping in unit test context
    }
}
