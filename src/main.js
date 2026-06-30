import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "@fontsource/baloo-2/400.css";
import "@fontsource/baloo-2/500.css";
import "@fontsource/baloo-2/600.css";
import "@fontsource/baloo-2/700.css";
import "@fontsource/baloo-2/800.css";
import "@fontsource/patrick-hand/400.css";
import "@fontsource/space-mono/400.css";
import "@fontsource/space-mono/700.css";
import "@fontsource-variable/fraunces";

let cfg = null;
let caps = null;
let categories = [];   // [{name, templates:[...]}]
let guides = [];       // [{name,title,kind}]
let active = null;
let editing = null;    // template id being edited, or null for new
let activeGuide = null;

const $ = (id) => document.getElementById(id);

function effectiveExecutionMode() {
  const mode = cfg?.execution_mode || "auto";
  if (mode === "copy_only") return "copy_only";
  if (mode === "full") return "full";
  return caps?.default_mode || "full";
}

function isCopyOnlyMode() {
  return effectiveExecutionMode() === "copy_only";
}

// ---- command assembly ----
function buildCommand(pattern, fields, values, dryFlag) {
  let out = pattern;
  (fields || []).forEach((f) => {
    const v = (values[f.key] ?? "").trim() || f.placeholder || `{${f.key}}`;
    out = out.split(`{${f.key}}`).join(v);
  });
  if (dryFlag) {
    // inject the dry-run flag right after the program name (first token)
    const parts = out.split(" ");
    parts.splice(1, 0, dryFlag);
    out = parts.join(" ");
  }
  return out;
}

function dryPattern(tpl) {
  if (tpl.dry_run && tpl.dry_run.pattern) return { mode: "pattern", val: tpl.dry_run.pattern };
  if (tpl.dry_run && tpl.dry_run.flag) return { mode: "flag", val: tpl.dry_run.flag };
  return null;
}

// ---- rendering ----
function renderTabs() {
  const tabs = $("tabs");
  tabs.innerHTML = "";
  categories.forEach((c) => {
    const b = document.createElement("button");
    b.className = "cd-tab" + (c.name === active ? " on" : "");
    b.textContent = c.name;
    b.onclick = () => { active = c.name; render(); };
    tabs.appendChild(b);
  });
  const add = document.createElement("button");
  add.className = "cd-tab add";
  add.textContent = "+ add template";
  add.onclick = () => openModal(null);
  tabs.appendChild(add);
}
function templateKind(tpl) {
  if (tpl.kind === "command" && !tpl.pattern && tpl.guide) {
    return "guide";
  }
  return tpl.kind || "command";
}

function isGuideOnly(tpl) {
  return templateKind(tpl) === "guide";
}

function isCommandTemplate(tpl) {
  return templateKind(tpl) === "command";
}
function isSupportedTemplateKind(tpl) {
  return ["command", "guide"].includes(templateKind(tpl));
}

function renderGrid() {
  const grid = $("grid");
  grid.innerHTML = "";
  const cat = categories.find((c) => c.name === active);
  const list = cat ? cat.templates : [];
  if (!list.length) {
    grid.innerHTML = '<div class="cd-empty">No templates in this tab yet.</div>';
    return;
  }
  list.forEach((tpl) => grid.appendChild(renderCard(tpl)));
}

//Guide-only cards should not call buildCommand.
// Guide-only cards should not read or render fields.
// Guide-only cards should not render the command output block.
// Guide-only cards should not expose execution actions.

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

function renderCard(tpl) {
  if (isGuideOnly(tpl)) return renderGuideCard(tpl);
  if (!isCommandTemplate(tpl)) return renderUnsupportedCard(tpl);
  return renderCommandCard(tpl);
}

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

