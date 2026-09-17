import assert from "node:assert/strict";
import { transformSync } from "@babel/core";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.ok(["vdom", "vapor"].includes(input.backend));
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

const vue = await loadRuntime();
const events = [];
const state = vue.reactive({ label: "initial", ready: true, ...input.context });
const record = (value) => events.push(value);
const fixture = { state, record, makeHandler: () => record, handlers: { "x;y": record } };
globalThis.__vaporSfcModules = { vue, "./fixture": fixture };

async function evaluate(code) {
  const compiled = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }) => ({
        visitor: {
          ImportDeclaration(path) {
            const source = path.node.source.value;
            assert.ok(Object.hasOwn(globalThis.__vaporSfcModules, source), source);
            const declarations = path.node.specifiers.map((specifier) => {
              const imported = t.isImportDefaultSpecifier(specifier)
                ? "default"
                : specifier.imported.name;
              assert.ok(Object.hasOwn(globalThis.__vaporSfcModules[source], imported), imported);
              return t.variableDeclarator(
                specifier.local,
                t.memberExpression(
                  t.memberExpression(t.identifier("globalThis"), t.identifier("__vaporSfcModules")),
                  t.stringLiteral(source),
                  true,
                ),
              );
            });
            for (let i = 0; i < declarations.length; i++) {
              const specifier = path.node.specifiers[i];
              const imported = t.isImportDefaultSpecifier(specifier)
                ? "default"
                : specifier.imported.name;
              declarations[i].init = t.memberExpression(
                declarations[i].init,
                t.stringLiteral(imported),
                true,
              );
            }
            path.replaceWith(t.variableDeclaration("const", declarations));
          },
        },
      }),
    ],
  }).code;
  return (await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`))
    .default;
}

const child = await evaluate(input.child);
globalThis.__vaporSfcModules["./Child.vue"] = { default: child };
const component = await evaluate(input.code);
if (input.scopedSelectors) component.__scopeId = "data-v-probe";
const app = (input.backend === "vapor" ? vue.createVaporApp : vue.createApp)(component);
const diagnostics = [];
app.config.warnHandler = (message) => diagnostics.push(message);
app.config.errorHandler = (error) => diagnostics.push(String(error));
const host = window.document.createElement("div");
window.document.body.append(host);
const snapshots = [];
const snapshot = () => {
  assert.deepEqual(diagnostics, [], "mounted SFC diagnostics");
  for (const selector of input.scopedSelectors ?? []) {
    const elements = host.querySelectorAll(selector);
    assert.ok(elements.length, `missing scoped selector target: ${selector}`);
    for (const element of elements)
      assert.ok(element.hasAttribute("data-v-probe"), element.outerHTML);
  }
  snapshots.push({
    tree: observeChildren(host),
    namespaces: [...host.querySelectorAll("*")].map((element) => element.namespaceURI),
    events: [...events],
  });
};
try {
  app.mount(host);
  await vue.nextTick();
  snapshot();
  for (const step of input.steps ?? []) {
    const retained = (step.preserve ?? []).map((selector) => {
      const node = host.querySelector(selector);
      assert.ok(node, `missing identity target: ${selector}`);
      return [selector, node];
    });
    if (step.patch) Object.assign(state, step.patch);
    else {
      const target = host.querySelector(step.click);
      assert.ok(target, `missing click target: ${step.click}`);
      target.click();
    }
    await vue.nextTick();
    for (const [selector, node] of retained)
      assert.equal(host.querySelector(selector), node, selector);
    snapshot();
  }
  const detachedButtons = [...host.querySelectorAll("button")];
  app.unmount();
  await vue.nextTick();
  assert.equal(host.childNodes.length, 0, "SFC unmount left nodes behind");
  assert.deepEqual(diagnostics, []);
  const settledEvents = [...events];
  const settledState = JSON.stringify(state);
  for (const button of detachedButtons) button.click();
  await vue.nextTick();
  assert.deepEqual(events, settledEvents, "unmounted component still emitted events");
  assert.equal(JSON.stringify(state), settledState, "unmounted component still updated state");
  snapshots.push({ tree: [], namespaces: [], events: [...events] });
  process.stdout.write(JSON.stringify(snapshots));
} finally {
  if (host.childNodes.length) app.unmount();
  delete globalThis.__vaporSfcModules;
  host.remove();
  await window.happyDOM.close();
}
