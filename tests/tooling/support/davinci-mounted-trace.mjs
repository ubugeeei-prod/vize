import assert from "node:assert/strict";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Window } from "happy-dom";
import { build } from "vite-plus";
import { evaluateCompiledRender } from "./davinci-runtime-trace.mjs";

/** One process owns one DOM and one Vue module, including its scheduler and effects. */
export async function traceMountedBackend({
  backend,
  code,
  context = {},
  steps = [],
  identities = false,
}) {
  assert.ok(backend === "vdom" || backend === "vapor", `unknown backend: ${backend}`);
  if (identities) validateLoopScenario(context, steps);
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
  ]) {
    globalThis[key] = key === "window" ? window : window[key];
  }

  const vue = await loadRuntime();
  const render = await evaluateCompiledRender(code, vue);
  const events = [];
  const state = vue.reactive({
    $slots: {},
    ...context,
    save: () => events.push("save"),
    saveParent: () => events.push("saveParent"),
    record: (value) => events.push(value),
  });
  const cache = [];
  const component =
    backend === "vapor"
      ? vue.defineVaporComponent({ setup: () => render(state) })
      : { setup: () => () => render(state, cache) };
  const app = (backend === "vapor" ? vue.createVaporApp : vue.createApp)(component);
  const diagnostics = [];
  app.config.warnHandler = (message) => diagnostics.push(message);
  app.config.errorHandler = (error) => diagnostics.push(String(error));
  const host = window.document.createElement("div");
  window.document.body.append(host);
  const snapshots = [];
  const nodeIdentities = new WeakMap();
  let nextIdentity = 0;
  let previousNodes = [];

  function observedIdentities() {
    const nodes = [...host.querySelectorAll("*")];
    for (const previous of previousNodes) {
      if (!nodes.includes(previous))
        assert.equal(previous.isConnected, false, "removed node is still connected");
    }
    previousNodes = nodes;
    for (const node of nodes) {
      if (!nodeIdentities.has(node)) nodeIdentities.set(node, nextIdentity++);
    }
    return nodes
      .filter((node) => node.hasAttribute("data-id"))
      .map((node) => [node.getAttribute("data-id"), nodeIdentities.get(node)]);
  }

  function snapshot() {
    assert.deepEqual(diagnostics, [], "mounted runtime diagnostics");
    snapshots.push({
      tree: observeChildren(host),
      events: [...events],
      ...(identities ? { identities: observedIdentities() } : {}),
    });
  }

  try {
    app.mount(host);
    await vue.nextTick();
    snapshot();
    for (const step of steps) {
      if (Object.hasOwn(step, "click")) {
        assert.equal(identities, true, "loop clicks require identity observations");
        assert.deepEqual(Object.keys(step), ["click"], "unexpected loop click fields");
        assert.equal(typeof step.click, "string", "loop target must be a data-id string");
        const targets = [...host.querySelectorAll("[data-id]")].filter(
          (node) => node.getAttribute("data-id") === step.click,
        );
        assert.equal(targets.length, 1, "expected one live interaction target");
        assert.ok(targets[0] instanceof window.HTMLButtonElement, "expected a native button");
        targets[0].click();
      } else if (Object.hasOwn(step, "activate")) {
        assert.deepEqual(Object.keys(step), ["activate"], "unexpected activation fields");
        assert.equal(step.activate, "button", "unsupported activation target");
        const targets = host.querySelectorAll("button");
        assert.equal(targets.length, 1, "expected one activation target");
        assert.ok(targets[0] instanceof window.HTMLButtonElement, "expected an HTML button");
        targets[0].click();
      } else if (step.patch) {
        Object.assign(state, step.patch);
      } else if (step.event) {
        const target = host.querySelector(step.selector);
        assert.ok(target, `missing interaction target: ${step.selector}`);
        if (Object.hasOwn(step, "value")) target.value = step.value;
        if (Object.hasOwn(step, "checked")) target.checked = step.checked;
        if (step.selectedValues) {
          for (const option of target.options)
            option.selected = step.selectedValues.includes(option.value);
        }
        const EventClass = Object.hasOwn(step, "key")
          ? window.KeyboardEvent
          : Object.hasOwn(step, "button")
            ? window.MouseEvent
            : window.Event;
        target.dispatchEvent(
          new EventClass(step.event, {
            bubbles: step.bubbles ?? true,
            cancelable: true,
            ...(Object.hasOwn(step, "key") ? { key: step.key } : {}),
            ...(Object.hasOwn(step, "button") ? { button: step.button } : {}),
          }),
        );
      } else {
        throw new Error(`unknown interaction step: ${JSON.stringify(step)}`);
      }
      await vue.nextTick();
      snapshot();
    }
    app.unmount();
    await vue.nextTick();
    assert.equal(host.childNodes.length, 0, "unmount left DOM nodes behind");
    snapshot();
    return snapshots;
  } finally {
    if (host.childNodes.length) app.unmount();
    host.remove();
    await window.happyDOM.close();
  }
}

