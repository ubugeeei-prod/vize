import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import Masonry from "./masonry.vue";

const items = [120, 80, 200, 60, 90].map((size, index) => ({ id: `tile-${index}`, size }));

const Probe = defineComponent({
  name: "MasonrySsrProbe",
  setup: () => () =>
    h(
      Masonry,
      {
        items,
        columns: 3,
        estimateHeight: (item: { size: number }) => item.size,
        getKey: (item: { id: string }) => item.id,
      },
      { item: ({ item }: { item: { id: string } }) => h("figure", item.id) },
    ),
});

test("renders a byte-identical estimated distribution across SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="masonry"/);
  assert.equal(html.match(/data-part="column"/g)?.length, 3);
  assert.match(html, /data-column="1"[^]*tile-1[^]*tile-3/);
});

test("hydrates the estimated distribution without mismatch warnings", async () => {
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
    assert.equal(host.querySelectorAll('[data-part="item"]').length, 5);
  } finally {
    app.unmount();
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
  }
  assert.deepEqual(diagnostics, []);
});
