import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { useSizeObserver, useVisibilityObserver } from "./measure.ts";

const SsrProbe = defineComponent({
  name: "MeasureSsrProbe",
  setup() {
    const sizes = useSizeObserver({ onResize: () => {} });
    const visibility = useVisibilityObserver({ onVisibilityChange: () => {} });
    return () =>
      h(
        "div",
        {
          "data-observed": String(sizes.observedCount.value),
          "data-visible-observed": String(visibility.observedCount.value),
        },
        "Measured content",
      );
  },
});

test("renders byte-identical SSR output without observers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<div data-observed="0" data-visible-observed="0">Measured content</div>',
  );
});

test("hydrates measurement consumers without diagnostics", async () => {
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
    assert.equal(host.firstElementChild?.getAttribute("data-observed"), "0");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
