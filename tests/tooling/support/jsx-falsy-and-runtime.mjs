// Execute actual compiled JSX with real Vue, Babel, DOM updates and hydration.
import assert from "node:assert/strict";
import { writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { compileFunction } from "node:vm";
import { transformSync } from "@babel/core";
import jsx from "@vue/babel-plugin-jsx";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";
import { vueVaporVersion } from "./vue-vapor-release.mjs";

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
const Vue = await loadRuntime();
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const SSR = fromVue("vue/server-renderer");
const compilerVersions = {
  vue: vueVaporVersion,
  babel: createRequire(import.meta.url)("@babel/core/package.json").version,
  babelJsx: createRequire(import.meta.url)("@vue/babel-plugin-jsx/package.json").version,
};

function executable(code, name, read, rows = []) {
  // Rewrite only module linkage, using Babel's real parser. Generated
  // expressions, comments, whitespace and runtime behavior stay untouched.
  const body = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const module = path.node.source.value;
            assert.ok(["vue", "@vue/server-renderer"].includes(module), module);
            const properties = path.node.specifiers.map((specifier) => {
              assert.equal(specifier.type, "ImportSpecifier");
              return t.objectProperty(specifier.imported, specifier.local);
            });
            path.replaceWith(
              t.variableDeclaration("const", [
                t.variableDeclarator(
                  t.objectPattern(properties),
                  t.identifier(module === "vue" ? "Vue" : "SSR"),
                ),
              ]),
            );
          },
          ExportNamedDeclaration(path) {
            assert.ok(path.node.declaration, "no export forwarding in these fixtures");
            path.replaceWith(path.node.declaration);
          },
        },
      }),
    ],
  }).code;
  return compileFunction(`${body}\nreturn ${name};`, ["Vue", "SSR", "read", "rows"])(
    Vue,
    SSR,
    read,
    rows,
  );
}

function oracle(source, read, rows) {
  const code = transformSync(source, {
    configFile: false,
    babelrc: false,
    plugins: [jsx],
    filename: "input.jsx",
  }).code;
  return { code, render: executable(code, "App", read, rows) };
}

const values = [
  ["zero", 0],
  ["negative-zero", -0],
  ["empty-string", ""],
  ["nan", NaN],
  ["null", null],
  ["undefined", undefined],
  ["false", false],
  ["true", true],
  ["array", []],
  ["nested-vnode", Vue.h("i", null, [Vue.h("b", null, "nested")])],
  ["boxed-false", new Boolean(false)],
  ["zero-bigint", 0n],
  ["one", 1],
  ["two", 2],
];
const receipt = {
  compilerVersions,
  fixtures,
  observations: [],
  nativeBytePair: "pending",
  failures: 0,
};
process.on("uncaughtExceptionMonitor", (error) => {
  receipt.failures += 1;
  receipt.exception = { message: error.message, stack: error.stack };
});
if (process.env.VIZE_JSX_FALSY_RECEIPT)
  process.on("exit", () => {
    writeFileSync(process.env.VIZE_JSX_FALSY_RECEIPT, `${JSON.stringify(receipt, null, 2)}\n`);
  });
function host() {
  const node = window.document.createElement("div");
  window.document.body.append(node);
  return node;
}
function mount(backend, render, node, warnings) {
  const component =
    backend === "vapor"
      ? Vue.defineVaporComponent({ setup: () => render({}) })
      : { setup: () => () => render({}, []) };
  const app = (backend === "vapor" ? Vue.createVaporApp : Vue.createApp)(component);
  app.config.warnHandler = (message) => warnings.push(message);
  app.config.errorHandler = (error) => warnings.push(String(error));
  app.mount(node);
  return app;
}

