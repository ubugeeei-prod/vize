import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import AspectRatio from "./aspect-ratio.vue";

const SsrProbe = defineComponent({
  name: "AspectRatioSsrProbe",
  setup() {
    return () =>
      h(
        AspectRatio,
        { as: "figure", ratio: 16 / 9 },
        {
          default: () => "Poster",
        },
      );
  },
});

test("renders byte-identical intrinsic ratio markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<figure/);
  assert.match(html, /data-vize-ui="aspect-ratio"/);
  assert.match(html, /data-state="valid"/);
  assert.match(html, /data-vize-aspect-ratio="1\.7777777777777777"/);
  assert.match(html, /--vize-ui-aspect-ratio:1\.7777777777777777/);
  assert.match(html, /aspect-ratio:var\(--vize-ui-aspect-ratio\)/);
  assert.match(html, /Poster/);
});

test("renders fallback ratio markup for invalid server input", async () => {
  const InvalidProbe = defineComponent({
    name: "AspectRatioInvalidSsrProbe",
    setup() {
      return () => h(AspectRatio, { ratio: 0 }, { default: () => "Fallback" });
    },
  });
  const html = await renderToString(createSSRApp(InvalidProbe));

  assert.match(html, /data-state="fallback"/);
  assert.match(html, /data-vize-aspect-ratio="1"/);
  assert.match(html, /--vize-ui-aspect-ratio:1/);
});
