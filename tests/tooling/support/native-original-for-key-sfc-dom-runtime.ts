import assert from "node:assert/strict";
import { runtime, runtimeModuleSource } from "./native-selected-sfc-dom-runtime.ts";

export type ForKeyFixture = {
  id: string;
  source: string;
  script: string;
  template: string;
  bindings: string[];
  immutableBindings: string[];
  updates: Record<string, unknown>;
  forKeyRuntime: { tag: string; initial: string[]; updated: string[] };
};

export function assertForKeyFixture(value: unknown): asserts value is ForKeyFixture {
  assert(value && typeof value === "object" && !Array.isArray(value));
  const fixture = value as Record<string, unknown>;
  for (const field of ["id", "source", "script", "template"]) {
    assert.equal(typeof fixture[field], "string", field);
    assert((fixture[field] as string).length > 0, field);
  }
  const bindings = fixture.bindings;
  const immutableBindings = fixture.immutableBindings;
  assert(Array.isArray(bindings) && bindings.length === 1);
  assert(bindings.every((name: unknown) => typeof name === "string" && name.length > 0));
  assert(Array.isArray(immutableBindings));
  assert(
    immutableBindings.every((name: unknown) => typeof name === "string" && bindings.includes(name)),
  );
  assert(fixture.updates && typeof fixture.updates === "object" && !Array.isArray(fixture.updates));
  for (const name of Object.keys(fixture.updates)) {
    assert(bindings.includes(name) && !immutableBindings.includes(name));
  }
  const expected = fixture.forKeyRuntime;
  assert(expected && typeof expected === "object" && !Array.isArray(expected));
  const vectors = expected as Record<string, unknown>;
  assert.equal(typeof vectors.tag, "string");
  assert((vectors.tag as string).length > 0);
  for (const phase of ["initial", "updated"]) {
    const values = vectors[phase];
    assert(Array.isArray(values));
    assert(values.every((value: unknown) => typeof value === "string"));
    // These independently declared vectors are the zero-based callback keys,
    // including UTF-16 string units; generated code never supplies this oracle.
    assert.deepEqual(
      values,
      values.map((_, index) => String(index)),
    );
  }
}

const dataUrl = (text: string) =>
  `data:text/javascript;base64,${Buffer.from(text).toString("base64")}`;
function shape(node: any): any {
  assert(runtime.isVNode(node));
  return {
    type:
      node.type === runtime.Fragment
        ? "fragment"
        : node.type === runtime.Text
          ? "text"
          : node.type === runtime.Comment
            ? "comment"
            : node.type,
    props: node.props ?? {},
    children: Array.isArray(node.children) ? node.children.map(shape) : node.children,
    patchFlag: node.patchFlag,
  };
}
const hostNode = (type: string, text = "") => ({
  type,
  text,
  children: [] as any[],
  parent: null as any,
  props: {},
});
const renderer = runtime.createRenderer({
  createElement: (type: string) => hostNode(type),
  createText: (text: string) => hostNode("text", text),
  createComment: (text: string) => hostNode("comment", text),
  setText: (node: any, text: string) => {
    node.text = text;
  },
  setElementText: (node: any, text: string) => {
    node.text = text;
    node.children = [];
  },
  patchProp: (node: any, key: string, _previous: any, value: any) => {
    node.props[key] = value;
  },
  parentNode: (node: any) => node.parent,
  nextSibling: (node: any) => node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
  insert: (node: any, parent: any, anchor: any) => {
    if (node.parent) {
      const index = node.parent.children.indexOf(node);
      if (index >= 0) node.parent.children.splice(index, 1);
    }
    const index = anchor ? parent.children.indexOf(anchor) : -1;
    parent.children.splice(index < 0 ? parent.children.length : index, 0, node);
    node.parent = parent;
  },
  remove: (node: any) => {
    const index = node.parent?.children.indexOf(node) ?? -1;
    if (index >= 0) node.parent.children.splice(index, 1);
    node.parent = null;
  },
});

export async function executeForKeyComponent(
  code: string,
  fixture: ForKeyFixture,
  native: boolean,
) {
  assertForKeyFixture(fixture);
  const loaded = await import(dataUrl(runtimeModuleSource(code)));
  const component = loaded.default;
  const originalSetup = component.setup;
  const originalRender = component.render;
  let state: any;
  let instance: any;
  let calls = 0;
  let renders = 0;
  assert.equal(typeof originalSetup, "function");
  assert.equal(typeof originalRender, "function");
  component.setup = (props: unknown, context: unknown) => {
    calls += 1;
    // getCurrentInstance is available in both Vue builds; app._instance is dev-only.
    instance = runtime.getCurrentInstance();
    assert(instance && instance.type.setup === component.setup);
    state = originalSetup(props, context);
    return state;
  };
  component.render = function (this: unknown, ...args: unknown[]) {
    renders += 1;
    return originalRender.apply(this, args);
  };
  const warnings: string[] = [];
  const app = renderer.createApp(component);
  const host = hostNode("root");
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  app.mount(host);
  assert.equal(calls, 1);
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
    if (fixture.immutableBindings.includes(name)) {
      assert.equal(typeof descriptor?.set, "undefined");
      if (native) {
        assert.equal(typeof descriptor?.get, "function");
        const original = state[name];
        assert.equal(Reflect.set(state, name, Symbol("forbidden mutation")), false);
        assert.equal(state[name], original);
      }
    } else {
      assert.equal(typeof descriptor?.get, "function");
      assert.equal(typeof descriptor?.set, "function");
    }
  }
  const collection = fixture.bindings[0];
  const patchFlag = fixture.immutableBindings.includes(collection) ? 64 : 256;
  function snapshot(values: string[]) {
    const result = shape(instance.subTree);
    assert.deepEqual(result, {
      type: "fragment",
      props: {},
      children: values.map((value) => ({
        type: fixture.forKeyRuntime.tag,
        props: {},
        children: value,
        patchFlag: 1,
      })),
      patchFlag,
    });
    const elements = host.children.filter((node: any) => node.type === fixture.forKeyRuntime.tag);
    assert.deepEqual(
      elements.map((node: any) => node.text),
      values,
    );
    assert.equal(
      host.children.length,
      values.length + 2,
      "the actual Fragment owns two host anchors",
    );
    return result;
  }
  const initial = snapshot(fixture.forKeyRuntime.initial);
  for (const [name, value] of Object.entries(fixture.updates)) {
    assert(fixture.bindings.includes(name) && !fixture.immutableBindings.includes(name));
    state[name] = value;
    assert.equal(state[name], value);
  }
  instance.proxy.$forceUpdate();
  await runtime.nextTick();
  const updated = snapshot(fixture.forKeyRuntime.updated);
  assert.equal(calls, 1);
  assert.equal(renders, 2);
  assert.deepEqual(warnings, []);
  app.unmount();
  await runtime.nextTick();
  assert.equal(instance.isUnmounted, true);
  assert.deepEqual(host.children, []);
  return { initial, updated, setupInvocations: calls, renders, unmounted: true };
}
