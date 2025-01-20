use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;

const FIXTURES_DIR: &str = "tests/fixtures/";
const GOLDEN_DIR: &str = "tests/golden/";

static INIT: Once = Once::new();

/// Allows fixtures to be excluded from package if desired
/// and generated on demand
fn init() {
    INIT.call_once(|| {
        generate_wavs();
    });
}

fn generate_wavs() {
    Command::new("make").assert().success();
}

/// Run the wav2c binary with the given input and compare the output with the golden file
fn wav_to_c_case(input_path: &Path, golden_path: &Path, output_path: Option<&Path>, args: &[&str]) {
    init();

    let generated_output = if let Some(output_path) = output_path {
        Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .arg(input_path)
            .arg("--output")
            .arg(output_path)
            .args(args)
            .assert()
            .success();
        fs::read_to_string(output_path).unwrap()
    } else {
        let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .arg(input_path)
            .args(args)
            .assert()
            .success();
        String::from_utf8(cmd.get_output().stdout.clone()).unwrap()
    };

    let golden_output = fs::read_to_string(golden_path).unwrap();

    pretty_assertions::assert_eq!(
        generated_output.trim(),
        golden_output.trim(),
        "Mismatch for {}",
        golden_path.file_name().unwrap().to_string_lossy()
    );

    if args.contains(&"--header") {
        let header_path = output_path.unwrap().with_extension("h");
        let generated_header = fs::read_to_string(&header_path).unwrap();
        let golden_header = fs::read_to_string(golden_path.with_extension("h")).unwrap();

        pretty_assertions::assert_eq!(
            generated_header.trim(),
            golden_header.trim(),
            "Mismatch for {} header",
            header_path.file_name().unwrap().to_string_lossy()
        );
    }
}

/// Compile the generated C file with GCC - compile only, no linking (-c) since no entry function
fn compile_with_gcc(file_path: &Path) {
    let temp_file = file_path.with_extension("o");
    Command::new("gcc")
        .arg("-c")
        .arg(file_path)
        .arg("--include")
        .arg("stdint.h")
        .arg("--include")
        .arg("stddef.h")
        .arg("-o")
        .arg(&temp_file)
        .assert()
        .success();
}

#[test]
fn test_wav_to_c_array() {
    let test_cases = vec![
        ("mono_8bit.wav", "mono_8bit.c"),
        ("stereo_16bit.wav", "stereo_16bit.c"),
        ("mono_32bit.wav", "mono_32bit.c"),
        ("stereo_8bit_low.wav", "stereo_8bit_low.c"),
    ];

    for (input, golden) in test_cases {
        let input_path = PathBuf::from(format!("{}/{}", FIXTURES_DIR, input));
        let golden_path = PathBuf::from(format!("{}/{}", GOLDEN_DIR, golden));

        wav_to_c_case(&input_path, &golden_path, None, &["--no-comment"]);
    }
}

#[test]
fn test_wav_to_c_array_file() {
    let test_cases = vec![
        ("mono_8bit.wav", "mono_8bit.c"),
        ("stereo_16bit.wav", "stereo_16bit.c"),
        ("mono_32bit.wav", "mono_32bit.c"),
        ("stereo_8bit_low.wav", "stereo_8bit_low.c"),
    ];

    for (input, golden) in test_cases {
        let input_path = PathBuf::from(format!("tests/fixtures/{}", input));
        let golden_path = PathBuf::from(format!("tests/golden/{}", golden));
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_path = temp_dir.path().join(golden);

        wav_to_c_case(
            &input_path,
            &golden_path,
            Some(&output_path),
            &["--no-comment"],
        );
        compile_with_gcc(&output_path);
    }
}

#[test]
fn test_wav_to_c_array_file_header() {
    let test_cases = vec![
        ("mono_8bit.wav", "mono_8bit.c"),
        ("stereo_16bit.wav", "stereo_16bit.c"),
        ("mono_32bit.wav", "mono_32bit.c"),
        ("stereo_8bit_low.wav", "stereo_8bit_low.c"),
    ];

    for (input, golden) in test_cases {
        let input_path = PathBuf::from(format!("tests/fixtures/{}", input));
        let golden_path = PathBuf::from(format!("tests/golden/{}", golden));
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_path = temp_dir.path().join(golden);

        wav_to_c_case(
            &input_path,
            &golden_path,
            Some(&output_path),
            &["--no-comment", "--header"],
        );
        compile_with_gcc(&output_path);
    }
}

#[test]
fn test_wav_to_c_array_file_base16() {
    let test_cases = vec![("mono_8bit.wav", "mono_8bit_base16.c")];

    for (input, golden) in test_cases {
        let input_path = PathBuf::from(format!("tests/fixtures/{}", input));
        let golden_path = PathBuf::from(format!("tests/golden/{}", golden));
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_path = temp_dir.path().join(golden);

        wav_to_c_case(
            &input_path,
            &golden_path,
            Some(&output_path),
            &["--no-comment", "--format", "base16"],
        );
        compile_with_gcc(&output_path);
    }
}

#[test]
fn test_wav_to_c_array_prefix() {
    let test_cases = vec![("mono_8bit.wav", "mono_8bit_prefix.c")];

    for (input, golden) in test_cases {
        let input_path = PathBuf::from(format!("tests/fixtures/{}", input));
        let golden_path = PathBuf::from(format!("tests/golden/{}", golden));
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output_path = temp_dir.path().join(golden);

        wav_to_c_case(
            &input_path,
            &golden_path,
            Some(&output_path),
            &["--no-comment", "--prefix", "/* john was here */"],
        );
        compile_with_gcc(&output_path);
    }
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
