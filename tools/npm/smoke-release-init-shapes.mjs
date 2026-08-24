import { VITE_VUE_JS_CHECKJS_SHAPE } from "./smoke-release-init-js-shapes.mjs";

/**
 * Matrix data for the fresh-project release smoke (#3956).
 *
 * A matrix cell is a (package manager, project shape) pair and both are data:
 * the manager table in `smoke-release-init-managers.mjs` holds the argv that
 * manager needs, the shape table here holds the files the project starts with,
 * the flags `vize init` is given, the plan text it must print, and the
 * diagnostics `vize check` must report for the clean/broken/repaired triple.
 * Adding the pnpm/yarn/bun/Vite+ rows, or the remaining project shapes from
 * #3956, is a new entry here rather than new code in the driver.
 */

/** Cells this slice runs. Further cells are added here, not in the driver. */
export const FRESH_INIT_MATRIX = [
  { packageManager: "npm", shape: "vite-vue-ts" },
  { packageManager: "npm", shape: "vite-vue-js-checkjs" },
  { packageManager: "pnpm", shape: "vite-vue-ts" },
  { packageManager: "yarn", shape: "vite-vue-ts" },
  { packageManager: "bun", shape: "vite-vue-ts" },
  { packageManager: "vp", shape: "vite-plus-vue-ts" },
];

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

function vitePlusVueTsFiles(peers) {
  const files = viteVueTsFiles(peers);
  const packageJson = JSON.parse(files["package.json"]);
  packageJson.name = "vize-fresh-init-vite-plus-vue-ts";
  packageJson.scripts = { dev: "vp dev", check: "vp check" };
  packageJson.devDependencies["vite-plus"] = peers["vite-plus"];
  files["package.json"] = `${JSON.stringify(packageJson, null, 2)}\n`;
  files["vite.config.ts"] = [
    'import { defineConfig } from "vite-plus";',
    "",
    "export default defineConfig({",
    "  plugins: [],",
    "});",
    "",
  ].join("\n");
  return files;
}

function viteDetection(manager, framework = "Vite") {
  return [
    `  framework:       ${framework} (vite.config.ts)`,
    `  package manager: ${manager.detectedPackageManager}`,
    "  language:        TypeScript (tsconfig.json)",
    framework === "Vite+" ? "  lint command:    vp lint" : "  lint command:    oxlint",
    "  vize config:     none",
    "  oxlint config:   none",
  ];
}

function configuredViteDetection(manager, framework = "Vite") {
  return [
    `  framework:       ${framework} (vite.config.ts)`,
    `  package manager: ${manager.detectedPackageManager}`,
    "  language:        TypeScript (tsconfig.json)",
    framework === "Vite+" ? "  lint command:    vp lint" : "  lint command:    oxlint",
    "  vize config:     vize.config.ts",
    "  oxlint config:   none",
  ];
}

const EXPECTED_VIZE_CONFIG = [
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
].join("\n");

const EXPECTED_EXTENSIONS = `${JSON.stringify({ recommendations: ["ubugeeei.vize"] }, null, 2)}\n`;

const CHECK_TRIPLE = {
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
      diagnostics: ["error:2:35 [TS2339] Property 'notAMethod' does not exist on type 'string'."],
    },
  ],
};

/**
 * Project shapes.
 *
 * `requires` lists the packed packages the generated plan installs; a cell whose
 * packages were not packed for this run is skipped rather than red-lighted, so
 * the narrower `release-npm-native` smoke stays green.
 */
export const PROJECT_SHAPES = {
  "vite-vue-js-checkjs": VITE_VUE_JS_CHECKJS_SHAPE,
  "vite-vue-ts": {
    id: "vite-vue-ts",
    // A `create vue` TypeScript app flattened to a single tsconfig: no project
    // references, so the shape asserts one deterministic program.
    requires: ["vize", "@vizejs/vite-plugin"],
    files: viteVueTsFiles,
    // `--no-lint` because the lint plan pulls `oxlint` and `oxlint-plugin-vize`,
    // and `oxlint-plugin-vize` is not packed by every caller of this smoke.
    initFlags: ["--yes", "--no-lint", "--vite", "--fmt", "--typecheck", "--editor"],
    detection: (manager) => viteDetection(manager),
    features: [
      "  lint      skipped    not selected",
      "  bundler   configured adds vize() to vite.config.ts",
      "  fmt       configured writes vize.config.ts",
      "  typecheck configured writes vize.config.ts",
      "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
    ],
    reconfiguredDetection: (manager) => configuredViteDetection(manager),
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
      "vize.config.ts": EXPECTED_VIZE_CONFIG,
      ".vscode/extensions.json": EXPECTED_EXTENSIONS,
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
    check: CHECK_TRIPLE,
  },
  "vite-plus-vue-ts": {
    id: "vite-plus-vue-ts",
    requires: ["vize", "@vizejs/vite-plugin"],
    files: vitePlusVueTsFiles,
    initFlags: ["--yes", "--no-lint", "--vite", "--fmt", "--typecheck", "--editor"],
    detection: (manager) => viteDetection(manager, "Vite+"),
    features: [
      "  lint      skipped    not selected",
      "  bundler   configured adds vize() to vite.config.ts",
      "  fmt       configured writes vize.config.ts",
      "  typecheck configured writes vize.config.ts",
      "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
    ],
    reconfiguredDetection: (manager) => configuredViteDetection(manager, "Vite+"),
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
    plannedDependencies: ["@vizejs/vite-plugin", "vize"],
    expectedDevDependencies: ["@vizejs/vite-plugin", "typescript", "vite", "vite-plus", "vize"],
    expectedScripts: {
      dev: "vp dev",
      check: "vp check",
      "vize:fmt": "vize fmt --check src",
      "vize:fmt:fix": "vize fmt --write src",
      "vize:check": "vize check",
    },
    expectedFiles: {
      "vize.config.ts": EXPECTED_VIZE_CONFIG,
      ".vscode/extensions.json": EXPECTED_EXTENSIONS,
      "vite.config.ts": [
        'import { defineConfig } from "vite-plus";',
        'import vize from "@vizejs/vite-plugin";',
        "",
        "export default defineConfig({",
        "  plugins: [vize()],",
        "});",
        "",
      ].join("\n"),
    },
    check: CHECK_TRIPLE,
  },
};
