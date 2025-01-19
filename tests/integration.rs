use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

fn wav_to_c_case(test_cases: &[(&str, &str)], args: &[&str], output_file: bool) {
    let fixtures_dir = "tests/fixtures/";
    let golden_dir = "tests/golden/";

    for (input, golden_output) in test_cases {
        let input_path = PathBuf::from(format!("{}/{}", fixtures_dir, input));
        let golden_path = PathBuf::from(format!("{}/{}", golden_dir, golden_output));
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_path = temp_dir.path().join(golden_output);

        let cmd = if output_file {
            Command::cargo_bin(env!("CARGO_PKG_NAME"))
                .unwrap()
                .arg(&input_path)
                .arg("--output")
                .arg(&output_path)
                .args(args)
                .assert()
                .success()
        } else {
            Command::cargo_bin(env!("CARGO_PKG_NAME"))
                .unwrap()
                .arg(&input_path)
                .args(args)
                .assert()
                .success()
        };

        let generated_output = if output_file {
            fs::read_to_string(&output_path).unwrap()
        } else {
            String::from_utf8(cmd.get_output().stdout.clone()).unwrap()
        };
        let golden_output = fs::read_to_string(&golden_path).unwrap();

        pretty_assertions::assert_eq!(
            generated_output.trim(),
            golden_output.trim(),
            "Mismatch for {}",
            input
        );

        if args.contains(&"--header") {
            let header_path = output_path.with_extension("h");
            let generated_header = fs::read_to_string(&header_path).unwrap();
            let golden_header = fs::read_to_string(golden_path.with_extension("h")).unwrap();

            pretty_assertions::assert_eq!(
                generated_header.trim(),
                golden_header.trim(),
                "Mismatch for {} header",
                input
            );
        }
    }
}

#[test]
fn test_wav_to_c_array() {
    let test_cases = vec![
        ("mono_8bit.wav", "mono_8bit.c"),
        ("stereo_16bit.wav", "stereo_16bit.c"),
        ("mono_32bit.wav", "mono_32bit.c"),
        ("stereo_8bit_low.wav", "stereo_8bit_low.c"),
    ];

    wav_to_c_case(&test_cases, &["--no-comment"], false);
}

#[test]
fn test_wav_to_c_array_file() {
    let test_cases = vec![
        ("mono_8bit.wav", "mono_8bit.c"),
        ("stereo_16bit.wav", "stereo_16bit.c"),
        ("mono_32bit.wav", "mono_32bit.c"),
        ("stereo_8bit_low.wav", "stereo_8bit_low.c"),
    ];

    wav_to_c_case(&test_cases, &["--no-comment"], true);
}

#[test]
fn test_wav_to_c_array_file_header() {
    let test_cases = vec![
        ("mono_8bit.wav", "mono_8bit.c"),
        ("stereo_16bit.wav", "stereo_16bit.c"),
        ("mono_32bit.wav", "mono_32bit.c"),
        ("stereo_8bit_low.wav", "stereo_8bit_low.c"),
    ];

    wav_to_c_case(&test_cases, &["--no-comment", "--header"], true);
}

#[test]
fn test_wav_to_c_array_prefix() {
    let test_cases = vec![("mono_8bit.wav", "mono_8bit_prefix.c")];

    wav_to_c_case(
        &test_cases,
        &["--no-comment", "--prefix", "/* john was here */"],
        true,
    );
}

#[test]
fn test_max_samples() {
    let input = "mono_8bit.wav";

    let input_path = PathBuf::from(format!("tests/fixtures/{}", input));

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg(&input_path)
        .arg("--max-samples")
        .arg("10")
        .assert()
        .failure();
}

#[test]
fn test_invalid_pcm_float() {
    let input = "mono_8bit_float.wav";

    let input_path = PathBuf::from(format!("tests/fixtures/{}", input));

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg(&input_path)
        .assert()
        .failure();
}

#[test]
fn test_invalid_file() {
    let input_path = PathBuf::from("src/main.rs");

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg(&input_path)
        .assert()
        .failure();
}
