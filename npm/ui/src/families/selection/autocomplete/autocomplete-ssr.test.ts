import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { renderAutocompleteTree } from "./autocomplete-fixture-tree.ts";

const SsrProbe = defineComponent({
  name: "AutocompleteSsrProbe",
  setup: () => renderAutocompleteTree,
});

test("renders byte-identical Autocomplete markup with history suggestions", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui-preset="autocomplete"/);
  assert.match(html, /data-history-count="2"/);
  assert.match(html, /role="combobox"/);
  assert.match(html, /data-clear/);
  assert.match(html, />vize</);
});

test("hydrates Autocomplete without mismatches or node replacement", async () => {
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
