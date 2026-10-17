import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "vite-plus/test";

import { loadConfig } from "../src/config.ts";
import { resolveViteConfigExport } from "../src/config/vite-runtime.mjs";

test("Vite functions receive serve/build while the authored workflow environment stays exact", async () => {
  for (const command of ["lint", "check", "fmt", "serve", "build"]) {
    const env = Object.freeze({ command, mode: "production", isSsrBuild: true });
    const seen: unknown[] = [];
    const viteEnv = { ...env, command: command === "build" ? "build" : "serve" };
    const resolved = await resolveViteConfigExport((actual: unknown) => {
      seen.push(actual);
      return { vue: { linter: { rules: { "vue/require-v-for-key": "error" } } } };
    }, env);
    assert.deepEqual(resolved, {
      typeChecker: { jsxTypecheck: true },
      linter: { rules: { "vue/require-v-for-key": "error" } },
    });
    assert.deepEqual(seen, [viteEnv]);
    if (command === "serve" || command === "build") {
      assert.equal(seen[0], env);
    }
    assert.deepEqual(env, { command, mode: "production", isSsrBuild: true });
  }
});

test("Vite settings retain shared values, overrides and explicit feature disables", async () => {
  const source = {
    vue: {
      typeChecker: { jsxTypecheck: false },
      linter: { preset: "essential" },
      languageServer: { formatting: false },
    },
    compiler: { vueVersion: 2.7, whitespace: "preserve" },
    typecheck: { strict: true },
    lint: { vue: { rules: { "a11y/img-alt": "error" }, helpLevel: "short" } },
    fmt: { printWidth: 90, vue: { singleQuote: true, tabWidth: 4 } },
  };
  const original = JSON.stringify(source);
  assert.deepEqual(await resolveViteConfigExport(source), {
    typeChecker: { jsxTypecheck: false, strict: true },
    languageServer: { formatting: false },
    compiler: { whitespace: "preserve", compatibility: { vueVersion: "2.7" } },
    linter: { preset: "essential", rules: { "a11y/img-alt": "error" } },
    formatter: { singleQuote: true, tabWidth: 4 },
  });
  assert.equal(JSON.stringify(source), original);
});

test("Vite+ source resolution never starts plugins or generated tasks", async () => {
  const factory = Object.assign(
    () => {
      throw new Error("must not run Vite+ factory");
    },
    {
      [Symbol.for("@vizejs/vite-plugin/vite-plus/source")]: async () => ({
        fmt: { vue: { printWidth: 100, singleQuote: true, tabWidth: 4 } },
        typecheck: false,
      }),
    },
  );
  assert.deepEqual(await resolveViteConfigExport(factory), {
    typeChecker: { jsxTypecheck: true, enabled: false },
    formatter: { printWidth: 100, singleQuote: true, tabWidth: 4 },
  });
});

test("Vite source cycles fail explicitly and preserve scoped entries and RegExp values", async () => {
  const cyclic = Object.assign(() => ({}), {
    [Symbol.for("@vizejs/vite-plugin/vite-plus/source")]: undefined as unknown,
  });
  cyclic[Symbol.for("@vizejs/vite-plugin/vite-plus/source")] = cyclic;
  await assert.rejects(resolveViteConfigExport(cyclic), /Circular Vue configuration source/);
  await assert.rejects(resolveViteConfigExport({ extends: {} }), /Custom config extends/);
  const include = /\.vue$/;
  const entries = [
    { files: ["packages/app/**/*.vue"], linter: { rules: { "vue/no-v-html": "error" } } },
  ];
  const resolved = (await resolveViteConfigExport({
    plugins: [{ name: "retain", include }],
    vue: { entries },
  })) as { entries: unknown[] };
  assert.deepEqual(resolved.entries, entries);
  assert.equal(include.source, "\\.vue$");
});

