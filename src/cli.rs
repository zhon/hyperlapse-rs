use clap::Parser;
use std::path::PathBuf;

/// Convert drone DNG RAW files into stabilized hyperlapse videos
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input directory containing DNG files (must be sequential: NAME_0001.DNG, NAME_0002.DNG, ...)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output MP4 file path (default: <input>/hyperlapse.mp4)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Frame rate for output video (default: 24)
    #[arg(short, long, default_value = "24")]
    pub framerate: u32,

    /// Resolution width (default: 1920)
    #[arg(long, default_value = "1920")]
    pub width: u32,

    /// Resolution height (default: 1080)
    #[arg(long, default_value = "1080")]
    pub height: u32,

    /// CRF for H.264 encoding (lower = higher quality, default: 18)
    #[arg(long, default_value = "18")]
    pub crf: u8,

    /// Stabilization preset: gentle (preserve motion) or aggressive (cancel all shake)
    #[arg(short, long, default_value = "aggressive", value_parser = ["gentle", "aggressive"])]
    pub preset: String,

    /// Use existing jpegs/ directory if present (skip RAW development)
    #[arg(long)]
    pub skip_development: bool,

    /// Dry run: show what would be done without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Start frame number (default: 1)
    #[arg(long, default_value = "1")]
    pub start_number: u32,

    /// Extra description for output filenames
    #[arg(long)]
    pub tag: Option<String>,

    /// Don't verify output against input count
    #[arg(long)]
    pub no_verify: bool,
}

impl Args {
    /// Returns true if gentle stabilization is requested
    pub fn is_gentle(&self) -> bool {
        self.preset.to_lowercase() == "gentle"
    }
}

/// Configuration derived from CLI arguments
#[derive(Debug, Clone)]
pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
    pub framerate: u32,
    pub width: u32,
    pub height: u32,
    pub crf: u8,
    pub preset: StabilizationPreset,
    pub skip_development: bool,
    pub dry_run: bool,
    pub verbose: bool,
    pub start_number: u32,
    pub tag: Option<String>,
    pub no_verify: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabilizationPreset {
    Gentle,
    Aggressive,
}

impl Config {
    pub fn from_args(args: Args) -> Self {
        let output = args.output.clone().unwrap_or_else(|| {
            let mut p = args.input.clone();
            p.push("hyperlapse");
            if let Some(tag) = &args.tag {
                p.set_extension(format!("{}.mp4", tag));
            } else {
                p.set_extension("mp4");
            }
            p
        });

        let preset = if args.is_gentle() {
            StabilizationPreset::Gentle
        } else {
            StabilizationPreset::Aggressive
        };

        Self {
            input: args.input,
            output: output,
            framerate: args.framerate,
            width: args.width,
            height: args.height,
            crf: args.crf,
            preset,
            skip_development: args.skip_development,
            dry_run: args.dry_run,
            verbose: args.verbose,
            start_number: args.start_number,
            tag: args.tag,
            no_verify: args.no_verify,
        }
    }

    /// Returns the intermediate JPEG output directory
    pub fn jpeg_dir(&self) -> PathBuf {
        let mut p = self.input.clone();
        p.push("jpegs");
        p
    }

    /// Returns the stabilization transform file path
    pub fn trf_path(&self) -> PathBuf {
        let mut p = self.input.clone();
        p.push("stabilize.trf");
        p
    }
}