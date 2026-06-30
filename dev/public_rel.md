1. Validate category names before saving
save_category currently joins the user-provided category name into a file path. This can allow writing outside the template directory if a category name contains path traversal.

Fix in src-tauri/src/store.rs:

```rust
fn is_plain_category_name(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
        && !trimmed.contains("..")
}
```

Use it before writing:

```rust
pub fn save_category(dir: &str, category: &str, templates: &[Template]) -> Result<(), String> {
    if !is_plain_category_name(category) {
        return Err("category name must be a plain file name".into());
    }

    let path = Path::new(dir);
    fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let file = path.join(format!("{}.toml", category));
    let s = toml::to_string_pretty(&CategoryFile {
        template: templates.to_vec(),
    })
    .map_err(|e| e.to_string())?;

    fs::write(file, s).map_err(|e| e.to_string())
}
```

Also validate in the frontend before saving.

2. Remove personal paths from README
Replace any path like:

```text
/Users/<name>/Github/Personal/...
```

with:

```text
/path/to/command-deck/onboarding/guides
```

3. Replace lab-specific GCP defaults
Replace project IDs, zones, VM names, login hosts, and cluster names with placeholders:

```text
YOUR_PROJECT_ID
YOUR_VM_NAME
YOUR_ZONE
user@login.cluster.edu
```

4. Remove or demote the install script

The macOS install script uses sudo, deletes the old app from /Applications, copies the new one, and removes quarantine attributes.

For public release:

remove it; or
rename it to dev_install_unsigned_macos.sh; and
add a warning that it is development-only.
Do not present quarantine bypass as the normal install path.

5. Make the security model explicit
Add this to README and SECURITY.md:

```md
Command Deck is a local command launcher. It is not a sandbox.

Templates can run arbitrary shell commands as the current user. Only use templates from sources you trust, and review generated commands before executing them. Treat `.toml` template files as executable code.
```

Should fix before public

6. Tighten CSP

Suggested tauri.conf.json after bundling fonts:

```json
"csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; script-src 'self'; frame-src 'self' data:"
```

Remove script-src 'unsafe-inline' if it is not needed.

7. Remove external Google Fonts
Use bundled fonts - bundle the current fonts as resources in the app and load them from there. (External fonts create a network dependency and privacy wrinkle.) IMPORTANT - are they copyright protected? Do I need to license them?

8. Make destructive templates less prominent

Move dangerous commands like rsync --delete into separate cards with explicit warnings.

Example:

```toml
[[template]]
id = "rsync-mirror-delete"
name = "Mirror with delete"
desc = "Danger: deletes destination files missing from source. Dry-run first."
pattern = "rsync -avzP --delete {src} {host}:{dst}"
dry_run = { flag = "-n" }
```

9. Change the Tauri identifier from personal naming to a neutral reverse-DNS style identifier you control.
