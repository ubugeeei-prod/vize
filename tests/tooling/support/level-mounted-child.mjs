import assert from "node:assert/strict";
import { evaluateCompiledRender } from "./davinci-runtime-trace.mjs";

/**
 * A child component compiled by the same backend and lane as its parent. The
 * render context exposes props plus `$emit`/`$slots`, as compiled templates
 * expect from a component instance.
 */
export async function childComponent(backend, vue, name, { code, props = [], emits = [] }) {
  assert.ok(Array.isArray(props) && Array.isArray(emits), "child props/emits must be arrays");
  const render = await evaluateCompiledRender(code, vue);
  const context = (instanceProps, emit, slots) =>
    new Proxy(instanceProps, {
      get: (target, key) =>
        key === "$emit" || key === "send"
          ? emit
          : key === "$slots"
            ? slots
            : Reflect.get(target, key),
      has: (target, key) =>
        key === "$emit" || key === "send" || key === "$slots" || Reflect.has(target, key),
    });
  if (backend === "vapor") {
    return vue.defineVaporComponent({
      name,
      props,
      emits,
      setup: (instanceProps, { emit, slots }) => render(context(instanceProps, emit, slots)),
    });
  }
  return {
    name,
    props,
    emits,
    setup: (instanceProps, { emit, slots }) => {
      const cache = [];
      return () => render(context(instanceProps, emit, slots), cache);
    },
  };
}
