#!/usr/bin/env zsh
set -euo pipefail

APP_NAME="Command Deck.app"
SCRIPT_DIR="${0:A:h}"
BUILT_APP="$SCRIPT_DIR/src-tauri/target/release/bundle/macos/$APP_NAME"
DEST_APP="/Applications/$APP_NAME"

log() {
  printf '\n==> %s\n' "$1"
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$1" >&2
    exit 1
  fi
}

cleanup_build_artifacts() {
  log "Cleaning build artifacts"
  rm -rf "$SCRIPT_DIR/dist" "$SCRIPT_DIR/src-tauri/target"
}

cd "$SCRIPT_DIR"

require_cmd npm
require_cmd sudo
require_cmd ditto
require_cmd xattr

if [[ ! -d node_modules ]]; then
  log "Installing npm dependencies"
  npm install
fi

if [[ -f "$SCRIPT_DIR/dev/env.local" ]]; then
  log "Loading personal configuration from dev/env.local"
  source "$SCRIPT_DIR/dev/env.local"
fi

log "Building Command Deck"
npm run tauri build

if [[ ! -d "$BUILT_APP" ]]; then
  printf 'Build did not produce expected app bundle: %s\n' "$BUILT_APP" >&2
  exit 1
fi

log "Requesting administrator access for /Applications"
sudo -v

log "Installing to /Applications"
if [[ -e "$DEST_APP" ]]; then
  sudo rm -rf "$DEST_APP"
fi
sudo ditto "$BUILT_APP" "$DEST_APP"

log "Removing quarantine attribute"
(
  cd /Applications
  sudo xattr -dr com.apple.quarantine "$APP_NAME" 2>/dev/null || true
)

cleanup_build_artifacts

log "Installed $DEST_APP"
