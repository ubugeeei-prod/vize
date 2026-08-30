import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import EmptyState from "./empty-state.vue";

const SsrProbe = defineComponent({
  name: "EmptyStateSsrProbe",
  setup() {
    return () =>
      h(
        EmptyState,
        {
          density: "comfortable",
          orientation: "block",
          tone: "neutral",
        },
        {
          default: () => "No projects yet",
        },
      );
  },
});

test("renders byte-identical default empty-state markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<section/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="empty-state"/);
  assert.match(html, /data-state="empty"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /data-density="comfortable"/);
  assert.match(html, /data-orientation="block"/);
  assert.match(html, /No projects yet/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
});

test("renders custom server markup without implicit accessibility attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "EmptyStateCustomSsrProbe",
      setup() {
        return () =>
          h(
            EmptyState,
            {
              as: "article",
              density: "compact",
              orientation: "inline",
              tone: "warning",
            },
            {
              default: () => "No matching filters",
            },
          );
      },
    }),
  );

  assert.match(html, /^<article/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="empty-state"/);
  assert.match(html, /data-state="empty"/);
  assert.match(html, /data-tone="warning"/);
  assert.match(html, /data-density="compact"/);
  assert.match(html, /data-orientation="inline"/);
  assert.match(html, /No matching filters/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
});
