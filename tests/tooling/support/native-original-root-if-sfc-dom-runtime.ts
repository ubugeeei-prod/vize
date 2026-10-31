import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { fromVue, runtime } from "./native-selected-sfc-dom-runtime.ts";
import {
  diagnosticValue,
  runtimeErrorDetails,
} from "./native-original-for-constant-sfc-dom-observation.ts";

export function rootIfErrorDetails(error: any) {
  return {
    ...runtimeErrorDetails(error),
    causePresent: Object.hasOwn(error ?? {}, "cause"),
    cause:
      error?.cause instanceof Error
        ? { ...runtimeErrorDetails(error.cause), cause: diagnosticValue(error.cause.cause) }
        : diagnosticValue(error?.cause),
  };
}

export type RootIfMode = "development" | "production";
export type RootIfFixture = {
  id: string;
  source: string;
  script: string;
  template: string;
  bindings: string[];
  immutableBindings: string[];
  updates: Record<string, boolean>;
  ifRuntime: { tag: string; text: string | null; initial: boolean; updated: boolean };
};

export function assertRootIfFixture(value: unknown): asserts value is RootIfFixture {
  assert(value && typeof value === "object" && !Array.isArray(value));
  const fixture = value as Record<string, unknown>;
  for (const field of ["id", "source", "script", "template"]) {
    assert.equal(typeof fixture[field], "string", field);
    assert((fixture[field] as string).length > 0, field);
  }
  const bindings = fixture.bindings;
  const immutable = fixture.immutableBindings;
  assert(Array.isArray(bindings) && bindings.length === 1);
  assert(bindings.every((name: unknown) => typeof name === "string" && name.length > 0));
  assert(Array.isArray(immutable));
  assert(immutable.every((name: unknown) => typeof name === "string" && bindings.includes(name)));
  assert(fixture.updates && typeof fixture.updates === "object" && !Array.isArray(fixture.updates));
  for (const [name, next] of Object.entries(fixture.updates)) {
    assert(bindings.includes(name) && !immutable.includes(name));
    assert.equal(typeof next, "boolean");
  }
  const expected = fixture.ifRuntime;
  assert(expected && typeof expected === "object" && !Array.isArray(expected));
  const vector = expected as Record<string, unknown>;
  assert.equal(typeof vector.tag, "string");
  assert((vector.tag as string).length > 0);
  assert(vector.text === null || typeof vector.text === "string");
  assert.equal(typeof vector.initial, "boolean");
  assert.equal(typeof vector.updated, "boolean");
}

const dataUrl = (text: string) =>
  `data:text/javascript;base64,${Buffer.from(text).toString("base64")}`;
const { parse } = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"))("@babel/parser");
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "createCommentVNode",
      "defineComponent",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);
function runtimeModuleSource(code: string) {
  const imports = parse(code, { sourceType: "module" }).program.body.filter(
    (node: any) => node.type === "ImportDeclaration" && node.source.value === "vue",
  );
  assert(imports.length > 0, "the whole component must import the real Vue runtime");
  let rewritten = code;
  for (const declaration of imports.toReversed()) {
    const { start, end } = declaration.source;
    assert.equal(declaration.source.type, "StringLiteral");
    assert(Number.isInteger(start) && Number.isInteger(end) && start >= 0 && end <= code.length);
    // Only actual ImportDeclaration source literals change address. Original
    // script text, comments, render functions and maps are never normalized.
    rewritten = rewritten.slice(0, start) + JSON.stringify(runtimeUrl) + rewritten.slice(end);
  }
  return rewritten;
}

