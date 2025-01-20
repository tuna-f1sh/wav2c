pub mod gen_wav;

use clap::{Parser, ValueEnum};
use gen_wav::generate_wav;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
enum SampleFormat {
    #[default]
    Int,
    Float,
}

impl From<SampleFormat> for hound::SampleFormat {
    fn from(sample_format: SampleFormat) -> Self {
        match sample_format {
            SampleFormat::Int => hound::SampleFormat::Int,
            SampleFormat::Float => hound::SampleFormat::Float,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the output WAV file
    output: PathBuf,

    /// Sample rate in Hz
    #[arg(short, long, default_value = "44100")]
    sample_rate: u32,

    /// Number of channels
    #[arg(short, long, default_value = "1")]
    channels: u16,

    /// Bits per sample
    #[arg(short, long, default_value = "16")]
    bits_per_sample: u16,

    /// Pitch in Hz
    #[arg(short, long, default_value = "440")]
    pitch: u32,

    /// Duration in seconds
    #[arg(short, long, default_value = "1.0")]
    duration: f32,

    /// Sample format
    #[arg(short = 'F', long, value_enum, default_value_t = SampleFormat::Int)]
    sample_format: SampleFormat,
}

fn main() {
    let args = Args::parse();

    let spec = hound::WavSpec {
        channels: args.channels,
        sample_rate: args.sample_rate,
        bits_per_sample: args.bits_per_sample,
        sample_format: args.sample_format.into(),
    };

    let duration = std::time::Duration::from_secs_f32(args.duration);

    generate_wav(args.output, spec, args.pitch as f32, duration);
}
