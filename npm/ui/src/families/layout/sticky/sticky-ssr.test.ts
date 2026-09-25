import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import Sticky from "./sticky.vue";

const Probe = defineComponent({
  name: "StickySsrProbe",
  setup: () => () => h(Sticky, { offset: 8, as: "nav" }, () => "Section navigation"),
});

test("renders byte-identical sticky markup without stuck state on the server", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<nav/);
  assert.match(html, /position:sticky/);
  assert.match(html, /top:8px/);
  assert.match(html, /data-state="idle"/);
  assert.doesNotMatch(html, /data-stuck/);
});

test("hydrates the server box without diagnostics", async () => {
  const host = document.createElement("div");
  host.innerHTML = await renderToString(createSSRApp(Probe));
  document.body.append(host);
  const serverNav = host.firstElementChild;
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
    assert.ok(host.firstElementChild === serverNav);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
