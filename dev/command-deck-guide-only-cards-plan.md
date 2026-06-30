# Plan: Add Guide-Only Cards to Command Deck

## Goal

Add a new template/card type for documentation-only cards.

Current workaround:

```toml
[[template]]
id = "git-basics-guide"
name = "Read: Git basics"
desc = "What Git is, why we use it, and what problem it solves."
guide = "git-basics.md"
pattern = "echo 'Open the guide: git-basics.md'"
dry_run = {}
fields = []
```

Target design:

```toml
[[template]]
id = "git-basics-guide"
kind = "guide"
name = "Read: Git basics"
desc = "What Git is, why we use it, and what problem it solves."
guide = "git-basics.md"
```

Guide-only cards should render with only:

```text
open guide
edit
delete
```

They should not show command preview, copy, dry-run, execute-in-app, or execute-in-terminal buttons.

---

## Why this is needed

The onboarding content contains many cards whose purpose is to open documentation rather than run commands.

The current `echo` workaround works, but it is not ideal because:

- it shows command-related actions for guide-only content;
- it makes the TOML noisier;
- it creates fake commands;
- it teaches the wrong mental model;
- it makes the interface look more dangerous than it is.

Guide-only cards are a small app-level improvement that make onboarding content feel intentional instead of improvised under fluorescent lighting.

---

## Scope

### In scope

- Add `kind = "guide"` support to templates.
- Keep existing command templates backward-compatible.
- Render guide-only cards differently in the UI.
- Allow hand-written TOML guide cards to work.
- Optionally update the add/edit modal to create guide-only cards.
- Convert onboarding TOML files from fake `echo` cards to real guide-only cards.

### Out of scope

- Full onboarding progress tracking.
- Search across all guides.
- Guide tagging.
- Auto-seeding the onboarding pack.
- User completion state.
- Remote guide hosting.

Those are useful later. They are not needed for the first clean implementation.

---

## Data model change

File:

```text
src-tauri/src/store.rs
```

Current `Template` assumes a command-oriented card.

Add a `kind` field and make `pattern` default to an empty string.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,

    #[serde(default = "default_template_kind")]
    pub kind: String,

    #[serde(default)]
    pub desc: String,

    #[serde(default)]
    pub pattern: String,

    #[serde(default)]
    pub guide: String,

    #[serde(default)]
    pub fields: Vec<Field>,

    #[serde(default)]
    pub dry_run: DryRun,

    /// filled in at load time from the file name; not stored in the file
    #[serde(skip)]
    pub category: String,
}

fn default_template_kind() -> String {
    "command".into()
}
```

### Why use a string instead of an enum?

A Rust enum would be cleaner, but a string is more forgiving for user-edited TOML.

Recommended valid values:

```text
command
guide
```

Unknown values can be handled in the frontend with an unsupported-card warning.

---

## Backward compatibility

Existing templates do not need to change.

If `kind` is missing, treat the template as:

```toml
kind = "command"
```

Existing command cards remain valid:

```toml
[[template]]
id = "git-status"
name = "Git status"
pattern = "git status --short --branch"
dry_run = {}
fields = []
```

Equivalent internal meaning:

```toml
[[template]]
id = "git-status"
kind = "command"
name = "Git status"
pattern = "git status --short --branch"
dry_run = {}
fields = []
```

This avoids breaking existing user template directories.

---

## Template behavior rules

| Kind      | Required fields         | Optional fields                      | UI behavior                              |
| --------- | ----------------------- | ------------------------------------ | ---------------------------------------- |
| `command` | `id`, `name`, `pattern` | `desc`, `guide`, `fields`, `dry_run` | Show command preview and command actions |
| `guide`   | `id`, `name`, `guide`   | `desc`                               | Show only guide action plus edit/delete  |

### Command card

Example:

```toml
[[template]]
id = "git-status"
kind = "command"
name = "Git status"
desc = "Show repository status."
guide = "git-basics.md"
pattern = "git status --short --branch"
dry_run = {}
fields = []
```

Shows:

```text
copy
dry-run
execute ▸ app
execute ▸ terminal
guide
edit
delete
```

### Guide-only card

Example:

```toml
[[template]]
id = "git-basics-guide"
kind = "guide"
name = "Read: Git basics"
desc = "What Git is and why we use it."
guide = "git-basics.md"
```

Shows:

```text
open guide
edit
delete
```

---

## Frontend helper functions

File:

```text
src/main.js
```

Add helpers near the rendering utilities:

```js
function templateKind(tpl) {
  return tpl.kind || "command";
}

