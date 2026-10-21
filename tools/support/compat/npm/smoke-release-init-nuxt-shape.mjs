const NUXT_VERSION = "4.5.1";

const APP_CLEAN = [
  "<template>",
  "  <main>{{ title }}</main>",
  "</template>",
  "",
  '<script setup lang="ts">',
  'const title: string = "vize";',
  "</script>",
  "",
].join("\n");

const APP_BROKEN = APP_CLEAN.replace(
  'const title: string = "vize";',
  'const title: number = "vize";',
);

const NUXT_CONFIG = ["export default defineNuxtConfig({", "  modules: [],", "});", ""].join("\n");

const CONFIGURED_NUXT_CONFIG = NUXT_CONFIG.replace("modules: []", 'modules: ["@vizejs/nuxt"]');

const EXPECTED_EXTENSIONS = `${JSON.stringify({ recommendations: ["ubugeeei.vize"] }, null, 2)}\n`;

function nuxtFiles(peers) {
  return {
    "package.json": `${JSON.stringify(
      {
        name: "vize-fresh-init-nuxt-vue-ts",
        version: "0.0.0",
        private: true,
        type: "module",
        scripts: {
          dev: "nuxt dev",
          build: "nuxt build",
          postinstall: "nuxt prepare",
        },
        dependencies: { vue: peers.vue },
        devDependencies: { nuxt: NUXT_VERSION, typescript: peers.typescript },
      },
      null,
      2,
    )}\n`,
    "nuxt.config.ts": NUXT_CONFIG,
    "tsconfig.json": `${JSON.stringify(
      {
        files: [],
        references: ["app", "server", "shared", "node"].map((part) => ({
          path: `./.nuxt/tsconfig.${part}.json`,
        })),
      },
      null,
      2,
    )}\n`,
    "app/app.vue": APP_CLEAN,
  };
}

function detection(manager) {
  return [
    "  framework:       Nuxt (nuxt.config.ts)",
    `  package manager: ${manager.detectedPackageManager}`,
    "  language:        TypeScript (tsconfig.json)",
    "  lint command:    oxlint",
    "  Vue settings:    defaults",
    "  oxlint config:   none",
  ];
}

function bootstrappedDetection(manager) {
  return [
    "  framework:       Nuxt (nuxt.config.ts)",
    `  package manager: ${manager.detectedPackageManager}`,
    "  language:        TypeScript (tsconfig.json)",
    "  lint command:    oxlint",
    "  Vue settings:    defaults",
    // The packed fixture materializes this unsupported name after dependency
    // bootstrap, so the CLI must surface it without treating lint as already
    // configured (#3474).
    "  oxlint config:   oxlint.config.mts — present but oxlint does not read this name (#3474)",
  ];
}

/**
 * One representative Nuxt cell joins the packed release to a real Nuxt
 * consumer. Package-manager compatibility is already covered by the Vite
 * shapes, so repeating this expensive `nuxt prepare` cell would add duration
 * without adding a distinct product contract.
 */
export const NUXT_VUE_TS_SHAPE = {
  id: "nuxt-vue-ts",
  requires: ["vize", "@vizejs/nuxt"],
  files: nuxtFiles,
  initFlags: ["--yes", "--no-lint", "--nuxt", "--fmt", "--typecheck", "--editor"],
  initialAbsentFiles: [".vscode"],
  detection,
  postBootstrapFiles: {
    "oxlint.config.mts": "export default {};\n",
  },
  bootstrappedDetection,
  features: [
    "  lint      skipped    not selected",
    "  bundler   configured adds @vizejs/nuxt to nuxt.config.ts",
    "  fmt       configured uses project settings and formatter defaults",
    "  typecheck configured uses tsconfig.json and project settings",
    "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
  ],
  reconfiguredDetection: detection,
  bootstrappedReconfiguredDetection: bootstrappedDetection,
  reconfiguredFeatures: [
    "  lint      skipped    not selected",
    "  bundler   unchanged  nuxt.config.ts already lists @vizejs/nuxt",
    "  fmt       unchanged  uses project settings and formatter defaults",
    "  typecheck unchanged  uses tsconfig.json and project settings",
    "  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize",
  ],
  createdFiles: [".vscode/extensions.json"],
  updatedFiles: ["nuxt.config.ts", "package.json"],
  addedScripts: ["vize:fmt", "vize:fmt:fix", "vize:check"],
  plannedDependencies: ["@vizejs/nuxt", "vize"],
  expectedDevDependencies: ["@vizejs/nuxt", "nuxt", "typescript", "vize"],
  expectedScripts: {
    dev: "nuxt dev",
    build: "nuxt build",
    postinstall: "nuxt prepare",
    "vize:fmt": "vize fmt --check app app.vue",
    "vize:fmt:fix": "vize fmt --write app app.vue",
    "vize:check": "nuxt prepare && vize check",
  },
  expectedFiles: {
    ".vscode/extensions.json": EXPECTED_EXTENSIONS,
    "nuxt.config.ts": CONFIGURED_NUXT_CONFIG,
  },
  check: {
    broken: { "app/app.vue": APP_BROKEN },
    brokenDiagnostics: [
      {
        file: "app/app.vue",
        diagnostics: ["error:6:7 [TS2322] Type 'string' is not assignable to type 'number'."],
      },
    ],
  },
};
