import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { build, createServer, type Plugin, type Rollup } from "vite";

import { vize } from "./index.ts";
import { removeHostVueSfcCompilers } from "./plugin/vue-version.ts";

const require = createRequire(import.meta.url);

const SFC = `<template>
  <main>
    <DevOnly><p>dev-only-marker</p></DevOnly>
  </main>
</template>
`;

/** A host transform ordered before the Vue compiler, shaped like Nuxt's `<DevOnly>` strip (#11641). */
function hostSourceTransform(seen: string[], replacement = "<p>fallback-marker</p>"): Plugin {
  return {
    name: "host:sfc-source",
    enforce: "pre",
    transform(code, id) {
      if (!/\.vue(?:\?|$)/.test(id) || !code.includes("<DevOnly>")) return null;
      seen.push(code);
      return code.replace(/<DevOnly>[\s\S]*?<\/DevOnly>/, replacement);
    },
  };
}

/** A host transform ordered after the compiler, which must keep seeing compiled JavaScript. */
function hostCompiledTransform(seen: string[]): Plugin {
  return {
    name: "host:compiled",
    enforce: "post",
    transform(code, id) {
      if (/\.vue\?vue&vize/.test(id)) seen.push(code);
      return null;
    },
  };
}

/** The original position of a generated line's first segment, decoded from v3 mappings. */
function firstOrigin(mappings: string, generatedLine: number) {
  const digits = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const origin = { source: 0, line: 0, column: 0 };
  for (const [index, line] of mappings.split(";").entries()) {
    for (const segment of line ? line.split(",") : []) {
      const fields: number[] = [];
      for (let i = 0, value = 0, shift = 0; i < segment.length; i++) {
        const digit = digits.indexOf(segment[i]);
        value += (digit & 31) << shift;
        shift += 5;
        if (digit & 32) continue;
        fields.push(value & 1 ? -(value >>> 1) : value >>> 1);
        value = 0;
        shift = 0;
      }
      if (fields.length < 4) continue;
      origin.source += fields[1];
      origin.line += fields[2];
      origin.column += fields[3];
      if (index === generatedLine) return { ...origin };
    }
  }
  return null;
}

function fixture(): string {
  const root = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vize-host-source-"));
  fs.writeFileSync(path.join(root, "entry.js"), 'import App from "./App.vue"; console.log(App);\n');
  fs.writeFileSync(path.join(root, "App.vue"), SFC);
  return root;
}

async function buildOutput(root: string, plugins: Plugin[]): Promise<string> {
  const result = (await build({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [...plugins, vize({ configMode: false })],
    build: {
      write: false,
      minify: false,
      rollupOptions: { input: path.join(root, "entry.js"), external: [/^vue(?:\/|$)/] },
    },
  })) as Rollup.RollupOutput;
  return result.output.map((item) => (item.type === "chunk" ? item.code : "")).join("\n");
}

void test("#11641 a build compiles the SFC source that host pre transforms produced", async () => {
  const root = fixture();
  try {
    const source: string[] = [];
    const compiled: string[] = [];
    const output = await buildOutput(root, [
      hostSourceTransform(source),
      hostCompiledTransform(compiled),
    ]);
    assert.equal(source.length, 1);
    assert.match(source[0], /^<template>/);
    assert.match(output, /fallback-marker/);
    assert.doesNotMatch(output, /dev-only-marker/);
    assert.equal(compiled.length, 1);
    assert.doesNotMatch(compiled[0], /<template>|<DevOnly>/);
    assert.match(compiled[0], /fallback-marker/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("#11641 an authored compile error in the host's output still fails the build", async () => {
  const root = fixture();
  try {
    await assert.rejects(
      buildOutput(root, [hostSourceTransform([], "<p v-for>broken-marker</p>")]),
      /v-for/,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("#11641 the dev server serves the SFC source that host pre transforms produced", async () => {
  const root = fixture();
  const source: string[] = [];
  const server = await createServer({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [hostSourceTransform(source), vize({ configMode: false, isProduction: false })],
    resolve: { alias: [{ find: /^vue$/u, replacement: require.resolve("vue") }] },
    optimizeDeps: { noDiscovery: true, include: [] },
    server: { middlewareMode: true, ws: false },
  });
  try {
    const entry = await server.transformRequest("/entry.js");
    const appUrl = /from\s+"([^"]*App\.vue[^"]*)"/.exec(entry?.code ?? "")?.[1];
    assert.ok(appUrl, entry?.code);
    for (const ssr of [false, true]) {
      const app = await server.transformRequest(appUrl, { ssr });
      assert.match(app?.code ?? "", ssr ? /ssrRender/ : /createElementBlock/);
      assert.match(app?.code ?? "", /fallback-marker/);
      assert.doesNotMatch(app?.code ?? "", /dev-only-marker/);
    }
    assert.equal(source.length, 2);
  } finally {
    await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("#11641 the compile stage takes the host compiler's place", () => {
  const hostVue = { name: "vite:vue", transform() {} };
  const sourceTransform = { name: "host:sfc-source", transform() {} };
  const compiledTransform = { name: "host:compiled", transform() {} };
  const stage = { name: "vite-plugin-vize:sfc", transform() {} };
  const replaced = [sourceTransform, hostVue, compiledTransform, stage];
  assert.equal(removeHostVueSfcCompilers(replaced, {}, stage), 1);
  assert.deepEqual(replaced, [sourceTransform, stage, compiledTransform]);

  const kept = [stage, sourceTransform, hostVue, compiledTransform];
  assert.equal(removeHostVueSfcCompilers(kept, { exclude: /legacy/ }, stage), 0);
  assert.deepEqual(kept, [sourceTransform, stage, hostVue, compiledTransform]);

  // Without a host compiler the stage keeps the place the config gave it.
  const alone = [sourceTransform, stage, compiledTransform];
  assert.equal(removeHostVueSfcCompilers(alone, {}, stage), 0);
  assert.deepEqual(alone, [sourceTransform, stage, compiledTransform]);
});

void test("#11641 a build's source map still names the authored SFC and its lines", async () => {
  const root = fixture();
  const app = `<script setup lang="ts">
const message: string = "mapped";
console.log(message);
</script>
<template><p>{{ message }}</p></template>
`;
  fs.writeFileSync(path.join(root, "App.vue"), app);
  try {
    const result = (await build({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [vize({ configMode: false })],
      build: {
        write: false,
        minify: false,
        sourcemap: true,
        rollupOptions: { input: path.join(root, "entry.js"), external: [/^vue(?:\/|$)/] },
      },
    })) as Rollup.RollupOutput;
    const chunk = result.output.find((item) => item.type === "chunk" && item.map);
    assert.ok(chunk && chunk.type === "chunk" && chunk.map);
    const sourceIndex = chunk.map.sources.findIndex((source) => /(?:^|\/)App\.vue$/.test(source));
    assert.notEqual(sourceIndex, -1, JSON.stringify(chunk.map.sources));
    assert.equal(chunk.map.sourcesContent?.[sourceIndex], app);
    // The `console.log` line of the chunk maps back to line 3 of the SFC.
    const generatedLine = chunk.code
      .split("\n")
      .findIndex((line) => line.includes("console.log(message)"));
    const origin = firstOrigin(chunk.map.mappings, generatedLine);
    assert.deepEqual(origin && { source: origin.source, line: origin.line }, {
      source: sourceIndex,
      line: 2,
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