function isGuideOnly(tpl) {
  return templateKind(tpl) === "guide";
}

function isCommandTemplate(tpl) {
  return templateKind(tpl) === "command";
}
```

Optional unsupported check:

```js
function isSupportedTemplateKind(tpl) {
  return ["command", "guide"].includes(templateKind(tpl));
}
```

---

## Rendering change

File:

```text
src/main.js
```

Current `renderCard(tpl)` assumes every template is command-based.

Modify `renderCard` so it branches early:

```js
function renderCard(tpl) {
  if (isGuideOnly(tpl)) return renderGuideCard(tpl);
  if (!isCommandTemplate(tpl)) return renderUnsupportedCard(tpl);
  return renderCommandCard(tpl);
}
```

The existing `renderCard` body can mostly become `renderCommandCard`.

This avoids threading guide-only conditionals through all command behavior.

---

## Guide card renderer

Add:

```js
function renderGuideCard(tpl) {
  const card = document.createElement("div");
  card.className = "cd-card guide-only";

  card.innerHTML = `
    <div class="cd-card-h">
      <h3 class="cd-card-name">
        <span>${esc(tpl.name)}</span>
        <small class="cd-kind-badge">guide</small>
      </h3>
      ${tpl.desc ? `<div class="cd-card-desc">${esc(tpl.desc)}</div>` : ""}
    </div>
    <div class="cd-body">
      <div class="cd-guide-summary">
        ${tpl.guide ? `Guide: <code>${esc(tpl.guide)}</code>` : "No guide linked."}
      </div>
      <div class="cd-actions">
        ${tpl.guide ? `<button class="cd-act guide">open guide</button>` : ""}
        <button class="cd-iconbtn edit">edit</button>
        <button class="cd-iconbtn del">delete</button>
      </div>
    </div>`;

  const guideBtn = card.querySelector(".guide");
  if (guideBtn) guideBtn.onclick = () => openGuide(tpl.guide);

  card.querySelector(".edit").onclick = () => openModal(tpl);
  card.querySelector(".del").onclick = () => deleteTemplate(tpl);

  return card;
}
```

### Notes

- Guide-only cards should not call `buildCommand`.
- Guide-only cards should not read or render `fields`.
- Guide-only cards should not render the command output block.
- Guide-only cards should not expose execution actions.

---

## Command card renderer

Rename the old `renderCard(tpl)` implementation to:

```js
function renderCommandCard(tpl) {
  // existing command-card logic
}
```

Keep the existing behavior:

- render fields;
- assemble command;
- show command preview;
- copy command;
- dry-run;
- execute in app;
- execute in terminal;
- open guide if linked;
- edit;
- delete.

Only minor adjustment needed:

```js
const dry = dryPattern(tpl);
```

should only run for command cards.

---

## Unsupported card renderer

Optional but useful.

```js
function renderUnsupportedCard(tpl) {
  const card = document.createElement("div");
  card.className = "cd-card unsupported";

  card.innerHTML = `
    <div class="cd-card-h">
      <h3 class="cd-card-name"><span>${esc(tpl.name || tpl.id || "Unsupported card")}</span></h3>
      <div class="cd-card-desc">
        Unsupported template kind: <code>${esc(templateKind(tpl))}</code>
      </div>
    </div>
    <div class="cd-body">
      <div class="cd-actions">
        <button class="cd-iconbtn edit">edit</button>
        <button class="cd-iconbtn del">delete</button>
      </div>
    </div>`;

  card.querySelector(".edit").onclick = () => openModal(tpl);
  card.querySelector(".del").onclick = () => deleteTemplate(tpl);

  return card;
}
```

This makes typos obvious.

A typo like:

```toml
kind = "guid"
```

should not silently behave like a command. Silent fallbacks are how bugs rent apartments.

---

## Modal editor changes

Files:

```text
src/index.html
src/main.js
src/styles.css
```

The modal currently assumes a command template and requires `pattern`.

### Add type selector

In `src/index.html`, add a field:

```html
<label>
  Type
  <select id="m-kind">
    <option value="command">Command</option>
    <option value="guide">Guide only</option>
  </select>
