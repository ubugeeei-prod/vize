import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  ALL_FEATURES,
  exists,
  read,
  runInit,
  temporaryProject,
  write,
  writeManifest,
} from "./init-support.ts";

test("a Nuxt project is configured through the Nuxt module, not the Vite plugin", async () => {
  const root = temporaryProject("nuxt");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "nuxt dev" },
    devDependencies: { nuxt: "^4.0.0" },
  });
  write(
    root,
    "nuxt.config.ts",
    `export default defineNuxtConfig({
  modules: ["@nuxt/image"],
  devtools: { enabled: true },
});
`,
  );
  write(root, "tsconfig.json", "{}\n");
  write(root, "yarn.lock", "# yarn lockfile v1\n");

  const result = await runInit(root, ALL_FEATURES);

  assert.deepEqual(result.commands, [
    {
      command: "yarn",
      args: ["add", "-D", "@vizejs/nuxt", "oxlint", "oxlint-plugin-vize", "vize"],
      cwd: root,
    },
  ]);
  assert.deepEqual(result.written, [
    "oxlint.config.ts",
    "nuxt.config.ts",
    "vize.config.ts",
    ".vscode/extensions.json",
    "package.json",
  ]);
  assert.equal(
    read(root, "nuxt.config.ts"),
    `export default defineNuxtConfig({
  modules: ["@vizejs/nuxt", "@nuxt/image"],
  devtools: { enabled: true },
});
`,
  );
  // Nuxt owns its Vite instance, so no Vite config is created for it.
  assert.equal(exists(root, "vite.config.ts"), false);
  assert.equal(result.plan?.lintTarget.kind, "oxlint");
});

test("Nuxt wins when both configs exist, and --vite overrides that", async () => {
  const root = temporaryProject("bundler-override");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "nuxt dev" },
    devDependencies: { nuxt: "^4.0.0" },
  });
  write(
    root,
    "vite.config.ts",
    `import { defineConfig } from "vite";

export default defineConfig({
  plugins: [],
});
`,
  );
  write(root, "nuxt.config.ts", "export default defineNuxtConfig({});\n");
  write(root, "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");

  const bundlerOnly = ["--yes", "--no-lint", "--no-fmt", "--no-typecheck", "--no-editor"] as const;

  // Nuxt owns the Vite instance, so it wins over the Vite config that is also
  // present.
  const detected = await runInit(root, [...bundlerOnly, "--bundler", "--dry-run"]);
  assert.deepEqual(detected.plan?.commands, [
    { command: "pnpm", args: ["add", "-D", "@vizejs/nuxt"], cwd: root },
  ]);
  assert.deepEqual(detected.plan?.updatedFiles, ["nuxt.config.ts"]);
  assert.equal(detected.output.split("\n")[1], "  framework:       Nuxt (nuxt.config.ts)");

  const overridden = await runInit(root, [...bundlerOnly, "--vite", "--dry-run"]);
  assert.deepEqual(overridden.plan?.commands, [
    { command: "pnpm", args: ["add", "-D", "@vizejs/vite-plugin"], cwd: root },
  ]);
  assert.deepEqual(overridden.plan?.updatedFiles, ["vite.config.ts"]);
  assert.equal(overridden.output.split("\n")[1], "  framework:       Vite (vite.config.ts)");
});

test("a hand-written lint block is never overwritten and never silently bypassed", async () => {
  const root = temporaryProject("hand-written");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "vp dev" },
    devDependencies: { "vite-plus": "^0.1.0" },
  });
  const handWritten = `import { defineConfig } from "vite-plus";
import legacy from "@vitejs/plugin-legacy";
import inspect from "vite-plugin-inspect";

export default defineConfig({
  lint: {
    plugins: ["eslint", "typescript"],
    rules: {
      "no-console": "warn",
    },
  },
  plugins: [legacy(), inspect()],
});
`;
  write(root, "vite.config.ts", handWritten);
  write(root, "tsconfig.json", "{}\n");
  write(root, "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");

  const result = await runInit(root, ALL_FEATURES);

  assert.equal(result.plan?.lintTarget.kind, "manual");
  assert.deepEqual(
    result.plan?.features.map((feature) => [feature.id, feature.outcome]),
    [
      ["lint", "blocked"],
      ["bundler", "configured"],
      ["fmt", "configured"],
      ["typecheck", "configured"],
      ["editor", "configured"],
    ],
  );
  // The lint block is untouched; only the plugin list gained an entry, and the
  // project's own plugins keep their order.
  assert.equal(
    read(root, "vite.config.ts"),
    `import { defineConfig } from "vite-plus";
import legacy from "@vitejs/plugin-legacy";
import inspect from "vite-plugin-inspect";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  lint: {
    plugins: ["eslint", "typescript"],
    rules: {
      "no-console": "warn",
    },
  },
  plugins: [vize(), legacy(), inspect()],
});
`,
  );
  // The bug this guards: falling back to an Oxlint config file would leave
  // `vp lint` reading the untouched hand-written block and reporting zero
  // vize/* diagnostics while exiting 0.
  assert.equal(exists(root, "oxlint.config.ts"), false);
  assert.equal(exists(root, ".oxlintrc.json"), false);
  assert.equal(
    result.plan?.features.find((feature) => feature.id === "lint")?.snippet,
    `import { createVizeLintConfig } from "oxlint-plugin-vize";

export default defineConfig({
  lint: {
    ...createVizeLintConfig({
      preset: "happy-path",
      settings: {
        helpLevel: "short",
      },
    }),
    // keep your existing lint keys here
  },
});
`,
  );
});

test("an oxlint config name oxlint never reads is reported, not counted as configured", async () => {
  const root = temporaryProject("unread-oxlint");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { lint: "oxlint src" },
    devDependencies: { vite: "^7.0.0" },
  });
  write(root, "oxlint.config.mjs", "export default {};\n");
  write(root, "package-lock.json", '{"lockfileVersion":3}\n');

  const result = await runInit(root, [
    "--yes",
    "--lint",
    "--no-fmt",
    "--no-typecheck",
    "--no-editor",
  ]);

  assert.equal(result.plan?.lintTarget.kind, "oxlint");
  assert.deepEqual(result.plan?.createdFiles, ["oxlint.config.ts", "vize.config.ts"]);
  assert.equal(read(root, "oxlint.config.mjs"), "export default {};\n");
  assert.equal(
    result.output.split("\n")[6],
    "  oxlint config:   oxlint.config.mjs — present but oxlint does not read this name (#3474)",
  );
});