test("canonical discovery reads Vite TypeScript settings and a stale dedicated file cannot override it", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-project-"));
  try {
    fs.writeFileSync(
      path.join(root, "vite.config.ts"),
      `export default { vue: { formatter: { singleQuote: true } } } satisfies Record<string, unknown>;`,
    );
    assert.equal((await loadConfig(root))?.formatter.singleQuote, true);
    assert.equal(await loadConfig(root, { viteConfig: false }), null);
    fs.writeFileSync(
      path.join(root, "vize.config.json"),
      JSON.stringify({ formatter: { singleQuote: false } }),
    );
    assert.equal((await loadConfig(root))?.formatter.singleQuote, true);
    assert.equal(
      (await loadConfig(root, { configFile: "vite.config.ts" }))?.formatter.singleQuote,
      true,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("automatic discovery stops at an unconfigured monorepo package boundary", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-monorepo-"));
  try {
    fs.writeFileSync(
      path.join(root, "vite.config.mjs"),
      `export default { vue: { formatter: { tabWidth: 8 } } };`,
    );
    const first = path.join(root, "packages/first");
    const second = path.join(root, "packages/second");
    fs.mkdirSync(path.join(first, "src"), { recursive: true });
    fs.mkdirSync(path.join(second, "src"), { recursive: true });
    fs.writeFileSync(path.join(first, "package.json"), "{}");
    fs.writeFileSync(path.join(second, "tsconfig.json"), "{}");
    fs.writeFileSync(
      path.join(first, "vite.config.mjs"),
      `export default { fmt: { vue: { tabWidth: 4 } } };`,
    );
    assert.equal(
      (await loadConfig(path.join(first, "src"), { mode: "auto" }))?.formatter.tabWidth,
      4,
    );
    assert.equal(await loadConfig(path.join(second, "src"), { mode: "auto" }), null);
    assert.equal(await loadConfig(path.join(first, "src")), null);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("Vite root projects scoped paths while preserving global entries and authored values", async () => {
  const root = path.join(os.tmpdir(), "vize-vite-root-fixture");
  const source = {
    root: "app",
    vue: {
      files: ["src/**/*.vue"],
      ignores: ["src/Ignored.vue", "!src/Keep.vue", String.raw`src/\[id\].vue`],
      typeChecker: { tsconfig: "tsconfig.json", corsaPath: "tools/corsa" },
      entries: [
        { files: ["src/**/*.vue"], linter: { rules: { "a11y/alt-text": "error" } } },
        { basePath: "packages/ui", files: ["*.vue"] },
      ],
    },
  };
  const original = JSON.stringify(source);
  const resolved = await resolveViteConfigExport(source, undefined, root);
  assert.deepEqual(resolved, {
    __vizeProjectRoot: path.join(root, "app"),
    basePath: path.join(root, "app"),
    files: ["src/**/*.vue"],
    ignores: ["src/Ignored.vue", "!src/Keep.vue", String.raw`src/\[id\].vue`],
    typeChecker: {
      jsxTypecheck: true,
      tsconfig: path.join(root, "app/tsconfig.json"),
      corsaPath: path.join(root, "app/tools/corsa"),
    },
    entries: [
      {
        basePath: path.join(root, "app"),
        files: ["src/**/*.vue"],
        linter: { rules: { "a11y/alt-text": "error" } },
      },
      { basePath: path.join(root, "app/packages/ui"), files: ["*.vue"] },
    ],
  });
  assert.equal(JSON.stringify(source), original);

  const projected = await resolveViteConfigExport(
    {
      root: "app",
      __vizeProjectRoot: "decoy",
      vue: { formatter: { singleQuote: true }, entries: [{ files: ["src/*.vue"] }] },
    },
    undefined,
    root,
  );
  assert.deepEqual(projected, {
    __vizeProjectRoot: path.join(root, "app"),
    typeChecker: { jsxTypecheck: true },
    formatter: { singleQuote: true },
    entries: [{ basePath: path.join(root, "app"), files: ["src/*.vue"] }],
  });
  await assert.rejects(resolveViteConfigExport({ vize: [] }), /move Vue settings to vue:/);
});

test("retired files and namespaces fail before evaluating callbacks while tsconfig-only projects stay config-free", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vue-config-migration-"));
  try {
    fs.writeFileSync(
      path.join(root, "tsconfig.json"),
      '{"vueCompilerOptions":{"strictTemplates":true}}',
    );
    assert.equal(await loadConfig(root), null);
    fs.writeFileSync(path.join(root, "vize.config.json"), "{}");
    await assert.rejects(loadConfig(root), /move Vue settings to vue:/);
    await assert.rejects(
      loadConfig(root, { configFile: "vize.config.json" }),
      /no longer supported/,
    );
    let evaluated = false;
    await assert.rejects(
      resolveViteConfigExport({
        vize() {
          evaluated = true;
          return {};
        },
      }),
      /move Vue settings to vue:/,
    );
    assert.equal(evaluated, false);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
