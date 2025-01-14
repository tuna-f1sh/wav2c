Quick Rust CLI tool to convert LPCM WAV files to C arrays.

Only PCM data is dumped to array; no header information is included. Intended use is for I2S audio playback on embedded systems.

# Usage

See the help message for usage information:

```bash
cargo run --release -- --help
```

The source file generated is designed to be included in target sources then definitions declared as externs in the target code.

For example, an audio file `audio.wav` (16-bit) with `wav2c -o audio.c audio.wav` will generate `audio.c` with the following definitions (use `--array-name` to change the array name). These can be included in the target source as `extern`:

```c
extern const size_t AUDIO_SAMPLE_NO;
extern const int16_t audio[];
```

Perhaps it might be useful to create a header file in the future but for now, the definitions are included in the source file.

# Input File and Array Size

The input file must be integer LPCM WAV format. Bit rates up to 32-bit are supported and any sample rate. Bare in mind that the array size will be very large for high bit rates/sample rates. The `--max-samples` option is used to sanity check the array size that will be generated.

Use an audio program to convert audio files to LPCM WAV format and/or downsample for size constraints. For example, using `ffmpeg`:

```bash
# Convert M4A to mono 16-bit 22.05 kHz WAV
ffmpeg -i input.m4a -ar 22050 -ac 1 -sample_fmt s16 output.wav
# Convert M4A to mono 8-bit 8.82 kHz WAV
ffmpeg -i input.m4a -ar 8820 -ac 1 -acodec pcm_u8 output.wav
```
