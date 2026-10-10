import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "vite-plus/test";

import { loadConfig } from "../src/config.ts";
import { resolveViteConfigExport } from "../src/config/vite-runtime.mjs";

test("Vite settings retain shared values, overrides and explicit feature disables", async () => {
  const source = {
    vize: {
      typeChecker: { jsxTypecheck: false },
      lint: { preset: "essential" },
      languageServer: { formatting: false },
    },
    compiler: { vueVersion: 2.7, whitespace: "preserve" },
    typecheck: { strict: true },
    lint: { vize: { rules: { "a11y/img-alt": "error" }, helpLevel: "short" } },
    fmt: { printWidth: 90, singleQuote: true, vize: { tabWidth: 4 } },
  };
  const original = JSON.stringify(source);
  assert.deepEqual(await resolveViteConfigExport(source), {
    typeChecker: { jsxTypecheck: false, strict: true },
    languageServer: { formatting: false },
    compiler: { whitespace: "preserve", compatibility: { vueVersion: "2.7" } },
    linter: { preset: "essential", rules: { "a11y/img-alt": "error" } },
    formatter: { printWidth: 90, singleQuote: true, tabWidth: 4 },
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
        extends: Promise.resolve({ fmt: { printWidth: 100, vize: { singleQuote: true } } }),
        fmt: { vize: { tabWidth: 4 } },
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
  const cyclic: { extends?: unknown } = {};
  cyclic.extends = cyclic;
  await assert.rejects(resolveViteConfigExport(cyclic), /Circular extends/);
  const include = /\.vue$/;
  const entries = [{ files: ["packages/app/**/*.vue"], formatter: { tabWidth: 4 } }];
  const resolved = (await resolveViteConfigExport({ vize: { vite: { include }, entries } })) as {
    vite: { include: RegExp };
    entries: unknown[];
  };
  assert.equal(resolved.vite.include, include);
  assert.deepEqual(resolved.entries, entries);
});

test("canonical discovery reads Vite TypeScript settings and keeps dedicated precedence", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-project-"));
  try {
    fs.writeFileSync(
      path.join(root, "vite.config.ts"),
      `export default { vize: { formatter: { singleQuote: true } } } satisfies Record<string, unknown>;`,
    );
    assert.equal((await loadConfig(root))?.formatter.singleQuote, true);
    assert.equal(await loadConfig(root, { viteConfig: false }), null);
    fs.writeFileSync(
      path.join(root, "vize.config.json"),
      JSON.stringify({ formatter: { singleQuote: false } }),
    );
    assert.equal((await loadConfig(root))?.formatter.singleQuote, false);
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
      `export default { vize: { formatter: { tabWidth: 8 } } };`,
    );
    const first = path.join(root, "packages/first");
    const second = path.join(root, "packages/second");
    fs.mkdirSync(path.join(first, "src"), { recursive: true });
    fs.mkdirSync(path.join(second, "src"), { recursive: true });
    fs.writeFileSync(path.join(first, "package.json"), "{}");
    fs.writeFileSync(path.join(second, "tsconfig.json"), "{}");
    fs.writeFileSync(
      path.join(first, "vite.config.mjs"),
      `export default { fmt: { vize: { tabWidth: 4 } } };`,
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
