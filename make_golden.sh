#!/bin/bash

fixture_dir='tests/fixtures'
fixture_files=$(fd -e .wav -I . $fixture_dir)
golden_dir='tests/golden'
cmd='cargo run --release --locked -- -v --no-comment --header --output'

function action {
  output=${2:-$golden_dir/$(basename "$1" .wav).c}
  rm -f "$output"
  $cmd "$output" "$1" "${@:3}"
}

for file in $fixture_files; do
  action "$file"
done

for file in $fixture_files; do
  output=$golden_dir/$(basename "$file" .wav)_base16.c
  action "$file" "$output" --format base16
done

action "$fixture_dir/mono_8bit.wav" "$golden_dir/mono_8bit_prefix.c" --prefix "/* john was here */"
