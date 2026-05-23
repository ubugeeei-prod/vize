const vizeDocsNavigation = (() => {
  const blogNavigationItems = [
    ["/blog", "Overview"],
    ["/blog/notes", "Notes"],
    ["/blog/releases", "Releases"],
  ];

  const blogPostLabels = [
    [
      "/blog/notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint",
      "Tooling Compare",
    ],
    ["/blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain", "Performance Tuning"],
    ["/blog/notes/2026-05-16-testing-agentic-coding-and-trust", "Testing & Agents"],
    ["/blog/notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface", "Vapor Mode"],
    [
      "/blog/notes/2026-05-16-vue-as-a-language-and-the-strongest-frontend-environment",
      "Vue as Language",
    ],
    ["/blog/notes/2026-05-16-why-musea-and-design-systems-matter-in-the-ai-era", "Musea & AI"],
    [
      "/blog/notes/2026-05-16-real-world-feedback-and-the-road-to-production-ready",
      "Production Ready",
    ],
    ["/blog/notes/2026-05-16-unofficial-personal-tooling-and-development-speed", "Personal Speed"],
    [
      "/blog/notes/2026-03-26-the-advantages-and-beauty-of-toolchains-and-vertical-integration",
      "Vertical Toolchains",
    ],
    ["/blog/notes/2026-03-26-why-ai-needs-deterministic-fast-static-analysis", "Static Analysis"],
    ["/blog/notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape", "Vue Tooling"],
    ["/blog/notes/2026-03-26-why-vize-needs-notes", "Notes Lane"],
    ["/blog/releases/2026-03-26-oxlint-plugin-vize-alpha", "Oxlint Alpha"],
    ["/blog/releases/2026-03-26-docs-blog-support", "Docs Blog"],
  ];

  const labelByPath = new Map([
    ["/", "Overview"],
    ["/getting-started", "Getting Started"],
    ["/stability", "Stability"],
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
    ["/architecture/language-engineering-practices", "Language Engineering"],
    ["/architecture/performance", "Performance"],
    ["/philosophy", "Philosophy"],
    ...blogNavigationItems,
    ...blogPostLabels,
  ]);

  const hiddenPathPatterns = [
    /^\/blog\/notes\/\d{4}-\d{2}-\d{2}-/,
    /^\/blog\/releases\/\d{4}-\d{2}-\d{2}-/,
  ];

  const navGroups = [
    {
      title: "Start",
      paths: ["/", "/getting-started", "/stability"],
    },
    {
      title: "Project Setup",
      paths: [
        "/guide/vite-plugin",
        "/integrations/nuxt",
        "/guide/configuration",
        "/guide/unplugin",
      ],
    },
    {
      title: "Static Analysis",
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
      title: "Tooling",
      paths: [
        "/guide/musea",
        "/integrations/vscode",
        "/integrations/mcp",
        "/guide/wasm",
        "/guide/cli",
      ],
    },
    {
      title: "Architecture",
      paths: [
        "/architecture/overview",
        "/architecture/crates",
        "/architecture/language-engineering-practices",
        "/architecture/performance",
        "/philosophy",
      ],
    },
    {
      title: "Blog",
      paths: blogNavigationItems.map(([path]) => path),
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

  function isHiddenFallbackPath(path) {
    return hiddenPathPatterns.some((pattern) => pattern.test(path));
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
      .filter(([path]) => !used.has(path) && !isHiddenFallbackPath(path))
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
