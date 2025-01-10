Quick Rust CLI tool to convert LPCM WAV files to C arrays.

Only PCM data is dumped to array; no header information is included. Intended use is for I2S audio playback on embedded systems.

# Usage

```bash
cargo run --release -- --help
```
