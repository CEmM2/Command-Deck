import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ---- state ----
let cfg = null;
let categories = [];   // [{name, templates:[...]}]
let active = null;
let editing = null;    // template id being edited, or null for new

const $ = (id) => document.getElementById(id);

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

function renderCard(tpl) {
  const card = document.createElement("div");
  card.className = "cd-card";
  const values = {};

  const dry = dryPattern(tpl);

  card.innerHTML = `
    <div class="cd-card-h">
      <h3 class="cd-card-name"><span>${esc(tpl.name)}</span></h3>
      ${tpl.desc ? `<div class="cd-card-desc">${esc(tpl.desc)}</div>` : ""}
    </div>
    <div class="cd-body">
      <div class="cd-fields"></div>
      <div class="cd-out"><code></code></div>
      <div class="cd-actions">
        <button class="cd-act copy">copy</button>
        <button class="cd-act dry" ${dry ? "" : "disabled title='no dry run for this command'"}>dry-run</button>
        <button class="cd-act exec">execute ▸ app</button>
        <button class="cd-act term">execute ▸ terminal</button>
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
  // edit / delete
  card.querySelector(".edit").onclick = () => openModal(tpl);
  card.querySelector(".del").onclick = () => deleteTemplate(tpl);

  return card;
}

function render() { renderTabs(); renderGrid(); $("note").textContent =
  `templates dir: ${cfg ? cfg.templatesDir || cfg.templates_dir : "?"}`; }

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
  $("m-desc").value = tpl ? tpl.desc : "";
  $("m-pattern").value = tpl ? tpl.pattern : "";
  $("m-dry").value = tpl && tpl.dry_run ? (tpl.dry_run.flag || "") : "";
  const dl = $("m-cats"); dl.innerHTML = "";
  categories.forEach((c) => { const o = document.createElement("option"); o.value = c.name; dl.appendChild(o); });
  updateTokens();
  $("modal").style.display = "flex";
}
$("m-pattern").oninput = updateTokens;
function updateTokens() {
  const m = ($("m-pattern").value.match(/\{[a-zA-Z0-9_]+\}/g) || []);
  const toks = [...new Set(m.map((t) => t.slice(1, -1)))];
  $("m-tokens").textContent = toks.length ? "Fields: " + toks.map((t) => `{${t}}`).join("  ") : "";
}
$("m-cancel").onclick = () => { $("modal").style.display = "none"; };
$("m-save").onclick = async () => {
  const name = $("m-name").value.trim();
  const catName = $("m-cat").value.trim();
  const pattern = $("m-pattern").value.trim();
  if (!name || !catName || !pattern) return;
  const m = (pattern.match(/\{[a-zA-Z0-9_]+\}/g) || []);
  const toks = [...new Set(m.map((t) => t.slice(1, -1)))];
  const dryFlag = $("m-dry").value.trim();

  const tpl = {
    id: editing || (name.toLowerCase().replace(/[^a-z0-9]+/g, "-") + "-" + Math.random().toString(36).slice(2, 6)),
    name,
    desc: $("m-desc").value.trim(),
    pattern,
    fields: toks.map((k) => ({ key: k, label: k, placeholder: "", default: "" })),
    dry_run: dryFlag ? { flag: dryFlag } : {},
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
  // strip the runtime-only `category` field before saving
  const clean = cat.templates.map(({ category, ...rest }) => rest);
  await invoke("save_category", { cfg, category: catName, templates: clean });
}

// ---- settings ----
$("open-settings").onclick = () => {
  $("s-dir").value = cfg.templates_dir || cfg.templatesDir || "";
  $("s-shell").value = cfg.shell || "";
  $("s-term").value = cfg.terminal || "terminal";
  $("settings").style.display = "flex";
};
$("s-cancel").onclick = () => { $("settings").style.display = "none"; };
$("s-save").onclick = async () => {
  const next = { templates_dir: $("s-dir").value.trim(), shell: $("s-shell").value.trim(), terminal: $("s-term").value };
  cfg = await invoke("set_config", { cfg: next });
  $("settings").style.display = "none";
  await reload();
};

// ---- util ----
function esc(s) { return (s || "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c])); }
function flash(btn, cls) { btn.classList.add("done"); const old = btn.textContent; btn.textContent = "✓"; setTimeout(() => { btn.classList.remove("done"); btn.textContent = old; }, 1200); }

// ---- boot ----
async function reload() {
  categories = await invoke("list_categories", { cfg });
  if (!active || !categories.find((c) => c.name === active)) {
    active = categories.length ? categories[0].name : null;
  }
  render();
}
async function boot() {
  cfg = await invoke("get_config");
  await reload();
}
boot();
