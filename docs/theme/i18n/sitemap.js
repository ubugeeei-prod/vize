/**
 * Sidebar structure shared by every locale: which pages exist, how they are
 * grouped, and which locales the site ships.
 *
 * Strings live in `locales/<code>.js`, one file per locale, keyed by the same
 * paths and the same group keys used here. Nothing on the docs site loads
 * modules, so each theme file is a plain script that registers itself on a
 * global; `theme/background.ts` concatenates them in order and inlines the
 * result, with `i18n/navigation.js` last.
 */
((sitemap) => {
  // Order matters: it is the order of the header language picker. Keep this in
  // sync with `i18n.locales` in `docs/vite.config.ts`.
  sitemap.supportedLocales = [
    { code: "en", name: "English" },
    { code: "ja", name: "日本語" },
    { code: "zh-CN", name: "简体中文" },
    { code: "pt-BR", name: "Português" },
    { code: "fr", name: "Français" },
  ];

  sitemap.blogNavigationPaths = ["/blog", "/blog/notes", "/blog/releases"];

  // Dated posts are reachable from the blog index; keeping them out of the
  // "More" fallback stops the sidebar from growing with every post.
  sitemap.hiddenPathPatterns = [
    /^\/blog\/notes\/\d{4}-\d{2}-\d{2}-/,
    /^\/blog\/releases\/\d{4}-\d{2}-\d{2}-/,
  ];

  // `key` selects the section heading from a locale's `ui.groups`.
  sitemap.navGroups = [
    {
      key: "start",
      paths: ["/", "/getting-started", "/stability", "/credits"],
    },
    {
      key: "projectSetup",
      paths: [
        "/guide/vite-plugin",
        "/integrations/nuxt",
        "/guide/workflows",
        "/guide/configuration",
        "/guide/jsx",
        "/guide/jsx-babel-compat",
        "/guide/troubleshooting",
        "/guide/unplugin",
      ],
    },
    {
      key: "staticAnalysis",
      paths: [
        "/guide/static-analysis",
        "/guide/cross-file-complexity",
        "/guide/analysis-diagnostics",
        "/guide/oxlint",
        "/guide/comment-annotations",
      ],
    },
    {
      key: "rules",
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
      key: "tooling",
      paths: [
        "/guide/musea",
        "/integrations/vscode",
        "/integrations/mcp",
        "/guide/wasm",
        "/guide/cli",
        "/guide/content-mapper",
      ],
    },
    {
      key: "architecture",
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
      key: "blog",
      paths: sitemap.blogNavigationPaths,
    },
  ];
})((globalThis.__vizeDocsSitemap ??= {}));
