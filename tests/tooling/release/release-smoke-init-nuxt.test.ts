import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  canRunFreshProjectShape,
  freshProjectLspShape,
  writePostBootstrapFiles,
} from "../../../tools/support/compat/npm/smoke-release-init-fresh.mjs";
import { PACKAGE_MANAGERS } from "../../../tools/support/compat/npm/smoke-release-init-managers.mjs";
import { NUXT_VUE_TS_SHAPE } from "../../../tools/support/compat/npm/smoke-release-init-nuxt-shape.mjs";
import { expectedInitOutput } from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";
import { FRESH_INIT_MATRIX } from "../../../tools/support/compat/npm/smoke-release-init-shapes.mjs";

const manager = PACKAGE_MANAGERS.npm;
const shape = NUXT_VUE_TS_SHAPE;
const projectRoot = "/fresh/nuxt-vue-ts";
const detection = [
  `[vize init] detected in ${projectRoot}:`,
  "  framework:       Nuxt (nuxt.config.ts)",
  "  package manager: npm",
  "  language:        TypeScript (tsconfig.json)",
  "  lint command:    oxlint",
  "  Vue settings:    defaults",
  "  oxlint config:   oxlint.config.mts — present but oxlint does not read this name (#3474)",
];
const editors = [
  "[vize init] editor integrations shipped with Vize:",
  "  VS Code: ubugeeei.vize (recommended in .vscode/extensions.json)",
  "  Zed: editors/zed",
  "  Neovim: editors/nvim",
  "  Vim: editors/vim",
  "  Helix: editors/helix",
  "  Emacs: editors/emacs",
];

function output(lines: string[]): string {
  return [...detection, "[vize init] plan:", ...lines, ""].join("\n");
}

test("the packed release adds one bounded Nuxt fresh-consumer cell", () => {
  assert.deepEqual(
    FRESH_INIT_MATRIX.filter((cell) => cell.shape === shape.id),
    [{ packageManager: "npm", shape: "nuxt-vue-ts" }],
  );
  assert.deepEqual(shape.requires, ["vize", "@vizejs/nuxt"]);
  assert.equal(
    canRunFreshProjectShape({ packed: new Map([["vize", "/packs/vize.tgz"]]) }, shape),
    false,
    "the native-only smoke must skip the Nuxt cell",
  );
  assert.equal(
    canRunFreshProjectShape(
      {
        packed: new Map([
          ["vize", "/packs/vize.tgz"],
          ["@vizejs/nuxt", "/packs/vizejs-nuxt.tgz"],
        ]),
      },
      shape,
    ),
    true,
    "a full packed release must run the Nuxt cell",
  );
});

test("the Nuxt consumer owns a complete current project and diagnostic triple", () => {
  const files = shape.files({ typescript: "6.0.3", vue: "3.5.34" });
  assert.equal(freshProjectLspShape(shape, files), "nuxt");
  assert.deepEqual(Object.keys(files), [
    "package.json",
    "nuxt.config.ts",
    "tsconfig.json",
    "app/app.vue",
  ]);
  assert.deepEqual(JSON.parse(files["package.json"]), {
    name: "vize-fresh-init-nuxt-vue-ts",
    version: "0.0.0",
    private: true,
    type: "module",
    scripts: { dev: "nuxt dev", build: "nuxt build", postinstall: "nuxt prepare" },
    dependencies: { vue: "3.5.34" },
    devDependencies: { nuxt: "4.5.1", typescript: "6.0.3" },
  });
  assert.deepEqual(JSON.parse(files["tsconfig.json"]), {
    files: [],
    references: ["app", "server", "shared", "node"].map((part) => ({
      path: `./.nuxt/tsconfig.${part}.json`,
    })),
  });
  assert.equal(files["nuxt.config.ts"], "export default defineNuxtConfig({\n  modules: [],\n});\n");
  assert.equal(
    files["app/app.vue"],
    '<template>\n  <main>{{ title }}</main>\n</template>\n\n<script setup lang="ts">\nconst title: string = "vize";\n</script>\n',
  );
  assert.deepEqual(shape.check, {
    broken: {
      "app/app.vue":
        '<template>\n  <main>{{ title }}</main>\n</template>\n\n<script setup lang="ts">\nconst title: number = "vize";\n</script>\n',
    },
    brokenDiagnostics: [
      {
        file: "app/app.vue",
        diagnostics: ["error:6:7 [TS2322] Type 'string' is not assignable to type 'number'."],
      },
    ],
  });
  assert.deepEqual(shape.detection(manager), [
    "  framework:       Nuxt (nuxt.config.ts)",
    "  package manager: npm",
    "  language:        TypeScript (tsconfig.json)",
    "  lint command:    oxlint",
    "  Vue settings:    defaults",
    "  oxlint config:   none",
  ]);
  assert.deepEqual(shape.bootstrappedDetection(manager), detection.slice(1));
  assert.deepEqual(shape.postBootstrapFiles, {
    "oxlint.config.mts": "export default {};\n",
  });
});