function renderCommandCard(tpl) {
  const card = document.createElement("div");
  card.className = "cd-card";
  const values = {};

  const dry = isCommandTemplate(tpl) ? dryPattern(tpl) : null;

  const copyOnly = isCopyOnlyMode();

  card.innerHTML = `
    <div class="cd-card-h">
      <h3 class="cd-card-name"><span>${esc(tpl.name)}</span></h3>
      ${tpl.desc ? `<div class="cd-card-desc">${esc(tpl.desc)}</div>` : ""}
    </div>
    <div class="cd-body">
      <div class="cd-fields"></div>
      <div class="cd-out"><code></code></div>
      ${copyOnly ? `<div class="cd-mode-note">Copy-only mode: paste this command into SSH, WSL, Git Bash, VS Code Remote, or your Linux workstation terminal.</div>` : ""}
      <div class="cd-actions">
        <button class="cd-act copy">copy</button>
        ${copyOnly ? "" : `<button class="cd-act dry" ${dry ? "" : "disabled title='no dry run for this command'"}>dry-run</button>`}
        ${copyOnly ? "" : `<button class="cd-act exec">execute ▸ app</button>`}
        ${copyOnly ? "" : `<button class="cd-act term">execute ▸ terminal</button>`}
        ${tpl.guide ? `<button class="cd-act guide" title="open ${esc(tpl.guide)}">guide</button>` : ""}
        <button class="cd-iconbtn edit">edit</button>
        <button class="cd-iconbtn del">delete</button>
      </div>
    </div>`;

  const fieldsWrap = card.querySelector(".cd-fields");
  const codeEl = card.querySelector(".cd-out code");

  const refresh = () => { codeEl.textContent = buildCommand(tpl.pattern, tpl.fields, values, null); };

  (tpl.fields || []).forEach((f) => {
    const wrap = document.createElement("div");
    wrap.className = "cd-field";
    wrap.innerHTML = `<label>${esc(f.label || f.key)}</label>`;
    const inp = document.createElement("input");
    inp.placeholder = f.placeholder || "";
    inp.value = f.default || "";
    if (f.default) values[f.key] = f.default;
    inp.oninput = () => { values[f.key] = inp.value; refresh(); };
    wrap.appendChild(inp);
    fieldsWrap.appendChild(wrap);
  });
  refresh();



  // copy
  card.querySelector(".copy").onclick = async (e) => {
    await navigator.clipboard.writeText(codeEl.textContent);
    flash(e.target, "copy");
  };
  // dry-run
  const dryBtn = card.querySelector(".dry");
  if (dry) {
    dryBtn.onclick = async () => {
      let cmd;
      if (dry.mode === "pattern") cmd = buildCommand(dry.val, tpl.fields, values, null);
      else cmd = buildCommand(tpl.pattern, tpl.fields, values, dry.val);
      openDrawer(`dry-run · ${tpl.name}`, "running…");
      try {
        const out = await invoke("run_capture", { cfg, command: cmd });
        setDrawer(out);
        setStatus("done");
      } catch (err) { setDrawer(String(err), true); setStatus("error"); }
    };
  }
  // execute in-app (streaming)
  card.querySelector(".exec").onclick = async () => {
    const cmd = buildCommand(tpl.pattern, tpl.fields, values, null);
    openDrawer(`execute · ${tpl.name}`, "running…");
    clearDrawer();
    try {
      await invoke("run_stream", { cfg, command: cmd });
    } catch (err) { appendDrawer(String(err), true); setStatus("error"); }
  };
  // execute in terminal
  card.querySelector(".term").onclick = async () => {
    const cmd = buildCommand(tpl.pattern, tpl.fields, values, null);
    try {
      await invoke("run_in_terminal", { cfg, command: cmd });
    } catch (err) { openDrawer("terminal handoff", String(err)); setStatus("error"); }
  };
  const guideBtn = card.querySelector(".guide");
  if (guideBtn) guideBtn.onclick = () => openGuide(tpl.guide);
  // edit / delete
  card.querySelector(".edit").onclick = () => openModal(tpl);
  card.querySelector(".del").onclick = () => deleteTemplate(tpl);

  return card;
}

