import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Text from "./text.vue";

const SsrProbe = defineComponent({
  name: "TextSsrProbe",
  setup() {
    return () =>
      h(
        Text,
        {
          size: "md",
          tone: "neutral",
          weight: "regular",
        },
        {
          default: () => "Readable copy",
        },
      );
  },
});

test("renders byte-identical neutral text markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<span/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="text"/);
  assert.match(html, /data-size="md"/);
  assert.match(html, /data-weight="regular"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /data-truncate="false"/);
  assert.match(html, /Readable copy/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});

test("renders consumer-owned server semantics without implicit accessibility attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "TextCustomSsrProbe",
      setup() {
        return () =>
          h(
            Text,
            {
              "aria-live": "polite",
              as: "p",
              role: "status",
              size: "xl",
              tabindex: "0",
              tone: "danger",
              truncate: true,
              weight: "bold",
            },
            {
              default: () => "Payment failed",
            },
          );
      },
    }),
  );

  assert.match(html, /^<p/);
  assert.match(html, /role="status"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="text"/);
  assert.match(html, /data-size="xl"/);
  assert.match(html, /data-weight="bold"/);
  assert.match(html, /data-tone="danger"/);
  assert.match(html, /data-truncate="true"/);
  assert.match(html, /Payment failed/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /style=/);
});
