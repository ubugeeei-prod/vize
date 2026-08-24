const MAIN_LABEL_CLEAN = '"root"';
const MAIN_LABEL_BROKEN = "42";

function mainJsSource(broken) {
  return [
    'import { createApp } from "vue";',
    'import App from "./App.vue";',
    "",
    "/** @type {string} */",
    `const mountLabel = ${broken ? MAIN_LABEL_BROKEN : MAIN_LABEL_CLEAN};`,
    "void mountLabel;",
    "",
    'createApp(App).mount("#app");',
    "",
  ].join("\n");
}

function appJsSource() {
  return [
    "<template>",
    '  <HelloWorld :msg="message" />',
    "</template>",
    "",
    "<script setup>",
    'import HelloWorld from "./components/HelloWorld.vue";',
    "",
    'const message = "vize";',
    "</script>",
    "",
  ].join("\n");
}

function helloWorldJsSource() {
  return [
    "<template>",
    '  <p class="greeting">{{ props.msg.toUpperCase() }}</p>',
    "</template>",
    "",
    "<script setup>",
    "const props = defineProps({",
    "  msg: { type: String, required: true },",
    "});",
    "</script>",
    "",
  ].join("\n");
}

function viteVueJsCheckJsFiles(peers) {
  return {
    "package.json": `${JSON.stringify(
      {
        name: "vize-fresh-init-vite-vue-js-checkjs",
        version: "0.0.0",
        private: true,
        type: "module",
        scripts: { dev: "vite", build: "vite build" },
        dependencies: { vue: peers.vue },
        devDependencies: { vite: peers.vite },
      },
      null,
      2,
    )}\n`,
    "index.html": '<div id="app"></div><script type="module" src="/src/main.js"></script>\n',
    "vite.config.js": [
      'import { defineConfig } from "vite";',
      "",
      "export default defineConfig({",
      "  plugins: [],",
      "});",
      "",
    ].join("\n"),
    "src/main.js": mainJsSource(false),
    "src/App.vue": appJsSource(),
    "src/components/HelloWorld.vue": helloWorldJsSource(),
  };
}

function viteJsDetection(manager) {
  return [
    "  framework:       Vite (vite.config.js)",
    `  package manager: ${manager.detectedPackageManager}`,
    "  language:        JavaScript (no tsconfig.json)",
    "  lint command:    oxlint",
    "  vize config:     none",
    "  oxlint config:   none",
  ];
}

function configuredViteJsDetection(manager) {
  return [
    "  framework:       Vite (vite.config.js)",
    `  package manager: ${manager.detectedPackageManager}`,
    "  language:        TypeScript (tsconfig.json)",
    "  lint command:    oxlint",
    "  vize config:     vize.config.ts",
    "  oxlint config:   none",
  ];
}

const EXPECTED_CHECKJS_TSCONFIG = `${JSON.stringify(
  {
    compilerOptions: {
      strict: true,
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "Bundler",
      jsx: "preserve",
      allowJs: true,
      checkJs: true,
      noEmit: true,
      skipLibCheck: true,
    },
    include: ["src/**/*"],
  },
  null,
  2,
)}\n`;

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

export const VITE_VUE_JS_CHECKJS_SHAPE = {
  id: "vite-vue-js-checkjs",
  requires: ["vize", "@vizejs/vite-plugin"],
  files: viteVueJsCheckJsFiles,
  initFlags: ["--yes", "--no-lint", "--vite", "--fmt", "--typecheck", "--editor"],
  initialAbsentFiles: [".vscode"],
  detection: viteJsDetection,
  features: [
    "  lint      skipped    not selected",
    "  bundler   configured adds vize() to vite.config.js",
    "  fmt       configured writes vize.config.ts",
    "  typecheck configured writes tsconfig.json and vize.config.ts",
    "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
  ],
  reconfiguredDetection: configuredViteJsDetection,
  reconfiguredFeatures: [
    "  lint      skipped    not selected",
    "  bundler   unchanged  vite.config.js already uses @vizejs/vite-plugin",
    "  fmt       unchanged  vize.config.ts already exists and was left unchanged",
    "  typecheck unchanged  vize.config.ts already exists and was left unchanged",
    "  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize",
  ],
  createdFiles: ["tsconfig.json", "vize.config.ts", ".vscode/extensions.json"],
  updatedFiles: ["vite.config.js", "package.json"],
  addedScripts: ["vize:fmt", "vize:fmt:fix", "vize:check"],
  plannedDependencies: ["@vizejs/vite-plugin", "vize"],
  expectedDevDependencies: ["@vizejs/vite-plugin", "vite", "vize"],
  expectedScripts: {
    dev: "vite",
    build: "vite build",
    "vize:fmt": "vize fmt --check src",
    "vize:fmt:fix": "vize fmt --write src",
    "vize:check": "vize check",
  },
  expectedFiles: {
    "tsconfig.json": EXPECTED_CHECKJS_TSCONFIG,
    "vize.config.ts": EXPECTED_VIZE_CONFIG,
    ".vscode/extensions.json": EXPECTED_EXTENSIONS,
    "vite.config.js": [
      'import { defineConfig } from "vite";',
      'import vize from "@vizejs/vite-plugin";',
      "",
      "export default defineConfig({",
      "  plugins: [vize()],",
      "});",
      "",
    ].join("\n"),
  },
  check: {
    broken: {
      "src/main.js": mainJsSource(true),
    },
    brokenDiagnostics: [
      {
        file: "src/main.js",
        diagnostics: ["error:5:7 [TS2322] Type 'number' is not assignable to type 'string'."],
      },
    ],
  },
};
