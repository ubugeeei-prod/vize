import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import DirectionProvider from "./direction-provider.vue";
import TabsList from "../../navigation/tabs/tabs-list.vue";
import TabsRoot from "../../navigation/tabs/tabs-root.vue";
import TabsTrigger from "../../navigation/tabs/tabs-trigger.vue";

const Probe = defineComponent({
  name: "DirectionSsrProbe",
  setup: () => () =>
    h(DirectionProvider, { dir: "rtl" }, () =>
      h(TabsRoot, { defaultValue: "a" }, () =>
        h(TabsList, null, () => [
          h(TabsTrigger, { value: "a" }, () => "A"),
          h(TabsTrigger, { value: "b" }, () => "B"),
        ]),
      ),
    ),
});

test("server output resolves inherited direction without reading the document", async () => {
  document.documentElement.dir = "ltr";
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div dir="rtl" data-vize-ui="direction-provider"/);
  assert.match(html, /data-vize-ui="tabs-root"[^>]*|dir="rtl"[^>]*data-vize-ui="tabs-root"/);
  assert.equal((html.match(/dir="rtl"/g) ?? []).length >= 2, true);
});

test("hydrates inherited direction without diagnostics", async () => {
  const host = document.createElement("div");
  host.innerHTML = await renderToString(createSSRApp(Probe));
  document.body.append(host);
  const serverRoot = host.firstElementChild;
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
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
