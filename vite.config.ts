import type { UserConfig } from "vite-plus";
import { defineConfig } from "vite-plus";
import { floatingPromiseTestPatterns } from "./tools/vite-plus/task-inputs.ts";
import { taskCatalog } from "./tools/vite-plus/task-groups.ts";
import { rootBuildTaskPlugin } from "./tools/vite-plus/task-helpers.ts";

const localGeneratedIgnorePatterns = [".cache/**", ".direnv/**", "target/**"];

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
      "**/__agent_only/**",
      "**/__ubugeeei__/**",
      "tests/_fixtures/**",
    ],
  },
  lint: {
    ignorePatterns: [
      ...localGeneratedIgnorePatterns,
      "**/__snapshots__/**",
      "**/__snapshot__/**",
      "**/__agent_only/**",
      "**/__ubugeeei__/**",
      "npm/vscode-vize/**",
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
