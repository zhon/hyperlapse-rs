// Hyperlapse-RS: Convert drone DNG files into stabilized hyperlapse videos
//
// This tool orchestrates:
// 1. RAW development via rawtherapee-cli (applies .pp3 profiles)
// 2. Hyperlapse assembly via ffmpeg (one frame per file)
// 3. Stabilization via ffmpeg + vidstab (two-pass)

use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod errors;
mod image_processor;
mod stabilizer;
mod validator;
mod workflow;

pub use cli::{Args, Config, StabilizationPreset};

/// Main entry point for the hyperlapse CLI
pub fn run(args: Args) -> Result<()> {
    let config = Config::from_args(args);
    workflow::process(&config).map_err(|e| e.into())
}

/// Check if a command exists in PATH
pub fn command_exists(name: &str) -> bool {
    std::process::Command::new(name).arg("--version").output().is_ok()
}