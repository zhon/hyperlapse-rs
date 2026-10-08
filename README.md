# hyperlapse-rs

A command-line tool to convert drone RAW files into stabilized hyperlapse videos.

## Features

- Convert DNG/CR2/NEF/ARW RAW files to stabilized hyperlapse video
- Automatic `.pp3` profile detection for consistent RAW development
- Two-pass vidstab stabilization via ffmpeg
- Configurable resolution, FPS, and quality settings
- `--dry-run` mode to preview commands before execution

## Requirements

- `rawtherapee-cli` - for RAW file development
- `ffmpeg` (with libvidstab) - for video assembly and stabilization

## Installation

```bash
cargo install --path .
```

## Usage

```
hyperlapse-rs -i <input_dir> -o <output_file> [OPTIONS]

Options:
  -i, --input <DIR>       Input directory with RAW files
  -o, --output <FILE>     Output video file path
  -w, --width <NUM>       Output width (default: 1920)
  -h, --height <NUM>      Output height (default: 1080)
  -f, --framerate <NUM>   Frame rate in fps (default: 24)
  -c, --crf <NUM>         CRF quality (18-23, default: 18)
  -p, --preset <PRESET>   Stabilization preset: gentle, aggressive (default: aggressive)
  --dry-run               Show commands without executing
  -v, --verbose           Show detailed output
  --verify                Verify output frame count
  --skip-development      Skip RAW development (use existing JPEGs)
  --start-number <NUM>    Starting frame number (default: 0001)
```

## Workflow

1. **RAW Development**: Converts RAW files to JPEGs using `rawtherapee-cli`
   - If `.pp3` profile exists in input directory, uses it
   - Otherwise, applies neutral default profile

2. **Hyperlapse Assembly**: Stitches JPEGs into video using `ffmpeg`

3. **Stabilization**: Two-pass vidstab processing
   - Pass 1: Motion detection (`vidstabdetect`)
   - Pass 2: Transform (`vidstabtransform`)

## Examples

```bash
# Basic conversion
hyperlapse-rs -i ./DRONE_FLIGHT_001 -o ./hyperlapse.mp4

# With specific output settings
hyperlapse-rs -i ./DRONE_FLIGHT_001 -o ./hyperlapse.mp4 -w 3840 -h 2160 -f 30 -c 20

# Preview commands without executing
hyperlapse-rs -i ./DRONE_FLIGHT_001 -o ./hyperlapse.mp4 --dry-run --verbose

# Gentle stabilization (less aggressive smoothing)
hyperlapse-rs -i ./DRONE_FLIGHT_001 -o ./hyperlapse.mp4 -p gentle

# Use existing developed JPEGs
hyperlapse-rs -i ./DRONE_FLIGHT_001 -o ./hyperlapse.mp4 --skip-development
```

## Testing

```bash
cargo test
```

## License

MIT