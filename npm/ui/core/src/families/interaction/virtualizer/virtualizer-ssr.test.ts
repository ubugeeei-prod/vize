import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { useVirtualizer } from "./virtualizer.ts";

const SsrProbe = defineComponent({
  name: "VirtualizerSsrProbe",
  setup() {
    const virtualizer = useVirtualizer({
      count: 1000,
      itemSize: 20,
      overscan: 1,
      initialRect: { width: 320, height: 60 },
    });
    return () =>
      h(
        "div",
        { "data-total": virtualizer.totalSize.value },
        virtualizer.virtualItems.value.map((item) =>
          h("div", { key: item.key, "data-index": item.index }, `Row ${item.index}`),
        ),
      );
  },
});

test("renders byte-identical SSR windows", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<div data-total="20000">' +
      '<div data-index="0">Row 0</div>' +
      '<div data-index="1">Row 1</div>' +
      '<div data-index="2">Row 2</div>' +
      '<div data-index="3">Row 3</div>' +
      "</div>",
  );
});

test("hydrates the server-rendered window without diagnostics", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTarget = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverTarget);
    await nextTick();
    assert.equal(host.querySelectorAll("[data-index]").length, 4);
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
