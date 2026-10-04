import assert from "node:assert/strict";
import { runtime, runtimeModuleSource } from "./native-selected-sfc-dom-runtime.ts";

import {
  diagnosticValue,
  runtimeErrorDetails,
} from "./native-original-for-constant-sfc-dom-observation.ts";
export { runtimeErrorDetails } from "./native-original-for-constant-sfc-dom-observation.ts";

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

export async function executeConstantComponent(
  code: string,
  fixture: any,
  native: boolean,
  mode: string,
  range = false,
  observe?: (record: any) => void,
) {
  const loaded = await import(dataUrl(runtimeModuleSource(code)));
  const component = loaded.default,
    originalSetup = component.setup,
    originalRender = component.render;
  let state: any,
    instance: any,
    calls = 0,
    renders = 0;
  assert.equal(typeof originalSetup, "function");
  assert.equal(typeof originalRender, "function");
  component.setup = (props: unknown, context: unknown) => {
    calls += 1;
    record("setup-call-before-comparisons");
    assert.equal(instance, undefined);
    instance = runtime.getCurrentInstance();
    record("setup-instance-before-comparisons");
    assert(instance && instance.type.setup === component.setup);
    assert.equal(instance.type.render, component.render);
    state = originalSetup(props, context);
    record("original-setup-result");
    return state;
  };
  component.render = function (this: unknown, ...args: unknown[]) {
    renders += 1;
    record("render-call");
    const rendered = originalRender.apply(this, args);
    record("original-render-result", observedShape(rendered));
    return rendered;
  };
  const warnings: string[] = [],
    errors: any[] = [],
    observedErrors: any[] = [];
  const app = renderer.createApp(component),
    host = hostNode("root");
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  app.config.errorHandler = (error: any, _instance: unknown, info: unknown) => {
    errors.push({ name: error.name, message: error.message, info });
    observedErrors.push({ info, error: runtimeErrorDetails(error) });
  };
  function hostSnapshot(node: any): any {
    return {
      type: node.type,
      text: node.text,
      props: { ...node.props },
      children: node.children.map(hostSnapshot),
    };
  }
  function observedShape(node: any): any {
    if (node === null || typeof node !== "object") return { isVNode: false, value: node };
    return {
      isVNode: runtime.isVNode(node),
      type:
        node.type === runtime.Fragment
          ? "fragment"
          : node.type === runtime.Text
            ? "text"
            : node.type === runtime.Comment
              ? "comment"
              : node.type,
      props: node.props,
      patchFlag: node.patchFlag,
      shapeFlag: node.shapeFlag,
      children: Array.isArray(node.children) ? node.children.map(observedShape) : node.children,
      trackedDynamicChildren: node.dynamicChildren?.length ?? null,
      callbackBlocks: Array.isArray(node.children)
        ? node.children.map((child: any) => Array.isArray(child?.dynamicChildren))
        : [],
    };
  }
  function record(phase: string, snapshot: any = undefined) {
    if (!observe) return;
    observe(
      diagnosticValue({
        phase,
        setupInvocations: calls,
        renders,
        warnings: [...warnings],
        errors: [...errors],
        observedErrors: [...observedErrors],
        host: hostSnapshot(host),
        vnode: observedShape(instance?.subTree),
        snapshot,
        isUnmounted: instance?.isUnmounted,
        stateKind: typeof state,
        bindings:
          state &&
          Object.keys(state).map((name) => ({
            name,
            descriptor: Object.getOwnPropertyDescriptor(state, name),
          })),
        scriptSetupDescriptor: state && Object.getOwnPropertyDescriptor(state, "__isScriptSetup"),
        sameApp: instance?.appContext.app === app,
        sameComponent: instance?.type === app._component,
      }),
    );
  }
  try {
    app.mount(host);
    record("mounted-before-comparisons");
    assert.equal(calls, 1);
    assert.equal(renders, 1);
    assert.equal(instance.appContext.app, app);
    assert.equal(instance.type, app._component);
    assert.deepEqual(Object.keys(state), fixture.bindings);
    assert.deepEqual(Object.getOwnPropertyDescriptor(state, "__isScriptSetup"), {
      value: true,
      writable: false,
      enumerable: false,
      configurable: false,
    });
    for (const name of fixture.bindings) {
      const descriptor = Object.getOwnPropertyDescriptor(state, name);
      record("binding-descriptor-before-comparison", { name, descriptor });
      assert.equal(typeof descriptor?.set, "undefined");
      if (native) {
        assert.equal(typeof descriptor?.get, "function");
        const original = state[name];
        record("binding-value-before-write", { name, value: original });
        const written = Reflect.set(state, name, Symbol("forbidden mutation"));
        record("binding-write-before-comparison", { name, written });
        assert.equal(written, false);
        const afterWrite = state[name];
        record("binding-value-after-write", { name, value: afterWrite });
        assert.equal(afterWrite, original);
      }
    }
    const initialNodes = host.children.filter((node: any) => node.type === fixture.tag);
    function snapshot() {
      const root = instance.subTree;
      return {
        tree: shape(root),
        trackedDynamicChildren: root.dynamicChildren?.length ?? null,
        callbackBlocks: Array.isArray(root.children)
          ? root.children.map((node: any) => Array.isArray(node.dynamicChildren))
          : [],
        hostValues: host.children
          .filter((node: any) => node.type === fixture.tag)
          .map((node: any) => node.text),
      };
    }
    const initial = snapshot();
    record("initial-before-force-update", initial);
    instance.proxy.$forceUpdate();
    await runtime.nextTick();
    record("force-updated-before-snapshot-comparisons");
    const updated = snapshot();
    record("updated-before-comparisons", updated);
    assert.equal(calls, 1);
    assert.equal(renders, 2);
    const updatedNodes = host.children.filter((node: any) => node.type === fixture.tag);
    const retained = initialNodes.map((node: any, index: number) => updatedNodes[index] === node);
    record("updated-host-identities-before-comparison", { retained });
    assert(
      retained.every(Boolean),
      "actual stable-list host identities survive normal force-update",
    );
    if (!range) {
      assert.deepEqual(initial, updated);
      assert.equal(initial.tree.type, "fragment");
      assert.equal(initial.tree.patchFlag, 64);
      assert.equal(initial.trackedDynamicChildren, fixture.dynamic ? fixture.values.length : 0);
      assert.deepEqual(
        initial.callbackBlocks,
        fixture.values.map(() => false),
      );
      assert.deepEqual(initial.hostValues, fixture.values);
      assert.deepEqual(
        initial.tree.children,
        fixture.values.map((value: string) => ({
          type: fixture.tag,
          props: {},
          children: fixture.dynamic ? value : fixture.text,
          patchFlag: fixture.dynamic ? 1 : 0,
        })),
      );
      assert.deepEqual(warnings, []);
      assert.deepEqual(errors, []);
      assert.equal(host.children.length, fixture.values.length + 2);
    } else if (mode === "development") {
      assert.deepEqual(errors, []);
      assert.deepEqual(
        warnings,
        Array(2).fill("The v-for range expects a positive integer value but got 2.5."),
      );
      assert.equal(initial.tree.patchFlag, 64);
      assert.deepEqual(initial.tree.children, []);
      assert.equal(initial.trackedDynamicChildren, 0);
      assert.deepEqual(initial.callbackBlocks, []);
      assert.deepEqual(initial.hostValues, []);
      assert.equal(host.children.length, 2);
      assert.deepEqual(initial, updated);
    } else {
      assert.deepEqual(warnings, []);
      assert.equal(errors.length, 2);
      for (const error of errors) {
        assert.equal(error.name, "RangeError");
        assert.equal(error.message, "Invalid array length");
        assert.equal(error.info, "https://vuejs.org/error-reference/#runtime-1");
      }
      assert.deepEqual(initial.tree, {
        type: "comment",
        props: {},
        children: null,
        patchFlag: 0,
      });
      assert.equal(initial.trackedDynamicChildren, null);
      assert.deepEqual(initial.callbackBlocks, []);
      assert.deepEqual(initial.hostValues, []);
      assert.equal(host.children.length, 1);
      assert.deepEqual(initial, updated);
    }
    app.unmount();
    await runtime.nextTick();
    record("unmounted-before-comparisons", updated);
    assert.deepEqual(host.children, []);
    assert.equal(instance.isUnmounted, true);
    return {
      initial,
      updated,
      retained,
      warnings,
      errors,
      setupInvocations: calls,
      renders,
      unmounted: instance.isUnmounted,
    };
  } catch (error) {
    record("failed-before-cleanup");
    throw error;
  } finally {
    // The existing genuine app is cleaned up even when an assertion interrupts.
    // Never retry its setup/render or replace an observed failed result.
    if (instance && !instance.isUnmounted) {
      app.unmount();
      await runtime.nextTick();
      record("failed-after-normal-cleanup");
    }
  }
}
