import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import {
  createCommentVNode,
  createSSRApp,
  defineComponent,
  Fragment,
  h,
  nextTick,
  useSlots,
} from "vue";
import { renderToString } from "vue/server-renderer";

import { hasSlotContent, isHandlerKey, mergeProps, presentSlotNames } from "./slot-utils.ts";

test("merges class, style, handlers, and last defined values", () => {
  const calls: string[] = [];
  const merged = mergeProps(
    {
      id: "a",
      class: ["x", { y: false }],
      style: "color: red; margin: 0",
      onClick: () => calls.push("first"),
    },
    {
      class: { z: true },
      style: { color: "blue" },
      onClick: () => calls.push("second"),
      title: "t",
    },
    { id: undefined, title: "u", "onUpdate:modelValue": () => calls.push("model") },
  );
  assert.equal(merged.id, "a");
  assert.equal(merged.title, "u");
  assert.equal(merged.class, "x z");
  assert.deepEqual(merged.style, { color: "blue", margin: "0" });
  merged.onClick(new MouseEvent("click"));
  merged["onUpdate:modelValue"]();
  assert.deepEqual(calls, ["first", "second", "model"]);
});

test("keeps an explicit undefined when no earlier value exists and passes single handlers through", () => {
  const handler = (): void => undefined;
  const merged = mergeProps({ value: undefined }, { onFocus: handler });
  assert.equal(Object.hasOwn(merged, "value"), true);
  assert.equal(merged.onFocus, handler);
  assert.deepEqual(mergeProps(), {});
});

test("recognizes handler keys", () => {
  assert.equal(isHandlerKey("onClick"), true);
  assert.equal(isHandlerKey("onUpdate:modelValue"), true);
  assert.equal(isHandlerKey("once"), false);
  assert.equal(isHandlerKey("on"), false);
});

test("detects slot content through comments, whitespace, and fragments", () => {
  assert.equal(hasSlotContent(undefined), false);
  assert.equal(
    hasSlotContent(() => [createCommentVNode("v-if")]),
    false,
  );
  assert.equal(
    hasSlotContent(() => ["   "]),
    false,
  );
  assert.equal(
    hasSlotContent(() => [h(Fragment, [createCommentVNode("x")])]),
    false,
  );
  assert.equal(
    hasSlotContent(() => [h(Fragment, [h("span")])]),
    true,
  );
  assert.equal(
    hasSlotContent((props: { readonly n: number }) => [String(props.n)], { n: 1 }),
    true,
  );
});

const Card = defineComponent(() => {
  const slots = useSlots();
  return () =>
    h("section", { "data-slots": presentSlotNames(slots).join(",") }, [
      hasSlotContent(slots["header"]) ? h("header", slots["header"]?.()) : null,
      slots["default"]?.(),
    ]);
});

test("skips wrappers for empty slots in the mounted DOM", () => {
  const wrapper = mount(Card, {
    slots: { header: () => [createCommentVNode("off")], default: () => "Body" },
  });
  assert.equal(wrapper.find("header").exists(), false);
  assert.equal(wrapper.attributes("data-slots"), "default");
  assert.equal(wrapper.text(), "Body");
  wrapper.unmount();
});

const Probe = defineComponent(
  () => () => h(Card, null, { header: () => "Title", default: () => "Body" }),
);

test("renders identical slot-aware markup on the server", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<section data-slots="header,default"><header>Title</header><!--[-->Body<!--]--></section>',
  );
});

test("hydrates server markup without mismatch diagnostics", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  try {
    const app = createSSRApp(Probe);
    app.mount(host);
    await nextTick();
    assert.equal(host.innerHTML, html);
    app.unmount();
  } finally {
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
  }
  assert.deepEqual(diagnostics, []);
});
