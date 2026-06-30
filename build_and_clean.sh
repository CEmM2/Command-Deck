#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

log() {
  printf '\n==> %s\n' "$1"
}

OS="$(uname -s)"
log "Detected OS: $OS"

if [[ "$OS" != "Darwin" && "$OS" != "Linux" ]]; then
  log "Warning: Operating system might not be fully supported by this script ($OS)."
fi

if [[ -f "$SCRIPT_DIR/dev/env.local" ]]; then
  log "Loading configuration from dev/env.local"
  source "$SCRIPT_DIR/dev/env.local"
fi

if [[ ! -d node_modules ]]; then
  log "Installing npm dependencies"
  npm install
fi

log "Building Command Deck"
npm run tauri build

log "Extracting bundles"
rm -rf "$SCRIPT_DIR/build_output"
mkdir -p "$SCRIPT_DIR/build_output"

if [[ -d "$SCRIPT_DIR/src-tauri/target/release/bundle" ]]; then
  cp -R "$SCRIPT_DIR/src-tauri/target/release/bundle/"* "$SCRIPT_DIR/build_output/"
else
  log "Warning: No bundle directory found. Build might have failed."
  exit 1
fi

log "Cleaning intermediate build artifacts"
rm -rf "$SCRIPT_DIR/dist" "$SCRIPT_DIR/src-tauri/target"

log "Build complete. Binaries are available in $SCRIPT_DIR/build_output/"