function render() {
  renderTabs();
  renderGrid();
  $("note").textContent =
    `templates dir: ${cfg ? cfg.templatesDir || cfg.templates_dir : "?"} · guides dir: ${cfg ? cfg.guidesDir || cfg.guides_dir : "?"}`;
}

function normalizeTheme(theme) {
  return theme === "bright" ? "bright" : "dark";
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = normalizeTheme(theme);
}

// ---- drawer (output) ----
function openDrawer(title, status) {
  $("drawer").classList.add("open");
  $("drawer-title").textContent = title;
  setStatus(status || "");
}
function clearDrawer() { $("drawer-body").innerHTML = ""; }
function setDrawer(text, isErr) {
  $("drawer-body").innerHTML = "";
  appendDrawer(text, isErr);
}
function appendDrawer(text, isErr) {
  const span = document.createElement("span");
  if (isErr) span.className = "err";
  span.textContent = text + "\n";
  $("drawer-body").appendChild(span);
  $("drawer-body").scrollTop = $("drawer-body").scrollHeight;
}
function setStatus(s) { $("drawer-status").textContent = s; }

$("drawer-close").onclick = () => $("drawer").classList.remove("open");
$("drawer-clear").onclick = () => clearDrawer();

// stream listeners
listen("run:line", (e) => {
  const { stream, line } = e.payload;
  appendDrawer(line, stream === "stderr");
});
listen("run:done", (e) => {
  setStatus(`exit ${e.payload.code}`);
});

// ---- modal (add/edit) ----
function openModal(tpl) {
  editing = tpl ? tpl.id : null;
  $("modal-title").textContent = tpl ? "Edit template" : "New template";
  $("m-name").value = tpl ? tpl.name : "";
  $("m-cat").value = tpl ? tpl.category : (active || "");
  $("m-kind").value = tpl ? templateKind(tpl) : "command";
  $("m-desc").value = tpl ? tpl.desc : "";
  $("m-guide").value = tpl ? (tpl.guide || "") : "";
  $("m-pattern").value = tpl ? tpl.pattern : "";
  $("m-dry").value = tpl && tpl.dry_run ? (tpl.dry_run.flag || "") : "";
  const dl = $("m-cats"); dl.innerHTML = "";
  categories.forEach((c) => { const o = document.createElement("option"); o.value = c.name; dl.appendChild(o); });
  const guideList = $("m-guides"); guideList.innerHTML = "";
  guides.forEach((g) => { const o = document.createElement("option"); o.value = g.name; guideList.appendChild(o); });
  updateModalKindState();
  $("modal").style.display = "flex";
}
$("m-pattern").oninput = () => {
  if (($("m-kind").value || "command") !== "guide") updateTokens();
};
$("m-kind").onchange = updateModalKindState;
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
function updateTokens() {
  const m = ($("m-pattern").value.match(/\{[a-zA-Z0-9_]+\}/g) || []);
  const toks = [...new Set(m.map((t) => t.slice(1, -1)))];
  $("m-tokens").textContent = toks.length ? "Fields: " + toks.map((t) => `{${t}}`).join("  ") : "";
}
$("m-cancel").onclick = () => { $("modal").style.display = "none"; };
$("m-save").onclick = async () => {
  const name = $("m-name").value.trim();
  const catName = $("m-cat").value.trim();
  const kind = $("m-kind").value || "command";
  const pattern = $("m-pattern").value.trim();
  const guide = $("m-guide").value.trim();

  if (!name || !catName) return;
  if (catName.includes("/") || catName.includes("\\") || catName.includes("..")) {
    alert("Category name cannot contain /, \\, or ..");
    return;
  }
  if (kind === "command" && !pattern) return;
  if (kind === "guide" && !guide) return;

  const m = (pattern.match(/\{[a-zA-Z0-9_]+\}/g) || []);
  const toks = [...new Set(m.map((t) => t.slice(1, -1)))];
  const dryFlag = $("m-dry").value.trim();

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

  // if editing and category changed, remove from old category first
  if (editing) {
    for (const c of categories) {
      const i = c.templates.findIndex((t) => t.id === editing);
      if (i >= 0 && c.name !== catName) {
        c.templates.splice(i, 1);
        await persist(c.name);
      } else if (i >= 0) {
        c.templates[i] = tpl;
      }
    }
  }
  let cat = categories.find((c) => c.name === catName);
  if (!cat) { cat = { name: catName, templates: [] }; categories.push(cat); }
  if (!cat.templates.find((t) => t.id === tpl.id)) cat.templates.push(tpl);

  await persist(catName);
  $("modal").style.display = "none";
  active = catName;
  render();
};

