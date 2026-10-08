use crate::cli::{Config, StabilizationPreset};
use crate::config::ProcConfig;
use crate::errors::{HyperlapseError, Result};
use crate::image_processor as img_proc;
use crate::stabilizer;
use crate::validator;

pub fn process(config: &Config) -> Result<()> {
    println!("Hyperlapse-RS v0.1.0");
    println!("========================");
    
    // Create processed config
    let mut proc_config = ProcConfig::new(config)?;

    // Count DNG files (do this early for dry-run preview)
    let dng_count = img_proc::count_dng_files(&proc_config.input_dir);
    proc_config.frame_count = dng_count;
    
    if config.dry_run {
        println!("DRY RUN - showing commands that would be executed:");
        if dng_count == 0 {
            println!("Warning: No DNG files found");
        } else {
            println!("Found {} DNG files", dng_count);
        }
        show_dry_run(&proc_config);
        return Ok(());
    }
    
    // Check dependencies
    println!("Checking dependencies...");
    validator::ensure_ffmpeg()?;
    
    if dng_count == 0 {
        return Err(HyperlapseError::NoDngFiles);
    }
    
    println!("Found {} DNG files", dng_count);
    
    // Check for existing jpegs
    if proc_config.skip_development || img_proc::has_developed_jpegs(&proc_config.jpeg_dir) {
        println!("Using existing JPEG development");
    } else {
        // Need to develop Raws
        println!("Developing RAW files...");
        needs_raw_development(&proc_config)?;
    }
    
    // Validate sequence
    println!("Validating file sequence...");
    let _files = img_proc::validate_sequence(
        &proc_config.input_dir,
        proc_config.start_number,
        proc_config.frame_count,
    )?;
    
    // Build hyperlapse
    println!("Building hyperlapse video...");
    build_hyperlapse(&proc_config)?;
    
    // Stabilization (two-pass vidstab)
    if proc_config.frame_count > 2 {
        println!("Applying stabilization (preset: {:?})...", proc_config.preset);
        stabilize_hyperlapse(&proc_config)?;
    } else {
        println!("Too few frames for stabilization, skipping");
    }
    
    // Verify output
    if proc_config.verify {
        println!("Verifying output...");
        let output_for_verify = &proc_config.output_file;
        stabilizer::verify_output(
            &output_for_verify.to_string_lossy(),
            proc_config.frame_count,
        )?;
        println!("Verification passed: {} frames", proc_config.frame_count);
    }
    
    println!("Done!");
    println!("Output: {}", proc_config.output_file.display());
    
    Ok(())
}

fn needs_raw_development(config: &ProcConfig) -> Result<()> {
    validator::ensure_rawtherapee()?;
    
    // Find matching profile
    let profile = img_proc::find_matching_profile(&config.input_dir);
    
    println!(
        "  Profile: {}",
        profile.as_deref().unwrap_or("none (neutral development)")
    );
    
    // Build rawtherapee command
    let cmd = img_proc::build_rawtherapee_cmd(
        &config.input_dir,
        &config.jpeg_dir,
        profile.as_deref(),
        config.dry_run,
    );
    
    if config.verbose {
        println!("  Command: {}", cmd.join(" "));
    }
    
    if !config.dry_run {
        let output = std::process::Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .map_err(|e| HyperlapseError::IoError(e))?;
        
        if !output.status.success() {
            return Err(HyperlapseError::RawDevelopmentError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        
        println!("  Developed {} frames", config.frame_count);
    }
    
    Ok(())
}

fn build_hyperlapse(config: &ProcConfig) -> Result<()> {
    let cmd = img_proc::build_hyperlapse_cmd(config);
    
    if config.verbose {
        println!("  Command: {}", cmd.join(" "));
    }
    
    let output = std::process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .output()
        .map_err(|e| HyperlapseError::IoError(e))?;
    
    if !output.status.success() {
        return Err(HyperlapseError::HyperlapseBuildError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    
    println!("  Built hyperlapse");
    Ok(())
}

fn stabilize_hyperlapse(config: &ProcConfig) -> Result<()> {
    let params = config.stabilization_params();
    
    // Check vidstab availability
    validator::check_vidstab_available()?;
    
    // Pass 1: detect motion
    let trf_path = &config.trf_file;
    
    let detect_cmd = vec![
        "ffmpeg".to_string(),
        "-y".to_string(),
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-framerate".to_string(),
        config.framerate.to_string(),
        "-start_number".to_string(),
        config.start_number.to_string(),
        "-i".to_string(),
        format!(
            "{}/{}",
            config.jpeg_dir.display(),
            format!("{}_%04d.jpg", config.basename_pattern())
        ),
        "-vf".to_string(),
        format!(
            "vidstabdetect=shakiness={}:accuracy={}:result={}",
            params.shakiness, params.accuracy, trf_path.display()
        ),
        "-f".to_string(),
        "null".to_string(),
        "-".to_string(),
    ];
    
    if config.verbose {
        println!("  Detect command: {}", detect_cmd.join(" "));
    }
    
    let output = std::process::Command::new(&detect_cmd[0])
        .args(&detect_cmd[1..])
        .output()
        .map_err(|e| HyperlapseError::IoError(e))?;
    
    if !output.status.success() {
        return Err(HyperlapseError::StabilizationDetectError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    
    // Pass 2: transform
    let transform_cmd = vec![
        "ffmpeg".to_string(),
        "-y".to_string(),
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-framerate".to_string(),
        config.framerate.to_string(),
        "-start_number".to_string(),
        config.start_number.to_string(),
        "-i".to_string(),
        format!(
            "{}/{}",
            config.jpeg_dir.display(),
            format!("{}_%04d.jpg", config.basename_pattern())
        ),
        "-vf".to_string(),
        format!(
            "vidstabtransform=input={}:smoothing={}:optzoom={}:zoom={}",
            trf_path.display(), params.smoothing, params.optzoom, params.zoom
        ),
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
    ];
    
    if config.verbose {
        println!("  Transform command: {}", transform_cmd.join(" "));
    }
    
    let output = std::process::Command::new(&transform_cmd[0])
        .args(&transform_cmd[1..])
        .output()
        .map_err(|e| HyperlapseError::IoError(e))?;
    
    if !output.status.success() {
        return Err(HyperlapseError::StabilizationTransformError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    
    println!("  Stabilization complete");
    Ok(())
}

fn show_dry_run(config: &ProcConfig) {
    println!("Input: {:?}", config.input_dir);
    println!("Output: {:?}", config.output_file);
    println!("Frames: {}", config.frame_count);
    println!("Resolution: {}x{}", config.width, config.height);
    println!("FPS: {}", config.framerate);
    println!("CRF: {}", config.crf);
    
    if !config.skip_development {
        let profile = img_proc::find_matching_profile(&config.input_dir);
        let profile_str = profile.as_deref().unwrap_or("none");
        if profile.is_some() {
            println!("Would develop with profile: {}", profile_str);
        } else {
            println!("Would develop with neutral profile");
        }
        println!(
            "  Command: rawtherapee-cli -j -Y -f -p {} -o {} -c {}",
            profile_str,
            config.jpeg_dir.display(),
            config.input_dir.display()
        );
    }

    println!(
        "Would assemble: ffmpeg -framerate {} -i {}/{}_{}.{}",
        config.framerate,
        config.input_dir.display(),
        config.basename_pattern(),
        "%04d",
        config.extension_string().to_lowercase()
    );
    
    if config.frame_count > 2 {
        println!(
            "Would stabilize with vidstab: shakiness={}, smoothing={}",
            config.stabilization_params().shakiness,
            config.stabilization_params().smoothing
        );
    }
}