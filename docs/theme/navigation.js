const vizeDocsNavigation = (() => {
  const labelByPath = new Map([
    ["/", "Overview"],
    ["/getting-started", "Getting Started"],
    ["/guide/configuration", "Configuration"],
    ["/guide/cli", "CLI"],
    ["/guide/vite-plugin", "Vite Plugin"],
    ["/guide/unplugin", "Bundler Integrations"],
    ["/guide/wasm", "WASM Bindings"],
    ["/guide/static-analysis", "Static Analysis"],
    ["/guide/analysis-diagnostics", "Diagnostics"],
    ["/guide/oxlint", "Oxlint Plugin"],
    ["/guide/comment-annotations", "Comment Annotations"],
    ["/rules", "Rules Overview"],
    ["/rules/vue", "Vue"],
    ["/rules/type-and-script", "Type & Script"],
    ["/rules/html", "HTML"],
    ["/rules/accessibility", "Accessibility"],
    ["/rules/ssr", "SSR"],
    ["/rules/vapor", "Vapor"],
    ["/rules/musea-and-css", "Musea & CSS"],
    ["/rules/cross-file", "Cross-file Analyzer"],
    ["/guide/musea", "Musea"],
    ["/integrations/nuxt", "Nuxt"],
    ["/integrations/vscode", "VS Code"],
    ["/integrations/mcp", "MCP Server"],
    ["/architecture/overview", "Architecture Overview"],
    ["/architecture/crates", "Crates"],
    ["/architecture/performance", "Performance"],
    ["/philosophy", "Philosophy"],
  ]);

  const navGroups = [
    {
      title: "Start",
      paths: ["/", "/getting-started"],
    },
    {
      title: "Use",
      paths: [
        "/guide/configuration",
        "/guide/cli",
        "/guide/vite-plugin",
        "/guide/unplugin",
        "/guide/wasm",
      ],
    },
    {
      title: "Analyze",
      paths: [
        "/guide/static-analysis",
        "/guide/analysis-diagnostics",
        "/guide/oxlint",
        "/guide/comment-annotations",
      ],
    },
    {
      title: "Rules",
      paths: [
        "/rules",
        "/rules/vue",
        "/rules/type-and-script",
        "/rules/html",
        "/rules/accessibility",
        "/rules/ssr",
        "/rules/vapor",
        "/rules/musea-and-css",
        "/rules/cross-file",
      ],
    },
    {
      title: "Integrations",
      paths: ["/guide/musea", "/integrations/nuxt", "/integrations/vscode", "/integrations/mcp"],
    },
    {
      title: "Architecture",
      paths: [
        "/architecture/overview",
        "/architecture/crates",
        "/architecture/performance",
        "/philosophy",
      ],
    },
  ];

  function canonicalPath(href) {
    const url = new URL(href, window.location.origin);
    const path = url.pathname.replace(/\/index\.html$/, "").replace(/\/$/, "");
    return path || "/";
  }

  function createSection(title, items) {
    const section = document.createElement("div");
    section.className = "nav-section";

    const heading = document.createElement("div");
    heading.className = "nav-title";
    heading.textContent = title;
    section.append(heading);

    const list = document.createElement("ul");
    list.className = "nav-list";
    for (const item of items) {
      list.append(item);
    }
    section.append(list);

    return section;
  }

  function applyNavigationOrder(root = document) {
    const nav = root.querySelector?.(".sidebar nav");
    if (!nav || nav.dataset.vizeNavigation === "structured") {
      return;
    }

    const itemsByPath = new Map();
    const unusedItems = [];
    for (const item of nav.querySelectorAll(".nav-item")) {
      const link = item.querySelector(".nav-link[href]");
      if (!link) {
        unusedItems.push(item);
        continue;
      }

      const path = canonicalPath(link.getAttribute("href"));
      link.textContent = labelByPath.get(path) ?? link.textContent.trim();
      if (itemsByPath.has(path)) {
        unusedItems.push(item);
        continue;
      }
      itemsByPath.set(path, item);
    }

    const nextNav = document.createDocumentFragment();
    const used = new Set();
    for (const group of navGroups) {
      const items = group.paths
        .map((path) => {
          used.add(path);
          return itemsByPath.get(path);
        })
        .filter(Boolean);

      if (items.length > 0) {
        nextNav.append(createSection(group.title, items));
      }
    }

    const remainingItems = [...itemsByPath]
      .filter(([path]) => !used.has(path))
      .map(([, item]) => item)
      .concat(unusedItems);
    if (remainingItems.length > 0) {
      nextNav.append(createSection("More", remainingItems));
    }

    nav.replaceChildren(nextNav);
    nav.dataset.vizeNavigation = "structured";
  }

  return {
    applyNavigationOrder,
    canonicalPath,
  };
})();

if (typeof globalThis !== "undefined") {
  globalThis.__vizeDocsNavigation = vizeDocsNavigation;
}

(() => {
  if (typeof document === "undefined") {
    return;
  }

  const start = () => vizeDocsNavigation.applyNavigationOrder(document);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
    return;
  }

  start();
})();
