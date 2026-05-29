#!/usr/bin/env bash
#
# Build the Tarot Battler battle engine to WebAssembly for the browser dev tool.
# Output lands in tools/ui/engine/ (battle_engine.js + battle_engine_bg.wasm),
# which is loaded by index.html and committed so the static site needs no build
# step to run.
#
# One-time prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli   # version must match the wasm-bindgen crate
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT/battle_engine"
cargo build --release --lib --target wasm32-unknown-unknown

wasm-bindgen --target web --no-typescript \
  --out-dir "$ROOT/tools/ui/engine" \
  "$ROOT/battle_engine/target/wasm32-unknown-unknown/release/battle_engine.wasm"

echo "Built battle engine WASM into tools/ui/engine/"
