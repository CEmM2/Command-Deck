# Command Deck

A small native macOS app (Tauri + Rust) for the commands you use often enough
to need but rarely enough to forget — rsync transfers, git worktrees, ssh
tunnels, and whatever else you add. Fill in the blanks, then **copy**,
**dry-run**, or **execute** — either streaming inside the app or handed off to a
real terminal.

---

## What's where

```
command-deck/
├── package.json            # frontend deps (Vite + Tauri JS API)
├── vite.config.js
├── src/                    # frontend (plain HTML/CSS/JS, no framework)
│   ├── index.html
│   ├── styles.css
│   └── main.js
├── src-tauri/              # Rust backend
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/              # PNGs; run `tauri icon` for a proper .icns
│   └── src/
│       ├── main.rs         # command wiring
│       ├── store.rs        # config + TOML template loading/saving
│       └── runner.rs       # dry-run / streaming exec / terminal handoff
└── templates/              # (created on first run in your config dir)
```

## Prerequisites (one time)

- **Rust** — https://rustup.rs  (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Node 18+** — for the frontend build (`brew install node`)
- **Xcode command-line tools** — `xcode-select --install`

## Build & run

```bash
cd command-deck
npm install

# dev mode (hot reload, opens the app window):
npm run tauri dev

# proper app icon (optional but recommended), then a release build:
npm run tauri icon src-tauri/icons/icon-source.png
npm run tauri build
```

The built app lands in
`src-tauri/target/release/bundle/macos/Command Deck.app` and a `.dmg` in
`bundle/dmg/`. Because it's unsigned, the first launch needs a
right-click → Open (or `xattr -dr com.apple.quarantine "Command Deck.app"`).

---

## How templates work

Templates live as **one TOML file per tab** in a directory you set in
**Settings** (default `~/.config/command-deck/templates/`). On first run the app
seeds `rsync.toml`, `git worktree.toml`, and `ssh.toml` there. Because they're
just files you can hand-edit them, keep them in git, or rsync them to the
cluster.

A template looks like:

```toml
[[template]]
id = "rsync-push"
name = "Push laptop -> remote"
desc = "Copy a local dir up to the cluster."
pattern = "rsync -avzP --delete {src} {host}:{dst}"
guide = "gcp-gpu-vm-cheatsheet.md" # optional local guide file
dry_run = { flag = "-n" }          # injected after the program name for dry-run
fields = [
  { key = "src",  label = "Local path",  placeholder = "./project/" },
  { key = "host", label = "Remote host", placeholder = "user@cluster" },
  { key = "dst",  label = "Remote path", placeholder = "~/project/" },
]
```

- Every `{token}` in `pattern` becomes an input field. Listing it in `fields`
  just adds a nice label/placeholder/default; an unlisted token still works.
- `guide` is optional. When set, the card shows a **guide** button that opens a
  matching `.html`, `.md`, `.markdown`, or `.txt` file from the guides directory
  in Settings (default `~/.config/command-deck/guides/`).
- **Dry-run options** (`dry_run = { ... }`):
  - `{ flag = "-n" }` — inject this flag after the program name (rsync-style).
  - `{ pattern = "..." }` — use a totally separate command for the preview.
  - `{}` — no meaningful dry-run; the button is disabled (e.g. `git worktree`).

You can also add/edit templates from the UI (**+ add template** / **edit**); the
app writes the changes straight back to the TOML file.

## The four actions per template

| Action | What it does |
|--------|--------------|
| **copy** | Copies the assembled command to the clipboard. |
| **dry-run** | Runs the dry-run variant, captures output, shows it in the drawer. Disabled if the template declares no dry-run. |
| **execute ▸ app** | Runs for real, streaming stdout/stderr live into the bottom drawer. Best for fire-and-forget rsync. |
| **execute ▸ terminal** | Hands the command to Terminal.app or iTerm in a new window. Best for interactive/long-lived things — ssh tunnels, anything that prompts. |

## Why commands "just work" with SSH

Commands run through your **login shell** (`$SHELL -lic '<command>'`), so your
`~/.ssh/config` host aliases, ssh-agent keys, and PATH resolve exactly as they
do in an interactive terminal. Set your shell in Settings if `$SHELL` isn't what
you want.

The first time you run something that needs an SSH key passphrase, do it via
**execute ▸ terminal** so the agent can prompt you; afterwards the in-app
streaming runner will reuse the agent.

## A word of caution

This app runs whatever command you assemble — that's the point. There's no
sandbox. Use **dry-run** first for anything destructive (note that
`rsync --delete` can remove files on the receiving side), and prefer the
terminal handoff when you want to see prompts before things happen.
