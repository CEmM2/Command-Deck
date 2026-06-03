use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A single fill-in field on a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub key: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub default: String,
}

/// How a template can be dry-run.
///   none            -> no meaningful dry run (button disabled)
///   flag = "-n"     -> inject this token into the command
///   pattern = "..." -> use a completely separate command pattern
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DryRun {
    #[serde(default)]
    pub flag: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub desc: String,
    pub pattern: String,
    #[serde(default)]
    pub fields: Vec<Field>,
    #[serde(default)]
    pub dry_run: DryRun,
    /// filled in at load time from the file name; not stored in the file
    #[serde(skip)]
    pub category: String,
}

/// One TOML file = one category. `[[template]]` array of tables.
#[derive(Debug, Deserialize, Serialize, Default)]
struct CategoryFile {
    #[serde(default)]
    template: Vec<Template>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub name: String,
    pub templates: Vec<Template>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory holding the per-category *.toml template files.
    pub templates_dir: String,
    /// Login shell used to run commands so SSH agent / PATH / ssh config resolve.
    pub shell: String,
    /// "iterm" or "terminal" for the external-terminal handoff.
    pub terminal: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs_home();
        let templates_dir = home
            .join(".config")
            .join("command-deck")
            .join("templates")
            .to_string_lossy()
            .into_owned();
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        Config {
            templates_dir,
            shell,
            terminal: "terminal".into(),
        }
    }
}

pub fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"))
}

fn config_path() -> PathBuf {
    dirs_home()
        .join(".config")
        .join("command-deck")
        .join("config.toml")
}

pub fn load_config() -> Config {
    let p = config_path();
    if let Ok(s) = fs::read_to_string(&p) {
        if let Ok(c) = toml::from_str::<Config>(&s) {
            return c;
        }
    }
    let c = Config::default();
    let _ = save_config(&c);
    c
}

pub fn save_config(c: &Config) -> Result<(), String> {
    let p = config_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let s = toml::to_string_pretty(c).map_err(|e| e.to_string())?;
    fs::write(&p, s).map_err(|e| e.to_string())
}

/// Load all categories from the templates dir. Seeds defaults on first run.
pub fn load_categories(dir: &str) -> Result<Vec<Category>, String> {
    let path = Path::new(dir);
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
        seed_defaults(path)?;
    }
    let mut cats: Vec<Category> = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "toml").unwrap_or(false))
        .collect();
    entries.sort();

    for p in entries {
        let cat_name = p
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "misc".into());
        let s = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let mut parsed: CategoryFile =
            toml::from_str(&s).map_err(|e| format!("{}: {}", cat_name, e))?;
        for t in parsed.template.iter_mut() {
            t.category = cat_name.clone();
        }
        cats.push(Category {
            name: cat_name,
            templates: parsed.template,
        });
    }
    Ok(cats)
}

/// Persist a category's templates back to its file.
pub fn save_category(dir: &str, category: &str, templates: &[Template]) -> Result<(), String> {
    let path = Path::new(dir);
    fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let file = path.join(format!("{}.toml", category));
    let cf = CategoryFile {
        template: templates.to_vec(),
    };
    let s = toml::to_string_pretty(&cf).map_err(|e| e.to_string())?;
    fs::write(file, s).map_err(|e| e.to_string())
}

fn seed_defaults(dir: &Path) -> Result<(), String> {
    let rsync = r#"
[[template]]
id = "rsync-push"
name = "Push laptop -> remote"
desc = "Copy a local dir up to the VM/cluster. Trailing slash on src copies contents."
pattern = "rsync -avzP --delete {src} {host}:{dst}"
dry_run = { flag = "-n" }
fields = [
  { key = "src",  label = "Local path",  placeholder = "./project/" },
  { key = "host", label = "Remote host", placeholder = "user@cluster" },
  { key = "dst",  label = "Remote path", placeholder = "~/project/" },
]

[[template]]
id = "rsync-pull"
name = "Pull remote -> laptop"
desc = "Bring results back down."
pattern = "rsync -avzP {host}:{src} {dst}"
dry_run = { flag = "-n" }
fields = [
  { key = "host", label = "Remote host", placeholder = "user@cluster" },
  { key = "src",  label = "Remote path", placeholder = "~/results/" },
  { key = "dst",  label = "Local path",  placeholder = "./results/" },
]
"#;

    let worktree = r#"
[[template]]
id = "wt-add"
name = "Create worktree + branch"
desc = "Spin a sibling dir on a new branch. No real dry run for git worktree."
pattern = "git worktree add -b {branch} {path} {base}"
dry_run = {}
fields = [
  { key = "path",   label = "Worktree dir", placeholder = "../proj-featurex" },
  { key = "branch", label = "New branch",   placeholder = "feature/x" },
  { key = "base",   label = "Base ref",     placeholder = "main", default = "main" },
]

[[template]]
id = "wt-remove"
name = "Remove worktree"
desc = "Detach and delete a worktree, then prune."
pattern = "git worktree remove {path} && git worktree prune"
dry_run = {}
fields = [
  { key = "path", label = "Worktree dir", placeholder = "../proj-featurex" },
]

[[template]]
id = "wt-list"
name = "List worktrees"
desc = "Show every worktree and its branch."
pattern = "git worktree list"
dry_run = {}
fields = []
"#;

    let ssh = r#"
[[template]]
id = "ssh-tunnel"
name = "Port-forward tunnel"
desc = "Forward a remote service port to localhost. Run in a real terminal so it stays open."
pattern = "ssh -N -L {lport}:localhost:{rport} {host}"
dry_run = {}
fields = [
  { key = "lport", label = "Local port",  placeholder = "8888" },
  { key = "rport", label = "Remote port", placeholder = "8888" },
  { key = "host",  label = "Remote host", placeholder = "user@cluster" },
]
"#;

    fs::write(dir.join("rsync.toml"), rsync.trim_start()).map_err(|e| e.to_string())?;
    fs::write(dir.join("git worktree.toml"), worktree.trim_start()).map_err(|e| e.to_string())?;
    fs::write(dir.join("ssh.toml"), ssh.trim_start()).map_err(|e| e.to_string())?;
    Ok(())
}
