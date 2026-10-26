import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire, registerHooks } from "node:module";
import { pathToFileURL } from "node:url";
import { inspect } from "node:util";
import { vueVaporBrowserRuntime, vueVaporVersion } from "./support/vue-vapor-release.mjs";

const require = createRequire(import.meta.url);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const [epoch, image, output] = process.argv.slice(2);
assert.ok(epoch === "base" || epoch === "head");
const native = require(path.resolve(image));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
fs.mkdirSync(output, { recursive: true });
const directory = path.join(output, "modules", epoch);
fs.mkdirSync(directory, { recursive: true });
if (!fs.existsSync(path.join(output, "node_modules")))
  fs.symlinkSync(path.resolve("tests/node_modules"), path.join(output, "node_modules"), "dir");
const { Window } = await import(fromUi.resolve("happy-dom"));
const window = new Window({ url: "http://localhost/" });
for (const name of [
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
  globalThis[name] = name === "window" ? window : window[name];
const vue = await import("vue");
const fromVue = createRequire(require.resolve("vue/package.json"));
const { SourceMapConsumer } = createRequire(fromVue.resolve("@vue/compiler-sfc"))("source-map-js");
const ssrEntry = fromVue.resolve("@vue/server-renderer");
const ssr = await import(pathToFileURL(ssrEntry));
registerHooks({
  resolve(specifier, context, next) {
    if (specifier === "@vue/server-renderer")
      return { url: pathToFileURL(ssrEntry).href, shortCircuit: true };
    return next(specifier, context);
  },
});
const records = [];
const report = {
  epoch,
  sourceSha: process.env.SOURCE_SHA,
  node: process.version,
  native: { image, sha256: hash(fs.readFileSync(image)), token: native.getNativeBuildToken() },
  runtimes: {
    ssr: { version: vue.version, entry: ssrEntry, sha256: hash(fs.readFileSync(ssrEntry)) },
    client: {
      version: vueVaporVersion,
      entry: vueVaporBrowserRuntime,
      sha256: hash(fs.readFileSync(vueVaporBrowserRuntime)),
    },
    domHost: {
      package: fromUi.resolve("happy-dom/package.json"),
      version: fromUi("happy-dom/package.json").version,
    },
  },
  records,
};
process.on("exit", () =>
  fs.writeFileSync(
    path.join(output, `${epoch}-native.json`),
    JSON.stringify(report, null, 2) + "\n",
  ),
);

const cases = [
  {
    id: "parameter-default",
    alias: String.raw`\u005frow = 'A'`,
    value: "_row",
    items: [undefined, "B"],
    addition: "C",
    expected: ["A", "B", "C"],
  },
  {
    id: "ordinary",
    alias: "row",
    value: "row",
    items: ["A", "B"],
    addition: "C",
    expected: ["A", "B", "C"],
    unchanged: true,
  },
  {
    id: "ordinary-pattern",
    alias: "{label: row}",
    value: "row",
    items: [{ label: "A" }, { label: "B" }],
    addition: { label: "C" },
    expected: ["A", "B", "C"],
    unchanged: true,
  },
  {
    id: "escaped",
    alias: String.raw`\u005frow`,
    value: "_row",
    items: ["A", "B"],
    addition: "C",
    expected: ["A", "B", "C"],
  },
  {
    id: "braced",
    alias: String.raw`\u{5f}row`,
    value: "_row",
    items: ["A", "B"],
    addition: "C",
    expected: ["A", "B", "C"],
  },
  {
    id: "unicode",
    alias: "行",
    value: "行",
    items: ["A", "B"],
    addition: "C",
    expected: ["A", "B", "C"],
  },
  {
    id: "key",
    alias: String.raw`(row, \u005fkey)`,
    value: "_key + ':' + row",
    items: { alpha: "A", beta: "B" },
    addition: { gamma: "C" },
    expected: ["alpha:A", "beta:B", "gamma:C"],
  },
  {
    id: "index",
    alias: String.raw`(row, key, \u005findex)`,
    value: "_index + ':' + key + ':' + row",
    items: { alpha: "A", beta: "B" },
    addition: { gamma: "C" },
    expected: ["0:alpha:A", "1:beta:B", "2:gamma:C"],
  },
  {
    id: "destructure",
    alias: String.raw`{label: \u005frow}`,
    value: "_row",
    items: [{ label: "A" }, { label: "B" }],
    addition: { label: "C" },
    expected: ["A", "B", "C"],
  },
  {
    id: "array",
    alias: String.raw`[\u005frow]`,
    value: "_row",
    items: [["A"], ["B"]],
    addition: ["C"],
    expected: ["A", "B", "C"],
  },
  {
    id: "default",
    alias: String.raw`{label: \u005frow = 'A'}`,
    value: "_row",
    items: [{}, { label: "B" }],
    addition: { label: "C" },
    expected: ["A", "B", "C"],
  },
  {
    id: "property-negative",
    alias: String.raw`{['\u005frow']: value}`,
    value: "value",
    items: [{ _row: "A" }, { _row: "B" }],
    addition: { _row: "C" },
    expected: ["A", "B", "C"],
  },
  {
    id: "shadowing",
    alias: String.raw`\u005frow`,
    value: "_row",
    items: [["A", "B"]],
    addition: ["C"],
    expected: ["A", "B", "C"],
    nested: true,
  },
];

for (const fixture of cases)
  for (const newline of ["LF", "CRLF"]) {
    const body = fixture.nested
      ? String.raw`<section v-for="\u005frow of items"><p v-for="\u005frow of _row" :key="_row" :title="_row">{{ _row }}</p></section>`
      : `<p v-for="${fixture.alias} in items" :key="${fixture.value}" :title="${fixture.value}">{{ ${fixture.value} }}</p>`;
    const source =
      `<script>export default {}</script>\n<template>\n${body}\n</template>`.replaceAll(
        "\n",
        newline === "LF" ? "\n" : "\r\n",
      );
    for (const backend of ["dom", "ssr", "vapor"]) {
      const options = {
        filename: `${fixture.id}.vue`,
        sourceMap: true,
        ssr: backend === "ssr",
        vapor: backend === "vapor",
      };
      const result = native.compileSfc(source, options);
      const file = path.join(directory, `${fixture.id}-${newline}-${backend}.mjs`);
      fs.writeFileSync(file, result.code);
      const packet = {
        runtimeInputText: inspect(
          {
            items: fixture.items,
            fallback: "A",
            addition: fixture.addition,
            expected: fixture.expected,
          },
          { depth: null, colors: false, sorted: false, compact: false },
        ),
        id: fixture.id,
        newline,
        backend,
        source,
        sourceSha256: hash(source),
        options,
        result,
        unchanged: fixture.unchanged ?? false,
        file,
      };
      records.push(packet);
      assert.deepEqual(result.errors, [], `${fixture.id}:${backend}`);
      assert.ok(result.map, `${fixture.id}:${backend}: actual complete module map`);
      if (result.map) {
        const map = JSON.parse(result.map);
        assert.deepEqual(map.sources, [options.filename]);
        assert.deepEqual(map.sourcesContent, [source]);
        assert.equal(map.version, 3);
        const consumer = new SourceMapConsumer(map);
        packet.mapSegments = [];
        const generated = result.code.split("\n"),
          original = source.split("\n");
        consumer.eachMapping((segment) => {
          assert.ok(segment.generatedLine >= 1 && segment.generatedLine <= generated.length);
          assert.ok(
            segment.generatedColumn >= 0 &&
              segment.generatedColumn <= generated[segment.generatedLine - 1].length,
          );
          if (segment.originalLine !== null) {
            assert.equal(segment.source, options.filename);
            assert.ok(segment.originalLine >= 1 && segment.originalLine <= original.length);
            assert.ok(
              segment.originalColumn >= 0 &&
                segment.originalColumn <= original[segment.originalLine - 1].length,
            );
          }
          packet.mapSegments.push(segment);
        });
        assert.ok(packet.mapSegments.length > 0);
      }
    }
  }

async function observe(packet, run) {
  try {
    await run();
  } catch (error) {
    const trace = error.stack ?? String(error);
    const file = path.join(directory, `${packet.id}-${packet.newline}-${packet.backend}.error.txt`);
    fs.writeFileSync(file, trace);
    packet.runtimeError = {
      name: error.name,
      message: error.message,
      stackFile: file,
      sha256: hash(trace),
    };
    if (epoch === "head") process.exitCode = 1;
  }
}

// SSR uses its actual installed Vue graph. Client modes use the repository's
// pinned genuine dual-renderer browser build under the existing DOM host.
for (const packet of records.filter((row) => row.backend === "ssr")) {
  await observe(packet, async () => {
    const fixture = cases.find((row) => row.id === packet.id);
    const module = await import(pathToFileURL(packet.file));
    const component = { ...module.default, props: ["items", "fallback"], inheritAttrs: false };
    const warnings = [];
    const app = vue.createSSRApp(component, {
      items: structuredClone(fixture.items),
      fallback: "A",
    });
    app.config.warnHandler = (warning) => warnings.push(warning);
    packet.serverHtml = await ssr.renderToString(app);
    packet.warnings = warnings;
    const host = window.document.createElement("div");
    host.innerHTML = packet.serverHtml;
    packet.observed = Array.from(host.querySelectorAll("p"), (element) => ({
      text: element.textContent,
      title: element.getAttribute("title"),
    }));
    if (epoch === "head")
      assert.deepEqual(
        packet.observed,
        fixture.expected.slice(0, -1).map((value) => ({ text: value, title: value })),
        `${packet.id}:ssr`,
      );
  });
}
const browserBytes = fs.readFileSync(vueVaporBrowserRuntime, "utf8");
const browserUrl = `data:text/javascript;base64,${Buffer.from(browserBytes).toString("base64")}`;
const client = await import(browserUrl);
assert.equal(client.version, vueVaporVersion);
registerHooks({
  resolve(specifier, context, next) {
    if (specifier === "vue") return { url: browserUrl, shortCircuit: true };
    return next(specifier, context);
  },
});
for (const packet of records.filter((row) => row.backend !== "ssr")) {
  await observe(packet, async () => {
    const fixture = cases.find((row) => row.id === packet.id);
    const module = await import(pathToFileURL(packet.file));
    const render = module.render ?? module.default?.render;
    assert.equal(typeof render, "function");
    const componentOptions = { render, props: ["items", "fallback"], inheritAttrs: false };
    const component =
      packet.backend === "vapor"
        ? client.defineVaporComponent({
            ...componentOptions,
            // The real Vapor callback receives setup state as its first argument.
            // This host exposes the same authored inputs used by DOM and SSR.
            setup: (props) => ({
              get items() {
                return props.items;
              },
              get fallback() {
                return props.fallback;
              },
            }),
          })
        : componentOptions;
    const state = client.reactive({ items: structuredClone(fixture.items), fallback: "A" });
    const app =
      packet.backend === "vapor"
        ? client.createVaporApp(component, state)
        : client.createApp(component, state);
    const warnings = [];
    app.config.warnHandler = (warning) => warnings.push(warning);
    const host = window.document.createElement("div");
    window.document.body.append(host);
    try {
      app.mount(host);
      await client.nextTick();
      packet.initialHtml = host.innerHTML;
      packet.observed = Array.from(host.querySelectorAll("p"), (element) => ({
        text: element.textContent,
        title: element.getAttribute("title"),
      }));
      if (epoch === "head")
        assert.deepEqual(
          packet.observed,
          fixture.expected.slice(0, -1).map((value) => ({ text: value, title: value })),
          `${packet.id}:${packet.backend}:mount`,
        );
      if (Array.isArray(state.items)) state.items.push(structuredClone(fixture.addition));
      else Object.assign(state.items, structuredClone(fixture.addition));
      await client.nextTick();
      packet.updatedHtml = host.innerHTML;
      packet.updated = Array.from(host.querySelectorAll("p"), (element) => ({
        text: element.textContent,
        title: element.getAttribute("title"),
      }));
      if (epoch === "head")
        assert.deepEqual(
          packet.updated,
          fixture.expected.map((value) => ({ text: value, title: value })),
          `${packet.id}:${packet.backend}:update`,
        );
    } finally {
      packet.warnings = warnings;
      app.unmount();
      host.remove();
    }
  });
}
console.log(
  JSON.stringify({
    epoch,
    packets: records.length,
    sourceSha: report.sourceSha,
    native: report.native,
    runtimes: report.runtimes,
  }),
);
