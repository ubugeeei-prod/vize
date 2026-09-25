import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { renderTransferListTree } from "./transfer-list-fixture-tree.ts";

const SsrProbe = defineComponent({
  name: "TransferListSsrProbe",
  setup: () => renderTransferListTree,
});

test("renders byte-identical TransferList markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /id="fruit-transfer-source-listbox"[^>]*role="listbox"/);
  assert.match(html, /id="fruit-transfer-target-listbox"/);
  assert.match(html, /aria-multiselectable="true"/);
  assert.match(html, /type="search"/);
  assert.match(html, /data-action="move-all-to-target"/);
  assert.match(html, /type="hidden"[^>]*name="fruits"[^>]*value="Banana"/);
});

test("hydrates TransferList without mismatches or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
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