function vnodeShape(node: any): any {
  assert(runtime.isVNode(node));
  return {
    type: node.type === runtime.Comment ? "comment" : node.type,
    key: node.key,
    props: node.props ?? {},
    children: Array.isArray(node.children) ? node.children.map(vnodeShape) : node.children,
    patchFlag: node.patchFlag,
  };
}
type HostNode = {
  type: string;
  text: string;
  children: HostNode[];
  parent: HostNode | null;
  props: Record<string, unknown>;
};
const hostNode = (type: string, text = ""): HostNode => ({
  type,
  text,
  children: [],
  parent: null,
  props: {},
});
function hostShape(node: HostNode): unknown {
  return {
    type: node.type,
    text: node.text,
    props: node.props,
    children: node.children.map(hostShape),
  };
}
const renderer = runtime.createRenderer({
  createElement: (type: string) => hostNode(type),
  createText: (text: string) => hostNode("text", text),
  createComment: (text: string) => hostNode("comment", text),
  setText: (node: HostNode, text: string) => {
    node.text = text;
  },
  setElementText: (node: HostNode, text: string) => {
    node.text = text;
    node.children = [];
  },
  patchProp: (node: HostNode, key: string, _previous: unknown, value: unknown) => {
    node.props[key] = value;
  },
  parentNode: (node: HostNode) => node.parent,
  nextSibling: (node: HostNode) =>
    node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
  insert: (node: HostNode, parent: HostNode, anchor: HostNode | null) => {
    if (node.parent) {
      const index = node.parent.children.indexOf(node);
      assert(index >= 0);
      node.parent.children.splice(index, 1);
    }
    const index = anchor ? parent.children.indexOf(anchor) : -1;
    if (anchor) assert(index >= 0);
    parent.children.splice(index < 0 ? parent.children.length : index, 0, node);
    node.parent = parent;
  },
  remove: (node: HostNode) => {
    assert(node.parent);
    const index = node.parent.children.indexOf(node);
    assert(index >= 0);
    node.parent.children.splice(index, 1);
    node.parent = null;
  },
});

