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
    /^\/reference\/(?:ui|composables)(?:\/|$)/,
    /^\/guide\/(?:ui|composables)\/.+/,
    /^\/guide\/(?:compiler-)?configuration-reference$/,
    /^\/rules\/(?:reference|project)\//,
    // Reached from their parent page: Experimentals links to both of these in
    // every locale, so listing them again would only lengthen the sidebar.
    /^\/guide\/experimentals-(?:reference|vue-rfcs)$/,
  ];

  // petite-vue has authored pages in English and Japanese only.
  const rulePaths = [
    "/rules",
    "/rules/all",
    "/rules/vue",
    "/rules/type-and-script",
    "/rules/html",
    "/rules/accessibility",
    "/rules/ssr",
    "/rules/vapor",
    "/rules/ecosystem",
    "/rules/musea-and-css",
    "/rules/cross-file",
  ];

  // Every published page belongs to exactly one group below or is hidden
  // above. `navigation.js` still collects anything left over into a trailing
  // "More" section, but that section is a safety net, not a place to file pages:
  // `tests/tooling/docs-navigation.test.ts` fails while it is non-empty.
  //
  // `paths` lists the pages all five locales publish. `pathsByLocale` adds the
  // pages only some locales have; each list must repeat the common paths.
  const startPaths = [
    "/",
    "/getting-started",
    "/guide/content-mapper",
    "/philosophy",
    "/guide/configuration",
  ];
  const projectSetupPaths = [
    "/guide/vite-plugin",
    "/integrations/nuxt",
    "/integrations/void",
    "/guide/workflows",
    "/guide/jsx",
    "/guide/jsx-babel-compat",
    "/guide/unplugin",
  ];
  const staticAnalysisPaths = [
    "/guide/static-analysis",
    "/guide/analysis-diagnostics",
    "/guide/cross-file-complexity",
    "/guide/oxlint",
    "/guide/comment-annotations",
    "/integrations/vscode",
  ];
  const checkSettingsPaths = [
    "/guide/static-analysis",
    "/guide/language-server",
    "/guide/analysis-diagnostics",
    "/guide/lint-execution",
    "/guide/check-performance",
    "/guide/cross-file-complexity",
    "/guide/oxlint",
    "/guide/comment-annotations",
    "/integrations/vscode",
  ];
  const formatterPaths = [
    "/guide/formatter",
    "/guide/formatter-import-sorting",
    "/guide/formatter-indent-width",
    "/guide/formatter-line-endings",
    "/guide/formatter-property-quotes",
    "/guide/formatter-jsx-quotes",
  ];
  const authoredRulePaths = [
    "/rules",
    "/rules/all",
    "/rules/options",
    "/rules/options-vue",
    "/rules/options-script",
    "/rules/options-type",
    "/rules/options-musea",
    "/rules/migration",
    ...rulePaths.slice(2),
    "/rules/petite-vue",
  ];
  const ruleGuidePaths = [
    "/rules/vue-sfc",
    "/rules/vue-components",
    "/rules/vue-directives",
    "/rules/vue-directive-validity",
    "/rules/vue-template-safety",
    "/rules/vue-template-structure",
    "/rules/vue-formatting",
    "/rules/accessibility-core",
    "/rules/accessibility-integrity",
    "/rules/accessibility-interactions",
    "/rules/accessibility-structure",
  ];
  const componentPaths = [
    "/guide/musea",
    "/guide/musea-globals",
    "/guide/musea-capture",
    "/guide/musea-snapshots",
    "/guide/musea-hosting",
    "/guide/ui-styles",
    "/guide/ui",
    "/guide/composables",
    "/guide/lib-pull",
    "/guide/auto-imports",
  ];
  const toolPaths = [
    "/guide/cli",
    "/guide/doctor",
    "/guide/compiler-inspector",
    "/integrations/mcp",
    "/guide/wasm",
    "/guide/troubleshooting",
    "/guide/testing",
  ];

  // `key` selects the section heading from a locale's `ui.groups`.
  sitemap.navGroups = [
    {
      key: "start",
      paths: startPaths,
      pathsByLocale: {
        en: [
          "/",
          "/getting-started",
          "/guide/content-mapper",
          "/guide/type-checking",
          "/philosophy",
          "/guide/vite-plus",
          "/guide/vite-plus-editor",
          "/guide/init",
          "/guide/migration",
          "/guide/configuration",
        ],
        ja: [
          "/",
          "/getting-started",
          "/guide/content-mapper",
          "/guide/type-checking",
          "/philosophy",
          "/guide/vite-plus",
          "/guide/vite-plus-editor",
          "/guide/init",
          "/guide/migration",
          "/guide/configuration",
        ],
      },
    },
    {
      key: "projectSetup",
      paths: projectSetupPaths,
      pathsByLocale: {
        en: [
          ...projectSetupPaths,
          "/guide/vapor",
          "/guide/jsr",
          "/guide/vue-configuration",
          "/guide/vite-scoped-compiler",
          "/guide/experimentals",
        ],
        ja: [
          ...projectSetupPaths,
          "/guide/vapor",
          "/guide/jsr",
          "/guide/vue-configuration",
          "/guide/vite-scoped-compiler",
          "/guide/experimentals",
        ],
      },
    },
    {
      key: "staticAnalysis",
      paths: staticAnalysisPaths,
      pathsByLocale: { en: checkSettingsPaths, ja: checkSettingsPaths },
    },
    {
      key: "formatting",
      paths: [],
      pathsByLocale: { en: formatterPaths, ja: formatterPaths },
    },
    {
      key: "rules",
      paths: rulePaths,
      pathsByLocale: {
        en: authoredRulePaths,
        ja: authoredRulePaths,
        "zh-CN": [...rulePaths, "/rules/petite-vue"],
        "pt-BR": [...rulePaths, "/rules/petite-vue"],
        fr: [...rulePaths, "/rules/petite-vue"],
      },
    },
    {
      key: "ruleGuides",
      paths: [],
      pathsByLocale: { en: ruleGuidePaths, ja: ruleGuidePaths },
    },
    {
      key: "tooling",
      paths: [
        "/guide/musea",
        "/guide/ui-styles",
        "/guide/ui",
        "/guide/composables",
        "/guide/lib-pull",
        "/guide/auto-imports",
      ],
      pathsByLocale: { en: componentPaths, ja: componentPaths },
    },
    {
      key: "tools",
      paths: toolPaths.filter((path) => path !== "/guide/doctor"),
      pathsByLocale: { en: toolPaths, ja: toolPaths },
    },
    {
      key: "architecture",
      paths: [
        "/stability",
        "/credits",
        "/contributing",
        "/architecture/overview",
        "/architecture/crates",
        "/architecture/source-guide",
        "/architecture/language-engineering-practices",
        "/architecture/performance",
        "/architecture/performance-blacksmith",
      ],
    },
    {
      key: "blog",
      paths: sitemap.blogNavigationPaths,
    },
  ];
})((globalThis.__vizeDocsSitemap ??= {}));
