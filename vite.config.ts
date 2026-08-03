import type { UserConfig } from "vite-plus";
import { defineConfig } from "vite-plus";
import { floatingPromiseTestPatterns } from "./tools/vite-plus/task-inputs.ts";
import { taskCatalog } from "./tools/vite-plus/task-groups.ts";
import { rootBuildTaskPlugin } from "./tools/vite-plus/task-helpers.ts";

const localGeneratedIgnorePatterns = [
  ".cache/**",
  ".direnv/**",
  "editors/vscode/.vscode-test/**",
  "npm/fresco-native/index.d.ts",
  "target/**",
];
const testOutputIgnorePattern = ["**", "target", "vize-tests", "**"].join("/");
/**
 * The VS Code / Neovim real-server scenario file carries deliberate authored
 * defects (a `vue/no-multi-spaces` warning next to a prop type error) that the
 * editor tests fix and re-read at runtime. Formatting it would erase the very
 * diagnostic the scenario asserts on.
 */
const editorScenarioFixtureIgnorePattern =
  "editors/vscode/test-fixtures/extension-host/real-vue/src/Scenario.vue";

/**
 * Root Vite+ configuration.
 *
 * The root config intentionally stays small: task helpers, package inputs, and
 * the task catalog live under `tools/vite-plus/` where they can carry richer
 * documentation and tighter type boundaries. This file should remain the place
 * that wires Vite+, repository lint/format policy, and the assembled task map
 * together.
 */
const config = {
  plugins: [rootBuildTaskPlugin()],
  build: {
    emptyOutDir: true,
    lib: {
      entry: "tests/tooling/support/vp-build-entry.ts",
      fileName: "vp-build",
      formats: ["es"],
    },
    outDir: "target/vp-build",
  },
  fmt: {
    ignorePatterns: [
      ...localGeneratedIgnorePatterns,
      "**/__snapshots__/**",
      "**/__snapshot__/**",
      testOutputIgnorePattern,
      "**/__ubugeeei__/**",
      "tests/_fixtures/**",
      editorScenarioFixtureIgnorePattern,
    ],
  },
  lint: {
    ignorePatterns: [
      ...localGeneratedIgnorePatterns,
      "**/__snapshots__/**",
      "**/__snapshot__/**",
      testOutputIgnorePattern,
      "**/__ubugeeei__/**",
      "editors/vscode/**",
      "tests/_fixtures/**",
    ],
    options: {
      typeAware: true,
    },
    overrides: [
      {
        files: floatingPromiseTestPatterns,
        rules: {
          "typescript/no-floating-promises": "off",
        },
      },
    ],
  },
  run: {
    cache: {
      scripts: true,
      tasks: true,
    },
    tasks: taskCatalog,
  },
} satisfies UserConfig;

export default defineConfig(config);
