import assert from "node:assert/strict";
import { test } from "node:test";

import { CONFIG_FREE_PROJECT_SHAPES } from "../../../tools/support/compat/npm/smoke-release-init-config-free.mjs";
import { PACKAGE_MANAGERS } from "../../../tools/support/compat/npm/smoke-release-init-managers.mjs";
import { expectedInitOutput } from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";

// Authored complete plans for the ordinary Vite JavaScript project. Running
// its package manager through vp does not make the project a Vite+ scaffold.
function ordinaryViteJavaScriptPlan(packageManager: "bun" | "pnpm") {
  return [
    "[vize init] detected in <fresh-project>:",
    "  framework:       Vite (vite.config.js)",
    `  package manager: ${packageManager}`,
    "  language:        JavaScript (no tsconfig.json)",
    "  lint command:    oxlint",
    "  Vue settings:    vue: in vite.config.*",
    "  oxlint config:   none",
    "[vize init] plan:",
    "  lint      skipped    not selected",
    "  bundler   configured adds vize() to vite.config.js",
    "  fmt       configured uses project settings and formatter defaults",
    "  typecheck configured writes tsconfig.json",
    "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
    "[vize init] would create tsconfig.json",
    "[vize init] would create .vscode/extensions.json",
    "[vize init] would update vite.config.js",
    "[vize init] would update package.json",
    "[vize init] would add scripts: vize:fmt, vize:fmt:fix, vize:check",
    `[vize init] would run: ${packageManager} add -D @vizejs/vite-plugin vize`,
    "",
  ].join("\n");
}

for (const [runner, installer] of [
  ["bun", "bun"],
  ["vp", "pnpm"],
] as const) {
  test(`${runner} JavaScript smoke uses the whole auto-detected ${installer} install plan`, () => {
    assert.equal(
      expectedInitOutput(
        CONFIG_FREE_PROJECT_SHAPES["vite-vue-js-checkjs"],
        "<fresh-project>",
        PACKAGE_MANAGERS[runner],
        "dry",
      ),
      ordinaryViteJavaScriptPlan(installer),
    );
  });
}
