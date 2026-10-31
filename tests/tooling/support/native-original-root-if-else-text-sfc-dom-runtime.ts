import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { fromVue, runtime as actualRuntime } from "./native-selected-sfc-dom-runtime.ts";
import {
  diagnosticValue,
  runtimeErrorDetails,
} from "./native-original-for-constant-sfc-dom-observation.ts";

type ObjectValue = Record<string, unknown>;
function object(value: unknown): asserts value is ObjectValue {
  assert(value && typeof value === "object" && !Array.isArray(value));
}
export function rootIfElseTextErrorDetails(error: unknown) {
  const cause =
    error !== null && typeof error === "object" ? Reflect.get(error, "cause") : undefined;
  return {
    ...runtimeErrorDetails(error),
    causePresent: error !== null && typeof error === "object" && Object.hasOwn(error, "cause"),
    cause:
      cause instanceof Error
        ? { ...runtimeErrorDetails(cause), cause: diagnosticValue(cause.cause) }
        : diagnosticValue(cause),
  };
}
export type RootIfElseTextMode = "development" | "production";
type Branch = { branch: 0 | 1; tag: string; text: string | null; patchFlag: 0 | 1 };
export type RootIfElseTextFixture = {
  id: string;
  source: string;
  script: string;
  template: string;
  bindings: string[];
  immutableBindings: string[];
  updates: Record<string, boolean | string>;
  conditionBinding: string;
  textBindings: [string | null, string | null];
  ifElseTextRuntime: { initial: Branch; updated: Branch };
};
export function assertRootIfElseTextFixture(
  value: unknown,
): asserts value is RootIfElseTextFixture {
  object(value);
  for (const field of ["id", "source", "script", "template"]) {
    assert(typeof value[field] === "string" && value[field].length > 0, field);
  }
  const bindings = value.bindings;
  assert(Array.isArray(bindings) && bindings.length >= 1 && bindings.length <= 3);
  assert.equal(new Set(bindings).size, bindings.length);
  assert(bindings.every((name: unknown) => typeof name === "string" && name.length > 0));
  assert(typeof value.conditionBinding === "string" && bindings.includes(value.conditionBinding));
  const immutable = value.immutableBindings;
  assert(Array.isArray(immutable) && immutable.length <= bindings.length);
  assert.equal(new Set(immutable).size, immutable.length);
  assert(immutable.every((name: unknown) => typeof name === "string" && bindings.includes(name)));
  assert(Array.isArray(value.textBindings) && value.textBindings.length === 2);
  assert(
    value.textBindings.every(
      (name: unknown) => name === null || (typeof name === "string" && bindings.includes(name)),
    ),
  );
  object(value.updates);
  for (const [name, next] of Object.entries(value.updates)) {
    assert(bindings.includes(name) && !immutable.includes(name));
    assert(typeof next === "boolean" || typeof next === "string");
  }
  object(value.ifElseTextRuntime);
  for (const phase of ["initial", "updated"]) {
    const vector = value.ifElseTextRuntime[phase];
    object(vector);
    assert(vector.branch === 0 || vector.branch === 1);
    assert(typeof vector.tag === "string" && vector.tag.length > 0);
    assert(vector.text === null || typeof vector.text === "string");
    assert(vector.patchFlag === 0 || vector.patchFlag === 1);
  }
}

type ImportNode = {
  type: string;
  source?: { type: string; value: string; start: number; end: number };
};
const parser = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"))(
  "@babel/parser",
) as {
  parse(code: string, options: { sourceType: "module" }): { program: { body: ImportNode[] } };
};
const dataUrl = (text: string) =>
  `data:text/javascript;base64,${Buffer.from(text).toString("base64")}`;
