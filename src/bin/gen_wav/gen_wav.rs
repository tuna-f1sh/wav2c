use hound::{WavSpec, WavWriter};
use std::f32::consts::PI;
use std::path::Path;
use std::time::Duration;

/// Generates simple sine wave WAV files for testing
pub(crate) fn generate_wav<P: AsRef<Path>>(
    path: P,
    spec: WavSpec,
    pitch: f32,
    duration_secs: Duration,
) {
    let mut writer = WavWriter::create(path, spec).unwrap();
    let amplitude = match spec.bits_per_sample {
        0..=8 => 127.0,
        9..=16 => 32_767.0,
        17..=32 => 2_147_483_647.0,
        _ => panic!("Unsupported bits per sample"),
    };

    let sample_count = (spec.sample_rate as u128 * duration_secs.as_micros() / 1_000_000) as u32;
    for t in 0..sample_count {
        let value =
            (amplitude * (2.0 * PI * pitch * t as f32 / spec.sample_rate as f32).sin()) as i32;
        for _ in 0..spec.channels {
            match spec.bits_per_sample {
                0..=8 => writer.write_sample(value as i8).unwrap(),
                9..=16 => writer.write_sample(value as i16).unwrap(),
                17..=32 => writer.write_sample(value).unwrap(),
                _ => unreachable!(),
            }
        }
    }

    writer.finalize().unwrap();
}
