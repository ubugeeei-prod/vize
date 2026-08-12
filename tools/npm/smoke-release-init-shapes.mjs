/**
 * Matrix data for the fresh-project release smoke (#3956).
 *
 * A matrix cell is a (package manager, project shape) pair and both are data:
 * the manager table holds the argv that manager needs, the shape table holds the
 * files the project starts with, the flags `vize init` is given, the plan text it
 * must print, and the diagnostics `vize check` must report for the
 * clean/broken/repaired triple. Adding the pnpm/yarn/bun/Vite+ rows, or the
 * remaining project shapes from #3956, is a new entry here rather than new code
 * in the driver.
 */

/**
 * Supported package managers.
 *
 * `bootstrap` materialises the project's own dependencies before `init` runs, so
 * `init` sees the lockfile a real project already has -- that lockfile is what
 * `detectPackageManager` reads, and it is what decides the installer the plan
 * prints. `redirect` forces every *transitive* resolution of a packed package
 * onto its tarball; direct dependencies are redirected on the command line
 * instead, because npm rejects an override that collides with a direct spec.
 *
 * `installFlags` deliberately omits `--legacy-peer-deps`, which the umbrella
 * install in `smoke-release-install.mjs` needs: a real user installing the plan
 * does not pass it, so a genuine peer conflict in the published packages has to
 * red-light here.
 */
export const PACKAGE_MANAGERS = {
  npm: {
    id: "npm",
    binary: "npm",
    binaryEnv: "NPM_BIN",
    lockfile: "package-lock.json",
    bootstrapArgs: ["install", "--no-audit", "--fund=false", "--include=optional"],
    installArgs: ["install", "-D"],
    installFlags: ["--no-audit", "--fund=false", "--include=optional"],
    runScriptArgs: (script, extra) =>
      extra.length === 0
        ? ["run", "--silent", script]
        : ["run", "--silent", script, "--", ...extra],
    redirect: (manifest, redirects) => {
      manifest.overrides = { ...manifest.overrides, ...redirects };
    },
  },
};

/** Cells this slice runs. Further cells are added here, not in the driver. */
export const FRESH_INIT_MATRIX = [{ packageManager: "npm", shape: "vite-vue-ts" }];

const APP_TITLE_CLEAN = 'const title: string = "vize";';
const APP_TITLE_BROKEN = 'const title: number = "vize";';
const GREETING_CLEAN = "greeting.toUpperCase()";
const GREETING_BROKEN = "greeting.notAMethod()";

function appSource(broken) {
  return [
    "<template>",
    '  <HelloWorld :msg="title" />',
    "</template>",
    "",
    '<script setup lang="ts">',
    'import HelloWorld from "./components/HelloWorld.vue";',
    "",
    broken ? APP_TITLE_BROKEN : APP_TITLE_CLEAN,
    "</script>",
    "",
  ].join("\n");
}

function helloWorldSource(broken) {
  return [
    "<template>",
    `  <p class="greeting">{{ ${broken ? GREETING_BROKEN : GREETING_CLEAN} }}</p>`,
    "</template>",
    "",
    '<script setup lang="ts">',
    "const props = defineProps<{ msg: string }>();",
    "",
    "const greeting = `Hello, ${props.msg}`;",
    "</script>",
    "",
  ].join("\n");
}

function viteVueTsFiles(peers) {
  return {
    "package.json": `${JSON.stringify(
      {
        name: "vize-fresh-init-vite-vue-ts",
        version: "0.0.0",
        private: true,
        type: "module",
        scripts: { dev: "vite", build: "vite build" },
        dependencies: { vue: peers.vue },
        devDependencies: { typescript: peers.typescript, vite: peers.vite },
      },
      null,
      2,
    )}\n`,
    "index.html": '<div id="app"></div><script type="module" src="/src/main.ts"></script>\n',
    "tsconfig.json": `${JSON.stringify(
      {
        compilerOptions: {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          jsx: "preserve",
          lib: ["ES2022", "DOM", "DOM.Iterable"],
          types: [],
          strict: true,
          noEmit: true,
          skipLibCheck: true,
        },
        include: ["src/**/*.ts", "src/**/*.vue"],
      },
      null,
      2,
    )}\n`,
    "vite.config.ts": [
      'import { defineConfig } from "vite";',
      "",
      "export default defineConfig({",
      "  plugins: [],",
      "});",
      "",
    ].join("\n"),
    "src/main.ts": [
      'import { createApp } from "vue";',
      'import App from "./App.vue";',
      "",
      'createApp(App).mount("#app");',
      "",
    ].join("\n"),
    "src/App.vue": appSource(false),
    "src/components/HelloWorld.vue": helloWorldSource(false),
  };
}

/**
 * Project shapes.
 *
 * `requires` lists the packed packages the generated plan installs; a cell whose
 * packages were not packed for this run is skipped rather than red-lighted, so
 * the narrower `release-npm-native` smoke stays green.
 */