const runtimeUrl = dataUrl(
  `import runtime from ${JSON.stringify(pathToFileURL(fromVue.resolve("vue")).href)};\n` +
    [
      "openBlock",
      "createElementBlock",
      "createElementVNode",
      "createCommentVNode",
      "toDisplayString",
      "defineComponent",
    ]
      .map((name) => `export const ${name} = runtime.${name};`)
      .join("\n"),
);
function runtimeModuleSource(code: string) {
  const imports = parser
    .parse(code, { sourceType: "module" })
    .program.body.filter(
      (node) => node.type === "ImportDeclaration" && node.source?.value === "vue",
    );
  assert(imports.length > 0, "the whole component must import the real Vue runtime");
  let rewritten = code;
  for (const declaration of imports.toReversed()) {
    assert(declaration.source);
    const { start, end } = declaration.source;
    assert.equal(declaration.source.type, "StringLiteral");
    assert(
      Number.isInteger(start) &&
        Number.isInteger(end) &&
        start >= 0 &&
        start < end &&
        end <= code.length,
    );
    // Only Babel's actual ImportDeclaration source literal changes address.
    rewritten = rewritten.slice(0, start) + JSON.stringify(runtimeUrl) + rewritten.slice(end);
  }
  return rewritten;
}
type VNode = { type: unknown; key: unknown; props: unknown; children: unknown; patchFlag: number };
type Component = {
  setup: (props: unknown, context: unknown) => unknown;
  render: (this: unknown, ...args: unknown[]) => unknown;
};
type Instance = {
  type: Component;
  appContext: { app: App };
  subTree: VNode;
  proxy: { $forceUpdate(): void };
  isUnmounted: boolean;
};
type HostNode = {
  type: string;
  text: string;
  children: HostNode[];
  parent: HostNode | null;
  props: ObjectValue;
};
type App = {
  config: { warnHandler?: (warning: string) => void };
  mount(host: HostNode): unknown;
  unmount(): void;
};
type HostOptions = {
  createElement(type: string): HostNode;
  createText(text: string): HostNode;
  createComment(text: string): HostNode;
  setText(node: HostNode, text: string): void;
  setElementText(node: HostNode, text: string): void;
  patchProp(node: HostNode, key: string, previous: unknown, value: unknown): void;
  parentNode(node: HostNode): HostNode | null;
  nextSibling(node: HostNode): HostNode | null;
  insert(node: HostNode, parent: HostNode, anchor: HostNode | null): void;
  remove(node: HostNode): void;
};
// A finite type boundary around the actual pinned Vue module; no runtime is substituted.
const runtime = actualRuntime as {
  Comment: symbol;
  isVNode(value: unknown): value is VNode;
  getCurrentInstance(): Instance | null;
  nextTick(): Promise<void>;
  toDisplayString(value: unknown): string;
  createRenderer(options: HostOptions): { createApp(component: Component): App };
};
function vnodeShape(node: unknown): unknown {
  assert(runtime.isVNode(node));
  return {
    type: node.type === runtime.Comment ? "comment" : node.type,
    key: node.key,
    props: node.props ?? {},
    children: Array.isArray(node.children) ? node.children.map(vnodeShape) : node.children,
    patchFlag: node.patchFlag,
  };
}
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
  createElement: (type) => hostNode(type),
  createText: (text) => hostNode("text", text),
  createComment: (text) => hostNode("comment", text),
  setText: (node, text) => {
    node.text = text;
  },
  setElementText: (node, text) => {
    node.text = text;
    node.children = [];
  },
  patchProp: (node, key, _previous, value) => {
    node.props[key] = value;
  },
  parentNode: (node) => node.parent,
  nextSibling: (node) => node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
  insert: (node, parent, anchor) => {
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
  remove: (node) => {
    assert(node.parent);
    const index = node.parent.children.indexOf(node);
    assert(index >= 0);
    node.parent.children.splice(index, 1);
    node.parent = null;
  },
});