test("the packed Nuxt fixture deterministically owns the unsupported oxlint config", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-nuxt-oxlint-"));
  try {
    assert.deepEqual(writePostBootstrapFiles(root, shape), shape.postBootstrapFiles);
    assert.equal(
      fs.readFileSync(path.join(root, "oxlint.config.mts"), "utf8"),
      "export default {};\n",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the Nuxt init contract pins the complete dry, apply, and idempotent plans", () => {
  const configured = [
    "  lint      skipped    not selected",
    "  bundler   configured adds @vizejs/nuxt to nuxt.config.ts",
    "  fmt       configured uses project settings and formatter defaults",
    "  typecheck configured uses tsconfig.json and project settings",
    "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
  ];
  const unchanged = [
    "  lint      skipped    not selected",
    "  bundler   unchanged  nuxt.config.ts already lists @vizejs/nuxt",
    "  fmt       unchanged  uses project settings and formatter defaults",
    "  typecheck unchanged  uses tsconfig.json and project settings",
    "  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize",
  ];
  assert.equal(
    expectedInitOutput(shape, projectRoot, manager, "dry", { bootstrapped: true }),
    output([
      ...configured,
      "[vize init] would create .vscode/extensions.json",
      "[vize init] would update nuxt.config.ts",
      "[vize init] would update package.json",
      "[vize init] would add scripts: vize:fmt, vize:fmt:fix, vize:check",
      "[vize init] would run: npm install -D @vizejs/nuxt vize",
    ]),
  );
  assert.equal(
    expectedInitOutput(shape, projectRoot, manager, "apply", { bootstrapped: true }),
    output([
      ...configured,
      "[vize init] will create .vscode/extensions.json",
      "[vize init] will update nuxt.config.ts",
      "[vize init] will update package.json",
      "[vize init] will add scripts: vize:fmt, vize:fmt:fix, vize:check",
      ...editors,
    ]),
  );
  assert.equal(
    expectedInitOutput(shape, projectRoot, manager, "rerun", { bootstrapped: true }),
    output([
      ...unchanged,
      "[vize init] nothing to do; the project is already configured",
      ...editors,
    ]),
  );
  assert.deepEqual(shape.expectedFiles, {
    ".vscode/extensions.json": '{\n  "recommendations": [\n    "ubugeeei.vize"\n  ]\n}\n',
    "nuxt.config.ts": 'export default defineNuxtConfig({\n  modules: ["@vizejs/nuxt"],\n});\n',
  });
  assert.deepEqual(shape.expectedScripts, {
    dev: "nuxt dev",
    build: "nuxt build",
    postinstall: "nuxt prepare",
    "vize:fmt": "vize fmt --check app app.vue",
    "vize:fmt:fix": "vize fmt --write app app.vue",
    "vize:check": "nuxt prepare && vize check",
  });
  assert.deepEqual(shape.expectedDevDependencies, ["@vizejs/nuxt", "nuxt", "typescript", "vize"]);
});
