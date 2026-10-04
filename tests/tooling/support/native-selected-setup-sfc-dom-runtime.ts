import assert from "node:assert/strict";
import { runtime, runtimeModuleSource } from "./native-selected-sfc-dom-runtime.ts";

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

export async function executeSetupComponent(code: string, fixture: any, native: boolean) {
  const rewritten = runtimeModuleSource(code);
  // Import-shaped original strings and comments are production source bytes.
  if (fixture.id === "unicode-import-shaped-var") {
    assert(rewritten.includes("/* from 'vue' 雪🌸 */"));
    assert(rewritten.includes("\"from 'vue' 雪🌸\""));
  }
  const loaded = await import(dataUrl(rewritten));
  const component = loaded.default,
    originalSetup = component.setup;
  let state: any,
    calls = 0;
  component.setup = (props: unknown, context: unknown) => {
    calls += 1;
    state = originalSetup(props, context);
    return state;
  };
  const warnings: string[] = [];
  const app = renderer.createApp(component),
    host = hostNode("root");
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  app.mount(host);
  assert.equal(calls, 1);
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
  function originalForHost(count: number, values?: string[]) {
    if (!fixture.forRuntime && !fixture.forValueRuntime) return;
    const expected = fixture.forRuntime ?? fixture.forValueRuntime;
    const elements = host.children.filter((node: any) => node.type === expected.tag);
    assert.equal(elements.length, count);
    assert.deepEqual(
      elements.map((node: any) => node.text),
      values ?? Array(count).fill(expected.text),
    );
    assert.equal(host.children.length, count + 2, "the actual Fragment owns two host anchors");
  }
  const initial = shape(app._instance.subTree);
  originalForHost(
    fixture.forRuntime?.initialChildren ?? fixture.forValueRuntime?.initial.length,
    fixture.forValueRuntime?.initial,
  );
  for (const [name, value] of Object.entries(fixture.updates)) {
    assert(fixture.bindings.includes(name) && !fixture.immutableBindings.includes(name));
    state[name] = value;
    assert.equal(state[name], value);
  }
  // Primitive lexical mutation requires an explicit render request. This
  // proves actual setup getter/setter semantics, not automatic ref reactivity.
  app._instance.proxy.$forceUpdate();
  await runtime.nextTick();
  const updated = shape(app._instance.subTree);
  originalForHost(
    fixture.forRuntime?.updatedChildren ?? fixture.forValueRuntime?.updated.length,
    fixture.forValueRuntime?.updated,
  );
  assert.equal(calls, 1);
  assert.deepEqual(warnings, []);
  app.unmount();
  assert.deepEqual(host.children, []);
  return { initial, updated, setupInvocations: calls, renders: 2, unmounted: true };
}
