import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import ResizableHandle from "./resizable-handle.vue";
import ResizableRoot from "./resizable-root.vue";

const Probe = defineComponent({
  name: "ResizableSsrProbe",
  setup() {
    return () =>
      h(ResizableRoot, { minWidth: 120, maxWidth: 480 }, () => [
        "Panel",
        h(ResizableHandle, { edge: "e" }),
        h(ResizableHandle, { edge: "se" }),
      ]);
  },
});

test("renders byte-identical sized markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /id="vize-v-\d+-resizable"/);
  assert.match(html, /width:320px/);
  assert.match(html, /--vize-resizable-height:240px/);
  assert.match(html, /role="separator"/);
  assert.match(html, /aria-valuenow="320"/);
  assert.match(html, /aria-valuemax="480"/);
  assert.match(html, /aria-controls="vize-v-\d+-resizable"/);
  assert.match(html, /aria-valuetext="Width 320 pixels, height 240 pixels"/);
});

test("hydrates handles without mismatches", async () => {
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverHandle = host.querySelector('[data-vize-ui="resizable-handle"]');
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    assert.deepEqual(diagnostics, []);
    const hydrated = host.querySelector('[data-vize-ui="resizable-handle"]');
    assert.ok(hydrated === serverHandle);
    hydrated?.dispatchEvent(
      new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }),
    );
    await nextTick();
    assert.equal(hydrated?.getAttribute("aria-valuenow"), "330");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