for (const [id, fixture] of Object.entries(fixtures).filter(([id]) => id !== "loop")) {
  const state = Vue.shallowReactive({ value: 0 });
  for (const backend of ["vdom", "vapor"]) {
    let calls = 0;
    const warnings = [];
    const read = () => {
      calls += 1;
      return state.value;
    };
    const render = executable(
      fixture.outputs[backend].code,
      fixture.outputs[backend].functionName,
      read,
    );
    const node = host();
    state.value = 0;
    const app = mount(backend, render, node, warnings);
    await Vue.nextTick();
    assert.equal(calls, 1, `${id}/${backend}: one initial evaluation`);
    let previousSpan = null;
    let previousTruthy = false;
    for (const [label, value] of values) {
      const before = calls;
      const changed = !Object.is(state.value, value);
      state.value = value;
      await Vue.nextTick();
      const delta = calls - before;
      // Initial zero was evaluated at mount; every distinct update is read
      // once. Falsy-to-falsy updates must also update the displayed value.
      assert.equal(delta, changed ? 1 : 0, `${id}/${backend}/${label}: source reads`);
      const reference = oracle(fixture.source, () => value);
      const referenceHost = host();
      const referenceWarnings = [];
      const referenceApp = mount("vdom", reference.render, referenceHost, referenceWarnings);
      assert.deepEqual(
        observeChildren(node),
        observeChildren(referenceHost),
        `${id}/${backend}/${label}`,
      );
      assert.deepEqual(referenceWarnings, []);
      const span = node.querySelector("span") ?? node.querySelector("section");
      if (previousTruthy && Boolean(value))
        assert.equal(span, previousSpan, "truthy update must retain RHS identity");
      previousTruthy = Boolean(value);
      previousSpan = span;
      receipt.observations.push({
        id,
        backend,
        label,
        html: node.innerHTML,
        tree: observeChildren(node),
        calls,
        updateCalls: delta,
        referenceHtml: referenceHost.innerHTML,
        referenceCode: reference.code,
        warnings: [...warnings],
      });
      referenceApp.unmount();
      referenceHost.remove();
    }
    assert.deepEqual(warnings, []);
    app.unmount();
    assert.equal(node.childNodes.length, 0);
    node.remove();
  }

  // Server HTML is kept raw and hydrated by the corresponding client code.
  // Comparing visible trees never replaces the separate warning/identity gate.
  for (const [label, value] of values) {
    let calls = 0;
    const read = () => {
      calls += 1;
      return value;
    };
    const server = executable(fixture.outputs.ssr.code, fixture.outputs.ssr.functionName, read);
    const pushes = [];
    server({}, (html) => pushes.push(html), null, {});
    const html = pushes.join("");
    assert.equal(calls, 1, "one server evaluation");
    const node = host();
    node.innerHTML = html;
    const originalRoot = node.firstElementChild;
    const warnings = [];
    const client = executable(fixture.outputs.vdom.code, fixture.outputs.vdom.functionName, read);
    const app = Vue.createSSRApp({ setup: () => () => client({}, []) });
    app.config.warnHandler = (message) => warnings.push(message);
    app.config.errorHandler = (error) => warnings.push(String(error));
    app.mount(node);
    await Vue.nextTick();
    receipt.observations.push({
      id,
      backend: "hydrate",
      label,
      serverHtml: html,
      hydratedHtml: node.innerHTML,
      tree: observeChildren(node),
      calls,
      warnings,
    });
    assert.deepEqual(warnings, [], `${id}/${label} hydration`);
    assert.equal(node.firstElementChild, originalRoot, "hydration retains the authored root");
    assert.equal(calls, 2, "one client hydration evaluation");
    const referenceHost = host();
    const reference = oracle(fixture.source, () => value);
    const referenceApp = mount("vdom", reference.render, referenceHost, []);
    assert.deepEqual(observeChildren(node), observeChildren(referenceHost));
    referenceApp.unmount();
    referenceHost.remove();
    app.unmount();
    node.remove();
  }
}

for (const backend of ["vdom", "vapor"]) {
  const rows = Vue.reactive([
    { id: "a", value: 0 },
    { id: "b", value: 1 },
  ]);
  const code = fixtures.loop.outputs[backend].code;
  const render = executable(
    code,
    fixtures.loop.outputs[backend].functionName,
    () => assert.fail("loop never calls read"),
    rows,
  );
  const node = host();
  const warnings = [];
  const app = mount(backend, render, node, warnings);
  const first = node.querySelector('[data-id="a"]');
  const second = node.querySelector('[data-id="b"]');
  assert.equal(first.textContent, "0");
  assert.equal(second.textContent, "X");
  rows[0].value = NaN;
  rows[1].value = 2;
  await Vue.nextTick();
  assert.equal(node.querySelector('[data-id="a"]'), first);
  assert.equal(node.querySelector('[data-id="b"]'), second);
  assert.equal(first.textContent, "NaN");
  rows.reverse();
  await Vue.nextTick();
  assert.equal(node.querySelectorAll("[data-id]")[0], second);
  assert.equal(node.querySelectorAll("[data-id]")[1], first);
  assert.deepEqual(warnings, []);
  receipt.observations.push({
    id: "loop",
    backend,
    html: node.innerHTML,
    tree: observeChildren(node),
    warnings,
  });
  app.unmount();
  node.remove();
}
await window.happyDOM.abort();
console.log(JSON.stringify(receipt));
