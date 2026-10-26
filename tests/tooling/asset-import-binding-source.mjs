// Real source-built N-API producers, complete packets, Node module semantics.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

const require = createRequire(import.meta.url);
const [epoch, image, output] = process.argv.slice(2);
assert.ok(epoch === "base" || epoch === "head");
const native = require(path.resolve(image));
fs.mkdirSync(output, { recursive: true });
const moduleRoot = path.join(output, "modules");
fs.mkdirSync(moduleRoot, { recursive: true });
const modules = path.join(output, "node_modules");
if (!fs.existsSync(modules)) fs.symlinkSync(path.resolve("tests/node_modules"), modules, "dir");
const { createVNode, createSSRApp } = await import("vue");
const { renderToString } = await import("vue/server-renderer");
const runtime = require.resolve("vue/package.json");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const packets = [];
const report = {
  epoch,
  sourceSha: process.env.SOURCE_SHA,
  node: process.version,
  native: {
    image: path.resolve(image),
    sha256: hash(fs.readFileSync(image)),
    buildToken: native.getNativeBuildToken(),
  },
  runtime: {
    package: runtime,
    sha256: hash(fs.readFileSync(runtime)),
    version: require(runtime).version,
    importEntry: import.meta.resolve("vue"),
    importEntrySha256: hash(fs.readFileSync(fileURLToPath(import.meta.resolve("vue")))),
    serverRendererEntry: import.meta.resolve("vue/server-renderer"),
    serverRendererEntrySha256: hash(
      fs.readFileSync(fileURLToPath(import.meta.resolve("vue/server-renderer"))),
    ),
  },
  scope:
    "real source native collector + existing rewriter; data URL asset modules; Node --check and actual Vue VNode/free-reference semantics; not a bundler speed claim",
  packets,
};
// Preserve the current complete packet even when a real assertion fails.
process.on("exit", () => {
  fs.writeFileSync(
    path.join(output, `${epoch}-native.json`),
    JSON.stringify(report, null, 2) + "\n",
  );
});
const asset = "<template><img src='./a.svg'></template>";
const cases = [
  [
    "normal-ts",
    "<script lang='ts'>const _imports_0: string = 'authored'; export default {value: _imports_0}</script>" +
      asset,
    1,
    "value",
  ],
  [
    "setup-ts",
    "<script setup lang='ts'>const _imports_0: string = 'authored'</script><template><img src='./a.svg' :title='_imports_0'></template>",
    1,
  ],
  [
    "type-only-negative",
    "<script setup lang='ts'>type _imports_0 = string; const title: _imports_0 = 'authored'</script><template><img src='./a.svg' :title='title'></template>",
    0,
  ],
  [
    "escaped-braced",
    String.raw`<script>const \u{5f}imports_\u{30} = 'authored'; export default {value: _imports_0}</script>` +
      asset,
    1,
    "value",
  ],
  [
    "template-long-pattern",
    "<template><img v-for='({ a, b, c, d, e, f, g, h, i, j, k, label: _imports_0 }, index) of items' :key='index' src='./a.svg' :title='_imports_0'></template>",
    1,
    "pattern",
  ],
  [
    "template-for-entity",
    "<template><img v-for='&#95;imports_0 of items' :key='_imports_0' src='./a.svg' :title='_imports_0'></template>",
    1,
    "for",
  ],
  [
    "template-for-escaped",
    String.raw`<template><img v-for='\u005fimports_0 of items' :key='_imports_0' src='./a.svg' :title='_imports_0'></template>`,
    1,
    "for",
  ],
  [
    "normal-const",
    "<script>const _imports_0 = 'authored'; export default {value: _imports_0}</script>" + asset,
    1,
    "value",
  ],
  [
    "normal-var",
    "<script>if (true) { var _imports_0 = 'authored' } export default {value: _imports_0}</script>" +
      asset,
    1,
    "value",
  ],
  [
    "normal-escaped",
    String.raw`<script>const \u005fimports_0 = 'authored'; export default {value: _imports_0}</script>` +
      asset,
    1,
    "value",
  ],
  [
    "normal-import",
    "<script>import {ref as _imports_0} from 'vue'; export default {value: _imports_0('authored').value}</script>" +
      asset,
    1,
    "value",
  ],
  [
    "setup-const",
    "<script setup>const _imports_0 = 'authored'</script><template><img src='./a.svg' :title='_imports_0'></template>",
    1,
  ],
  [
    "setup-import",
    "<script setup>import {ref as _imports_0} from 'vue'; const title = _imports_0('authored')</script><template><img src='./a.svg' :title='title'></template>",
    1,
  ],
  [
    "both-blocks",
    "<script>const _imports_0 = 0; export default {}</script><script setup>const _imports_1 = 1</script>" +
      asset,
    2,
  ],
  ["free-global", "<script>export default {read: () => _imports_0}</script>" + asset, 1, "global"],
  [
    "escaped-global",
    String.raw`<script>export default {read: () => \u005fimports_0}</script>` + asset,
    1,
    "global",
  ],
  [
    "template-for-in",
    "<template><img v-for='_imports_0 in items' :key='_imports_0' src='./a.svg' :title='_imports_0'></template>",
    1,
    "for",
  ],
  [
    "template-for-of",
    "<template><img v-for='_imports_0 of items' :key='_imports_0' src='./a.svg' :title='_imports_0'></template>",
    1,
    "for",
  ],
  [
    "template-for-pattern",
    "<template><img v-for='({label: _imports_0}, i) of items' :key='i' src='./a.svg' :title='_imports_0'></template>",
    1,
    "pattern",
  ],
  [
    "template-slot",
    "<template><Child v-slot='{_imports_0}'><img src='./a.svg' :title='_imports_0'></Child></template>",
    1,
    "slot",
  ],
  [
    "template-slot-entity",
    "<template><Child v-slot='{value: &#95;imports_0}'><img src='./a.svg' :title='_imports_0'></Child></template>",
    1,
    "slot-entity",
  ],
  [
    "template-callback",
    "<template><Child :render=\"_imports_0 => h('img', {src: './a.svg', title: _imports_0})\"><img src=\"./a.svg\"></Child></template>",
    1,
  ],
  ["template-handler", "<template><img src='./a.svg' @click='_imports_0++; go()'></template>", 1],
  ["template-context", "<template><img src='./a.svg'>{{ _imports_0 }}</template>", 1],
  [
    "property-negative",
    "<script>const obj = {_imports_0: 'authored'}; export default {value: obj._imports_0}</script>" +
      asset,
    0,
    "value",
  ],
  [
    "string-negative",
    "<script>const value = '_imports_0'; const sameUrl = './a.svg'; export default {value, sameUrl}</script>" +
      asset,
    0,
    "string",
  ],
  ["comment-negative", "<script>/* _imports_0 */ export default {}</script>" + asset, 0],
  [
    "nested-local-negative",
    "<script>function read(_imports_0) { return _imports_0 }; export default {value: read('authored')}</script>" +
      asset,
    0,
    "value",
  ],
  [
    "leading-zero-negative",
    "<script>const _imports_00 = 'authored'; export default {value: _imports_00}</script>" + asset,
    0,
    "value",
  ],
  [
    "template-property-negative",
    "<template><img src='./a.svg' :title='obj._imports_0'></template>",
    0,
  ],
  ["ordinary", "<script>export default {value: '日本語😀'}</script>" + asset, 0],
  [
    "urls-order",
    "<script>const _imports_0 = 0, _imports_2 = 2; export default {}</script><template><img src='./a.svg?raw#x'><image href='./b.svg#home'/><img src='./a.svg?raw#x'></template>",
    1,
  ],
];

