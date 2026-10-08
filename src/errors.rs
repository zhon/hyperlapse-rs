use thiserror::Error;

/// Errors that can occur during hyperlapse processing
#[derive(Error, Debug)]
pub enum HyperlapseError {
    #[error("Input directory not found: {0}")]
    InputNotFound(String),

    #[error("No DNG files found in input directory")]
    NoDngFiles,

    #[error("Sequential numbering gap detected at frame {frame}")]
    SequenceGap { frame: u32 },

    #[error("No .pp3 profile found and required")]
    NoProfile,

    #[error("rawtherapee-cli not found in PATH")]
    RawTherapeeNotFound,

    #[error("ffmpeg not found in PATH")]
    FFmpegNotFound,

    #[error("vidstab filter not available in ffmpeg build")]
    VidstabNotAvailable,

    #[error("Error developing RAW file: {0}")]
    RawDevelopmentError(String),

    #[error("Error building hyperlapse: {0}")]
    HyperlapseBuildError(String),

    #[error("Stabilization transform detection failed: {0}")]
    StabilizationDetectError(String),

    #[error("Stabilization transform failed: {0}")]
    StabilizationTransformError(String),

    #[error("Output verification failed: expected {expected} frames, got {actual}")]
    VerificationFailed { expected: usize, actual: usize },

    #[error("Dry run mode: command would execute: {0}")]
    DryRun(String),

    #[error("Frame count mismatch: {0}")]
    FrameMismatch(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result type alias for hyperlapse operations
pub type Result<T> = std::result::Result<T, HyperlapseError>;

impl HyperlapseError {
    /// Wrap an IO error with context
    pub fn with_context(self, context: &str) -> Self {
        match self {
            HyperlapseError::IoError(e) => HyperlapseError::IoError(e),
            _ => HyperlapseError::with_context_from_string(self.to_string(), context),
        }
    }

    fn with_context_from_string(original: String, context: &str) -> Self {
        HyperlapseError::ConfigError(format!("{}: {}", context, original))
    }
    }
