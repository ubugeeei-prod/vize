const vizeDocsI18nNavigation = (() => {
  const supportedLocales = [
    { code: "en", name: "English" },
    { code: "ja", name: "日本語" },
    { code: "zh-CN", name: "简体中文" },
    { code: "pt-BR", name: "Português" },
    { code: "fr", name: "Français" },
  ];
  const localeCodes = new Set(supportedLocales.map(({ code }) => code));

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
    ["/blog/notes/2026-05-16-personal-tooling-and-development-speed", "Personal Speed"],
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
    ["/credits", "Credits"],
    ["/guide/configuration", "Configuration"],
    ["/guide/workflows", "User Workflows"],
    ["/guide/jsx", "JSX & TSX"],
    ["/guide/troubleshooting", "Troubleshooting"],
    ["/guide/cli", "CLI"],
    ["/guide/vite-plugin", "Vite Plugin"],
    ["/guide/unplugin", "Bundler Integrations"],
    ["/guide/wasm", "WASM Bindings"],
    ["/guide/static-analysis", "Static Analysis"],
    ["/guide/cross-file-complexity", "Cross-file Complexity"],
    ["/guide/analysis-diagnostics", "Diagnostics"],
    ["/guide/oxlint", "Oxlint Plugin"],
    ["/guide/comment-annotations", "Comment Annotations"],
    ["/rules", "Rules Overview"],
    ["/rules/all", "All Rules"],
    ["/rules/vue", "Vue"],
    ["/rules/type-and-script", "Type & Script"],
    ["/rules/html", "HTML"],
    ["/rules/accessibility", "Accessibility"],
    ["/rules/ssr", "SSR"],
    ["/rules/vapor", "Vapor"],
    ["/rules/musea-and-css", "Musea & CSS"],
    ["/rules/cross-file", "Cross-file Rules"],
    ["/guide/musea", "Musea"],
    ["/integrations/nuxt", "Nuxt"],
    ["/integrations/vscode", "VS Code"],
    ["/integrations/mcp", "MCP Server"],
    ["/architecture/overview", "Architecture Overview"],
    ["/architecture/crates", "Crates"],
    ["/architecture/source-guide", "Source Guide"],
    ["/architecture/language-engineering-practices", "Language Engineering"],
    ["/architecture/performance", "Performance"],
    ["/philosophy", "Philosophy"],
    ...blogNavigationItems,
    ...blogPostLabels,
  ]);

  const localizedLabels = {
    ja: {
      "/": "概要",
      "/getting-started": "はじめに",
      "/stability": "安定性",
      "/credits": "クレジット",
      "/guide/configuration": "設定",
      "/guide/workflows": "ユーザーワークフロー",
      "/guide/jsx": "JSX & TSX",
      "/guide/troubleshooting": "トラブルシューティング",
      "/guide/cli": "CLI",
      "/guide/vite-plugin": "Vite プラグイン",
      "/guide/unplugin": "バンドラー統合",
      "/guide/wasm": "WASM バインディング",
      "/guide/static-analysis": "静的解析",
      "/guide/cross-file-complexity": "ファイル横断の複雑性",
      "/guide/analysis-diagnostics": "診断",
      "/guide/oxlint": "Oxlint プラグイン",
      "/guide/comment-annotations": "コメント注釈",
      "/rules": "ルール概要",
      "/rules/all": "すべてのルール",
      "/rules/vue": "Vue",
      "/rules/type-and-script": "型とスクリプト",
      "/rules/html": "HTML",
      "/rules/accessibility": "アクセシビリティ",
      "/rules/ssr": "SSR",
      "/rules/vapor": "Vapor",
      "/rules/musea-and-css": "Musea と CSS",
      "/rules/cross-file": "ファイル横断ルール",
      "/guide/musea": "Musea",
      "/integrations/nuxt": "Nuxt",
      "/integrations/vscode": "VS Code",
      "/integrations/mcp": "MCP サーバー",
      "/architecture/overview": "アーキテクチャ概要",
      "/architecture/crates": "クレート",
      "/architecture/source-guide": "ソースガイド",
      "/architecture/language-engineering-practices": "言語エンジニアリング",
      "/architecture/performance": "パフォーマンス",
      "/philosophy": "思想",
      "/blog": "概要",
      "/blog/notes": "ノート",
      "/blog/releases": "リリース",
    },
    "zh-CN": {
      "/": "概述",
      "/getting-started": "入门",
      "/stability": "稳定性",
      "/credits": "致谢",
      "/guide/configuration": "配置",
      "/guide/workflows": "用户工作流",
      "/guide/jsx": "JSX 与 TSX",
      "/guide/troubleshooting": "故障排除",
      "/guide/cli": "CLI",
      "/guide/vite-plugin": "Vite 插件",
      "/guide/unplugin": "打包器集成",
      "/guide/wasm": "WASM 绑定",
      "/guide/static-analysis": "静态分析",
      "/guide/cross-file-complexity": "跨文件复杂度",
      "/guide/analysis-diagnostics": "诊断",
      "/guide/oxlint": "Oxlint 插件",
      "/guide/comment-annotations": "注释标注",
      "/rules": "规则概述",
      "/rules/all": "全部规则",
      "/rules/vue": "Vue",
      "/rules/type-and-script": "类型与脚本",
      "/rules/html": "HTML",
      "/rules/accessibility": "无障碍",
      "/rules/ssr": "SSR",
      "/rules/vapor": "Vapor",
      "/rules/musea-and-css": "Musea 与 CSS",
      "/rules/cross-file": "跨文件规则",
      "/guide/musea": "Musea",
      "/integrations/nuxt": "Nuxt",
      "/integrations/vscode": "VS Code",
      "/integrations/mcp": "MCP 服务器",
      "/architecture/overview": "架构概述",
      "/architecture/crates": "Crate",
      "/architecture/source-guide": "源码指南",
      "/architecture/language-engineering-practices": "语言工程",
      "/architecture/performance": "性能",
      "/philosophy": "理念",
      "/blog": "概述",
      "/blog/notes": "笔记",
      "/blog/releases": "发布",
    },
    "pt-BR": {
      "/": "Visão geral",
      "/getting-started": "Primeiros passos",
      "/stability": "Estabilidade",
      "/credits": "Créditos",
      "/guide/configuration": "Configuração",
      "/guide/workflows": "Fluxos de trabalho",
      "/guide/jsx": "JSX e TSX",
      "/guide/troubleshooting": "Solução de problemas",
      "/guide/cli": "CLI",
      "/guide/vite-plugin": "Plugin do Vite",
      "/guide/unplugin": "Integrações com bundlers",
      "/guide/wasm": "Bindings WASM",
      "/guide/static-analysis": "Análise estática",
      "/guide/cross-file-complexity": "Complexidade entre arquivos",
      "/guide/analysis-diagnostics": "Diagnósticos",
      "/guide/oxlint": "Plugin do Oxlint",
      "/guide/comment-annotations": "Anotações em comentários",
      "/rules": "Visão geral das regras",
      "/rules/all": "Todas as regras",
      "/rules/vue": "Vue",
      "/rules/type-and-script": "Tipos e scripts",
      "/rules/html": "HTML",
      "/rules/accessibility": "Acessibilidade",
      "/rules/ssr": "SSR",
      "/rules/vapor": "Vapor",
      "/rules/musea-and-css": "Musea e CSS",
      "/rules/cross-file": "Regras entre arquivos",
      "/guide/musea": "Musea",
      "/integrations/nuxt": "Nuxt",
      "/integrations/vscode": "VS Code",
      "/integrations/mcp": "Servidor MCP",
      "/architecture/overview": "Visão geral da arquitetura",
      "/architecture/crates": "Crates",
      "/architecture/source-guide": "Guia do código-fonte",
      "/architecture/language-engineering-practices": "Engenharia de linguagens",
      "/architecture/performance": "Desempenho",
      "/philosophy": "Filosofia",
      "/blog": "Visão geral",
      "/blog/notes": "Notas",
      "/blog/releases": "Versões",
    },
    fr: {
      "/": "Vue d’ensemble",
      "/getting-started": "Bien démarrer",
      "/stability": "Stabilité",
      "/credits": "Crédits",
      "/guide/configuration": "Configuration",
      "/guide/workflows": "Flux de travail",
      "/guide/jsx": "JSX et TSX",
      "/guide/troubleshooting": "Dépannage",
      "/guide/cli": "CLI",
      "/guide/vite-plugin": "Plugin Vite",
      "/guide/unplugin": "Intégrations des bundlers",
      "/guide/wasm": "Bindings WASM",
      "/guide/static-analysis": "Analyse statique",
      "/guide/cross-file-complexity": "Complexité inter-fichiers",
      "/guide/analysis-diagnostics": "Diagnostics",
      "/guide/oxlint": "Plugin Oxlint",
      "/guide/comment-annotations": "Annotations de commentaires",
      "/rules": "Vue d’ensemble des règles",
      "/rules/all": "Toutes les règles",
      "/rules/vue": "Vue",
      "/rules/type-and-script": "Types et scripts",
      "/rules/html": "HTML",
      "/rules/accessibility": "Accessibilité",
      "/rules/ssr": "SSR",
      "/rules/vapor": "Vapor",
      "/rules/musea-and-css": "Musea et CSS",
      "/rules/cross-file": "Règles inter-fichiers",
      "/guide/musea": "Musea",
      "/integrations/nuxt": "Nuxt",
      "/integrations/vscode": "VS Code",
      "/integrations/mcp": "Serveur MCP",
      "/architecture/overview": "Vue d’ensemble de l’architecture",
      "/architecture/crates": "Crates",
      "/architecture/source-guide": "Guide du code source",
      "/architecture/language-engineering-practices": "Ingénierie des langages",
      "/architecture/performance": "Performances",
      "/philosophy": "Philosophie",
      "/blog": "Vue d’ensemble",
      "/blog/notes": "Notes",
      "/blog/releases": "Versions",
    },
  };

  const localizedUi = {
    en: {
      groups: [
        "Start",
        "Project Setup",
        "Static Analysis",
        "Rules",
        "Tooling",
        "Architecture",
        "Blog",
      ],
      language: "Language",
      more: "More",
      search: "Search",
      searchPlaceholder: "Search documentation...",
      footer: 'Released under the <a href="https://opensource.org/licenses/MIT">MIT License</a>.',
    },
    ja: {
      groups: [
        "スタート",
        "プロジェクト設定",
        "静的解析",
        "ルール",
        "ツール",
        "アーキテクチャ",
        "ブログ",
      ],
      language: "言語",
      more: "その他",
      search: "検索",
      searchPlaceholder: "ドキュメントを検索...",
      footer:
        '<a href="https://opensource.org/licenses/MIT">MIT License</a> の下で公開されています。',
    },
    "zh-CN": {
      groups: ["开始", "项目设置", "静态分析", "规则", "工具", "架构", "博客"],
      language: "语言",
      more: "更多",
      search: "搜索",
      searchPlaceholder: "搜索文档...",
      footer: '根据 <a href="https://opensource.org/licenses/MIT">MIT License</a> 发布。',
    },
    "pt-BR": {
      groups: [
        "Início",
        "Configuração do projeto",
        "Análise estática",
        "Regras",
        "Ferramentas",
        "Arquitetura",
        "Blog",
      ],
      language: "Idioma",
      more: "Mais",
      search: "Pesquisar",
      searchPlaceholder: "Pesquisar na documentação...",
      footer: 'Publicado sob a <a href="https://opensource.org/licenses/MIT">Licença MIT</a>.',
    },
    fr: {
      groups: [
        "Accueil",
        "Configuration du projet",
        "Analyse statique",
        "Règles",
        "Outils",
        "Architecture",
        "Blog",
      ],
      language: "Langue",
      more: "Plus",
      search: "Rechercher",
      searchPlaceholder: "Rechercher dans la documentation...",
      footer: 'Publié sous <a href="https://opensource.org/licenses/MIT">licence MIT</a>.',
    },
  };

  const hiddenPathPatterns = [
    /^\/blog\/notes\/\d{4}-\d{2}-\d{2}-/,
    /^\/blog\/releases\/\d{4}-\d{2}-\d{2}-/,
  ];

  const navGroups = [
    {
      title: "Start",
      paths: ["/", "/getting-started", "/stability", "/credits"],
    },
    {
      title: "Project Setup",
      paths: [
        "/guide/vite-plugin",
        "/integrations/nuxt",
        "/guide/workflows",
        "/guide/configuration",
        "/guide/jsx",
        "/guide/troubleshooting",
        "/guide/unplugin",
      ],
    },
    {
      title: "Static Analysis",
      paths: [
        "/guide/static-analysis",
        "/guide/cross-file-complexity",
        "/guide/analysis-diagnostics",
        "/guide/oxlint",
        "/guide/comment-annotations",
      ],
    },
    {
      title: "Rules",
      paths: [
        "/rules",
        "/rules/all",
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
        "/architecture/source-guide",
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

  function pathLocale(path) {
    const firstSegment = path.split("/").find(Boolean);
    return localeCodes.has(firstSegment) ? firstSegment : "en";
  }

  function currentLocale() {
    return pathLocale(globalThis.window?.location?.pathname ?? "/");
  }

  function canonicalPath(href) {
    const origin = globalThis.window?.location?.origin ?? "https://vizejs.dev";
    const url = new URL(href, origin);
    let path = url.pathname.replace(/\/index\.html$/, "").replace(/\/$/, "") || "/";
    const locale = pathLocale(path);
    if (locale !== "en") {
      path = path.slice(locale.length + 1) || "/";
    }
    return path;
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

    const locale = currentLocale();
    const itemsByPath = new Map();
    const unusedItems = [];
    for (const item of nav.querySelectorAll(".nav-item")) {
      const link = item.querySelector(".nav-link[href]");
      if (!link) {
        unusedItems.push(item);
        continue;
      }

      const href = link.getAttribute("href");
      const url = new URL(href, window.location.origin);
      if (pathLocale(url.pathname) !== locale) {
        continue;
      }

      const path = canonicalPath(href);
      link.textContent =
        localizedLabels[locale]?.[path] ?? labelByPath.get(path) ?? link.textContent.trim();
      if (itemsByPath.has(path)) {
        unusedItems.push(item);
        continue;
      }
      itemsByPath.set(path, item);
    }

    const nextNav = document.createDocumentFragment();
    const used = new Set();
    for (const [groupIndex, group] of navGroups.entries()) {
      const items = group.paths
        .map((path) => {
          used.add(path);
          return itemsByPath.get(path);
        })
        .filter(Boolean);

      if (items.length > 0) {
        nextNav.append(createSection(localizedUi[locale].groups[groupIndex], items));
      }
    }

    const remainingItems = [...itemsByPath]
      .filter(([path]) => !used.has(path) && !isHiddenFallbackPath(path))
      .map(([, item]) => item)
      .concat(unusedItems);
    if (remainingItems.length > 0) {
      nextNav.append(createSection(localizedUi[locale].more, remainingItems));
    }

    nav.replaceChildren(nextNav);
    nav.dataset.vizeNavigation = "structured";
  }

  function localizedPagePath(locale) {
    const logicalPath = canonicalPath(window.location.pathname);
    const localePrefix = locale === "en" ? "" : `/${locale}`;
    const pagePath = logicalPath === "/" ? "" : logicalPath;
    return `${localePrefix}${pagePath}/index.html`.replace(/^\/\//, "/");
  }

  function applyLocalizedChrome(root = document) {
    const locale = currentLocale();
    const ui = localizedUi[locale];
    const headerTitle = root.querySelector?.(".header-title");
    if (headerTitle) {
      headerTitle.setAttribute("href", locale === "en" ? "/index.html" : `/${locale}/index.html`);
    }

    const searchButton = root.querySelector?.(".search-button");
    const searchText = searchButton?.querySelector("span");
    if (searchText) searchText.textContent = ui.search;
    if (searchButton) searchButton.setAttribute("aria-label", ui.search);

    const searchInput = root.querySelector?.(".search-input");
    if (searchInput) searchInput.setAttribute("placeholder", ui.searchPlaceholder);

    const footerMessage = root.querySelector?.(".footer-message");
    if (footerMessage) footerMessage.innerHTML = ui.footer;
  }

  function installLocaleSwitcher(root = document) {
    const headerActions = root.querySelector?.(".header-actions");
    if (!headerActions || headerActions.querySelector(".docs-locale")) return;

    const locale = currentLocale();
    const wrapper = document.createElement("label");
    wrapper.className = "docs-locale";
    wrapper.setAttribute("aria-label", localizedUi[locale].language);

    const label = document.createElement("span");
    label.textContent = localizedUi[locale].language;
    const select = document.createElement("select");
    select.className = "docs-locale-select";
    select.setAttribute("aria-label", localizedUi[locale].language);
    for (const supportedLocale of supportedLocales) {
      const option = document.createElement("option");
      option.value = supportedLocale.code;
      option.textContent = supportedLocale.name;
      option.selected = supportedLocale.code === locale;
      select.append(option);
    }
    select.addEventListener("change", () => {
      window.location.href = `${localizedPagePath(select.value)}${window.location.search}${window.location.hash}`;
    });
    wrapper.append(label, select);

    const searchButton = headerActions.querySelector(".search-button");
    headerActions.insertBefore(wrapper, searchButton);
  }

  function initialize(root = document) {
    applyNavigationOrder(root);
    applyLocalizedChrome(root);
    installLocaleSwitcher(root);
  }

  return {
    applyNavigationOrder,
    canonicalPath,
    currentLocale,
    initialize,
  };
})();

if (typeof globalThis !== "undefined") {
  globalThis.__vizeDocsNavigation = vizeDocsI18nNavigation;
}

(() => {
  if (typeof document === "undefined") {
    return;
  }

  const start = () => vizeDocsI18nNavigation.initialize(document);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start, { once: true });
    return;
  }

  start();
})();
