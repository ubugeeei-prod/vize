import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { normalizePath, optimizeDeps, resolveConfig } from "vite";
import { registerSourceGalleryDependencies } from "./gallery-dependencies.ts";

const require = createRequire(import.meta.url);

void test("a cold empty app prebundles the middleware gallery's real highlighting modules", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-gallery-dependencies-"));
  try {
    await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
    const vue = require.resolve("vue/dist/vue.runtime.esm-bundler.js");
    const original = await resolveConfig(
      { root, configFile: false, cacheDir: path.join(root, ".original") },
      "serve",
    );
    assert.deepEqual((await optimizeDeps(original)).optimized, {});
    const config = await resolveConfig(
      {
        root,
        configFile: false,
        cacheDir: path.join(root, ".cache"),
        resolve: { alias: { vue } },
        optimizeDeps: { exclude: ["monaco-editor"] },
      },
      "serve",
    );
    const entries = config.optimizeDeps.entries;
    await registerSourceGalleryDependencies(config);
    assert.equal(config.optimizeDeps.entries, entries);
    assert.deepEqual(config.optimizeDeps.exclude, ["monaco-editor"]);
    const metadata = await optimizeDeps(config);
    const optimizedSources = Object.values(metadata.optimized).map((entry) => entry.src);
    for (const id of [
      "highlight.js/lib/core",
      "highlight.js/lib/languages/xml",
      "highlight.js/lib/languages/json",
      "highlight.js/lib/languages/javascript",
      "highlight.js/lib/languages/typescript",
      "highlight.js/lib/languages/css",
      "highlight.js/lib/languages/bash",
    ]) {
      assert.ok(
        optimizedSources.includes(normalizePath(fileURLToPath(import.meta.resolve(id)))),
        `${id} must join the first bundle`,
      );
    }
    assert.ok(
      optimizedSources.includes(normalizePath(vue)),
      "the actual user Vue alias must be preserved",
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

void test("gallery defaults preserve explicit optimizer disablement and dependency exclusions", async () => {
  for (const optimizeDeps of [
    { noDiscovery: true, include: [] },
    { disabled: true },
    { disabled: "dev" as const },
  ]) {
    const config = await resolveConfig({ configFile: false, optimizeDeps }, "serve");
    const before = structuredClone(config.optimizeDeps);
    await registerSourceGalleryDependencies(config);
    assert.deepEqual(config.optimizeDeps, before);
  }
  const included = require.resolve("marked");
  const config = await resolveConfig(
    { configFile: false, optimizeDeps: { include: [included], exclude: ["highlight.js"] } },
    "serve",
  );
  await registerSourceGalleryDependencies(config);
  assert.ok(config.optimizeDeps.include?.includes(included));
  assert.ok(config.optimizeDeps.include?.every((id) => !id.includes("highlight.js")));
  const build = await resolveConfig({ configFile: false }, "build");
  const before = structuredClone(build.optimizeDeps);
  await registerSourceGalleryDependencies(build);
  assert.deepEqual(build.optimizeDeps, before);
});
