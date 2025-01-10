//! Convert a .wav file to a C array for use in embedded systems.
use clap::{Parser, ValueEnum};
use log::{info, warn, LevelFilter};
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

/// Maximum number of samples to process to prevent massive arrays
const MAX_SAMPLES: usize = 220_000;
/// Samples per line in the output C array for formatting
const SAMPLES_PER_LINE: usize = 8;

/// Error type for the application
enum WavToCError {
    /// std:io error
    IoError(std::io::Error),
    /// Hound wav decoding error
    HoundError(hound::Error),
    /// Incompatible input file
    InvalidInput(String),
    /// Output file already exists
    OutputExists(PathBuf),
}

/// Format for the output array values
#[derive(Debug, Default, Clone, ValueEnum)]
enum ArrayFormat {
    #[default]
    /// Signed 16-bit integers in base 10
    Base10,
    /// Signed 16-bit integers in hexadecimal
    Base16,
}

impl fmt::Display for WavToCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WavToCError::IoError(e) => write!(f, "IO error: {}", e),
            WavToCError::HoundError(e) => write!(f, "Hound error: {}", e),
            WavToCError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
            WavToCError::OutputExists(p) => {
                write!(f, "Output file already exists: {}", p.display())
            }
        }
    }
}

impl fmt::Debug for WavToCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for WavToCError {}

impl From<std::io::Error> for WavToCError {
    fn from(err: std::io::Error) -> Self {
        WavToCError::IoError(err)
    }
}

impl From<hound::Error> for WavToCError {
    fn from(err: hound::Error) -> Self {
        WavToCError::HoundError(err)
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the input .wav file
    input: PathBuf,

    /// Name of the array (optional, defaults to the input file name without extension)
    #[arg(short, long)]
    array_name: Option<String>,

    /// Path to the output file (optional, defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Number format for the output array
    #[arg(short, long, value_enum, default_value_t = ArrayFormat::Base10)]
    format: ArrayFormat,

    /// Max samples to sanity check the array size
    ///
    /// 220,000 samples of 16 bit 44.1kHz audio is about 5 seconds/440 kB. For embedded systems, you may want to adjust sample rate of input file to fit memory constraints before increasing this value.
    #[arg(short, long, default_value_t = MAX_SAMPLES)]
    max_samples: usize,

    /// Do not include a comment with the file information
    #[arg(short, long)]
    no_comment: bool,

    /// File to read and write to the output file before the array
    #[arg(short = 'H', long, conflicts_with = "prefix")]
    prefix_file: Option<PathBuf>,

    /// String to prepend to the output file before the array
    #[arg(short, long, conflicts_with = "prefix_file")]
    prefix: Option<String>,

    /// Enable verbose output (can be repeated for more verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn setup_logging(verbose: u8) {
    match verbose {
        0 => env_logger::Builder::new().parse_default_env().init(),
        1 => env_logger::Builder::new()
            .filter_level(LevelFilter::Info)
            .init(),
        2 => env_logger::Builder::new()
            .filter_level(LevelFilter::Debug)
            .init(),
        _ => env_logger::Builder::new()
            .filter_level(LevelFilter::Trace)
            .init(),
    };
}

#[derive(Debug, Default)]
struct WavToCOptions<'a> {
    max_samples: Option<usize>,
    no_comment: bool,
    format: ArrayFormat,
    prefix: Option<&'a str>,
}

fn wav_to_c_array(
    wav_path: &Path,
    array_name: &str,
    output_path: Option<&Path>,
    options: WavToCOptions,
) -> Result<(), WavToCError> {
    if !wav_path.exists() {
        return Err(WavToCError::InvalidInput(
            "Input file does not exist.".to_string(),
        ));
    }

    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();
    let file_spec = format!(
        "Sample rate: {} Hz, Channels: {}, Bits per sample: {}",
        spec.sample_rate, spec.channels, spec.bits_per_sample
    );

    let wave_file = wav_path.file_name().unwrap().to_string_lossy();
    info!("Processing file: {}", wave_file);
    info!("{}", file_spec);

    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample > 16 {
        return Err(WavToCError::InvalidInput(
            "Only <= 16-bit PCM audio is currently supported.".to_string(),
        ));
    }

    let samples: Vec<i16> = match spec.channels {
        1 => reader.samples::<i16>().collect::<Result<Vec<_>, _>>()?,
        2 => {
            warn!("Merging stereo channels into mono.");
            reader
                .samples::<i16>()
                .collect::<Result<Vec<_>, _>>()?
                .chunks(2)
                .map(|pair| {
                    let left = pair[0] as i32;
                    let right = pair[1] as i32;
                    ((left + right) / 2) as i16
                })
                .collect()
        }
        _ => {
            return Err(WavToCError::InvalidInput(
                "Only mono or stereo audio is supported.".to_string(),
            ));
        }
    };

    if let Some(max_samples) = options.max_samples {
        if samples.len() > max_samples {
            return Err(WavToCError::InvalidInput(format!(
                "Too many samples ({}), maximum is {}",
                samples.len(),
                max_samples
            )));
        }
    }

    // strip spaces from the array name
    let safe_array_name = array_name.replace(" ", "_");
    let mut c_code = if !options.no_comment {
        format!(
            "/*\n\
            /* Generated by {} v{} from {}\n\
            /* {}\n\
            /*\n\
            /* {}\n\
            */\n\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            wave_file,
            file_spec,
            env!("CARGO_PKG_HOMEPAGE")
        )
    } else {
        String::new()
    };

    if let Some(prefix) = options.prefix {
        c_code.push_str(prefix);
        c_code.push_str("\n\n");
    }

    c_code.push_str(&format!(
        "#define {}_SAMPLE_NO {}\n\n\
        const int16_t {}[] = {{",
        safe_array_name.to_uppercase(),
        samples.len(),
        safe_array_name
    ));

    for (i, sample) in samples.iter().enumerate() {
        if i % SAMPLES_PER_LINE == 0 {
            c_code.push_str("\n\t");
        }
        match options.format {
            ArrayFormat::Base10 => c_code.push_str(&format!(" {},", sample)),
            ArrayFormat::Base16 => c_code.push_str(&format!(" {:#x},", sample)),
        }
    }

    c_code.push_str("\n};");

    if let Some(output_path) = output_path {
        std::fs::write(output_path, c_code)?;
        info!("Output written to: {}", output_path.display());
    } else {
        println!("{}", c_code);
    }

    Ok(())
}

fn main() -> Result<(), WavToCError> {
    let args = Args::parse();

    setup_logging(args.verbose);

    if let Some(output_path) = &args.output {
        if output_path.exists() {
            return Err(WavToCError::OutputExists(output_path.to_path_buf()));
        }
    }

    // use the input file name as the array name if not provided
    // converted to lowercase ascii
    let array_name = args.array_name.unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .to_ascii_lowercase()
    });

    let prefix = if let Some(prefix_file) = &args.prefix_file {
        Some(std::fs::read_to_string(prefix_file)?)
    } else {
        args.prefix
    };

    let options = WavToCOptions {
        max_samples: Some(args.max_samples),
        no_comment: args.no_comment,
        format: args.format,
        prefix: prefix.as_deref(),
    };

    wav_to_c_array(&args.input, &array_name, args.output.as_deref(), options)?;
    Ok(())
}