export async function executeRootIfElseTextComponent(
  code: string,
  fixture: RootIfElseTextFixture,
  native: boolean,
  mode: RootIfElseTextMode,
  observe?: (observation: unknown) => void,
) {
  assertRootIfElseTextFixture(fixture);
  assert.equal(process.env.NODE_ENV ?? "development", mode);
  const rewritten = runtimeModuleSource(code);
  if (fixture.script.includes("/* from 'vue' 雪🌸 */"))
    assert(rewritten.includes("/* from 'vue' 雪🌸 */"));
  const loaded: unknown = (await import(dataUrl(rewritten))).default;
  object(loaded);
  assert.equal(typeof loaded.setup, "function");
  assert.equal(typeof loaded.render, "function");
  const component = loaded as unknown as Component;
  const originalSetup = component.setup;
  const originalRender = component.render;
  let state: ObjectValue | undefined;
  let instance: Instance | null = null;
  let setupInvocations = 0;
  let renders = 0;
  component.setup = (props, context) => {
    setupInvocations++;
    instance = runtime.getCurrentInstance();
    record("setup-instance-before-comparisons");
    assert(instance && instance.type.setup === component.setup);
    const returned = originalSetup(props, context);
    record("original-setup-return-before-comparisons", { returned });
    object(returned);
    state = returned;
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
  app.config.warnHandler = (warning) => warnings.push(warning);
  const current = () => {
    assert(instance);
    return instance;
  };
  const bindings = () => {
    assert(state);
    return state;
  };
  function observedVNode(node: unknown) {
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
  function snapshot(vector: Branch, phase: string) {
    record(phase);
    const selectedText = fixture.textBindings[vector.branch];
    assert.equal(
      selectedText === null ? null : runtime.toDisplayString(bindings()[selectedText]),
      vector.text,
    );
    const vnode = vnodeShape(current().subTree);
    assert.deepEqual(vnode, {
      type: vector.tag,
      key: vector.branch,
      props: { key: vector.branch },
      children: vector.text,
      patchFlag: vector.patchFlag,
    });
    const actualHost = host.children.map(hostShape);
    assert.deepEqual(actualHost, [
      { type: vector.tag, text: vector.text ?? "", props: {}, children: [] },
    ]);
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
      record(phase, { error: rootIfElseTextErrorDetails(error) });
    } catch {
      /* Preserve the first runtime failure. */
    }
  }
  try {
    app.mount(host);
    record("mounted-before-comparisons");
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 1);
    assert.equal(current().appContext.app, app);
    assert.equal(current().type.render, component.render);
    assert.deepEqual(Object.keys(bindings()), fixture.bindings);
    assert.deepEqual(Object.getOwnPropertyDescriptor(bindings(), "__isScriptSetup"), {
      value: true,
      writable: false,
      enumerable: false,
      configurable: false,
    });
    for (const name of fixture.bindings) {
      const descriptor = Object.getOwnPropertyDescriptor(bindings(), name);
      record("binding-before-comparisons", { name, descriptor });
      if (fixture.immutableBindings.includes(name)) {
        assert.equal(typeof descriptor?.set, "undefined");
        if (native) {
          assert.equal(typeof descriptor?.get, "function");
          const original = bindings()[name];
          const written = Reflect.set(bindings(), name, Symbol("forbidden mutation"));
          record("binding-write-before-comparisons", { name, written, value: bindings()[name] });
          assert.equal(written, false);
          assert.equal(bindings()[name], original);
        }
      } else {
        assert.equal(typeof descriptor?.get, "function");
        assert.equal(typeof descriptor?.set, "function");
      }
    }
    record("initial-state-before-comparisons");
    assert.equal(
      bindings()[fixture.conditionBinding],
      fixture.ifElseTextRuntime.initial.branch === 0,
    );
    const initial = snapshot(fixture.ifElseTextRuntime.initial, "initial-before-comparisons");
    for (const [name, value] of Object.entries(fixture.updates)) {
      bindings()[name] = value;
      record("updated-binding-before-comparison", { name, value: bindings()[name] });
      assert.equal(bindings()[name], value);
    }
    current().proxy.$forceUpdate();
    await runtime.nextTick();
    record("updated-state-before-comparisons");
    assert.equal(
      bindings()[fixture.conditionBinding],
      fixture.ifElseTextRuntime.updated.branch === 0,
    );
    const updated = snapshot(fixture.ifElseTextRuntime.updated, "updated-before-comparisons");
    assert.equal(setupInvocations, 1);
    assert.equal(renders, 2);
    assert.deepEqual(warnings, []);
    unmountAttempted = true;
    app.unmount();
    await runtime.nextTick();
    record("unmounted-before-comparisons");
    assert.equal(current().isUnmounted, true);
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
