Degraded Windows Mode Implementation
Overview
Successfully implemented the Degraded Windows Mode (Copy-Only Mode), allowing Command Deck to serve gracefully as a guided command reference tool on Windows without attempting local execution.

Changes Made
1. Backend Paths & Config (src-tauri/Cargo.toml, store.rs)
dirs Crate Integration: Replaced hardcoded HOME/.config paths with platform-native configuration directories.
Prioritizes ~/.config/command-deck if it already exists (for backwards compatibility).
Falls back to dirs::config_dir() which seamlessly maps to %APPDATA%\command-deck on Windows, and ~/.config/command-deck on Linux.
Execution Mode: Added the execution_mode attribute to Config, falling back safely to "auto".
2. Frontend Settings & Rendering (src/index.html, main.js)
Settings UI: Added an Execution mode dropdown with options for Auto, Copy-only, and Full execution.
Conditional Rendering: Added effectiveExecutionMode() and isCopyOnlyMode() helpers. When the app determines it's in copy-only mode:
The dry-run, execute ▸ app, and execute ▸ terminal buttons are suppressed entirely.
A helpful banner (.cd-mode-note) instructs users to "paste this command into SSH, WSL, Git Bash, VS Code Remote, or your Linux workstation terminal."
3. Styling (src/styles.css)
Styled .cd-mode-note using the standard theme variables (--text-muted, --surface-secondary, --accent left-border) to keep the banner quiet but noticeable.
4. Documentation Updates
README.md: Outlined Windows's "copy-only mode" explicit functionality and added the platform behavior matrix.
onboarding/guides/start-here.md: Added a dedicated ## Windows users block advising how they should consume templates on Windows systems (SSH/WSL over PowerShell).
Validation
cargo check verifies the backend successfully leverages dirs without complaints.
Frontend JS accurately modifies state using the configured caps and cfg from Tauri.