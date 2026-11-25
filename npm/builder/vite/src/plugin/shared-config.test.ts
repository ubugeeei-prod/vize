import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { createLogger } from "../transform.ts";
import { resolveSharedConfig } from "./shared-config.ts";

const env = { command: "build", mode: "production" } as const;

await test("Nuxt reads shared Vue settings from the project Vite config", async () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-shared-config-"));
  try {
    writeFileSync(
      path.join(root, "vite.config.ts"),
      `export default {
  vue: {
    compiler: { whitespace: "preserve", templateSyntax: "strict" }
  }
};\n`,
    );

    const resolved = await resolveSharedConfig(
      {},
      root,
      env,
      createLogger(false),
      { vue: { script: {}, features: {} } },
      "nuxt",
    );

    assert.equal(resolved?.compiler?.whitespace, "preserve");
    assert.equal(resolved?.compiler?.templateSyntax, "strict");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("Nuxt's plugin-vue compiler module never shadows the project's Vue compiler settings", async () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-host-compiler-"));
  try {
    writeFileSync(
      path.join(root, "vite.config.ts"),
      `export default { vue: { compiler: { whitespace: "preserve" } } };\n`,
    );
    // Nuxt 4.7 resolves `vite.vue` with the app's `vue/compiler-sfc` module.
    const compilerSfc = await import("data:text/javascript,export const parse = () => {};");

    const resolved = await resolveSharedConfig(
      {},
      root,
      env,
      createLogger(false),
      {
        vue: { isProduction: true, script: {}, template: {}, features: {}, compiler: compilerSfc },
      },
      "nuxt",
    );

    assert.equal(resolved?.compiler?.whitespace, "preserve");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("ordinary Vite hosts keep using their already resolved project config", async () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-vite-shared-config-"));
  try {
    writeFileSync(
      path.join(root, "vite.config.ts"),
      `export default { vue: { compiler: { whitespace: "preserve" } } };\n`,
    );

    const resolved = await resolveSharedConfig({}, root, env, createLogger(false), {
      vue: { compiler: { whitespace: "condense" } },
    });

    assert.equal(resolved?.compiler?.whitespace, "condense");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
