# Linux Support Implementation Walkthrough

## Overview

Added robust Linux support to Command Deck, bringing it up to parity for template execution, dry runs, and terminal handoffs across Ubuntu, Pop!_OS, and other Linux distributions.

## Backend Changes (Rust)

- **OS Specific Implementation**: Split `run_in_terminal` into macOS, Linux, and Windows targets. 
  - The macOS implementation continues to support `open` and AppleScript for Terminal.app, iTerm, and Warp.
  - The Linux implementation uses a prioritized candidates list (`xdg-terminal-exec`, `gnome-terminal`, `konsole`, `xfce4-terminal`, `xterm`) backed by a new `command_exists` helper.
  - The Windows stub safely errors out indicating that terminal execution is currently unsupported (to lay ground for Degraded Windows mode).
- **Runtime Capabilities**: Introduced a new `runtime_capabilities` Tauri command to securely feed OS context (`std::env::consts::OS`) to the frontend without requiring a separate plugin.
- **Safer Shell Fallbacks**: Updated the default configuration fallback shell to `/bin/sh` from macOS's `/bin/zsh`.
- **Packaging Targets**: Added `appimage` and `deb` targets to `tauri.conf.json`'s `bundle.targets` to provide out-of-the-box binaries for Linux users.

## Frontend Updates (HTML/JS)

- **Dynamic Terminal Selection**:
  - The hardcoded terminal `<select>` list was removed from `index.html`.
  - The application now fetches `runtime_capabilities` in the `boot()` initialization flow.
  - The Settings modal dynamically injects valid terminal options (`GNOME Terminal`, `Konsole`, `XFCE`, etc. vs. `iTerm`, `Terminal.app`) based on the detected operating system.

## Documentation

- **README.md**: Added the official Linux support matrix outlining what works (Execution, Dry-runs, AppImage) and any associated limits.
