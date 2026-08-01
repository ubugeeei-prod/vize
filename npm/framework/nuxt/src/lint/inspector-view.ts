/** Render the dependency-free UI served by the in-process lint-plan inspector. */
export function renderNuxtLintInspectorHtml(nonce: string): string {
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>Vize Nuxt Lint Plan</title>
  <style nonce="${nonce}">
    :root { color-scheme: light dark; font: 14px/1.5 ui-sans-serif, system-ui, sans-serif; }
    * { box-sizing: border-box; }
    body { margin: 0; background: #0d1117; color: #e6edf3; }
    header { position: sticky; top: 0; z-index: 1; padding: 16px 20px; background: #161b22; border-bottom: 1px solid #30363d; }
    h1 { margin: 0 0 12px; font-size: 18px; }
    h2 { margin: 24px 0 10px; font-size: 16px; }
    form { display: flex; gap: 8px; flex-wrap: wrap; }
    input { flex: 1 1 320px; min-width: 180px; padding: 8px 10px; color: inherit; background: #0d1117; border: 1px solid #484f58; border-radius: 6px; }
    button { padding: 8px 12px; color: #fff; background: #238636; border: 0; border-radius: 6px; cursor: pointer; }
    button.secondary { background: #30363d; }
    button:disabled { opacity: .6; cursor: wait; }
    main { max-width: 1100px; margin: 0 auto; padding: 0 20px 32px; }
    #status { min-height: 22px; margin-top: 8px; color: #8c959f; }
    #status.error { color: #ff7b72; }
    .card { margin: 8px 0; padding: 12px; background: #161b22; border: 1px solid #30363d; border-radius: 8px; }
    .meta { color: #8c959f; overflow-wrap: anywhere; }
    .badge { display: inline-block; margin: 2px 4px 2px 0; padding: 1px 6px; border-radius: 10px; background: #30363d; }
    .severity-error { color: #ff7b72; } .severity-warn { color: #d29922; } .severity-off { color: #8c959f; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: 7px 8px; text-align: left; border-bottom: 1px solid #30363d; overflow-wrap: anywhere; }
    th { color: #8c959f; font-weight: 600; }
    code { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }
  </style>
</head>
<body>
  <header>
    <h1>Vize Nuxt Lint Plan</h1>
    <form id="inspect-form">
      <input id="file" name="file" autocomplete="off" placeholder="app/pages/index.vue" aria-label="Project-relative file">
      <button type="submit">Inspect file</button>
      <button id="refresh" class="secondary" type="button">Refresh plan</button>
    </form>
    <div id="status" role="status" aria-live="polite"></div>
  </header>
  <main>
    <section><h2>Effective rules</h2><div id="effective"></div></section>
    <section><h2>Ordered config items</h2><div id="items"></div></section>
  </main>
  <script nonce="${nonce}">
    const fileInput = document.getElementById("file");
    const form = document.getElementById("inspect-form");
    const refresh = document.getElementById("refresh");
    const status = document.getElementById("status");
    const effective = document.getElementById("effective");
    const items = document.getElementById("items");
    const buttons = Array.from(document.querySelectorAll("button"));

    function node(tag, text, className) {
      const value = document.createElement(tag);
      if (text !== undefined) value.textContent = String(text);
      if (className) value.className = className;
      return value;
    }

    function badges(values) {
      const wrapper = node("div");
      for (const value of values || []) wrapper.append(node("span", value, "badge"));
      return wrapper;
    }

    function renderItems(payload) {
      items.replaceChildren();
      for (const item of payload.items || []) {
        const card = node("article", undefined, "card");
        card.append(node("strong", item.name || "unnamed"));
        if (item.globalIgnore) card.append(node("span", " global ignore", "meta"));
        if (item.basePath) card.append(node("div", "base: " + item.basePath, "meta"));
        if (item.files) card.append(node("div", "files", "meta"), badges(item.files));
        if (item.ignores && item.ignores.length) card.append(node("div", "ignores", "meta"), badges(item.ignores));
        const rules = Object.entries(item.rules || {});
        if (rules.length) card.append(badges(rules.map(([name, severity]) => name + ": " + severity)));
        items.append(card);
      }
    }

    function renderEffective(payload) {
      effective.replaceChildren();
      const file = payload.files && payload.files[0];
      if (!file) {
        effective.append(node("p", "Enter a project-relative file to explain its effective rules.", "meta"));
        return;
      }
      const summary = node("div", undefined, "card");
      summary.append(node("strong", file.path));
      if (file.ignored) summary.append(node("div", "Ignored by: " + (file.ignoredBy || []).join(", "), "severity-warn"));
      else summary.append(node("div", "Matched: " + (file.matchedItems || []).join(" → "), "meta"));
      effective.append(summary);
      if (!file.rules || !file.rules.length) return;
      const table = node("table");
      const head = node("tr");
      for (const label of ["Rule", "Severity", "Set by"]) head.append(node("th", label));
      const thead = node("thead"); thead.append(head); table.append(thead);
      const tbody = node("tbody");
      for (const rule of file.rules) {
        const row = node("tr");
        row.append(node("td", rule.name), node("td", rule.severity, "severity-" + rule.severity), node("td", rule.setBy));
        tbody.append(row);
      }
      table.append(tbody); effective.append(table);
    }

    async function load(fresh) {
      const url = new URL("api", location.href);
      const file = fileInput.value.trim();
      if (file) url.searchParams.set("file", file);
      if (fresh) url.searchParams.set("fresh", "1");
      buttons.forEach(button => { button.disabled = true; });
      status.className = ""; status.textContent = fresh ? "Refreshing…" : "Loading…";
      try {
        const response = await fetch(url, { headers: { accept: "application/json" } });
        const payload = await response.json();
        if (!response.ok) throw new Error(payload.error || "Inspector request failed");
        renderEffective(payload); renderItems(payload);
        status.textContent = "Resolved " + (payload.items || []).length + " config items";
      } catch (error) {
        status.className = "error";
        status.textContent = error instanceof Error ? error.message : String(error);
      } finally {
        buttons.forEach(button => { button.disabled = false; });
      }
    }

    form.addEventListener("submit", event => { event.preventDefault(); void load(false); });
    refresh.addEventListener("click", () => { void load(true); });
    void load(false);
  </script>
</body>
</html>`;
}
