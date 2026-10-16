#!/bin/sh
# Build the game into web/ for any static host (no Next.js needed):
#
#   tools/build-web.sh            # release build into web/game
#   python3 -m http.server -d web 8080   # then open http://localhost:8080
#
# Needs: rustup with the wasm32-unknown-unknown target, and wasm-bindgen-cli at
# the same version as the wasm-bindgen crate in game/Cargo.lock:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version "$(grep -A1 'name = "wasm-bindgen"' game/Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p')" --locked
set -eu
cd "$(dirname "$0")/.."
python3 tools/gen-world.py
(cd game && cargo build --release --target wasm32-unknown-unknown)
rm -rf web/game
wasm-bindgen --target web --no-typescript --out-dir web/game game/target/wasm32-unknown-unknown/release/luckymonkey.wasm
mkdir -p web/game/data && cp game/data/*.json web/game/data/
echo "built: serve web/ (e.g. python3 -m http.server -d web 8080)"
