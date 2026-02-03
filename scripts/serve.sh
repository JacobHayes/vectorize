#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Check for wasm-pack, install if missing
if ! command -v wasm-pack &>/dev/null; then
  echo "Installing wasm-pack..."
  cargo install wasm-pack
fi

# Build the wasm package
echo "Building wasm package..."
wasm-pack build --target web

# Start server
echo "Starting server at http://localhost:8080"
python3 -m http.server 8080 --bind 127.0.0.1
