// Execute complete compiled spread-child modules (#6888) against the real
// `@vue/babel-plugin-jsx` output, through mount, updates, SSR and hydration.
//
// Vize spreads into its own BAIL Fragment block, so server HTML carries that
// fragment's `<!--[-->`/`<!--]-->` hydration markers where Babel's flat child
// array has none. Markers are compared with those two comments removed;
// everything else (text, elements, reads, iterations, keyed identity,
// warnings, TypeErrors) must match exactly.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, symlinkSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { transformSync } from "@babel/core";
import jsx from "@vue/babel-plugin-jsx";
import tsSyntax from "@babel/plugin-syntax-typescript";
import { Window } from "happy-dom";

let payload = "";
for await (const chunk of process.stdin) payload += chunk;
const fixtures = JSON.parse(payload);
const window = new Window();
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
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const Vue = fromUi("vue");
const { renderToString } = fromUi("vue/server-renderer");
const workspace = mkdtempSync(join(tmpdir(), "vize-jsx-spread-runtime-"));
const versions = {
  vue: fromUi("vue/package.json").version,
  babel: createRequire(import.meta.url)("@babel/core/package.json").version,
  babelJsx: createRequire(import.meta.url)("@vue/babel-plugin-jsx/package.json").version,
};
const stateSource = readFileSync(
  new URL("../../_fixtures/differential/jsx/spread-child-state.mjs", import.meta.url),
  "utf8",
);

function reference(source, lang) {
  const plugins = [jsx];
  if (lang === "tsx") plugins.push([tsSyntax, { isTSX: true }]);
  const code = transformSync(source, {
    plugins,
    babelrc: false,
    configFile: false,
    filename: `fixture.${lang}`,
  }).code;
  // The recorded Babel facet retains its TypeScript syntax. Only this known
  // fixture's type assertions are erased for Node execution, through Babel's
  // actual AST. JSX expressions and all generated code are otherwise intact.
  const runtimeCode =
    lang === "tsx"
      ? transformSync(code, {
          plugins: [
            [tsSyntax, { isTSX: true }],
            () => ({
              visitor: {
                TSAsExpression(path) {
                  path.replaceWith(path.node.expression);
                },
              },
            }),
          ],
          babelrc: false,
          configFile: false,
          filename: "fixture.tsx",
        }).code
      : code;
  return { code, runtimeCode };
}

async function program(id, kind, code) {
  const root = join(workspace, `${id}-${kind}`);
  mkdirSync(join(root, "node_modules"), { recursive: true });
  symlinkSync(dirname(fromUi.resolve("vue/package.json")), join(root, "node_modules/vue"));
  writeFileSync(join(root, "spread-child-state.mjs"), stateSource);
  writeFileSync(join(root, "compiled.mjs"), code);
  return {
    ...(await import(pathToFileURL(join(root, "compiled.mjs")).href)),
    ...(await import(pathToFileURL(join(root, "spread-child-state.mjs")).href)),
  };
}

function iterable(module, values) {
  return {
    [Symbol.iterator]() {
      module.stats.iterations += 1;
      return values[Symbol.iterator]();
    },
  };
}

async function observe(module) {
  const warnings = [];
  const host = window.document.createElement("div");
  const app = Vue.createApp(module.App);
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => {
    throw error;
  };
  module.state.value = iterable(module, []);
  app.mount(host);
  assert.equal(module.stats.reads, 1);
  assert.equal(module.stats.iterations, 1);
  const phases = [];
  let retained = null;
  const values = [
    [
      "mixed",
      () => [
        0,
        -0,
        NaN,
        false,
        null,
        undefined,
        "",
        ["nested"],
        Vue.h("em", null, [Vue.h("strong", null, "VNode")]),
      ],
    ],
    ["keyed", () => [Vue.h("em", { key: "stable" }, "first"), "tail"]],
    ["reorder", () => ["lead", Vue.h("em", { key: "stable" }, "updated")]],
    ["empty", () => []],
  ];
  for (const [label, value] of values) {
    const reads = module.stats.reads;
    const iterations = module.stats.iterations;
    module.state.value = iterable(module, value());
    await Vue.nextTick();
    assert.equal(module.stats.reads - reads, 1, `${String(label)}: argument read once`);
    assert.equal(
      module.stats.iterations - iterations,
      1,
      `${String(label)}: iterator obtained once`,
    );
    if (label === "keyed") retained = host.querySelector("em");
    if (label === "reorder") assert.equal(host.querySelector("em"), retained);
    phases.push({ label, html: host.innerHTML, reads: 1, iterations: 1 });
  }
  app.unmount();
  assert.deepEqual(warnings, []);

  module.state.value = iterable(module, [0, false, ["nested"], Vue.h("em", null, "SSR")]);
  const server = Vue.createSSRApp(module.App);
  server.config.warnHandler = (message) => warnings.push(message);
  let reads = module.stats.reads;
  const html = await renderToString(server);
  assert.equal(module.stats.reads - reads, 1);
  const hydrationHost = window.document.createElement("div");
  hydrationHost.innerHTML = html;
  const hydration = Vue.createSSRApp(module.App);
  hydration.config.warnHandler = (message) => warnings.push(message);
  reads = module.stats.reads;
  hydration.mount(hydrationHost);
  assert.equal(module.stats.reads - reads, 1);
  const hydrated = hydrationHost.innerHTML;
  module.state.value = iterable(module, ["hydrated update", Vue.h("b", null, "new")]);
  await Vue.nextTick();
  const afterHydration = hydrationHost.innerHTML;
  hydration.unmount();
  assert.deepEqual(warnings, []);

  const invalid = [];
  for (const value of [null, undefined, false, 0, {}]) {
    module.state.value = value;
    reads = module.stats.reads;
    const failing = Vue.createSSRApp(module.App);
    failing.config.warnHandler = () => {};
    failing.config.errorHandler = (error) => {
      throw error;
    };
    await assert.rejects(renderToString(failing), TypeError);
    assert.equal(module.stats.reads - reads, 1);
    invalid.push({ type: value === null ? "null" : typeof value, error: "TypeError", reads: 1 });
  }
  return { phases, html, hydrated, afterHydration, warnings, keyedRetention: true, invalid };
}

const unmark = (html) => html.replaceAll("<!--[-->", "").replaceAll("<!--]-->", "");
function comparable(observation) {
  return {
    ...observation,
    html: unmark(observation.html),
    hydrated: unmark(observation.hydrated),
    afterHydration: unmark(observation.afterHydration),
  };
}

const observations = {};
try {
  for (const [id, fixture] of Object.entries(fixtures)) {
    const babel = reference(fixture.source, fixture.lang);
    const expected = await observe(await program(id, "reference", babel.runtimeCode));
    assert.deepEqual(fixture.diagnostics, []);
    const actual = await observe(await program(id, "native", fixture.code));
    assert.deepEqual(comparable(actual), comparable(expected), `${id}: runtime observations`);
    observations[id] = actual;
  }
  process.stdout.write(JSON.stringify({ versions, observations, failures: 0 }));
} finally {
  await window.happyDOM.close();
  rmSync(workspace, { recursive: true, force: true });
}
