import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { usePointerGrace } from "./pointer-grace.ts";

const SsrProbe = defineComponent({
  name: "PointerGraceSsrProbe",
  setup() {
    const grace = usePointerGrace({ delay: 300 });
    return () =>
      h(
        "div",
        { "data-pending": String(grace.isPending.value) },
        grace.polygon.value === null ? "idle" : "armed",
      );
  },
});

test("renders byte-identical pointer-grace SSR output without timers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-pending="false">idle</div>');
});

test("hydrates pointer grace without replacement or diagnostics", async () => {
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
    assert.equal(host.firstElementChild?.getAttribute("data-pending"), "false");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
