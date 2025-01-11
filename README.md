Quick Rust CLI tool to convert LPCM WAV files to C arrays.

Only PCM data is dumped to array; no header information is included. Intended use is for I2S audio playback on embedded systems.

# Usage

See the help message for usage information:

```bash
cargo run --release -- --help
```

The source file generated is designed to be included in target sources then definitions declared as externs in the target code.

For example, an audio file `audio.wav` with `wav2c -o audio.c audio.wav` will generate `audio.c` with the following definitions (use `--array-name` to change the array name):

```c
extern const size_t AUDIO_SAMPLE_NO;
extern const uint8_t audio[];
```
