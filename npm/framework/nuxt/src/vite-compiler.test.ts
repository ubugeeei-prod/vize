import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { nuxtTypeResolutionFallback } from "./vite-compiler.ts";

void test("a production build into the cache buildDir reads the tsconfig Nuxt wrote to typesDir", () => {
  // After `nuxt prepare` or `nuxt dev`, `nuxt build` moves buildDir into the
  // cache; the generated tsconfigs stay in typesDir and the cache buildDir only
  // receives the server tsconfig.
  const rootDir = mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-types-dir-"));
  try {
    const typesDir = path.join(rootDir, ".nuxt");
    const buildDir = path.join(rootDir, "node_modules/.cache/nuxt/.nuxt");
    mkdirSync(typesDir, { recursive: true });
    mkdirSync(buildDir, { recursive: true });
    writeFileSync(path.join(typesDir, "tsconfig.json"), "{}\n");
    writeFileSync(path.join(buildDir, "tsconfig.server.json"), "{}\n");

    const fallback = nuxtTypeResolutionFallback({ buildDir, typesDir });

    assert.deepEqual(fallback, {
      typeResolutionFallbackTsconfig: path.join(typesDir, "tsconfig.json"),
    });
    assert.equal(existsSync(fallback.typeResolutionFallbackTsconfig ?? ""), true);
  } finally {
    rmSync(rootDir, { recursive: true, force: true });
  }
});

void test("Nuxt releases without typesDir keep the generated tsconfig in buildDir", () => {
  const buildDir = path.resolve("/repo/app/.nuxt");
  assert.deepEqual(nuxtTypeResolutionFallback({ buildDir }), {
    typeResolutionFallbackTsconfig: path.join(buildDir, "tsconfig.json"),
  });
  assert.deepEqual(nuxtTypeResolutionFallback({ buildDir, typesDir: "" }), {
    typeResolutionFallbackTsconfig: path.join(buildDir, "tsconfig.json"),
  });
});

void test("no generated directory leaves type resolution without a fallback", () => {
  assert.deepEqual(nuxtTypeResolutionFallback({}), {});
});
