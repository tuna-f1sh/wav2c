use hound::{SampleFormat, WavSpec, WavWriter};
use std::f32::consts::PI;
use std::path::Path;

/// Generates WAV files for testing with sine wave at 440 Hz (the A4 pitch)
fn generate_wav<P: AsRef<Path>>(
    path: P,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    duration_secs: u32,
    sample_format: SampleFormat,
) {
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        sample_format,
    };

    let mut writer = WavWriter::create(path, spec).unwrap();
    let amplitude = match bits_per_sample {
        0..=8 => 127.0,
        9..=16 => 32_767.0,
        17..=32 => 2_147_483_647.0,
        _ => panic!("Unsupported bits per sample"),
    };

    let sample_count = sample_rate * duration_secs;
    for t in 0..sample_count {
        let value = (amplitude * (2.0 * PI * 440.0 * t as f32 / sample_rate as f32).sin()) as i32;
        for _ in 0..channels {
            match bits_per_sample {
                0..=8 => writer.write_sample(value as i8).unwrap(),
                9..=16 => writer.write_sample(value as i16).unwrap(),
                17..=32 => writer.write_sample(value).unwrap(),
                _ => unreachable!(),
            }
        }
    }
    writer.finalize().unwrap();
}

fn main() {
    let output_dir = "tests/fixtures/";
    std::fs::create_dir_all(output_dir).unwrap();

    generate_wav(
        format!("{}/mono_8bit.wav", output_dir),
        44100,
        1,
        8,
        2,
        SampleFormat::Int,
    );
    generate_wav(
        format!("{}/stereo_16bit.wav", output_dir),
        44100,
        2,
        16,
        2,
        SampleFormat::Int,
    );
    generate_wav(
        format!("{}/mono_32bit.wav", output_dir),
        22050,
        1,
        32,
        2,
        SampleFormat::Int,
    );
    generate_wav(
        format!("{}/stereo_8bit_low.wav", output_dir),
        11025,
        2,
        8,
        2,
        SampleFormat::Int,
    );

    generate_wav(
        format!("{}/mono_8bit_float.wav", output_dir),
        44100,
        1,
        32,
        1,
        SampleFormat::Float,
    );

    println!("Test WAV files generated in '{}'.", output_dir);
}
