import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createServer, normalizePath, optimizeDeps, resolveConfig } from "vite";
import { registerSourceGalleryDependencies } from "./gallery-dependencies.ts";

const require = createRequire(import.meta.url);

void test("a cold empty app serves the gallery's bare dependency IDs in its first generation", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-gallery-dependencies-"));
  let server: Awaited<ReturnType<typeof createServer>> | undefined;
  try {
    await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
    const original = await resolveConfig({ root, configFile: false }, "serve");
    assert.deepEqual((await optimizeDeps(original)).optimized, {});
    for (const ignored of ["__tests__", "coverage"]) {
      await mkdir(path.join(root, ignored));
      await writeFile(
        path.join(root, ignored, "ignored.html"),
        '<script type="module">import "missing-ignored-dependency"</script>',
      );
    }
    await writeFile(
      path.join(root, "extra.html"),
      '<script type="module">import "app-highlight"</script>',
    );
    const vue = require.resolve("vue/dist/vue.runtime.esm-bundler.js");
    const gallery = fileURLToPath(new URL("../gallery/", import.meta.url));
    server = await createServer({
      root,
      configFile: false,
      cacheDir: path.join(root, ".cache"),
      resolve: {
        alias: {
          vue,
          "app-highlight": fileURLToPath(import.meta.resolve("highlight.js/lib/languages/python")),
        },
      },
      server: {
        host: "127.0.0.1",
        port: 0,
        preTransformRequests: false,
        fs: { allow: [root, gallery] },
      },
      plugins: [
        { name: "source-gallery-dependencies", configResolved: registerSourceGalleryDependencies },
      ],
    });
    await server.listen();
    const transformed = await server.transformRequest(
      "/@fs/" + normalizePath(path.join(gallery, "main.ts")),
    );
    const vueUrl = transformed?.code.match(/from "([^"]*vue\.js\?[^"]*)"/)?.[1];
    assert.ok(vueUrl, "the real gallery's Vue import must use an optimized browser URL");
    assert.equal((await fetch(new URL(vueUrl, server.resolvedUrls!.local[0]))).status, 200);
    const optimizer = server.environments.client.depsOptimizer!;
    for (const id of [
      "highlight.js/lib/core",
      "highlight.js/lib/languages/xml",
      "highlight.js/lib/languages/json",
      "highlight.js/lib/languages/javascript",
      "highlight.js/lib/languages/typescript",
      "highlight.js/lib/languages/css",
      "highlight.js/lib/languages/bash",
      "marked",
      "monaco-editor",
      "app-highlight",
    ]) {
      assert.ok(
        optimizer.metadata.optimized[id],
        `${id} must join the first bundle under its browser import ID`,
      );
    }
    assert.equal(optimizer.metadata.optimized.vue.src, normalizePath(vue));
    assert.deepEqual(optimizer.metadata.discovered, {});
  } finally {
    await server?.close();
    await rm(root, { recursive: true, force: true });
  }
});

void test("gallery defaults preserve disablement, dependency exclusions, and explicit app entries", async () => {
  for (const optimizeDeps of [
    { noDiscovery: true, include: [] },
    { noDiscovery: true, include: ["vue"] },
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
    {
      configFile: false,
      optimizeDeps: {
        entries: ["custom.html", "!ignored/**"],
        include: [included],
        exclude: ["highlight.js"],
      },
    },
    "serve",
  );
  await registerSourceGalleryDependencies(config);
  assert.deepEqual(config.optimizeDeps.entries?.slice(0, 2), ["custom.html", "!ignored/**"]);
  assert.deepEqual(config.optimizeDeps.include, [included]);
  assert.deepEqual(config.optimizeDeps.exclude, ["highlight.js"]);
  const metadata = await optimizeDeps(config);
  assert.ok(Object.keys(metadata.optimized).every((id) => !id.startsWith("highlight.js")));
  const build = await resolveConfig({ configFile: false }, "build");
  const before = structuredClone(build.optimizeDeps);
  await registerSourceGalleryDependencies(build);
  assert.deepEqual(build.optimizeDeps, before);
});

void test("explicit build inputs retain their app scan instead of crawling unrelated HTML", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-gallery-build-input-"));
  try {
    const entry = path.join(root, "app[entry].ts");
    await writeFile(entry, 'import "app-highlight";');
    await writeFile(
      path.join(root, "ignored.html"),
      '<script type="module">import "missing-ignored-dependency"</script>',
    );
    const config = await resolveConfig(
      {
        root,
        configFile: false,
        resolve: {
          alias: {
            "app-entry": entry,
            "app-highlight": fileURLToPath(
              import.meta.resolve("highlight.js/lib/languages/python"),
            ),
          },
        },
        build: { rollupOptions: { input: { app: "app-entry" } } },
      },
      "serve",
    );
    await registerSourceGalleryDependencies(config);
    const metadata = await optimizeDeps(config);
    assert.ok(metadata.optimized["app-highlight"]);
    assert.ok(metadata.optimized["highlight.js/lib/languages/css"]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

void test("the gallery document and styles request no external origins", async () => {
  const gallery = fileURLToPath(new URL("../gallery/", import.meta.url));
  const external =
    /(?:href|src)\s*=\s*["']?(?:https?:)?\/\/|url\(\s*["']?(?:https?:)?\/\/|@import\s+["'](?:https?:)?\/\//iu;
  const documents = (await readdir(gallery, { recursive: true })).filter((file) =>
    /\.(?:html|css|vue)$/u.test(file),
  );
  assert.ok(documents.includes("index.html"));
  for (const file of documents) {
    const source = await readFile(path.join(gallery, file), "utf8");
    assert.doesNotMatch(source, external, file);
  }
});
