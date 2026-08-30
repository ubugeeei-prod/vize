import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Separator from "./separator.vue";

const SsrProbe = defineComponent({
  name: "SeparatorSsrProbe",
  setup() {
    return () => h(Separator, { ariaLabel: "Pane boundary", as: "div", orientation: "vertical" });
  },
});

test("renders byte-identical semantic separator markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<div/);
  assert.match(html, /role="separator"/);
  assert.match(html, /aria-orientation="vertical"/);
  assert.match(html, /aria-label="Pane boundary"/);
  assert.match(html, /data-vize-ui="separator"/);
  assert.match(html, /data-state="semantic"/);
  assert.match(html, /data-orientation="vertical"/);
});

test("renders decorative server markup without semantic ARIA", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "SeparatorDecorativeSsrProbe",
      setup() {
        return () =>
          h(Separator, { ariaLabel: "Ignored label", decorative: true, orientation: "vertical" });
      },
    }),
  );

  assert.match(html, /role="presentation"/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /data-state="decorative"/);
  assert.match(html, /data-orientation="vertical"/);
  assert.doesNotMatch(html, /role="separator"/);
  assert.doesNotMatch(html, /aria-orientation=/);
  assert.doesNotMatch(html, /aria-label=/);
});
