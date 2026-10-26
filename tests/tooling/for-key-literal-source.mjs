// Actual source-produced native images and the pinned genuine Vapor runtime.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire, registerHooks } from "node:module";
import { pathToFileURL } from "node:url";
import { inspect } from "node:util";
import { vueVaporBrowserRuntime, vueVaporVersion } from "./support/vue-vapor-release.mjs";
const require = createRequire(import.meta.url);
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const [providers, output] = process.argv.slice(2);
fs.mkdirSync(output, { recursive: true });
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const { Window } = await import(ui.resolve("happy-dom"));
const window = new Window({ url: "http://localhost/" });
for (const key of [
  "window",
  "document",
  "Document",
  "Node",
  "Text",
  "Comment",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Event",
  "ShadowRoot",
])
  globalThis[key] = key === "window" ? window : window[key];
const runtimeBytes = fs.readFileSync(vueVaporBrowserRuntime);
const runtimeUrl = `data:text/javascript;base64,${runtimeBytes.toString("base64")}`;
const vue = await import(runtimeUrl);
registerHooks({
  resolve(specifier, context, next) {
    return specifier === "vue" ? { url: runtimeUrl, shortCircuit: true } : next(specifier, context);
  },
});
const report = {
  node: process.version,
  runtime: { version: vueVaporVersion, sha256: hash(runtimeBytes), file: vueVaporBrowserRuntime },
  rows: [],
};
const marker = "_for_item0.value";
const fixtures = [
  ["escaped-string", String.raw`\u005frow`, "capture('_for_item0.value')", marker],
  ["escaped-template", String.raw`\u005frow`, "capture(`_for_item0.value`)", marker],
  ["escaped-regex", String.raw`\u005frow`, "capture(/_for_item0.value/.source)", marker],
  [
    "escaped-comment",
    String.raw`\u005frow`,
    "capture(/* _for_item0.value */ '_for_item0.value')",
    marker,
  ],
  [
    "escaped-property",
    String.raw`\u005frow`,
    "capture(holder._for_item0.value)",
    "authored-property",
  ],
  [
    "escaped-static-property",
    String.raw`\u005frow`,
    "capture(({_for_item0: {value: 'authored-property'}})._for_item0.value)",
    "authored-property",
  ],
  [
    "callback-local-shadow",
    String.raw`\u005frow`,
    "(_row => capture(_row + '_for_item0.value'))('shadow')",
    "shadow_for_item0.value",
  ],
  ["ordinary-string-control", "row", "capture('_for_item0.value')", marker],
  [
    "escaped-key-index-literals",
    String.raw`(row, \u005fkey, \u005findex)`,
    "capture('_for_key0.value:_for_index0.value')",
    "_for_key0.value:_for_index0.value",
  ],
  [
    "callback-member-shadow",
    String.raw`\u005frow`,
    "(_for_item0 => capture(_for_item0.value))({value: 'shadow'})",
    "shadow",
  ],
  ["outer-loop", null, null, "outer:A"],
];
for (const epoch of ["base", "head"]) {
  const receipt = JSON.parse(fs.readFileSync(path.join(providers, `${epoch}-native.json`), "utf8"));
  const image = path.join(providers, `${epoch}.node`);
  const native = require(image);
  assert.equal(hash(fs.readFileSync(image)), receipt.native.sha256);
  assert.equal(native.getNativeBuildToken(), receipt.native.token);
  for (const [id, alias, key, expected] of fixtures)
    for (const newline of ["LF", "CRLF"]) {
      const body =
        id === "outer-loop"
          ? String.raw`<div v-for="\u005fouter of groups" :key="_outer.id"><p v-for="\u005frow of _outer.items" :key="capture(_outer.id + ':' + _row)">ok</p></div>`
          : `<p v-for="${alias} of items" :key="${key}">ok</p>`;
      const source = `<script>export default {}</script>\n<template>${body}</template>`.replaceAll(
        "\n",
        newline === "LF" ? "\n" : "\r\n",
      );
      const options = { filename: `${id}.vue`, vapor: true, sourceMap: true };
      const runtimeInputs = {
        items: ["A"],
        groups: [{ id: "outer", items: ["A"] }],
        holder: { _for_item0: { value: "authored-property" } },
      };
      const result = native.compileSfc(source, options);
      const row = {
        epoch,
        sourceSha: receipt.sourceSha,
        image: { file: image, sha256: receipt.native.sha256, token: receipt.native.token },
        id,
        newline,
        source,
        sourceSha256: hash(source),
        options,
        runtimeInputs,
        runtimeInputsSha256: hash(JSON.stringify(runtimeInputs)),
        callbackContract: "Record each argument and return that same value",
        result,
        calls: [],
        warnings: [],
        expectedArgument: expected,
      };
      report.rows.push(row);
      assert.deepEqual(result.errors, []);
      assert.deepEqual(JSON.parse(result.map).sourcesContent, [source]);
      const file = path.join(output, `${epoch}-${id}-${newline}.mjs`);
      fs.writeFileSync(file, result.code);
      const compiled = await import(pathToFileURL(file));
      const component = vue.defineVaporComponent({
        render: compiled.default.render,
        props: ["items", "groups", "holder", "capture"],
        inheritAttrs: false,
        setup: (props) => ({
          get items() {
            return props.items;
          },
          get groups() {
            return props.groups;
          },
          get holder() {
            return props.holder;
          },
          get capture() {
            return props.capture;
          },
        }),
      });
      const app = vue.createVaporApp(component, {
        ...runtimeInputs,
        capture: () => (value) => {
          row.calls.push(value);
          return value;
        },
      });
      app.config.warnHandler = (warning) => row.warnings.push(warning);
      const host = window.document.createElement("div");
      window.document.body.append(host);
      try {
        app.mount(host);
        await vue.nextTick();
        row.html = host.innerHTML;
        assert.equal(host.textContent, "ok");
      } catch (error) {
        row.runtimeError = { name: error.name, message: error.message, stack: error.stack };
      } finally {
        app.unmount();
        host.remove();
      }
      row.callsText = inspect(row.calls, { depth: null, colors: false });
      row.matches =
        !row.runtimeError &&
        row.calls.length > 0 &&
        row.calls.every((value) => value === row.expectedArgument);
    }
}
fs.writeFileSync(path.join(output, "for-key-literal.json"), JSON.stringify(report, null, 2) + "\n");
for (const after of report.rows.filter((row) => row.epoch === "head")) {
  const before = report.rows.find(
    (row) => row.epoch === "base" && row.id === after.id && row.newline === after.newline,
  );
  assert.equal(after.source, before.source);
  assert.deepEqual(after.options, before.options);
  assert.deepEqual(after.runtimeInputs, before.runtimeInputs);
  assert.equal(after.runtimeInputsSha256, before.runtimeInputsSha256);
}
console.log(
  JSON.stringify(
    report.rows.map(({ epoch, id, newline, calls, matches }) => ({
      epoch,
      id,
      newline,
      calls,
      matches,
    })),
  ),
);
assert.ok(
  report.rows
    .filter(
      (row) => row.epoch === "base" && ["escaped-string", "escaped-template"].includes(row.id),
    )
    .every((row) => row.matches),
  "actual before source must retain escaped callback literals",
);
assert.ok(
  report.rows.filter((row) => row.epoch === "head").every((row) => row.matches),
  "source head must preserve authored callback literals",
);
