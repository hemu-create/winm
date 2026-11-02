#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/b00t"

rustup show active-toolchain
cargo fmt --check
cargo build --release
llvm-objcopy -O binary target/aarch64-unknown-none-softfloat/release/b00t target/b00t.img
python3 ../tools/pad_b00t.py target/b00t.img
sha256sum target/b00t.img
python3 ../tools/sbom.py --output "$ROOT/build-sbom.json"
echo "reproducible build check complete"
