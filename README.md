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
│       ├── store.rs        # config + TOML template/guide loading/saving
│       └── runner.rs       # dry-run / streaming exec / terminal handoff
└── com_dat/                # optional local guides / reference docs
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
seeds useful starter tabs there. Because they're just files you can hand-edit
them, keep them in git, or rsync them to the cluster.

Current seeded tabs include:

- `rsync.toml`
- `git worktree.toml`
- `ssh.toml`
- `pbs.toml`
- `gcp gpu vm.toml`
- `gcp gpu admin.toml`
- `gpu sync.toml`
- `gpu remote.toml`
- `gpu tmux.toml`
- `gpu profiling.toml`

If a templates directory already exists, Command Deck adds missing seeded tabs
without overwriting files you already edited.

A template looks like:

```toml
[[template]]
id = "rsync-push"
name = "Push laptop -> remote"
desc = "Copy a local dir up to the cluster."
pattern = "rsync -avzP --delete {src} {host}:{dst}"
kind = "command"                   # optional; default is command
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

## Guides

Guides are local `.html`, `.md`, `.markdown`, or `.txt` files in the directory
set in **Settings** (default `~/.config/command-deck/guides/`). The **guides**
button opens a browser-style guide view inside the app.

Set a template's optional `guide` field to connect a command card to a guide:

```toml
guide = "gcp-gpu-vm-cheatsheet.md"
```

When that file exists in the guides directory, the card shows a **guide** action
button. HTML guides render in a sandboxed frame; Markdown and text guides render
directly in the app.

## Guide-only cards

A guide-only card opens documentation without assembling or running a command:

```toml
[[template]]
id = "git-basics-guide"
kind = "guide"
name = "Read: Git basics"
desc = "What Git is and why we use it."
guide = "git-basics.md"
```

Guide-only cards require `guide` and do not require `pattern`, `fields`, or `dry_run`.

If `kind` is omitted, Command Deck treats the template as a normal command card:

```toml
kind = "command"
```

For this repo's current GCP notes, point the guides directory at:

```text
/Users/shmuelosovski/Github/Personal/command-deck/com_dat
```

## Appearance

Settings includes a **Theme** selector with `Dark` and `Bright` modes. The
current visual style is adapted from the reference CSS kept under
`com_dat/styles_ref/`, using the same chunky card borders, shadows, and
light/dark color-token approach while keeping Command Deck's own HTML structure.

## Template actions

| Action | What it does |
|--------|--------------|
| **copy** | Copies the assembled command to the clipboard. |
| **dry-run** | Runs the dry-run variant, captures output, shows it in the drawer. Disabled if the template declares no dry-run. |
| **execute ▸ app** | Runs for real, streaming stdout/stderr live into the bottom drawer. Best for fire-and-forget rsync. |
| **execute ▸ terminal** | Hands the command to Terminal.app, iTerm, or Warp in a new window/tab. Best for interactive/long-lived things — ssh tunnels, anything that prompts. |
| **guide** | Opens the linked local guide file, when the template declares `guide`. |

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