async function deleteTemplate(tpl) {
  const cat = categories.find((c) => c.name === tpl.category);
  if (!cat) return;
  cat.templates = cat.templates.filter((t) => t.id !== tpl.id);
  await persist(cat.name);
  render();
}

async function persist(catName) {
  const cat = categories.find((c) => c.name === catName);
  if (!cat) return;
  // strip runtime-only fields; omit command-only fields for guide cards
  const clean = cat.templates.map(({ category, ...rest }) => {
    if ((rest.kind || "command") === "guide") {
      const { pattern, fields, dry_run, ...guideRest } = rest;
      return guideRest;
    }
    return rest;
  });
  await invoke("save_category", { cfg, category: catName, templates: clean });
}

// ---- settings ----
$("open-settings").onclick = () => {
  $("s-dir").value = cfg.templates_dir || cfg.templatesDir || "";
  $("s-guides-dir").value = cfg.guides_dir || cfg.guidesDir || "";
  $("s-shell").value = cfg.shell || "";

  const termSelect = $("s-term");
  termSelect.innerHTML = "";
  if (caps && caps.os === "linux") {
    termSelect.innerHTML = `
      <option value="default">Default terminal</option>
      <option value="gnome-terminal">GNOME Terminal</option>
      <option value="konsole">Konsole</option>
      <option value="xfce4-terminal">XFCE Terminal</option>
      <option value="xterm">xterm</option>
      <option value="custom">Custom</option>
    `;
  } else {
    termSelect.innerHTML = `
      <option value="terminal">Terminal.app</option>
      <option value="iterm">iTerm</option>
      <option value="warp">Warp</option>
    `;
  }
  
  $("s-term").value = cfg.terminal || (caps && caps.os === "linux" ? "default" : "terminal");
  $("s-execution-mode").value = cfg.execution_mode || "auto";
  $("s-theme").value = normalizeTheme(cfg.theme);
  $("settings").style.display = "flex";
};
$("s-cancel").onclick = () => { $("settings").style.display = "none"; };
$("s-save").onclick = async () => {
  const next = {
    templates_dir: $("s-dir").value.trim(),
    guides_dir: $("s-guides-dir").value.trim(),
    shell: $("s-shell").value.trim(),
    terminal: $("s-term").value,
    execution_mode: $("s-execution-mode").value,
    theme: normalizeTheme($("s-theme").value),
  };
  cfg = await invoke("set_config", { cfg: next });
  applyTheme(cfg.theme);
  $("settings").style.display = "none";
  await reload();
};

// ---- guides ----
$("open-guides").onclick = async () => {
  await reloadGuides();
  renderGuideList();
  $("guides").style.display = "flex";
};
$("guides-close").onclick = () => { $("guides").style.display = "none"; };

async function reloadGuides() {
  guides = await invoke("list_guides", { cfg });
}

