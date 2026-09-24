import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  isPolymorphicTag,
  renderPolymorphic,
  resolvePolymorphicAttributes,
} from "./polymorphic.ts";

const Badge = defineComponent({
  props: { tone: { type: String, required: true } },
  setup:
    (props, { slots }) =>
    () =>
      h("span", { "data-tone": props.tone }, slots["default"]?.()),
});

test("distinguishes native tags from components", () => {
  assert.equal(isPolymorphicTag("div"), true);
  assert.equal(isPolymorphicTag(Badge), false);
});

test("buttons default to type=button while explicit types win", () => {
  assert.deepEqual(resolvePolymorphicAttributes("button", {}), { type: "button" });
  assert.deepEqual(resolvePolymorphicAttributes("button", { type: "submit" }), { type: "submit" });
});

test("disabled anchors and generic elements expose aria-disabled", () => {
  assert.deepEqual(resolvePolymorphicAttributes("a", { href: "/x", disabled: true, id: "l" }), {
    "aria-disabled": "true",
    tabindex: -1,
    id: "l",
  });
  assert.deepEqual(resolvePolymorphicAttributes("div", { disabled: true }), {
    "aria-disabled": "true",
  });
  assert.deepEqual(resolvePolymorphicAttributes("a", { href: "/x" }), { href: "/x" });
  assert.deepEqual(resolvePolymorphicAttributes(Badge, { tone: "info" }), { tone: "info" });
  const empty = "" as "div";
  assert.throws(() => resolvePolymorphicAttributes(empty, {}), /VIZE_UI_POLYMORPHIC_AS/);
});

test("renders native tags and components with slot children in the DOM", () => {
  const wrapper = mount(
    defineComponent(
      () => () =>
        h("div", [
          renderPolymorphic("button", { class: "b" }, () => "Save"),
          renderPolymorphic(Badge, { tone: "info" }, () => "New"),
          renderPolymorphic("a", { href: "/docs" }, "Docs"),
        ]),
    ),
  );
  assert.equal(
    wrapper.html(),
    '<div><button type="button" class="b">Save</button><span data-tone="info">New</span><a href="/docs">Docs</a></div>',
  );
  wrapper.unmount();
});

const Probe = defineComponent(
  () => () => renderPolymorphic("a", { disabled: true, href: "/x" }, "Off"),
);

test("renders identical markup on the server", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<a aria-disabled="true" tabindex="-1">Off</a>');
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
