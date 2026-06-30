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
    /// Directory holding local guide files shown in the app.
    #[serde(default = "default_guides_dir")]
    pub guides_dir: String,
    /// Login shell used to run commands so SSH agent / PATH / ssh config resolve.
    pub shell: String,
    /// "terminal", "iterm", or "warp" for the external-terminal handoff.
    pub terminal: String,
    /// "dark" or "bright" for the UI theme.
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        Config {
            templates_dir: default_templates_dir(),
            guides_dir: default_guides_dir(),
            shell,
            terminal: "terminal".into(),
            theme: default_theme(),
        }
    }
}

pub fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

fn config_path() -> PathBuf {
    dirs_home()
        .join(".config")
        .join("command-deck")
        .join("config.toml")
}

fn command_deck_config_dir() -> PathBuf {
    dirs_home().join(".config").join("command-deck")
}

fn default_templates_dir() -> String {
    command_deck_config_dir()
        .join("templates")
        .to_string_lossy()
        .into_owned()
}

fn default_guides_dir() -> String {
    command_deck_config_dir()
        .join("guides")
        .to_string_lossy()
        .into_owned()
}

fn default_theme() -> String {
    "dark".into()
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
    } else {
        seed_missing_defaults(path)?;
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

#[derive(Debug, Clone, Serialize)]
pub struct Guide {
    pub name: String,
    pub title: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuideContent {
    pub name: String,
    pub title: String,
    pub kind: String,
    pub body: String,
}

pub fn list_guides(dir: &str) -> Result<Vec<Guide>, String> {
    let path = Path::new(dir);
    fs::create_dir_all(path).map_err(|e| e.to_string())?;

    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && guide_kind(p).is_some())
        .collect();
    entries.sort();

    let mut guides = Vec::new();
    for p in entries {
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let body = fs::read_to_string(&p).unwrap_or_default();
        let kind = guide_kind(&p).unwrap_or("text").to_string();
        let title = guide_title(&name, &kind, &body);
        guides.push(Guide { name, title, kind });
    }
    Ok(guides)
}

pub fn read_guide(dir: &str, name: &str) -> Result<GuideContent, String> {
    if !is_plain_file_name(name) {
        return Err("guide name must be a file name in the guides directory".into());
    }

    let path = Path::new(dir).join(name);
    if !path.is_file() {
        return Err(format!("guide not found: {name}"));
    }

    let kind = guide_kind(&path)
        .ok_or("unsupported guide file type")?
        .to_string();
    let body = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let title = guide_title(name, &kind, &body);

    Ok(GuideContent {
        name: name.to_string(),
        title,
        kind,
        body,
    })
}

fn guide_kind(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") | Some("htm") => Some("html"),
        Some("md") | Some("markdown") => Some("markdown"),
        Some("txt") => Some("text"),
        _ => None,
    }
}

fn guide_title(name: &str, kind: &str, body: &str) -> String {
    let parsed = match kind {
        "html" => html_title(body).or_else(|| html_h1(body)),
        "markdown" => markdown_title(body),
        _ => None,
    };
    parsed.unwrap_or_else(|| title_from_file_name(name))
}

fn html_title(body: &str) -> Option<String> {
    extract_between_case_insensitive(body, "<title>", "</title>")
}

fn html_h1(body: &str) -> Option<String> {
    let lower = body.to_lowercase();
    let open_start = lower.find("<h1")?;
    let open_end = lower[open_start..].find('>')? + open_start + 1;
    let close = lower[open_end..].find("</h1>")? + open_end;
    Some(strip_tags(&body[open_end..close]).trim().to_string()).filter(|s| !s.is_empty())
}