</label>
```

Place it near the template name/category fields.

### Load kind in modal

In `openModal(tpl)`:

```js
$("m-kind").value = tpl ? (tpl.kind || "command") : "command";
```

### Update validation

Current logic:

```js
if (!name || !catName || !pattern) return;
```

Replace with:

```js
const kind = $("m-kind").value || "command";
const pattern = $("m-pattern").value.trim();
const guide = $("m-guide").value.trim();

if (!name || !catName) return;
if (kind === "command" && !pattern) return;
if (kind === "guide" && !guide) return;
```

### Construct template by kind

```js
const tpl = {
  id: editing || (name.toLowerCase().replace(/[^a-z0-9]+/g, "-") + "-" + Math.random().toString(36).slice(2, 6)),
  kind,
  name,
  desc: $("m-desc").value.trim(),
  guide,
  pattern: kind === "command" ? pattern : "",
  fields: kind === "command"
    ? toks.map((k) => ({ key: k, label: k, placeholder: "", default: "" }))
    : [],
  dry_run: kind === "command" && dryFlag ? { flag: dryFlag } : {},
  category: catName,
};
```

### Disable command-specific fields for guide-only mode

Add:

```js
function updateModalKindState() {
  const kind = $("m-kind").value || "command";
  const guideOnly = kind === "guide";

  $("m-pattern").disabled = guideOnly;
  $("m-dry").disabled = guideOnly;

  if (guideOnly) {
    $("m-tokens").textContent = "";
  } else {
    updateTokens();
  }
}
```

Wire it:

```js
$("m-kind").onchange = updateModalKindState;
```

Call it inside `openModal(tpl)` after setting values:

```js
updateModalKindState();
```

Also adjust:

```js
$("m-pattern").oninput = updateTokens;
```

to avoid updating tokens when guide-only mode is active:

```js
$("m-pattern").oninput = () => {
  if (($("m-kind").value || "command") !== "guide") updateTokens();
};
```

---

## Persisting guide-only cards

The existing `persist(catName)` function strips only the runtime `category` field:

```js
const clean = cat.templates.map(({ category, ...rest }) => rest);
```

This should continue to work.

Guide-only cards will save as:

```toml
[[template]]
id = "git-basics-guide"
name = "Read: Git basics"
kind = "guide"
desc = "What Git is and why we use it."
guide = "git-basics.md"
pattern = ""
fields = []
dry_run = {}
```

That is acceptable but noisier than ideal.

### Optional cleaner persistence

Later, omit command-only fields for guide cards before saving:

```js
const clean = cat.templates.map(({ category, ...rest }) => {
  if ((rest.kind || "command") === "guide") {
    const { pattern, fields, dry_run, ...guideRest } = rest;
    return guideRest;
  }
  return rest;
});
```

This produces cleaner TOML.

Recommended, but not required.

---

## Styling

File:

```text
src/styles.css
```

Add simple styles:

```css
.cd-card.guide-only {
  opacity: 0.98;
}

.cd-guide-summary {
  font-size: 0.95rem;
  opacity: 0.85;
  margin-bottom: 0.75rem;
}

