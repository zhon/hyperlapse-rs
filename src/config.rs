use crate::cli::StabilizationPreset;
use crate::errors::Result;
use std::path::PathBuf;

/// Processed configuration ready for workflow
#[derive(Debug, Clone)]
pub struct ProcConfig {
    /// Input directory containing DNG files
    pub input_dir: PathBuf,
    
    /// JPEG development directory
    pub jpeg_dir: PathBuf,
    
    /// Output video file path
    pub output_file: PathBuf,
    
    /// Transform file for stabilization
    pub trf_file: PathBuf,
    
    /// Output frame rate
    pub framerate: u32,
    
    /// Output resolution
    pub width: u32,
    pub height: u32,
    
    /// Encoding CRF
    pub crf: u8,
    
    /// Stabilization preset
    pub preset: StabilizationPreset,
    
    /// Number of frames to process
    pub frame_count: usize,
    
    /// Starting frame number
    pub start_number: u32,
    
    /// Whether to skip RAW development
    pub skip_development: bool,
    
    /// Whether to verify output
    pub verify: bool,
    
    /// Whether to dry run
    pub dry_run: bool,
    
    /// Verbose mode
    pub verbose: bool,
}

impl ProcConfig {
    /// Create a processed configuration from base config
    pub fn new(config: &crate::cli::Config) -> Result<Self> {
        let input_dir = &config.input;
        
        if !input_dir.is_dir() {
            return Err(crate::errors::HyperlapseError::InputNotFound(
                input_dir.display().to_string(),
            ));
        }

        let jpeg_dir = config.jpeg_dir();
        let output_file = config.output.clone();
        let trf_file = config.trf_path();

        Ok(Self {
            input_dir: input_dir.clone(),
            jpeg_dir,
            output_file,
            trf_file,
            framerate: config.framerate,
            width: config.width,
            height: config.height,
            crf: config.crf,
            preset: config.preset,
            frame_count: 0,
            start_number: config.start_number,
            skip_development: config.skip_development,
            verify: !config.no_verify,
            dry_run: config.dry_run,
            verbose: config.verbose,
        })
    }

    /// Get stabilization parameters based on preset
    pub fn stabilization_params(&self) -> StabilizationParams {
        match self.preset {
            StabilizationPreset::Gentle => StabilizationParams {
                shakiness: 1,
                accuracy: 15,
                smoothing: 10,
                optzoom: 1,
                zoom: 0.98,
            },
            StabilizationPreset::Aggressive => StabilizationParams {
                shakiness: 8,
                accuracy: 15,
                smoothing: 45,
                optzoom: 1,
                zoom: 0.98,
            },
        }
    }

    /// Get the base name pattern (NAME prefix, no trailing underscore)
    /// For "HYPERLAPSE_0001.DNG" returns "HYPERLAPSE"
    pub fn basename_pattern(&self) -> String {
        if let Ok(mut entries) = std::fs::read_dir(&self.input_dir) {
            if let Some(Ok(first)) = entries.next() {
                let name = first.file_name().to_string_lossy().to_string();
                // Find the underscore and extract prefix before it
                if let Some(underscore_pos) = name.find('_') {
                    // Find the extension
                    if let Some(dot_pos) = name.rfind('.') {
                        let stem = &name[..dot_pos];
                        // Return the part before underscore
                        return stem[..underscore_pos].to_string();
                    }
                }
            }
        }
        "HYPERLAPSE".to_string()
    }

    /// Get the file extension
    pub fn extension_string(&self) -> &str {
        "DNG"
    }
}

/// Stabilization parameters for vidstab
#[derive(Debug, Clone)]
pub struct StabilizationParams {
    pub shakiness: u32,
    pub accuracy: u32,
    pub smoothing: u32,
    pub optzoom: u32,
    pub zoom: f64,
}