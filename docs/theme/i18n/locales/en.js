/**
 * English sidebar labels and chrome strings.
 *
 * English is also the fallback for every other locale: `navigation.js` falls
 * back to a label here whenever a locale has no entry of its own, which is why
 * this file — and only this file — carries the blog post titles.
 */
((locales) => {
  locales.en = {
    labels: {
      "/": "Overview",
      "/getting-started": "Getting Started",
      "/stability": "Stability",
      "/credits": "Credits",
      "/guide/configuration": "Configuration",
      "/guide/workflows": "User Workflows",
      "/guide/jsx": "JSX & TSX",
      "/guide/jsx-babel-compat": "Babel JSX Compat",
      "/guide/troubleshooting": "Troubleshooting",
      "/guide/cli": "CLI",
      "/guide/vite-plugin": "Vite Plugin",
      "/guide/unplugin": "Bundler Integrations",
      "/guide/wasm": "WASM Bindings",
      "/guide/static-analysis": "Static Analysis",
      "/guide/cross-file-complexity": "Cross-file Complexity",
      "/guide/analysis-diagnostics": "Diagnostics",
      "/guide/oxlint": "Oxlint Plugin",
      "/guide/comment-annotations": "Comment Annotations",
      "/rules": "Rules Overview",
      "/rules/all": "All Rules",
      "/rules/vue": "Vue",
      "/rules/type-and-script": "Type & Script",
      "/rules/html": "HTML",
      "/rules/accessibility": "Accessibility",
      "/rules/ssr": "SSR",
      "/rules/vapor": "Vapor",
      "/rules/musea-and-css": "Musea & CSS",
      "/rules/cross-file": "Cross-file Rules",
      "/guide/musea": "Musea",
      "/integrations/nuxt": "Nuxt",
      "/integrations/vscode": "VS Code",
      "/integrations/mcp": "MCP Server",
      "/architecture/overview": "Architecture Overview",
      "/architecture/crates": "Crates",
      "/architecture/source-guide": "Source Guide",
      "/architecture/language-engineering-practices": "Language Engineering",
      "/architecture/performance": "Performance",
      "/philosophy": "Philosophy",
      "/blog": "Overview",
      "/blog/notes": "Notes",
      "/blog/releases": "Releases",
      "/blog/notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint":
        "Tooling Compare",
      "/blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain": "Performance Tuning",
      "/blog/notes/2026-05-16-testing-agentic-coding-and-trust": "Testing & Agents",
      "/blog/notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface": "Vapor Mode",
      "/blog/notes/2026-05-16-vue-as-a-language-and-the-strongest-frontend-environment":
        "Vue as Language",
      "/blog/notes/2026-05-16-why-musea-and-design-systems-matter-in-the-ai-era": "Musea & AI",
      "/blog/notes/2026-05-16-real-world-feedback-and-the-road-to-production-ready":
        "Production Ready",
      "/blog/notes/2026-05-16-personal-tooling-and-development-speed": "Personal Speed",
      "/blog/notes/2026-03-26-the-advantages-and-beauty-of-toolchains-and-vertical-integration":
        "Vertical Toolchains",
      "/blog/notes/2026-03-26-why-ai-needs-deterministic-fast-static-analysis": "Static Analysis",
      "/blog/notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape": "Vue Tooling",
      "/blog/notes/2026-03-26-why-vize-needs-notes": "Notes Lane",
      "/blog/releases/2026-03-26-oxlint-plugin-vize-alpha": "Oxlint Alpha",
      "/blog/releases/2026-03-26-docs-blog-support": "Docs Blog",
    },
    ui: {
      groups: {
        start: "Start",
        projectSetup: "Project Setup",
        staticAnalysis: "Static Analysis",
        rules: "Rules",
        tooling: "Tooling",
        architecture: "Architecture",
        blog: "Blog",
      },
      language: "Language",
      more: "More",
      search: "Search",
      searchPlaceholder: "Search documentation...",
      footer: 'Released under the <a href="https://opensource.org/licenses/MIT">MIT License</a>.',
    },
  };
})((globalThis.__vizeDocsLocales ??= {}));
