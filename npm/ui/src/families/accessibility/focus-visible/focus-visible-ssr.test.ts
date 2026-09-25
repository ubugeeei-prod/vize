import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import FocusVisibleProvider from "./focus-visible-provider.vue";

const Probe = defineComponent({
  name: "FocusVisibleSsrProbe",
  setup: () => () =>
    h(FocusVisibleProvider, null, ({ modality }: { readonly modality: string | null }) => [
      h("button", { type: "button" }, `Modality ${modality ?? "none"}`),
    ]),
});

test("renders byte-identical markup without modality or focus attributes", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /data-vize-ui="focus-visible-provider"/);
  assert.doesNotMatch(html, /data-vize-modality/);
  assert.doesNotMatch(html, /data-focus-visible/);
  assert.match(html, /Modality none/);
});

test("hydrates without diagnostics even when the document already has a modality", async () => {
  document.body.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Tab" }));
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
    assert.equal(serverRoot?.getAttribute("data-vize-modality"), "keyboard");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
