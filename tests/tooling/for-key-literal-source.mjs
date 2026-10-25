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
const fixtures = [
  ["escaped-string", String.raw`\u005frow`, "capture('_for_item0.value')"],
  ["escaped-template", String.raw`\u005frow`, "capture(`_for_item0.value`)"],
  ["ordinary-string-control", "row", "capture('_for_item0.value')"],
];
for (const epoch of ["base", "head"]) {
  const receipt = JSON.parse(fs.readFileSync(path.join(providers, `${epoch}-native.json`), "utf8"));
  const image = path.join(providers, `${epoch}.node`);
  const native = require(image);
  assert.equal(hash(fs.readFileSync(image)), receipt.native.sha256);
  assert.equal(native.getNativeBuildToken(), receipt.native.token);
  for (const [id, alias, key] of fixtures)
    for (const newline of ["LF", "CRLF"]) {
      const source =
        `<script>export default {}</script>\n<template><p v-for="${alias} of items" :key="${key}">ok</p></template>`.replaceAll(
          "\n",
          newline === "LF" ? "\n" : "\r\n",
        );
      const options = { filename: `${id}.vue`, vapor: true, sourceMap: true };
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
        result,
        calls: [],
        warnings: [],
        expectedArgument: "_for_item0.value",
      };
      report.rows.push(row);
      assert.deepEqual(result.errors, []);
      assert.deepEqual(JSON.parse(result.map).sourcesContent, [source]);
      const file = path.join(output, `${epoch}-${id}-${newline}.mjs`);
      fs.writeFileSync(file, result.code);
      const compiled = await import(pathToFileURL(file));
      const component = vue.defineVaporComponent({
        render: compiled.default.render,
        props: ["items", "capture"],
        inheritAttrs: false,
        setup: (props) => ({
          get items() {
            return props.items;
          },
          get capture() {
            return props.capture;
          },
        }),
      });
      const app = vue.createVaporApp(component, {
        items: ["A"],
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
    .filter((row) => row.epoch === "base" && row.id.startsWith("escaped"))
    .every((row) => row.matches),
  "actual before source must retain escaped callback literals",
);
assert.ok(
  report.rows
    .filter((row) => row.epoch === "head" && row.id.startsWith("escaped"))
    .every((row) => row.matches),
  "source head must preserve authored callback literals",
);
