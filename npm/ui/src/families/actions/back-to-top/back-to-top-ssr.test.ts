import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import BackToTop from "./back-to-top.vue";

const Probe = defineComponent({
  name: "BackToTopSsrProbe",
  setup: () => () => h(BackToTop, { threshold: 200 }),
});

test("renders a hidden, deterministic button on the server", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<button type="button" hidden/);
  assert.match(html, /data-vize-ui="back-to-top"/);
  assert.match(html, /data-state="hidden"/);
  assert.match(html, /Back to top/);
});

test("hydrates without diagnostics and starts listening after mount", async () => {
  const host = document.createElement("div");
  host.innerHTML = await renderToString(createSSRApp(Probe));
  document.body.append(host);
  const serverButton = host.firstElementChild;
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
    assert.ok(host.firstElementChild === serverButton);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