export async function executeRootIfComponent(
  code: string,
  fixture: RootIfFixture,
  native: boolean,
  mode: RootIfMode,
  observe?: (observation: unknown) => void,
) {
  assertRootIfFixture(fixture);
  assert.equal(process.env.NODE_ENV ?? "development", mode);
  const rewritten = runtimeModuleSource(code);
  if (fixture.id === "commented-unicode") assert(rewritten.includes("/* from 'vue' 雪🌸 */"));
  const component = (await import(dataUrl(rewritten))).default;
  const originalSetup = component.setup;
  const originalRender = component.render;
  assert.equal(typeof originalSetup, "function");
  assert.equal(typeof originalRender, "function");
  let state: any;
  let instance: any;
  let setupInvocations = 0;
  let renders = 0;
  component.setup = (props: unknown, context: unknown) => {
    setupInvocations++;
    instance = runtime.getCurrentInstance();
    record("setup-instance-before-comparisons");
    assert(instance && instance.type.setup === component.setup);
    state = originalSetup(props, context);
    record("original-setup-result");
    return state;
  };
  component.render = function (this: unknown, ...args: unknown[]) {
    renders++;
    const rendered = originalRender.apply(this, args);
    record("original-render-result", { rendered: observedVNode(rendered) });
    return rendered;
  };
  const warnings: string[] = [];
  const app = renderer.createApp(component);
  const host = hostNode("root");
  app.config.warnHandler = (warning: string) => warnings.push(warning);

  function observedVNode(node: any) {
    return runtime.isVNode(node) ? vnodeShape(node) : { isVNode: false, value: node };
  }
  function record(phase: string, details?: unknown) {
    observe?.(
      diagnosticValue({
        phase,
        native,
        mode,
        vnode: observedVNode(instance?.subTree),
        host: host.children.map(hostShape),
        setupInvocations,
        renders,
        warnings: [...warnings],
        isUnmounted: instance?.isUnmounted,
        sameApp: instance?.appContext.app === app,
        sameSetup: instance?.type.setup === component.setup,
        sameRender: instance?.type.render === component.render,
        stateKind: typeof state,
        bindings:
          state &&
          Object.keys(state).map((name) => ({
            name,
            descriptor: Object.getOwnPropertyDescriptor(state, name),
          })),
        scriptSetupDescriptor: state && Object.getOwnPropertyDescriptor(state, "__isScriptSetup"),
        details,
      }),
    );
  }
  function snapshot(visible: boolean, phase: string) {
    // Record actual output before any expected mode, key, VNode or host check.
    record(phase);
    const text = visible ? fixture.ifRuntime.text : mode === "development" ? "v-if" : "";
    const type = visible ? fixture.ifRuntime.tag : "comment";
    const vnode = vnodeShape(instance.subTree);
    assert.deepEqual(vnode, {
      type,
      key: visible ? 0 : null,
      props: visible ? { key: 0 } : {},
      children: text,
      patchFlag: 0,
    });
    const actualHost = host.children.map(hostShape);
    assert.deepEqual(actualHost, [{ type, text: text ?? "", props: {}, children: [] }]);
    return { vnode, host: actualHost };
  }
  let failed = false;
  let firstFailure: unknown;
  let unmountAttempted = false;
  let result: unknown;
  function retainFailure(error: unknown, phase: string) {
    if (!failed) {
      failed = true;
      firstFailure = error;
    }
    try {
      record(phase, { error: rootIfErrorDetails(error) });
    } catch {
      // A diagnostic callback must not replace the original runtime failure.
    }
  }
  try {
    app.mount(host);
    record("mounted-before-comparisons");
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 1);
    assert.equal(instance.appContext.app, app);
    assert.equal(instance.type.render, component.render);
    assert.deepEqual(Object.keys(state), fixture.bindings);
    assert.deepEqual(Object.getOwnPropertyDescriptor(state, "__isScriptSetup"), {
      value: true,
      writable: false,
      enumerable: false,
      configurable: false,
    });
    for (const name of fixture.bindings) {
      const descriptor = Object.getOwnPropertyDescriptor(state, name);
      record("binding-before-comparisons", { name, descriptor });
      if (fixture.immutableBindings.includes(name)) {
        assert.equal(typeof descriptor?.set, "undefined");
        if (native) {
          assert.equal(typeof descriptor?.get, "function");
          const original = state[name];
          const written = Reflect.set(state, name, Symbol("forbidden mutation"));
          record("binding-write-before-comparisons", { name, written, value: state[name] });
          assert.equal(written, false);
          assert.equal(state[name], original);
        }
      } else {
        assert.equal(typeof descriptor?.get, "function");
        assert.equal(typeof descriptor?.set, "function");
      }
    }
    record("initial-state-before-comparisons");
    assert.equal(state[fixture.bindings[0]], fixture.ifRuntime.initial);
    const initial = snapshot(fixture.ifRuntime.initial, "initial-before-comparisons");
    for (const [name, value] of Object.entries(fixture.updates)) {
      state[name] = value;
      record("updated-binding-before-comparison", { name, value: state[name] });
      assert.equal(state[name], value);
    }
    instance.proxy.$forceUpdate();
    await runtime.nextTick();
    record("updated-state-before-comparisons");
    assert.equal(state[fixture.bindings[0]], fixture.ifRuntime.updated);
    const updated = snapshot(fixture.ifRuntime.updated, "updated-before-comparisons");
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 2);
    assert.deepEqual(warnings, []);
    unmountAttempted = true;
    app.unmount();
    await runtime.nextTick();
    record("unmounted-before-comparisons");
    assert.equal(instance.isUnmounted, true);
    assert.deepEqual(host.children, []);
    assert.deepEqual(warnings, []);
    result = { initial, updated, setupInvocations, renders, unmounted: true, warnings };
  } catch (error) {
    retainFailure(error, "runtime-failure");
  } finally {
    if (!unmountAttempted) {
      try {
        try {
          record("failure-cleanup-before-unmount");
        } catch (error) {
          retainFailure(error, "failure-cleanup-observation-error");
        }
        app.unmount();
        await runtime.nextTick();
        record("failure-cleanup-after-unmount");
      } catch (error) {
        retainFailure(error, "failure-cleanup-error");
      }
    }
  }
  if (failed) throw firstFailure;
  return result;
}
