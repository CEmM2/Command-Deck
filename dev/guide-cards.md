# Guide-Only Cards — Implementation Walkthrough

## Summary

Implemented the full [guide-only cards plan](file:///Users/shmuelosovski/Github/Personal/command-deck/command-deck-guide-only-cards-plan.md) across 4 source files, 1 doc file, and 9 onboarding TOML files.

## Verified (user-completed)

These sections were already implemented by the user and verified correct:

| Section                   | File                                                                                                | Status |
| ------------------------- | --------------------------------------------------------------------------------------------------- | ------ |
| Data model (`kind` field) | [store.rs](file:///Users/shmuelosovski/Github/Personal/command-deck/src-tauri/src/store.rs#L29-L57) | ✅      |
| Frontend helpers          | [main.js](file:///Users/shmuelosovski/Github/Personal/command-deck/src/main.js#L53-L66)             | ✅      |
| Rendering dispatcher      | [main.js](file:///Users/shmuelosovski/Github/Personal/command-deck/src/main.js#L117-L121)           | ✅      |
| Guide card renderer       | [main.js](file:///Users/shmuelosovski/Github/Personal/command-deck/src/main.js#L85-L115)            | ✅      |
| Command card renderer     | [main.js](file:///Users/shmuelosovski/Github/Personal/command-deck/src/main.js#L147-L237)           | ✅      |
| Unsupported card renderer | [main.js](file:///Users/shmuelosovski/Github/Personal/command-deck/src/main.js#L123-L145)           | ✅      |

## Changes Made

### Modal Editor Changes

#### [index.html](file:///Users/shmuelosovski/Github/Personal/command-deck/src/index.html#L57-L61)
- Added `<select id="m-kind">` between category and description fields
- Options: `Command` (default) and `Guide only`

#### [main.js](file:///Users/shmuelosovski/Github/Personal/command-deck/src/main.js#L286-L376)
- **openModal**: loads `kind` from template, calls `updateModalKindState()` instead of `updateTokens()`
- **updateModalKindState()**: disables `pattern` and `dry-run` fields when "Guide only" is selected, clears token hints
- **m-pattern.oninput**: guarded to skip token updates in guide mode
- **m-kind.onchange**: wired to `updateModalKindState`
- **Validation**: command requires `pattern`, guide requires `guide` file
- **Template construction**: guide cards get empty `pattern`, `fields`, and `dry_run`

---

### Cleaner Persistence

[persist()](file:///Users/shmuelosovski/Github/Personal/command-deck/src/main.js#L386-L398) now strips `pattern`, `fields`, and `dry_run` from guide cards before saving, producing cleaner TOML output.

---

### Styling

[styles.css](file:///Users/shmuelosovski/Github/Personal/command-deck/src/styles.css#L812-L838): Added 4 new rules:
- `.cd-card.guide-only` — subtle opacity
- `.cd-guide-summary` — guide file label styling
- `.cd-kind-badge` — pill badge for "guide" label
- `.cd-card.unsupported` — dashed outline for unknown kinds

---

### README Update

[README.md](file:///Users/shmuelosovski/Github/Personal/command-deck/README.md#L90): Added `kind = "command"` to the existing template example, and a new "Guide-only cards" section documenting the feature.

---

### Onboarding TOML Conversion

Converted **27 fake echo guide cards** across 9 files from:

```toml
pattern = "echo 'Open the guide: foo.md'"
dry_run = {}
fields = []
```

to:

```toml
kind = "guide"
```

| File                        | Guide cards converted | Command cards preserved |
| --------------------------- | --------------------- | ----------------------- |
| Start Here.toml             | 3                     | 1                       |
| Terminal and Files.toml     | 2                     | 3                       |
| Git and GitHub.toml         | 3                     | 10                      |
| Python Environments.toml    | 4                     | 7                       |
| Containers and HPC.toml     | 3                     | 5                       |
| Writing and Publishing.toml | 5                     | 7                       |
| Simulation Tools.toml       | 3                     | 5                       |
| Research Map.toml           | 1                     | 2                       |
| GCP.toml                    | 3                     | 7                       |

## Verification

- ✅ `cargo check` passes — Rust backend compiles with the `kind` field
- ✅ Zero remaining `echo 'Open the guide:` patterns in onboarding TOML
- ✅ All command cards preserved exactly (content, fields, dry_run unchanged)