function renderGuideList() {
  const list = $("guide-list");
  list.innerHTML = "";
  $("guides-note").textContent = cfg ? (cfg.guides_dir || cfg.guidesDir || "") : "";

  if (!guides.length) {
    list.innerHTML = '<div class="cd-guide-empty">No guides found. Add .html or .md files to the guides directory.</div>';
    $("guide-title").textContent = "No guides";
    $("guide-body").innerHTML = "";
    return;
  }

  guides.forEach((guide) => {
    const btn = document.createElement("button");
    btn.className = "cd-guide-item" + (guide.name === activeGuide ? " on" : "");
    btn.innerHTML = `${esc(guide.title)}<small>${esc(guide.name)}</small>`;
    btn.onclick = () => openGuide(guide.name);
    list.appendChild(btn);
  });
}

async function openGuide(name) {
  if (!name) return;
  try {
    const guide = await invoke("read_guide", { cfg, name });
    activeGuide = guide.name;
    $("guides").style.display = "flex";
    $("guide-title").textContent = guide.title;
    const body = $("guide-body");
    body.innerHTML = "";

    if (guide.kind === "html") {
      const frame = document.createElement("iframe");
      frame.className = "cd-guide-frame";
      frame.setAttribute("sandbox", "");
      frame.srcdoc = guide.body;
      body.appendChild(frame);
    } else if (guide.kind === "markdown") {
      const wrap = document.createElement("div");
      wrap.className = "cd-guide-md";
      wrap.innerHTML = renderMarkdown(guide.body);
      body.appendChild(wrap);
    } else {
      const pre = document.createElement("pre");
      pre.className = "cd-guide-md";
      pre.textContent = guide.body;
      body.appendChild(pre);
    }

    renderGuideList();
  } catch (err) {
    $("guides").style.display = "flex";
    $("guide-title").textContent = "Guide error";
    $("guide-body").innerHTML = `<div class="cd-guide-empty">${esc(String(err))}</div>`;
  }
}

function renderMarkdown(md) {
  const lines = md.split(/\r?\n/);
  const out = [];
  let inCode = false;
  let inList = false;

  const closeList = () => {
    if (inList) {
      out.push("</ul>");
      inList = false;
    }
  };

  for (const line of lines) {
    if (line.startsWith("```")) {
      if (inCode) {
        out.push("</code></pre>");
        inCode = false;
      } else {
        closeList();
        out.push("<pre><code>");
        inCode = true;
      }
      continue;
    }

    if (inCode) {
      out.push(esc(line) + "\n");
      continue;
    }

    if (!line.trim()) {
      closeList();
      continue;
    }

    if (line.startsWith("### ")) {
      closeList();
      out.push(`<h3>${inlineMarkdown(line.slice(4))}</h3>`);
    } else if (line.startsWith("## ")) {
      closeList();
      out.push(`<h2>${inlineMarkdown(line.slice(3))}</h2>`);
    } else if (line.startsWith("# ")) {
      closeList();
      out.push(`<h1>${inlineMarkdown(line.slice(2))}</h1>`);
    } else if (line.startsWith("- ")) {
      if (!inList) {
        out.push("<ul>");
        inList = true;
      }
      out.push(`<li>${inlineMarkdown(line.slice(2))}</li>`);
    } else {
      closeList();
      out.push(`<p>${inlineMarkdown(line)}</p>`);
    }
  }

  closeList();
  if (inCode) out.push("</code></pre>");
  return out.join("\n");
}

function inlineMarkdown(s) {
  return esc(s).replace(/`([^`]+)`/g, "<code>$1</code>");
}

// ---- util ----
function esc(s) { return (s || "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c])); }
function flash(btn, cls) { btn.classList.add("done"); const old = btn.textContent; btn.textContent = "✓"; setTimeout(() => { btn.classList.remove("done"); btn.textContent = old; }, 1200); }

// ---- boot ----
async function reload() {
  await reloadGuides();
  categories = await invoke("list_categories", { cfg });
  if (!active || !categories.find((c) => c.name === active)) {
    active = categories.length ? categories[0].name : null;
  }
  render();
}
async function boot() {
  cfg = await invoke("get_config");
  caps = await invoke("runtime_capabilities");
  applyTheme(cfg.theme);
  await reload();
}
boot();
