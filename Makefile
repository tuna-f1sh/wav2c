# Directories
FIXTURE_DIR := tests/fixtures
GOLDEN_DIR := tests/golden
GEN_WAV := cargo run --bin gen_wav --release --
CMD := cargo run --release --locked -- -v -f --no-comment --header --output

# WAV files to generate
WAV_FILES := mono_8bit.wav stereo_16bit.wav mono_32bit.wav stereo_8bit_low.wav mono_8bit_float.wav

# Derived paths
FIXTURE_PATHS := $(addprefix $(FIXTURE_DIR)/, $(WAV_FILES))
# Excluding _float.wav files
GOLDEN_BASE := $(addsuffix .c, $(basename $(subst $(FIXTURE_DIR)/, $(GOLDEN_DIR)/, $(filter-out $(FIXTURE_DIR)/mono_8bit_float.wav, $(FIXTURE_PATHS)))))
GOLDEN_BASE16 := $(addsuffix _base16.c, $(basename $(subst $(FIXTURE_DIR)/, $(GOLDEN_DIR)/, $(filter-out $(FIXTURE_DIR)/mono_8bit_float.wav, $(FIXTURE_PATHS)))))
GOLDEN_PREFIX := $(GOLDEN_DIR)/mono_8bit_prefix.c

# Default target
all: fixtures golden

# Generate WAV files
$(FIXTURE_DIR):
	mkdir -p $@

$(FIXTURE_DIR)/%.wav:
	$(GEN_WAV) $@

$(FIXTURE_DIR)/mono_8bit.wav:
	$(GEN_WAV) -c 1 -b 8 -s 44100 -d 1 $@

$(FIXTURE_DIR)/stereo_16bit.wav:
	$(GEN_WAV) -c 2 -b 16 -s 44100 -d 1 $@

$(FIXTURE_DIR)/mono_32bit.wav:
	$(GEN_WAV) -c 1 -b 32 -s 22050 -d 1 $@

$(FIXTURE_DIR)/stereo_8bit_low.wav:
	$(GEN_WAV) -c 2 -b 8 -s 11025 -d 1 $@

$(FIXTURE_DIR)/mono_8bit_float.wav:
	$(GEN_WAV) -c 1 -b 32 -s 44100 -d 1 -F float $@

fixtures: $(FIXTURE_DIR) $(FIXTURE_PATHS) | Makefile

# Generate C golden files
$(GOLDEN_DIR):
	mkdir -p $@

$(GOLDEN_DIR)/%.c: $(FIXTURE_DIR)/%.wav
	$(CMD) $@ $<

$(GOLDEN_DIR)/%_base16.c: $(FIXTURE_DIR)/%.wav
	$(CMD) $@ $< --format base16

$(GOLDEN_DIR)/mono_8bit_prefix.c: $(FIXTURE_DIR)/mono_8bit.wav
	$(CMD) $@ $< --prefix "/* john was here */"

golden: $(GOLDEN_DIR) $(GOLDEN_BASE) $(GOLDEN_BASE16) $(GOLDEN_PREFIX) | Makefile

# Clean up generated files
clean:
	rm -f $(FIXTURE_PATHS) $(GOLDEN_BASE) $(GOLDEN_BASE16) $(GOLDEN_PREFIX)

.PHONY: all fixtures golden clean