export function validateLoopScenario(context, steps) {
  const validateState = (state) => {
    assert.ok(
      state !== null && typeof state === "object" && !Array.isArray(state),
      "expected loop state object",
    );
    for (const key of Object.keys(state)) {
      assert.ok(
        /^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(key) &&
          !["save", "record", "$event", "$slots", "__proto__", "constructor", "prototype"].includes(
            key,
          ),
        "unsupported loop state key",
      );
    }
  };
  validateState(context);
  assert.ok(Array.isArray(steps), "expected loop steps array");
  for (const step of steps) {
    assert.ok(
      step !== null && typeof step === "object" && !Array.isArray(step),
      "expected loop step object",
    );
    assert.equal(Object.keys(step).length, 1, "expected one loop step field");
    if (Object.hasOwn(step, "click")) {
      assert.equal(typeof step.click, "string", "loop target must be a data-id string");
    } else {
      assert.ok(Object.hasOwn(step, "patch"), "unsupported loop step");
      validateState(step.patch);
    }
  }
}

export async function loadRuntime() {
  const result = await build({
    configFile: false,
    logLevel: "silent",
    define: {
      "process.env.NODE_ENV": JSON.stringify("development"),
      __VUE_OPTIONS_API__: "true",
      __VUE_PROD_DEVTOOLS__: "false",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
    },
    build: {
      write: false,
      minify: false,
      lib: {
        entry: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")),
        formats: ["es"],
      },
    },
  });
  const outputs = Array.isArray(result) ? result : [result];
  const chunks = outputs.flatMap((output) => output.output.filter((item) => item.type === "chunk"));
  assert.equal(chunks.length, 1, "Vue runtime must bundle into one self-contained module");
  return import(`data:text/javascript;base64,${Buffer.from(chunks[0].code).toString("base64")}`);
}

/** Ignore backend anchor comments; retain text, attributes, and live form state. */
export function observeChildren(parent) {
  const children = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3) {
      if (!node.data) continue;
      if (typeof children.at(-1) === "string") children[children.length - 1] += node.data;
      else children.push(node.data);
    } else if (node.nodeType === 1) {
      const attributes = Object.fromEntries(
        [...node.attributes]
          .map(({ name, value }) => [name, value])
          .sort(([a], [b]) => a.localeCompare(b)),
      );
      const element = { tag: node.localName, attributes, children: observeChildren(node) };
      if (node.localName === "input") {
        element.value = node.value;
        element.checked = node.checked;
      }
      if (node.localName === "button") element.disabled = node.disabled;
      if (node.localName === "select") element.value = node.value;
      if (node.localName === "option") element.selected = node.selected;
      children.push(element);
    }
  }
  return children;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  process.stdout.write(`${JSON.stringify(await traceMountedBackend(input))}\n`);
}
