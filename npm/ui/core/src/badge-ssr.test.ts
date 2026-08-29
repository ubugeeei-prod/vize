import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Badge from "./badge.vue";

const SsrProbe = defineComponent({
  name: "BadgeSsrProbe",
  setup() {
    return () =>
      h(
        Badge,
        {
          tone: "neutral",
          variant: "label",
        },
        {
          default: () => "Beta",
        },
      );
  },
});

test("renders byte-identical label badge markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<span/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="badge"/);
  assert.match(html, /data-variant="label"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /Beta/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
});

test("renders custom server markup without implicit accessibility attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "BadgeCustomSsrProbe",
      setup() {
        return () =>
          h(
            Badge,
            {
              as: "strong",
              tone: "success",
              variant: "status",
            },
            {
              default: () => "Online",
            },
          );
      },
    }),
  );

  assert.match(html, /^<strong/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="badge"/);
  assert.match(html, /data-variant="status"/);
  assert.match(html, /data-tone="success"/);
  assert.match(html, /Online/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
});
