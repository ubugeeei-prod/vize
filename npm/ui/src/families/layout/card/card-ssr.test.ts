import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Card from "./card.vue";

const SsrProbe = defineComponent({
  name: "CardSsrProbe",
  setup() {
    return () =>
      h(Card, null, {
        default: () => "Account overview",
      });
  },
});

test("renders byte-identical default card markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<section/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="card"/);
  assert.match(html, /data-variant="card"/);
  assert.match(html, /data-density="comfortable"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /Account overview/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
});

test("renders custom server markup without implicit accessibility attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "CardCustomSsrProbe",
      setup() {
        return () =>
          h(
            Card,
            {
              "aria-label": "Billing summary",
              as: "article",
              density: "spacious",
              role: "region",
              tone: "accent",
              variant: "panel",
            },
            {
              default: () => "Plan details",
            },
          );
      },
    }),
  );

  assert.match(html, /^<article/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="card"/);
  assert.match(html, /data-variant="panel"/);
  assert.match(html, /data-density="spacious"/);
  assert.match(html, /data-tone="accent"/);
  assert.match(html, /role="region"/);
  assert.match(html, /aria-label="Billing summary"/);
  assert.match(html, /Plan details/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
});
