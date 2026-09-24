import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { fixtureEmoji, renderEmojiPickerTree } from "./emoji-picker-fixture-tree.ts";

const SsrProbe = defineComponent({
  name: "EmojiPickerSsrProbe",
  setup: () => () => renderEmojiPickerTree({ defaultSkinTone: 2, recent: [fixtureEmoji[4]] }),
});

test("renders byte-identical EmojiPicker markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /id="emoji-grid"[^>]*role="grid"/);
  assert.match(html, /aria-rowcount="7"/);
  assert.match(html, /role="rowgroup"[^>]*aria-labelledby="emoji-section-0"/);
  assert.match(html, /id="emoji-cell-1-2"[^>]*role="gridcell"[^>]*aria-label="wave"/);
  assert.match(html, /👋🏼/);
  assert.match(html, /role="radiogroup"/);
  assert.match(html, /data-skin-tone="2"/);
});

test("hydrates EmojiPicker without mismatches or node replacement", async () => {
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
