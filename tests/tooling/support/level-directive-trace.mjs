import assert from "node:assert/strict";
import process from "node:process";
import { Window } from "happy-dom";
import { evaluateCompiledRender } from "./davinci-runtime-trace.mjs";
import { childComponent, loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";
import { officialCompilerVapor } from "./vue-vapor-release.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.ok(Array.isArray(input.steps) && input.steps.length > 0);
assert.ok(Array.isArray(input.directives) && input.directives.length > 0);
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
const compile = (source) =>
  officialCompilerVapor.compile(source, { mode: "module", prefixIdentifiers: true }).code;
const code = input.code ?? compile(input.source);
const render = await evaluateCompiledRender(code, vue);
const state = vue.reactive(input.context ?? {});
const app = vue.createVaporApp(vue.defineVaporComponent({ setup: () => render(state) }));
for (const [name, child] of Object.entries(input.components ?? {}))
  app.component(
    name,
    await childComponent("vapor", vue, name, {
      ...child,
      code: child.code ?? compile(child.source),
    }),
  );
const hooks = [];
for (const name of input.directives) {
  assert.match(name, /^[A-Za-z][A-Za-z0-9-]*$/u);
  app.directive(name, (element, getValue, getArgument, modifiers = {}) => {
    const observe = (phase) =>
      hooks.push({
        name,
        phase,
        tag: element.localName,
        value: getValue?.() ?? null,
        arg: getArgument?.() ?? null,
        modifiers: { ...modifiers },
      });
    observe("setup");
    vue.watchPostEffect(() => observe("effect"));
    return () => observe("cleanup");
  });
}
const diagnostics = [];
app.config.warnHandler = (message) => diagnostics.push(message);
app.config.errorHandler = (error) => diagnostics.push(String(error));
const host = window.document.createElement("div");
window.document.body.append(host);
const snapshots = [];
const snapshot = () => {
  assert.deepEqual(diagnostics, [], "directive runtime diagnostics");
  snapshots.push(JSON.parse(JSON.stringify({ tree: observeChildren(host), hooks })));
};
try {
  app.mount(host);
  await vue.nextTick();
  snapshot();
  for (const step of input.steps) {
    assert.ok(step.patch && Object.keys(step).length === 1);
    Object.assign(state, step.patch);
    await vue.nextTick();
    snapshot();
  }
  app.unmount();
  await vue.nextTick();
  assert.equal(host.childNodes.length, 0, "unmount left directive DOM behind");
  snapshot();
  process.stdout.write(`${JSON.stringify(snapshots)}\n`);
} finally {
  if (host.childNodes.length) app.unmount();
  host.remove();
  await window.happyDOM.close();
}
