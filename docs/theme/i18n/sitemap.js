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
    // Reached from their parent page: Experimentals links to both of these in
    // every locale, so listing them again would only lengthen the sidebar.
    /^\/guide\/experimentals-(?:reference|vue-rfcs)$/,
    // Linked from Type Checking in the two locales with an authored recipe.
    /^\/guide\/experimental-router-types$/,
  ];

  // Every published page belongs to exactly one group below or is hidden
  // above. `navigation.js` still collects anything left over into a trailing
  // "More" section, but that section is a safety net, not a place to file pages:
  // `tests/tooling/docs-navigation.test.ts` fails while it is non-empty.
  //
  // `paths` lists the pages all five locales publish. `pathsByLocale` adds the
  // pages only some locales have; each list must repeat the common paths.
  // `section` derives both from one ordered list: zh-CN, pt-BR and fr publish
  // every page except these.
  // Operational failures belong beside lint execution, outside the configurable catalogue.
  const operationalDiagnosticPaths = [
    "/rules/reference/parser-template",
    "/rules/reference/parser-sfc",
    "/rules/reference/parser-jsx",
    "/rules/reference/type-project-configuration",
  ];
  const englishAndJapaneseOnly = new Set([
    ...operationalDiagnosticPaths,
    "/guide/init",
    "/guide/vite-plus",
    "/guide/migration",
    "/guide/lint-execution",
    "/guide/formatter",
    "/guide/formatter-import-sorting",
    "/guide/formatter-indent-width",
    "/guide/formatter-line-endings",
    "/guide/formatter-property-quotes",
    "/guide/formatter-jsx-quotes",
    "/guide/type-checking",
    "/guide/type-checking-beyond-vue-tsc",
    "/guide/check-performance",
    "/guide/language-server",
    "/guide/vite-plus-editor",
    "/guide/vapor",
    "/guide/jsr",
    "/guide/vue-configuration",
    "/guide/vite-scoped-compiler",
    "/guide/experimentals",
    "/guide/musea-globals",
    "/guide/musea-capture",
    "/guide/musea-snapshots",
    "/guide/musea-hosting",
    "/guide/doctor",
  ]);
  const section = (key, paths) => {
    const common = paths.filter((path) => !englishAndJapaneseOnly.has(path));
    return common.length === paths.length
      ? { key, paths }
      : { key, paths: common, pathsByLocale: { en: paths, ja: paths } };
  };

  // One collapsible sidebar section per rule category. A rule page belongs to
  // the section that owns its ID namespace (`vue/...`, `a11y/...`); project
  // rule pages live in one directory. The rule generator reads this table too,
  // so a rule's page, its category page and its sidebar section always agree.
  sitemap.ruleSections = [
    { page: "/rules/vue", namespaces: ["vue"] },
    { page: "/rules/type-and-script", namespaces: ["script", "type"] },
    { page: "/rules/html", namespaces: ["html"] },
    { page: "/rules/accessibility", namespaces: ["a11y"] },
    { page: "/rules/ssr", namespaces: ["ssr"] },
    { page: "/rules/vapor", namespaces: ["vapor"] },
    { page: "/rules/ecosystem", namespaces: ["ecosystem", "nuxt"] },
    { page: "/rules/css", namespaces: ["css"] },
    { page: "/rules/musea", namespaces: ["musea"] },
    { page: "/rules/petite-vue", namespaces: ["petite-vue"] },
    { page: "/rules/cross-file", directory: "/rules/project/" },
  ];

  // The pages a section lists, from the routes a locale publishes. A rule's
  // page slug starts with its namespace: `vue/no-v-html` is `vue-no-v-html`.
  sitemap.sectionPaths = (section, routes) =>
    section.paths ??
    routes
      .filter((route) => !operationalDiagnosticPaths.includes(route))
      .filter((route) =>
        section.directory
          ? route.startsWith(section.directory)
          : section.namespaces.some((namespace) =>
              route.startsWith(`/rules/reference/${namespace}-`),
            ),
      )
      .sort((left, right) => left.localeCompare(right));

  // Options and the ESLint map are authored in English and Japanese only.
  const ruleOptionPaths = [
    "/rules/options-vue",
    "/rules/options-events",
    "/rules/options-attributes",
    "/rules/options-props",
    "/rules/options-script",
    "/rules/options-type",
    "/rules/options-musea",
  ];

  // `key` selects the section heading from a locale's `ui.groups`. Product
  // sections come first, then the rule reference, then the project itself.
  sitemap.navGroups = [
    section("start", [
      "/",
      "/getting-started",
      "/philosophy",
      "/guide/init",
      "/guide/vite-plus",
      "/guide/workflows",
      "/guide/migration",
      "/guide/troubleshooting",
    ]),
    section("linter", [
      "/guide/static-analysis",
      "/guide/lint-execution",
      "/guide/analysis-diagnostics",
      "/guide/comment-annotations",
      "/guide/cross-file-complexity",
      "/guide/oxlint",
      ...operationalDiagnosticPaths,
    ]),
    section("formatter", [
      "/guide/formatter",
      "/guide/formatter-import-sorting",
      "/guide/formatter-indent-width",
      "/guide/formatter-line-endings",
      "/guide/formatter-property-quotes",
      "/guide/formatter-jsx-quotes",
    ]),
    section("typeChecker", [
      "/guide/type-checking",
      "/guide/type-checking-beyond-vue-tsc",
      "/guide/content-mapper",
      "/guide/check-performance",
    ]),
    section("editor", [
      "/guide/language-server",
      "/guide/vite-plus-editor",
      "/integrations/vscode",
    ]),
    section("compiler", [
      "/guide/vite-plugin",
      "/integrations/nuxt",
      "/integrations/void",
      "/guide/unplugin",
      "/guide/jsx",
      "/guide/jsx-babel-compat",
      "/guide/vapor",
      "/guide/compiler-inspector",
      "/guide/wasm",
      "/guide/jsr",
    ]),
    section("configuration", [
      "/guide/configuration",
      "/guide/configuration-reference",
      "/guide/compiler-configuration-reference",
      "/guide/vue-configuration",
      "/guide/vite-scoped-compiler",
      "/guide/experimentals",
    ]),
    // The MCP server serves Musea's component metadata to assistants.
    section("components", [
      "/guide/musea",
      "/guide/musea-globals",
      "/guide/musea-capture",
      "/guide/musea-snapshots",
      "/guide/musea-hosting",
      "/integrations/mcp",
      "/guide/ui-styles",
      "/guide/ui",
      "/guide/composables",
      "/guide/lib-pull",
      "/guide/auto-imports",
    ]),
    section("tools", ["/guide/cli", "/guide/doctor", "/guide/testing"]),
    {
      key: "rules",
      paths: ["/rules", "/rules/all"],
      pathsByLocale: {
        en: ["/rules", "/rules/all", "/rules/migration"],
        ja: ["/rules", "/rules/all", "/rules/migration"],
      },
      // Each section collapses under its own page.
      sections: [
        { page: "/rules/options", paths: ruleOptionPaths, locales: ["en", "ja"] },
        ...sitemap.ruleSections,
      ],
    },
    section("architecture", [
      "/stability",
      "/contributing",
      "/credits",
      "/architecture/overview",
      "/architecture/crates",
      "/architecture/source-guide",
      "/architecture/language-engineering-practices",
      "/architecture/performance",
      "/architecture/performance-blacksmith",
    ]),
    {
      key: "blog",
      paths: sitemap.blogNavigationPaths,
    },
  ];
})((globalThis.__vizeDocsSitemap ??= {}));
