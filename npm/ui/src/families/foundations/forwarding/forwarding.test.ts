import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, reactive, shallowRef } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  toHandlerKey,
  useEmitAsProps,
  useForwardExpose,
  useForwardProps,
  useForwardPropsEmits,
} from "./forwarding.ts";

const Counter = defineComponent({
  props: { label: { type: String, default: "Count" }, step: { type: Number, default: 1 } },
  emits: {
    "update:modelValue": (_value: number) => true,
    "value-change": (_value: number) => true,
  },
  setup(props, { emit, expose }) {
    const count = shallowRef(0);
    const increment = (): void => {
      count.value += props.step;
      emit("update:modelValue", count.value);
      emit("value-change", count.value);
    };
    expose({ increment, count });
    return () =>
      h("button", { type: "button", onClick: increment }, `${props.label}:${count.value}`);
  },
});

const Wrapper = defineComponent({
  props: {
    label: { type: String, default: undefined },
    step: { type: Number, default: undefined },
  },
  emits: {
    "update:modelValue": (_value: number) => true,
    "value-change": (_value: number) => true,
  },
  setup(props, { emit, expose }) {
    const forwarded = useForwardPropsEmits(props, emit, ["update:modelValue", "value-change"]);
    const { forwardRef, exposed } = useForwardExpose<{ increment: () => void }>();
    expose(exposed);
    return () => h(Counter, { ...forwarded.value, ref: forwardRef });
  },
});

test("converts event names to Vue handler keys", () => {
  assert.equal(toHandlerKey("update:modelValue"), "onUpdate:modelValue");
  assert.equal(toHandlerKey("value-change"), "onValueChange");
  assert.equal(toHandlerKey("close"), "onClose");
});

test("re-emits selected events with their payloads", () => {
  const seen: unknown[][] = [];
  const handlers = useEmitAsProps(
    (event: "a" | "b", ...payload: unknown[]) => void seen.push([event, ...payload]),
    ["a"],
  );
  handlers.onA(1, 2);
  assert.deepEqual(seen, [["a", 1, 2]]);
  assert.equal(Object.isFrozen(handlers), true);
});

test("forwards only defined props so child defaults survive", async () => {
  const props = reactive<{ label?: string | undefined; step?: number | undefined }>({
    label: undefined,
    step: 2,
  });
  const forwarded = useForwardProps(props);
  assert.deepEqual(forwarded.value, { step: 2 });
  props.label = "Items";
  await nextTick();
  assert.deepEqual(forwarded.value, { label: "Items", step: 2 });
});

test("wrappers forward props, emits, and the child's exposed API in the DOM", async () => {
  const wrapper = mount(Wrapper, { props: { step: 5 } });
  assert.equal(wrapper.text(), "Count:0");

  await wrapper.find("button").trigger("click");
  assert.deepEqual(wrapper.emitted("update:modelValue"), [[5]]);
  assert.deepEqual(wrapper.emitted("value-change"), [[5]]);

  const exposed = wrapper.vm as unknown as { increment: () => void; $el: Element };
  exposed.increment();
  await nextTick();
  assert.equal(wrapper.text(), "Count:10");
  assert.equal(wrapper.emitted("update:modelValue")?.length, 2);
  wrapper.unmount();
});

test("forwarded expose binds element methods and is empty before mount", () => {
  const { forwardRef, exposed, current } = useForwardExpose<{ focus: () => void }>();
  assert.equal(exposed.$el, null);
  assert.equal(exposed.focus, undefined);
  assert.equal("focus" in exposed, false);

  const input = document.createElement("input");
  document.body.append(input);
  forwardRef(input);
  assert.equal(current.value, input);
  assert.equal(exposed.$el, input);
  exposed.focus?.();
  assert.equal(document.activeElement, input);
  input.remove();
});

test("renders identical wrapper markup on the server and hydrates cleanly", async () => {
  const Probe = defineComponent(() => () => h(Wrapper, { label: "Stars" }));
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<button type="button">Stars:0</button>');

  const host = document.createElement("div");
  host.innerHTML = outputs[0];
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  try {
    const app = createSSRApp(Probe);
    app.mount(host);
    host.querySelector("button")?.click();
    await nextTick();
    assert.equal(host.textContent, "Stars:1");
    app.unmount();
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
  }
  assert.deepEqual(diagnostics, []);
});
