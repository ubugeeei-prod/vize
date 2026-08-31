import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  assertMuseaNativeLoaded,
  assertMuseaNativeSelection,
  resolveMuseaArtifacts,
} from "../../tools/benchmarks/scripts/musea-artifacts.mjs";
import { createMuseaStages } from "../../tools/benchmarks/scripts/musea-stages.mjs";

function digestOf(values: string[]): string {
  const hash = createHash("sha256");
  for (const value of values) {
    const text = value ?? "";
    hash.update(`${Buffer.byteLength(text)}:`);
    hash.update(text);
  }
  return hash.digest("hex");
}

async function withFixture(fn: (root: string) => Promise<void>): Promise<void> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-stages-"));
  try {
    writeFixtureRuntime(root);
    await fn(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function writeFixtureRuntime(root: string): void {
  const pluginDist = path.join(root, "npm", "builder", "vite-musea", "dist");
  const nuxtDist = path.join(root, "npm", "framework", "musea-nuxt", "dist");
  const nativeDir = path.join(root, "npm", "native");
  fs.mkdirSync(path.join(pluginDist, "chunks"), { recursive: true });
  fs.mkdirSync(nuxtDist, { recursive: true });
  fs.mkdirSync(nativeDir, { recursive: true });

  fs.writeFileSync(
    path.join(pluginDist, "chunks", "runtime.mjs"),
    'export const marker = "chunk";\n',
  );
  fs.writeFileSync(
    path.join(pluginDist, "index.mjs"),
    `import { createRequire } from "node:module";
import { marker } from "./chunks/runtime.mjs";
const require = createRequire(import.meta.url);
export const runtimeMarker = marker;
export function musea() {
  let root = "";
  let native = null;
  return [{
    name: "vite-plugin-musea",
    config(userConfig) {
      return { build: { rollupOptions: { input: {
        "musea-static-entry": userConfig.build.rollupOptions.input,
        "musea-static-runtime": "virtual:musea-static-runtime",
      } } } };
    },
    configResolved(config) { root = config.root; },
    options() { return null; },
    buildStart() { native = require("@vizejs/native"); },
    resolveId(id) { return id.endsWith(".art.vue") ? "\\0fixture:" + id : null; },
    load(id) {
      if (!id.startsWith("\\0")) return null;
      return JSON.stringify({ id, marker, root, native });
    },
    transform(code) { return { code: "transformed:" + code }; },
  }];
}
`,
  );
  fs.writeFileSync(
    path.join(nuxtDist, "index.mjs"),
    `import path from "node:path";
import { fileURLToPath } from "node:url";
export function nuxtMusea() {
  const srcDir = path.dirname(fileURLToPath(import.meta.url));
  return {
    name: "fixture-nuxt",
    resolveId(id) { return id.startsWith("#") || id.startsWith("nuxt/") ? "\\0nuxt" : null; },
    load(id) { return id === "\\0nuxt" ? "export * from '" + path.join(srcDir, "auto-imports.js") + "';" : null; },
  };
}
`,
  );
  fs.writeFileSync(path.join(nativeDir, "vize-vitrine.fixture.node"), "fixture-native-binary");
  fs.writeFileSync(
    path.join(nativeDir, "index.js"),
    'module.exports = require("./native-binding");\n',
  );
  fs.writeFileSync(
    path.join(nativeDir, "native-binding.js"),
    `const path = require("node:path");
const bindingPath = path.join(__dirname, "vize-vitrine.fixture.node");
require.cache[bindingPath] = { id: bindingPath, filename: bindingPath, loaded: true, exports: {} };
module.exports = { loaderDir: __dirname, bindingPath };
`,
  );
  fs.writeFileSync(
    path.join(nativeDir, "native-targets.js"),
    'module.exports = { nativeTargets: () => ["fixture"] };\n',
  );
  fs.writeFileSync(path.join(nativeDir, "package.json"), '{"name":"@vizejs/native"}\n');
}

test("the runtime snapshot preserves package-relative ESM semantics", async () => {
  await withFixture(async (root) => {
    const artifacts = resolveMuseaArtifacts(root);
    assert.doesNotThrow(() => assertMuseaNativeSelection(artifacts));
    const sourcePlugin = await import(`${pathToFileURL(artifacts.museaPlugin.source).href}?source`);
    const pinnedPlugin = await import(
      `${pathToFileURL(artifacts.museaPlugin.measuredPath).href}?pin`
    );
    assert.equal(sourcePlugin.runtimeMarker, "chunk");
    assert.equal(pinnedPlugin.runtimeMarker, sourcePlugin.runtimeMarker);

    for (const artifact of [artifacts.museaPlugin, artifacts.museaNuxt]) {
      assert.equal(path.relative(path.dirname(artifact.source), artifact.source), "index.mjs");
      assert.equal(
        path.relative(path.dirname(artifact.measuredPath), artifact.measuredPath),
        "index.mjs",
      );
      assert.match(
        artifact.measuredPath,
        /node_modules[/\\]\.cache[/\\]vize-musea-benchmark[/\\][a-f0-9]{64}[/\\]package[/\\]dist[/\\]index\.mjs$/,
      );
    }

    const generatedPaths: Array<{ entry: string; generated: string }> = [];
    for (const [kind, entry] of [
      ["source", artifacts.museaNuxt.source],
      ["pinned", artifacts.museaNuxt.measuredPath],
    ] as const) {
      const { nuxtMusea } = await import(`${pathToFileURL(entry).href}?${kind}`);
      const plugin = nuxtMusea();
      const code = plugin.load(plugin.resolveId("#imports"));
      generatedPaths.push({ entry, generated: code.match(/'([^']+)'/)?.[1] ?? "" });
    }
    assert.deepEqual(
      generatedPaths.map(({ entry, generated }) => ({
        generated,
        expected: path.join(path.dirname(fs.realpathSync(entry)), "auto-imports.js"),
      })),
      generatedPaths.map(({ entry }) => ({
        generated: path.join(path.dirname(fs.realpathSync(entry)), "auto-imports.js"),
        expected: path.join(path.dirname(fs.realpathSync(entry)), "auto-imports.js"),
      })),
    );
  });
});

test("all six stages return raw output and hash only in observe", async () => {
  await withFixture(async (root) => {
    const artifacts = resolveMuseaArtifacts(root);
    const previous = process.env.NAPI_RS_NATIVE_LIBRARY_PATH;
    process.env.NAPI_RS_NATIVE_LIBRARY_PATH = artifacts.native.measuredPath;
    try {
      const files = [
        path.join(root, "corpus", "A.art.vue"),
        path.join(root, "corpus", "B.art.vue"),
      ];
      const stages = createMuseaStages({ artifacts, workDir: path.join(root, "corpus"), files });
      assert.deepEqual(
        stages.map(({ id, label, units, unitLabel }) => ({ id, label, units, unitLabel })),
        [
          {
            id: "musea-options",
            label: "options: preserve configured Rollup inputs",
            units: 1,
            unitLabel: "build hooks",
          },
          {
            id: "musea-build-start",
            label: "buildStart: scan + parse art files",
            units: 2,
            unitLabel: "art files",
          },
          {
            id: "musea-load",
            label: "load: generate art modules",
            units: 2,
            unitLabel: "art files",
          },
          {
            id: "musea-transform",
            label: "transform: TS to JS on generated modules",
            units: 2,
            unitLabel: "art files",
          },
          {
            id: "musea-nuxt-virtual",
            label: "musea-nuxt: resolve Nuxt mock specifiers",
            units: 18,
            unitLabel: "resolutions",
          },
          {
            id: "musea-plugin-total",
            label: "whole plugin: config + options + buildStart + load + transform",
            units: 2,
            unitLabel: "art files",
          },
        ],
      );

      const outputs = [];
      for (const stage of stages) {
        await stage.prepare();
        const output = await stage.run();
        outputs.push(output);
        assert.match(await stage.observe(output), /^[a-f0-9]{64}$/);
      }
      assert.doesNotThrow(() => assertMuseaNativeLoaded(artifacts));
      assert.equal(outputs[0], null);
      assert.equal(outputs[1], undefined);
      assert.ok(Array.isArray(outputs[2]));
      assert.ok(Array.isArray(outputs[3]));
      assert.ok(Array.isArray(outputs[4]));
      assert.deepEqual(Object.keys(outputs[5]), [
        "rollupOptions",
        "optionsReturned",
        "modules",
        "transformed",
      ]);
      assert.match(
        outputs[2].join("\n"),
        new RegExp(artifacts.native.measuredPath.replaceAll("\\", "\\\\")),
      );

      const optionsDigest = digestOf([
        JSON.stringify({
          before: {
            input: {
              "musea-static-entry": "musea-user-entry.html",
              "musea-static-runtime": "virtual:musea-static-runtime",
            },
          },
          after: {
            input: {
              "musea-static-entry": "musea-user-entry.html",
              "musea-static-runtime": "virtual:musea-static-runtime",
            },
          },
          returned: null,
        }),
      ]);
      assert.equal(await stages[0].observe(outputs[0]), optionsDigest);
    } finally {
      if (previous === undefined)
        Reflect.deleteProperty(process.env, "NAPI_RS_NATIVE_LIBRARY_PATH");
      else process.env.NAPI_RS_NATIVE_LIBRARY_PATH = previous;
    }
  });
});