function images(node, output = []) {
  if (!node || typeof node !== "object") return output;
  if (node.type === "img") output.push(node);
  if (Array.isArray(node.children)) for (const child of node.children) images(child, output);
  return output;
}

for (const [name, original, expected, semantic] of cases) {
  for (const newline of ["LF", "CRLF"]) {
    const source = original
      .replaceAll("><", ">\n<")
      .replaceAll("\n", newline === "LF" ? "\n" : "\r\n");
    const options = { filename: `${name}.vue`, sourceMap: true };
    const result = native.compileSfc(source, options);
    const assets = native.collectSfcTemplateAssetUrls(source, undefined, options.filename);
    const rewritten = native.rewriteSfcTemplateAssetReferences(result.code, assets);
    const imports = assets
      .map(
        (row) =>
          `import ${row.varName} from ${JSON.stringify("data:text/javascript," + encodeURIComponent(`export default ${JSON.stringify("asset:" + row.url.split("#")[0])}`))};`,
      )
      .join("\n");
    const assembled = imports + "\n" + rewritten;
    const file = path.join(moduleRoot, `${name}-${newline}.mjs`);
    fs.writeFileSync(file, assembled);
    const check = spawnSync(process.execPath, ["--check", file], { encoding: "utf8" });
    const packet = {
      name,
      newline,
      source,
      sourceSha256: hash(source),
      options,
      result,
      assets,
      rewritten,
      assembled,
      check: {
        status: check.status,
        signal: check.signal,
        stdout: check.stdout,
        stderr: check.stderr,
      },
    };
    packets.push(packet);
    assert.deepEqual(result.errors, [], `${name} compilation`);
    if (epoch === "head") {
      assert.equal(assets[0].varName, `_imports_${expected}`, name);
      assert.equal(check.status, 0, check.stderr);
    }
    if (check.status === 0) {
      globalThis._imports_0 = "authored-global";
      try {
        const compiled = await import(pathToFileURL(file));
        const component = compiled.default ?? { render: compiled.render };
        if (semantic === "value") assert.equal(component.value, "authored");
        if (semantic === "string") {
          assert.equal(component.value, "_imports_0");
          assert.equal(component.sameUrl, "./a.svg", "normal-script string ownership");
        }
        if (semantic === "global") {
          packet.observedFreeValue = component.read();
          if (epoch === "head") assert.equal(packet.observedFreeValue, "authored-global");
        }
        if (semantic === "for" || semantic === "pattern") {
          const items =
            semantic === "for" ? ["authored-iteration"] : [{ label: "authored-iteration" }];
          const nodes = images(component.render({ items }, []));
          packet.observedImages = nodes.map((node) => node.props);
          if (epoch === "head") {
            assert.equal(nodes.length, 1);
            assert.equal(nodes[0].props.src, "asset:./a.svg");
            assert.equal(nodes[0].props.title, "authored-iteration");
          }
        }
        if (semantic === "slot" || semantic === "slot-entity") {
          const props =
            semantic === "slot" ? { _imports_0: "authored-slot" } : { value: "authored-slot" };
          const app = createSSRApp(component);
          app.component("Child", {
            setup(_props, { slots }) {
              return () => createVNode("div", null, slots.default(props));
            },
          });
          packet.observedHtml = await renderToString(app);
          if (epoch === "head") {
            assert.ok(packet.observedHtml.includes('src="asset:./a.svg"'), packet.observedHtml);
            assert.ok(packet.observedHtml.includes('title="authored-slot"'), packet.observedHtml);
          }
        }
        if (result.map) {
          const map = JSON.parse(result.map);
          assert.deepEqual(map.sources, [options.filename]);
          assert.deepEqual(map.sourcesContent, [source]);
        }
        packet.execution = "completed";
      } finally {
        delete globalThis._imports_0;
      }
    }
  }
}
fs.writeFileSync(path.join(output, `${epoch}-native.json`), JSON.stringify(report, null, 2) + "\n");
console.log(
  JSON.stringify({
    epoch,
    cases: packets.length,
    native: report.native,
    syntaxFailures: packets
      .filter((row) => row.check.status !== 0)
      .map((row) => row.name + ":" + row.newline),
  }),
);