fn markdown_title(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

fn extract_between_case_insensitive(body: &str, open: &str, close: &str) -> Option<String> {
    let lower = body.to_lowercase();
    let open_lower = open.to_lowercase();
    let close_lower = close.to_lowercase();
    let start = lower.find(&open_lower)? + open.len();
    let end = lower[start..].find(&close_lower)? + start;
    Some(strip_tags(&body[start..end]).trim().to_string()).filter(|s| !s.is_empty())
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn title_from_file_name(name: &str) -> String {
    Path::new(name)
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_else(|| name.into())
        .replace(['-', '_'], " ")
}

fn is_plain_file_name(name: &str) -> bool {
    let path = Path::new(name);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
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
    for (file_name, contents) in extra_default_files() {
        fs::write(dir.join(file_name), contents.trim_start()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_missing_defaults(dir: &Path) -> Result<(), String> {
    for (file_name, contents) in extra_default_files() {
        let file = dir.join(file_name);
        if !file.exists() {
            fs::write(file, contents.trim_start()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn extra_default_files() -> [(&'static str, &'static str); 7] {
    [
        ("pbs.toml", pbs_defaults()),
        ("gcp gpu vm.toml", gcp_gpu_vm_defaults()),
        ("gcp gpu admin.toml", gcp_gpu_admin_defaults()),
        ("gpu sync.toml", gpu_sync_defaults()),
        ("gpu remote.toml", gpu_remote_defaults()),
        ("gpu tmux.toml", gpu_tmux_defaults()),
        ("gpu profiling.toml", gpu_profiling_defaults()),
    ]
}

fn pbs_defaults() -> &'static str {
    r#"
[[template]]
id = "pbs-submit"
name = "Submit PBS job"
desc = "Submit a PBS script with common resource flags."
pattern = "qsub -N {name} -q {queue} -l walltime={walltime},mem={mem},nodes={nodes}:ppn={ppn} {script}"
dry_run = {}
fields = [
  { key = "name",     label = "Job name", placeholder = "experiment" },
  { key = "queue",    label = "Queue",    placeholder = "short" },
  { key = "walltime", label = "Walltime", placeholder = "02:00:00" },
  { key = "mem",      label = "Memory",   placeholder = "16gb" },
  { key = "nodes",    label = "Nodes",    placeholder = "1", default = "1" },
  { key = "ppn",      label = "PPN",      placeholder = "4", default = "4" },
  { key = "script",   label = "Script",   placeholder = "run.pbs" },
]

[[template]]
id = "pbs-interactive"
name = "Interactive PBS shell"
desc = "Request an interactive compute session."
pattern = "qsub -I -q {queue} -l walltime={walltime},mem={mem},nodes={nodes}:ppn={ppn}"
dry_run = {}
fields = [
  { key = "queue",    label = "Queue",    placeholder = "short" },
  { key = "walltime", label = "Walltime", placeholder = "02:00:00" },
  { key = "mem",      label = "Memory",   placeholder = "16gb" },
  { key = "nodes",    label = "Nodes",    placeholder = "1", default = "1" },
  { key = "ppn",      label = "PPN",      placeholder = "4", default = "4" },
]

[[template]]
id = "pbs-queue"
name = "My PBS queue"
desc = "Show your queued and running PBS jobs."
pattern = "qstat -u {user}"
dry_run = {}
fields = [
  { key = "user", label = "Username", placeholder = "$USER", default = "$USER" },
]

[[template]]
id = "pbs-job-info"
name = "PBS job details"
desc = "Show full details for a PBS job."
pattern = "qstat -f {job_id}"
dry_run = {}
fields = [
  { key = "job_id", label = "Job ID", placeholder = "123456" },
]

[[template]]
id = "pbs-cancel"
name = "Cancel PBS job"
desc = "Cancel one PBS job by ID."
pattern = "qdel {job_id}"
dry_run = {}
fields = [
  { key = "job_id", label = "Job ID", placeholder = "123456" },
]

[[template]]
id = "pbs-tail-output"
name = "Tail PBS output"
desc = "Follow a PBS output or error file."
pattern = "tail -f {file}"
dry_run = {}
fields = [
  { key = "file", label = "Output file", placeholder = "job.o123456" },
]

[[template]]
id = "pbs-jupyter-tunnel"
name = "PBS Jupyter tunnel"
desc = "Forward Jupyter from a compute node through the cluster login host."
pattern = "ssh -N -L {local_port}:{compute_node}:{remote_port} {login_host}"
dry_run = {}
fields = [
  { key = "local_port",   label = "Local port",   placeholder = "8888", default = "8888" },
  { key = "compute_node", label = "Compute node", placeholder = "node001" },
  { key = "remote_port",  label = "Remote port",  placeholder = "8888", default = "8888" },
  { key = "login_host",   label = "Login host",   placeholder = "user@login.cluster.edu" },
]
"#
}

fn gcp_gpu_vm_defaults() -> &'static str {
    r#"
[[template]]
id = "gcp-gpu-status"
name = "Status"
desc = "Show the current status for the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances describe {vm} --project={project} --zone={zone} --format=\"get(status)\""
dry_run = {}
fields = [
  { key = "vm",      label = "VM",      placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project", label = "Project", placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",    label = "Zone",    placeholder = "us-central1-a", default = "us-central1-a" },
]

[[template]]
id = "gcp-gpu-detailed-status"
name = "Detailed status"
desc = "Show status plus start and stop timestamps."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances describe {vm} --project={project} --zone={zone} --format=\"table(name,status,lastStartTimestamp,lastStopTimestamp)\""
dry_run = {}
fields = [
  { key = "vm",      label = "VM",      placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project", label = "Project", placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",    label = "Zone",    placeholder = "us-central1-a", default = "us-central1-a" },
]

[[template]]
id = "gcp-gpu-start"
name = "Start VM"
desc = "Start the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances start {vm} --project={project} --zone={zone}"
dry_run = {}
fields = [
  { key = "vm",      label = "VM",      placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project", label = "Project", placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",    label = "Zone",    placeholder = "us-central1-a", default = "us-central1-a" },
]

[[template]]
id = "gcp-gpu-stop"
name = "Stop VM"
desc = "Stop the GPU VM while keeping disks."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances stop {vm} --project={project} --zone={zone}"
dry_run = {}
fields = [
  { key = "vm",      label = "VM",      placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project", label = "Project", placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",    label = "Zone",    placeholder = "us-central1-a", default = "us-central1-a" },
]

[[template]]
id = "gcp-gpu-ssh-iap"
name = "SSH via gcloud/IAP"
desc = "SSH into the GPU VM through Identity-Aware Proxy."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute ssh {vm} --project={project} --zone={zone} --tunnel-through-iap"
dry_run = {}
fields = [
  { key = "vm",      label = "VM",      placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project", label = "Project", placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",    label = "Zone",    placeholder = "us-central1-a", default = "us-central1-a" },
]

[[template]]
id = "gcp-gpu-list-matching"
name = "List matching instances"
desc = "List Compute Engine instances matching a name filter."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances list --project={project} --filter=\"name~{name_filter}\""
dry_run = {}
fields = [
  { key = "project",     label = "Project",     placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "name_filter", label = "Name filter", placeholder = "gpu", default = "gpu" },
]
"#
}

fn gcp_gpu_admin_defaults() -> &'static str {
    r#"
[[template]]
id = "gcp-gpu-reset"
name = "Reset VM"
desc = "Hard reset the GPU VM if SSH or boot is stuck."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances reset {vm} --project={project} --zone={zone}"
dry_run = {}
fields = [
  { key = "vm",      label = "VM",      placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project", label = "Project", placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",    label = "Zone",    placeholder = "us-central1-a", default = "us-central1-a" },
]

[[template]]
id = "gcp-gpu-add-temp-ip"
name = "Add temporary external IP"
desc = "Attach an ephemeral external IP for package installation."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances add-access-config {vm} --project={project} --zone={zone} --network-interface={network_interface}"
dry_run = {}
fields = [
  { key = "vm",                label = "VM",                placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project",           label = "Project",           placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",              label = "Zone",              placeholder = "us-central1-a", default = "us-central1-a" },
  { key = "network_interface", label = "Network interface", placeholder = "nic0", default = "nic0" },
]

[[template]]
id = "gcp-gpu-remove-temp-ip"
name = "Remove temporary external IP"
desc = "Remove the ephemeral external IP after setup."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute instances delete-access-config {vm} --project={project} --zone={zone} --access-config-name=\"{access_config}\" --network-interface={network_interface}"
dry_run = {}
fields = [
  { key = "vm",                label = "VM",                placeholder = "gpu-dev", default = "gpu-dev" },
  { key = "project",           label = "Project",           placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "zone",              label = "Zone",              placeholder = "us-central1-a", default = "us-central1-a" },
  { key = "access_config",     label = "Access config",     placeholder = "External NAT", default = "External NAT" },
  { key = "network_interface", label = "Network interface", placeholder = "nic0", default = "nic0" },
]

[[template]]
id = "gcp-gpu-iap-firewall-rules"
name = "Check IAP firewall rules"
desc = "List firewall rules allowing IAP SSH traffic."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "gcloud compute firewall-rules list --project={project} --filter=\"sourceRanges:{source_range}\" --format=\"table(name,allowed,targetTags)\""
dry_run = {}
fields = [
  { key = "project",      label = "Project",      placeholder = "iucc-computational-design", default = "iucc-computational-design" },
  { key = "source_range", label = "Source range", placeholder = "35.235.240.0/20", default = "35.235.240.0/20" },
]
"#
}

fn gpu_sync_defaults() -> &'static str {
    r#"
[[template]]
id = "gpu-sync-repo"
name = "Sync repo to GPU"
desc = "Mirror the local repo into the GPU VM project directory."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "rsync -az --delete --exclude=.git --exclude=.venv --exclude=__pycache__ --exclude=.pytest_cache --exclude=.mypy_cache --exclude=.ruff_cache {local_dir} {host}:{remote_repo}/"
dry_run = { flag = "-n" }
fields = [
  { key = "local_dir",   label = "Local dir",   placeholder = "./", default = "./" },
  { key = "host",        label = "Host",        placeholder = "gpu", default = "gpu" },
  { key = "remote_repo", label = "Remote repo", placeholder = "/data/repos/NumerixWeave", default = "/data/repos/NumerixWeave" },
]

[[template]]
id = "gpu-upload-file"
name = "Upload one file"
desc = "Upload a local file to an explicit remote path."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "rsync -az {local_file} {host}:{remote_file}"
dry_run = { flag = "-n" }
fields = [
  { key = "local_file",  label = "Local file",  placeholder = "examples/fem/demos/demo_RMT/config_implicit.yaml" },
  { key = "host",        label = "Host",        placeholder = "gpu", default = "gpu" },
  { key = "remote_file", label = "Remote file", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/config_implicit.yaml" },
]

[[template]]
id = "gpu-pull-file"
name = "Pull one file"
desc = "Download one file from the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "rsync -az {host}:{remote_file} {local_dir}/"
dry_run = { flag = "-n" }
fields = [
  { key = "host",        label = "Host",        placeholder = "gpu", default = "gpu" },
  { key = "remote_file", label = "Remote file", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/imp.log" },
  { key = "local_dir",   label = "Local dir",   placeholder = "results/demo_RMT", default = "results/demo_RMT" },
]

[[template]]
id = "gpu-pull-results"
name = "Pull results folder"
desc = "Download everything inside the remote results folder."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "rsync -azP {host}:{remote_results}/ {local_results}/"
dry_run = { flag = "-n" }
fields = [
  { key = "host",           label = "Host",           placeholder = "gpu", default = "gpu" },
  { key = "remote_results", label = "Remote results", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/results", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/results" },
  { key = "local_results",  label = "Local results",  placeholder = "results", default = "results" },
]

[[template]]
id = "gpu-preview-pull-results"
name = "Preview pull results"
desc = "Preview downloading the remote results folder."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "rsync -azPn {host}:{remote_results}/ {local_results}/"
dry_run = {}
fields = [
  { key = "host",           label = "Host",           placeholder = "gpu", default = "gpu" },
  { key = "remote_results", label = "Remote results", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/results", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/results" },
  { key = "local_results",  label = "Local results",  placeholder = "results", default = "results" },
]

[[template]]
id = "gpu-mirror-pull-results"
name = "Mirror pull results"
desc = "Mirror remote results locally, deleting local files missing remotely."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "rsync -azP --delete {host}:{remote_results}/ {local_results}/"
dry_run = { flag = "-n" }
fields = [
  { key = "host",           label = "Host",           placeholder = "gpu", default = "gpu" },
  { key = "remote_results", label = "Remote results", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/results", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT/results" },
  { key = "local_results",  label = "Local results",  placeholder = "results", default = "results" },
]
"#
}

fn gpu_remote_defaults() -> &'static str {
    r#"
[[template]]
id = "gpu-ssh-health"
name = "SSH health check"
desc = "Verify SSH, hostname, and GPU visibility."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'whoami && hostname && nvidia-smi'"
dry_run = {}
fields = [
  { key = "host", label = "Host", placeholder = "gpu", default = "gpu" },
]

[[template]]
id = "gpu-tool-check"
name = "GPU tool check"
desc = "Check CUDA, Nsight, Python, and GPU tools on the VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'nvidia-smi; nvcc --version || true; which ncu || true; which nsys || true; python3 --version'"
dry_run = {}
fields = [
  { key = "host", label = "Host", placeholder = "gpu", default = "gpu" },
]

[[template]]
id = "gpu-uv-sync"
name = "Remote uv sync"
desc = "Run uv sync in the remote repo."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'cd {remote_repo} && uv sync'"
dry_run = {}
fields = [
  { key = "host",        label = "Host",        placeholder = "gpu", default = "gpu" },
  { key = "remote_repo", label = "Remote repo", placeholder = "/data/repos/NumerixWeave", default = "/data/repos/NumerixWeave" },
]

[[template]]
id = "gpu-list-recent-files"
name = "List recent remote files"
desc = "Show recently modified files under a remote directory."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'cd {remote_dir} && find . -maxdepth {depth} -type f -printf \"%TY-%Tm-%Td %TH:%TM %s %p\\n\" | sort | tail -{count}'"
dry_run = {}
fields = [
  { key = "host",       label = "Host",       placeholder = "gpu", default = "gpu" },
  { key = "remote_dir", label = "Remote dir", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT" },
  { key = "depth",      label = "Depth",      placeholder = "3", default = "3" },
  { key = "count",      label = "Count",      placeholder = "50", default = "50" },
]
"#
}

fn gpu_tmux_defaults() -> &'static str {
    r#"
[[template]]
id = "gpu-tmux-start-sim"
name = "Start simulation in tmux"
desc = "Start a detached tmux simulation on the GPU VM and keep the shell open after exit."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} \"tmux new -d -s {session} 'bash -lc \\\"cd {remote_dir} && {command} > {log} 2>&1; echo EXIT_CODE=\\$? >> {log}; exec bash\\\"'\""
dry_run = {}
fields = [
  { key = "host",       label = "Host",       placeholder = "gpu", default = "gpu" },
  { key = "session",    label = "Session",    placeholder = "imp", default = "imp" },
  { key = "remote_dir", label = "Remote dir", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT" },
  { key = "command",    label = "Command",    placeholder = "uv run python run.py config_implicit.yaml", default = "uv run python run.py config_implicit.yaml" },
  { key = "log",        label = "Log",        placeholder = "imp.log", default = "imp.log" },
]

[[template]]
id = "gpu-tmux-list"
name = "List tmux sessions"
desc = "List tmux sessions on the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'tmux ls || true'"
dry_run = {}
fields = [
  { key = "host", label = "Host", placeholder = "gpu", default = "gpu" },
]

[[template]]
id = "gpu-tmux-attach"
name = "Attach tmux session"
desc = "Attach to a tmux session on the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh -t {host} 'tmux attach -t {session}'"
dry_run = {}
fields = [
  { key = "host",    label = "Host",    placeholder = "gpu", default = "gpu" },
  { key = "session", label = "Session", placeholder = "imp", default = "imp" },
]

[[template]]
id = "gpu-tmux-tail-log"
name = "Tail simulation log"
desc = "Follow a simulation log on the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'tail -f {remote_dir}/{log}'"
dry_run = {}
fields = [
  { key = "host",       label = "Host",       placeholder = "gpu", default = "gpu" },
  { key = "remote_dir", label = "Remote dir", placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT" },
  { key = "log",        label = "Log",        placeholder = "imp.log", default = "imp.log" },
]

[[template]]
id = "gpu-tmux-capture"
name = "Recent tmux output"
desc = "Capture recent output from a tmux pane."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'tmux capture-pane -pt {session} -S -{lines}'"
dry_run = {}
fields = [
  { key = "host",    label = "Host",    placeholder = "gpu", default = "gpu" },
  { key = "session", label = "Session", placeholder = "imp", default = "imp" },
  { key = "lines",   label = "Lines",   placeholder = "200", default = "200" },
]

[[template]]
id = "gpu-tmux-kill"
name = "Kill tmux session"
desc = "Stop one tmux session on the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'tmux kill-session -t {session}'"
dry_run = {}
fields = [
  { key = "host",    label = "Host",    placeholder = "gpu", default = "gpu" },
  { key = "session", label = "Session", placeholder = "imp", default = "imp" },
]
"#
}

fn gpu_profiling_defaults() -> &'static str {
    r#"
[[template]]
id = "gpu-nsight-compute"
name = "Nsight Compute run"
desc = "Run Nsight Compute on a command in the remote demo directory."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'cd {remote_dir} && ncu -o {profile_path} -f --set full {command}'"
dry_run = {}
fields = [
  { key = "host",         label = "Host",         placeholder = "gpu", default = "gpu" },
  { key = "remote_dir",   label = "Remote dir",   placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT" },
  { key = "profile_path", label = "Profile path", placeholder = "/data/profiles/imp-profile", default = "/data/profiles/imp-profile" },
  { key = "command",      label = "Command",      placeholder = "uv run python run.py config_implicit.yaml", default = "uv run python run.py config_implicit.yaml" },
]

[[template]]
id = "gpu-nsight-systems"
name = "Nsight Systems run"
desc = "Run Nsight Systems on a command in the remote demo directory."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "ssh {host} 'cd {remote_dir} && nsys profile -o {profile_path} -f true {command}'"
dry_run = {}
fields = [
  { key = "host",         label = "Host",         placeholder = "gpu", default = "gpu" },
  { key = "remote_dir",   label = "Remote dir",   placeholder = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT", default = "/data/repos/NumerixWeave/examples/fem/demos/demo_RMT" },
  { key = "profile_path", label = "Profile path", placeholder = "/data/profiles/imp-nsys", default = "/data/profiles/imp-nsys" },
  { key = "command",      label = "Command",      placeholder = "uv run python run.py config_implicit.yaml", default = "uv run python run.py config_implicit.yaml" },
]

[[template]]
id = "gpu-pull-profile"
name = "Pull profile file"
desc = "Download one Nsight profile report from the GPU VM."
guide = "gcp-gpu-vm-cheatsheet.md"
pattern = "rsync -azP {host}:{remote_profile_file} {local_profiles}/"
dry_run = { flag = "-n" }
fields = [
  { key = "host",                label = "Host",                placeholder = "gpu", default = "gpu" },
  { key = "remote_profile_file", label = "Remote profile file", placeholder = "/data/profiles/imp-profile.ncu-rep" },
  { key = "local_profiles",      label = "Local profiles",      placeholder = "profiles", default = "profiles" },
]
"#
}

#[cfg(test)]
mod tests {
    use super::{list_guides, load_categories, read_guide, Config};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn seeded_defaults_are_parseable_and_include_added_categories() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "command-deck-seed-test-{}-{suffix}",
            std::process::id()
        ));

        let categories = load_categories(dir.to_str().unwrap()).unwrap();
        let expected = [
            ("pbs", "pbs-submit"),
            ("gcp gpu vm", "gcp-gpu-status"),
            ("gcp gpu admin", "gcp-gpu-reset"),
            ("gpu sync", "gpu-sync-repo"),
            ("gpu remote", "gpu-ssh-health"),
            ("gpu tmux", "gpu-tmux-start-sim"),
            ("gpu profiling", "gpu-nsight-compute"),
        ];

        for (category_name, template_id) in expected {
            let category = categories
                .iter()
                .find(|category| category.name == category_name)
                .unwrap_or_else(|| panic!("{category_name} defaults should be seeded"));

            assert!(category
                .templates
                .iter()
                .any(|template| template.id == template_id));
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn existing_template_dir_gets_missing_added_defaults() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "command-deck-existing-seed-test-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("custom.toml"),
            r#"
[[template]]
id = "custom"
name = "Custom"
pattern = "echo hello"
"#
            .trim_start(),
        )
        .unwrap();

        let categories = load_categories(dir.to_str().unwrap()).unwrap();

        assert!(dir.join("pbs.toml").exists());
        assert!(dir.join("gcp gpu vm.toml").exists());
        assert!(dir.join("gcp gpu admin.toml").exists());
        assert!(dir.join("gpu sync.toml").exists());
        assert!(dir.join("gpu remote.toml").exists());
        assert!(dir.join("gpu tmux.toml").exists());
        assert!(dir.join("gpu profiling.toml").exists());
        assert!(categories.iter().any(|category| category.name == "custom"));
        assert!(categories.iter().any(|category| category.name == "pbs"));
        assert!(categories
            .iter()
            .any(|category| category.name == "gcp gpu vm"));
        assert!(categories
            .iter()
            .any(|category| category.name == "gcp gpu admin"));
        assert!(categories
            .iter()
            .any(|category| category.name == "gpu sync"));
        assert!(categories
            .iter()
            .any(|category| category.name == "gpu remote"));
        assert!(categories
            .iter()
            .any(|category| category.name == "gpu tmux"));
        assert!(categories
            .iter()
            .any(|category| category.name == "gpu profiling"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn old_config_without_guides_dir_still_loads() {
        let cfg: Config = toml::from_str(
            r#"
templates_dir = "/tmp/command-deck/templates"
shell = "/bin/zsh"
terminal = "terminal"
"#,
        )
        .unwrap();

        assert_eq!(cfg.templates_dir, "/tmp/command-deck/templates");
        assert!(cfg.guides_dir.ends_with(".config/command-deck/guides"));
        assert_eq!(cfg.theme, "dark");
    }

    #[test]
    fn guides_are_listed_and_read_from_guides_dir() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "command-deck-guides-test-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("gpu-guide.md"),
            r#"
# GPU Guide

Use `ssh gpu`.
"#
            .trim_start(),
        )
        .unwrap();
        fs::write(
            dir.join("setup.html"),
            r#"<!doctype html><html><head><title>Setup Guide</title></head><body></body></html>"#,
        )
        .unwrap();
        fs::write(dir.join("ignored.toml"), "").unwrap();

        let guides = list_guides(dir.to_str().unwrap()).unwrap();
        assert_eq!(guides.len(), 2);
        assert!(guides
            .iter()
            .any(|guide| guide.name == "gpu-guide.md" && guide.title == "GPU Guide"));
        assert!(guides
            .iter()
            .any(|guide| guide.name == "setup.html" && guide.title == "Setup Guide"));

        let guide = read_guide(dir.to_str().unwrap(), "gpu-guide.md").unwrap();
        assert_eq!(guide.kind, "markdown");
        assert!(guide.body.contains("ssh gpu"));

        let traversal = read_guide(dir.to_str().unwrap(), "../gpu-guide.md");
        assert!(traversal.is_err());

        let _ = fs::remove_dir_all(dir);
    }
}
