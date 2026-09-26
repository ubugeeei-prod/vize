import assert from "node:assert/strict";
import process from "node:process";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";
import { evaluateCompiledRender } from "./davinci-runtime-trace.mjs";
import { officialCompilerVapor, vueVaporVersion } from "./vue-vapor-release.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(vueVaporVersion, "3.6.0-rc.9");
assert.ok(
  [
    "single",
    "cancel",
    "static",
    "group",
    "group-div",
    "nested-static",
    "nested-single",
    "nested-cancel",
    "group-nested",
  ].includes(input.scenario),
);
const compile = (source) =>
  officialCompilerVapor.compile(source, { mode: "module", prefixIdentifiers: true }).code;
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
const render = await evaluateCompiledRender(input.code ?? compile(input.source), vue);
const events = [];
const pending = [];
const diagnostics = [];
const hook = (phase) => (element) => events.push([phase, element.getAttribute("data-id")]);
const heldHook = (phase) => (element, done) => {
  events.push([phase, element.getAttribute("data-id")]);
  pending.push({ phase, element, done });
};
const state = vue.reactive({
  show: true,
  label: "A",
  items: [
    { id: "a", text: "A" },
    { id: "b", text: "B" },
  ],
  beforeEnter: hook("before-enter"),
  enter: heldHook("enter"),
  afterEnter: hook("after-enter"),
  enterCancelled: hook("enter-cancelled"),
  beforeLeave: hook("before-leave"),
  leave: heldHook("leave"),
  afterLeave: hook("after-leave"),
  leaveCancelled: hook("leave-cancelled"),
  send: () => events.push(["click", state.label]),
});
const app = vue.createVaporApp(vue.defineVaporComponent({ setup: () => render(state) }));
app.config.warnHandler = (message) => diagnostics.push(message);
app.config.errorHandler = (error) => diagnostics.push(String(error));
const host = window.document.createElement("div");
window.document.body.append(host);
const snapshots = [];
const identities = new WeakMap();
let nextId = 0;
let mounted = false;
function snapshot() {
  assert.deepEqual(diagnostics, [], "transition runtime diagnostics");
  const nodes = [...host.querySelectorAll("[data-id]")];
  for (const node of nodes) if (!identities.has(node)) identities.set(node, nextId++);
  snapshots.push({
    tree: observeChildren(host),
    events: [...events],
    identities: nodes.map((node) => [node.getAttribute("data-id"), identities.get(node)]),
  });
}
async function complete(phase) {
  const callbacks = pending.filter((entry) => entry.phase === phase);
  assert.ok(callbacks.length, `${phase} hook was called`);
  for (const entry of callbacks) {
    pending.splice(pending.indexOf(entry), 1);
    entry.done();
  }
  await vue.nextTick();
}
try {
  app.mount(host);
  mounted = true;
  await vue.nextTick();
  snapshot();
  if (input.scenario.startsWith("group")) {
    state.items = [
      { id: "b", text: "B2" },
      { id: "c", text: "C" },
    ];
    await vue.nextTick();
    snapshot();
    await complete("leave");
    await complete("enter");
    snapshot();
    state.items = [
      { id: "c", text: "C2" },
      { id: "b", text: "B3" },
    ];
    await vue.nextTick();
    snapshot();
    state.items = [];
    await vue.nextTick();
    snapshot();
    await complete("leave");
    snapshot();
  } else if (input.scenario.endsWith("static")) {
    state.label = "B";
    await vue.nextTick();
    host.querySelector("button").click();
    await vue.nextTick();
    snapshot();
  } else {
    state.show = false;
    await vue.nextTick();
    snapshot();
    await complete("leave");
    snapshot();
    state.show = true;
    await vue.nextTick();
    snapshot();
    state.label = "B";
    await vue.nextTick();
    host.querySelector("button").click();
    await vue.nextTick();
    snapshot();
    if (input.scenario.endsWith("single")) {
      await complete("enter");
      snapshot();
      state.show = false;
      await vue.nextTick();
      snapshot();
    } else {
      state.show = false;
      await vue.nextTick();
      snapshot();
      await complete("enter");
      snapshot();
      await complete("leave");
      snapshot();
    }
  }
  app.unmount();
  mounted = false;
  await vue.nextTick();
  snapshot();
  for (const entry of pending.splice(0)) entry.done();
  await vue.nextTick();
  snapshot();
  assert.equal(host.childNodes.length, 0, "transition unmount left nodes behind");
  process.stdout.write(`${JSON.stringify(snapshots)}\n`);
} finally {
  if (mounted) app.unmount();
  host.remove();
  await window.happyDOM.close();
}
