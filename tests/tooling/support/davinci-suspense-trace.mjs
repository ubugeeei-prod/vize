import assert from "node:assert/strict";
import process from "node:process";
import { Window } from "happy-dom";
import { loadRuntime, observeChildren } from "./davinci-mounted-trace.mjs";
import { evaluateCompiledRender } from "./davinci-runtime-trace.mjs";
import { officialCompilerVapor, vueVaporVersion } from "./vue-vapor-release.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.ok(["resolve", "remove-pending", "unmount-pending"].includes(input.scenario));
assert.equal(vueVaporVersion, "3.6.0-rc.9", "review Suspense contract on runtime upgrades");
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
assert.equal(vue.version, vueVaporVersion);
const render = await evaluateCompiledRender(input.code ?? compile(input.source), vue);
const childRender = await evaluateCompiledRender(
  input.childCode ?? compile(input.childSource),
  vue,
);
const events = [];
const diagnostics = [];
const state = vue.reactive({
  visible: true,
  label: "A",
  waiting: "waiting",
  limit: 0,
  record: (event) => events.push(event),
});
let release;
const gate = new Promise((resolve) => {
  release = resolve;
});
let setupFinished;
const finished = new Promise((resolve) => {
  setupFinished = resolve;
});
const AsyncChild = vue.defineVaporComponent({
  name: "AsyncChild",
  props: ["label"],
  emits: ["send"],
  async setup(props, { emit }) {
    events.push("setup");
    vue.onMounted(() => events.push("mounted"));
    vue.onUnmounted(() => events.push("unmounted"));
    vue.onScopeDispose(() => events.push("disposed"));
    const [promise, restore] = vue.withAsyncContext(() => gate);
    await promise;
    restore();
    try {
      return childRender(
        new Proxy(props, {
          get: (target, key) => (key === "send" ? emit : Reflect.get(target, key)),
        }),
      );
    } finally {
      setupFinished();
    }
  },
});
const app = vue.createVaporApp(vue.defineVaporComponent({ setup: () => render(state) }));
// rc.9's renderer primitive and async Vapor setup require the published bridge.
app.use(vue.vaporInteropPlugin);
app.component("AsyncChild", AsyncChild);
app.config.warnHandler = (message) => diagnostics.push(message);
app.config.errorHandler = (error) => diagnostics.push(String(error));
const host = window.document.createElement("div");
window.document.body.append(host);
const snapshots = [];
const ids = new WeakMap();
let nextId = 0;
let previous = [];
let mounted = false;
function snapshot() {
  assert.deepEqual(diagnostics, [], "Suspense runtime diagnostics");
  const nodes = [...host.querySelectorAll("*")];
  for (const node of previous)
    if (!nodes.includes(node))
      assert.equal(node.isConnected, false, "removed branch remains connected");
  previous = nodes;
  for (const node of nodes) if (!ids.has(node)) ids.set(node, nextId++);
  snapshots.push({
    tree: observeChildren(host),
    events: [...events],
    identities: nodes
      .filter((node) => node.hasAttribute("data-id"))
      .map((node) => [node.getAttribute("data-id"), ids.get(node)]),
  });
}
async function resolveChild() {
  release();
  // Wait for the controlled setup continuation and its runtime Promise handlers.
  await finished;
  await gate;
  await vue.nextTick();
  assert.deepEqual(diagnostics, [], "async setup completion diagnostics");
}
const announcements = [];
const originalInfo = console.info;
console.info = (...args) => announcements.push(args.join(" "));
try {
  app.mount(host);
  mounted = true;
  await vue.nextTick();
  snapshot();
  state.waiting = "still waiting";
  state.label = "B";
  await vue.nextTick();
  snapshot();
  if (input.scenario === "resolve") {
    await resolveChild();
    snapshot();
    state.label = "C";
    await vue.nextTick();
    const button = host.querySelector("button");
    assert.ok(button, "resolved async child is mounted");
    button.click();
    await vue.nextTick();
    snapshot();
    state.visible = false;
    await vue.nextTick();
    snapshot();
  } else if (input.scenario === "remove-pending") {
    state.visible = false;
    await vue.nextTick();
    snapshot();
    await resolveChild();
    snapshot();
  } else {
    app.unmount();
    mounted = false;
    await vue.nextTick();
    snapshot();
    await resolveChild();
    snapshot();
  }
  if (mounted) {
    app.unmount();
    mounted = false;
    await vue.nextTick();
    snapshot();
  }
  assert.equal(host.childNodes.length, 0, "Suspense unmount left nodes behind");
  assert.ok(
    announcements.every(
      (message) =>
        message === "<Suspense> is an experimental feature and its API will likely change.",
    ),
  );
  process.stdout.write(`${JSON.stringify(snapshots)}\n`);
} finally {
  console.info = originalInfo;
  if (mounted) app.unmount();
  host.remove();
  await window.happyDOM.close();
}
