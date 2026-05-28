#!/usr/bin/env bash
set -euo pipefail

cargo build --release -q
BIN="./target/release/glyf"

hyperfine -N -w 5 -r 50 "$BIN --help" "$BIN --version"
hyperfine -N -w 5 -r 50 "$BIN -c U+0041"
hyperfine -N -w 3 -r 30 "$BIN -s musical" "$BIN -s smile" "$BIN piano"
hyperfine -N -w 3 -r 10 "$BIN -l"
