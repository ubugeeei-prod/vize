import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import StickyStackItem from "./sticky-stack-item.vue";
import StickyStack from "./sticky-stack.vue";

const Probe = defineComponent({
  name: "StickyStackSsrProbe",
  setup: () => () =>
    h(StickyStack, { offset: 4 }, () => [
      h(StickyStackItem, { as: "header", estimatedHeight: 56 }, () => "App bar"),
      h(StickyStackItem, { as: "nav", estimatedHeight: 40 }, () => "Tabs"),
    ]),
});

test("renders byte-identical estimated offsets across SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  // Items register while rendering, after the root's attributes are serialized.
  assert.match(html, /--vize-ui-sticky-stack-height:4px/);
  assert.match(html, /<header[^>]*position:sticky;top:4px;/);
  assert.match(html, /<nav[^>]*position:sticky;top:60px;/);
});

test("hydrates estimated offsets without mismatch warnings", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.querySelectorAll('[data-vize-ui="sticky-stack-item"]').length, 2);
  } finally {
    app.unmount();
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
  }
  assert.deepEqual(diagnostics, []);
});