export const PROJECT_SHAPES = {
  "vite-vue-ts": {
    id: "vite-vue-ts",
    // A `create vue` TypeScript app flattened to a single tsconfig: no project
    // references, so the shape asserts one deterministic program.
    requires: ["vize", "@vizejs/vite-plugin"],
    files: viteVueTsFiles,
    // `--no-lint` because the lint plan pulls `oxlint` and `oxlint-plugin-vize`,
    // and `oxlint-plugin-vize` is not packed by every caller of this smoke.
    initFlags: ["--yes", "--no-lint", "--vite", "--fmt", "--typecheck", "--editor"],
    detection: [
      "  framework:       Vite (vite.config.ts)",
      "  package manager: npm",
      "  language:        TypeScript (tsconfig.json)",
      "  lint command:    oxlint",
      "  vize config:     none",
      "  oxlint config:   none",
    ],
    features: [
      "  lint      skipped    not selected",
      "  bundler   configured adds vize() to vite.config.ts",
      "  fmt       configured writes vize.config.ts",
      "  typecheck configured writes vize.config.ts",
      "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
    ],
    reconfiguredDetection: [
      "  framework:       Vite (vite.config.ts)",
      "  package manager: npm",
      "  language:        TypeScript (tsconfig.json)",
      "  lint command:    oxlint",
      "  vize config:     vize.config.ts",
      "  oxlint config:   none",
    ],
    reconfiguredFeatures: [
      "  lint      skipped    not selected",
      "  bundler   unchanged  vite.config.ts already uses @vizejs/vite-plugin",
      "  fmt       unchanged  vize.config.ts already exists and was left unchanged",
      "  typecheck unchanged  vize.config.ts already exists and was left unchanged",
      "  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize",
    ],
    createdFiles: ["vize.config.ts", ".vscode/extensions.json"],
    updatedFiles: ["vite.config.ts", "package.json"],
    addedScripts: ["vize:fmt", "vize:fmt:fix", "vize:check"],
    /** Exactly the dependency plan, in the order the planner emits it. */
    plannedDependencies: ["@vizejs/vite-plugin", "vize"],
    /** What the project must declare afterwards -- nothing more, nothing less. */
    expectedDevDependencies: ["@vizejs/vite-plugin", "typescript", "vite", "vize"],
    expectedScripts: {
      dev: "vite",
      build: "vite build",
      "vize:fmt": "vize fmt --check src",
      "vize:fmt:fix": "vize fmt --write src",
      "vize:check": "vize check",
    },
    expectedFiles: {
      "vize.config.ts": [
        'import { defineConfig } from "vize";',
        "",
        "export default defineConfig({",
        "  compiler: {",
        '    templateSyntax: "standard",',
        "  },",
        "  formatter: {",
        "    singleAttributePerLine: false,",
        "    sortBlocks: true,",
        "  },",
        "  typeChecker: {",
        "    enabled: true,",
        "    strict: true,",
        "    jsxTypecheck: true,",
        "  },",
        "  vite: {",
        '    scanPatterns: ["src/**/*.vue"],',
        "  },",
        "});",
        "",
      ].join("\n"),
      ".vscode/extensions.json": `${JSON.stringify({ recommendations: ["ubugeeei.vize"] }, null, 2)}\n`,
      "vite.config.ts": [
        'import { defineConfig } from "vite";',
        'import vize from "@vizejs/vite-plugin";',
        "",
        "export default defineConfig({",
        "  plugins: [vize()],",
        "});",
        "",
      ].join("\n"),
    },
    /**
     * The broken half of the clean/broken/repaired triple. The clean and
     * repaired states are `files()` itself, so a repair cannot silently drift
     * from the state the clean run already proved.
     */
    check: {
      broken: {
        "src/App.vue": appSource(true),
        "src/components/HelloWorld.vue": helloWorldSource(true),
      },
      // Authored positions taken from vue-tsc 3.3.9 over this exact project:
      //   src/App.vue(2,16): error TS2322: Type 'number' is not assignable to type 'string'.
      //   src/App.vue(8,7): error TS2322: Type 'string' is not assignable to type 'number'.
      //   src/components/HelloWorld.vue(2,35): error TS2339: Property 'notAMethod'
      //     does not exist on type 'string'.
      brokenDiagnostics: [
        {
          file: "src/App.vue",
          diagnostics: [
            "error:2:16 [TS2322] Type 'number' is not assignable to type 'string'.",
            "error:8:7 [TS2322] Type 'string' is not assignable to type 'number'.",
          ],
        },
        {
          file: "src/components/HelloWorld.vue",
          diagnostics: [
            "error:2:35 [TS2339] Property 'notAMethod' does not exist on type 'string'.",
          ],
        },
      ],
    },
  },
};
