(() => {
  "use strict";

  const DESTRUCTIVE = new Set(["delete_card"]);
  const READ_ONLY = new Set([
    "list_projects",
    "list_board_summary",
    "find_cards",
    "get_card",
    "get_card_context",
  ]);

  const root = document.getElementById("tools");
  const errorEl = document.getElementById("error");
  const filterInput = document.getElementById("filter-input");
  const countEl = document.getElementById("tool-count");
  const versionPill = document.getElementById("version-pill");

  Promise.all([
    fetch("./tools.json").then((r) => {
      if (!r.ok) throw new Error(`tools.json: ${r.status}`);
      return r.json();
    }),
    fetch("./version.txt")
      .then((r) => (r.ok ? r.text() : ""))
      .catch(() => ""),
  ])
    .then(([tools, version]) => {
      if (version) {
        versionPill.textContent = `v${version.trim()}`;
        versionPill.hidden = false;
      }
      const cards = tools.map(buildCard);
      cards.forEach((c) => root.appendChild(c));
      countEl.textContent = `${tools.length} tool${tools.length === 1 ? "" : "s"}`;
      filterInput.addEventListener("input", () => applyFilter(tools, cards));
    })
    .catch((err) => {
      errorEl.hidden = false;
      errorEl.textContent = `Could not load tools.json — ${err.message}`;
    });

  function buildCard(tool) {
    const card = document.createElement("article");
    card.className = "tool-card";
    card.dataset.name = tool.name;
    card.dataset.description = tool.description || "";

    const destructive = DESTRUCTIVE.has(tool.name);
    const readOnly = READ_ONLY.has(tool.name);
    if (destructive) card.classList.add("destructive");

    const header = document.createElement("div");
    header.className = "tool-header";

    const name = document.createElement("span");
    name.className = "tool-name";
    name.textContent = tool.name;
    header.appendChild(name);

    if (destructive) header.appendChild(badge("destructive", "destructive"));
    else if (readOnly) header.appendChild(badge("readonly", "read-only"));
    else header.appendChild(badge("mutating", "mutating"));

    if (hasProgrammaticAnnotation(tool)) {
      header.appendChild(badge("programmatic", "programmatic"));
    }

    card.appendChild(header);

    const desc = document.createElement("p");
    desc.className = "tool-description";
    desc.textContent = tool.description || "";
    card.appendChild(desc);

    const schema = tool.inputSchema || tool.input_schema || {};
    const params = paramsFromSchema(schema);

    const sectionTitle = document.createElement("div");
    sectionTitle.className = "section-title";
    sectionTitle.textContent = "Parameters";
    card.appendChild(sectionTitle);

    if (params.length === 0) {
      const none = document.createElement("p");
      none.className = "no-params";
      none.textContent = "No parameters.";
      card.appendChild(none);
    } else {
      const ul = document.createElement("ul");
      ul.className = "params";
      params.forEach((p) => ul.appendChild(renderParam(p)));
      card.appendChild(ul);
    }

    const details = document.createElement("details");
    details.className = "schema";
    const summary = document.createElement("summary");
    summary.textContent = "Raw JSON schema";
    details.appendChild(summary);
    const pre = document.createElement("pre");
    const code = document.createElement("code");
    code.textContent = JSON.stringify(schema, null, 2);
    pre.appendChild(code);
    details.appendChild(pre);
    card.appendChild(details);

    return card;
  }

  function badge(klass, label) {
    const b = document.createElement("span");
    b.className = `badge ${klass}`;
    b.textContent = label;
    return b;
  }

  function hasProgrammaticAnnotation(tool) {
    const ann = tool.annotations;
    if (!ann) return false;
    const callers = ann.allowedCallers || ann.allowed_callers;
    return Array.isArray(callers) && callers.length > 0;
  }

  function paramsFromSchema(schema) {
    const properties = schema.properties || {};
    const required = new Set(schema.required || []);
    return Object.entries(properties).map(([name, def]) => ({
      name,
      type: def.type || "any",
      required: required.has(name),
      description: def.description || "",
      defaultValue: def.default,
    }));
  }

  function renderParam(p) {
    const li = document.createElement("li");
    li.className = "param";

    const name = document.createElement("span");
    name.className = "param-name";
    name.textContent = p.name;
    li.appendChild(name);

    const meta = document.createElement("span");
    meta.className = `param-meta ${p.required ? "required" : "optional"}`;
    let metaText = `${p.type}`;
    metaText += p.required ? " · required" : " · optional";
    if (p.defaultValue !== undefined) metaText += ` · default ${JSON.stringify(p.defaultValue)}`;
    meta.textContent = metaText;
    li.appendChild(meta);

    const desc = document.createElement("span");
    desc.className = "param-desc";
    desc.textContent = p.description;
    li.appendChild(desc);

    return li;
  }

  function applyFilter(tools, cards) {
    const q = filterInput.value.trim().toLowerCase();
    let visible = 0;
    cards.forEach((card) => {
      const haystack = `${card.dataset.name} ${card.dataset.description}`.toLowerCase();
      const show = !q || haystack.includes(q);
      card.style.display = show ? "" : "none";
      if (show) visible += 1;
    });
    countEl.textContent =
      q
        ? `${visible} of ${tools.length} tools`
        : `${tools.length} tool${tools.length === 1 ? "" : "s"}`;
  }
})();