.cd-kind-badge {
  font-size: 0.7rem;
  font-weight: 700;
  opacity: 0.75;
  border: 1px solid currentColor;
  border-radius: 999px;
  padding: 0.1rem 0.45rem;
  margin-left: 0.5rem;
  vertical-align: middle;
}

.cd-card.unsupported {
  outline: 2px dashed currentColor;
}
```

Keep styling modest. This is a guide card, not a nightclub flyer.

---

## README update

File:

```text
README.md
```

Add a section:

````md
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
````

Also update the existing template example to include:

```toml
kind = "command" # optional; default is command
```

---

## Convert onboarding TOML

After the app supports guide-only cards, convert fake guide cards.

Before:

```toml
[[template]]
id = "uv-vs-conda-guide"
name = "Read: uv vs conda"
desc = "Which environment tool to choose for the job."
guide = "uv-vs-conda.md"
pattern = "echo 'Open the guide: uv-vs-conda.md'"
dry_run = {}
fields = []
```

After:

```toml
[[template]]
id = "uv-vs-conda-guide"
kind = "guide"
name = "Read: uv vs conda"
desc = "Which environment tool to choose for the job."
guide = "uv-vs-conda.md"
```

This makes the onboarding files shorter, clearer, and less fake.

---

## Testing checklist

### Existing command cards

- Existing seeded templates load without changes.
- Existing command cards still show command preview.
- Copy still copies assembled command.
- Dry-run still works.
- Execute in app still works.
- Execute in terminal still works.
- Guide button still opens linked guide.

### Guide-only cards

Create a TOML file:

```toml
[[template]]
id = "test-guide-card"
kind = "guide"
name = "Read: Test guide"
desc = "A guide-only card."
guide = "test-guide.md"
```

Create:

```text
test-guide.md
```

Expected behavior:

- Card appears in the tab.
- No command preview appears.
- No copy button appears.
- No dry-run button appears.
- No execute buttons appear.
- `open guide` opens the guide.
- Edit works.
- Delete works.

### Modal

- New command card requires `pattern`.
- New guide card requires `guide`.
- Guide mode disables or ignores pattern/dry-run fields.
- Editing an existing command card keeps it as command.
- Editing an existing guide card keeps it as guide.

### Persistence

- Saving a command card keeps command fields.
- Saving a guide card does not create broken command behavior.
- Reloading the app preserves `kind = "guide"`.

### Bad input

Test typo:

```toml
kind = "guid"
```

Expected:

- Card renders as unsupported, or at least does not run commands.
- User can edit/delete it.

---

## Suggested PR breakdown

### PR 1: Schema and docs

Files:

```text
src-tauri/src/store.rs
README.md
```

Changes:

- Add `kind`.
- Default `kind` to `command`.
- Make `pattern` default to empty string.
- Document guide-only cards.

### PR 2: Rendering

Files:

```text
src/main.js
src/styles.css
```

Changes:

- Split `renderCard` into guide/command/unsupported renderers.
- Add guide-only UI.
- Add small styling.

### PR 3: Modal editor

Files:

```text
src/index.html
src/main.js
src/styles.css
```

Changes:

- Add type selector.
- Add kind-based validation.
- Disable command fields for guide cards.

### PR 4: Onboarding cleanup

Files:

```text
onboarding/templates/*.toml
```

Changes:

- Replace fake `echo` guide cards with `kind = "guide"`.

---

## Minimal implementation path

The fastest useful version is:

1. Add `kind` to Rust `Template`.
2. Make `pattern` default to empty.
3. Add frontend `isGuideOnly`.
4. Render guide cards without command buttons.
5. Leave modal support for later.

That lets hand-written TOML support guide-only cards immediately.

Minimal supported TOML:

```toml
[[template]]
id = "git-basics-guide"
kind = "guide"
name = "Read: Git basics"
desc = "What Git is and why we use it."
guide = "git-basics.md"
```

This is the best first cut: small, backward-compatible, and directly useful for onboarding.
