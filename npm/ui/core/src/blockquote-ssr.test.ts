import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Blockquote from "./blockquote.vue";

const SsrProbe = defineComponent({
  name: "BlockquoteSsrProbe",
  setup() {
    return () =>
      h(
        Blockquote,
        {
          cite: "https://example.com/source",
          size: "md",
          tone: "neutral",
        },
        {
          default: () => "Readable pull quote",
        },
      );
  },
});

test("renders byte-identical native blockquote markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<blockquote/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="blockquote"/);
  assert.match(html, /cite="https:\/\/example\.com\/source"/);
  assert.match(html, /data-size="md"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /Readable pull quote/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});

test("renders consumer-owned server semantics without implicit accessibility attrs", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "BlockquoteCustomSsrProbe",
      setup() {
        return () =>
          h(
            Blockquote,
            {
              "aria-label": "Customer quote",
              as: "figure",
              role: "group",
              size: "lg",
              tabindex: "0",
              tone: "accent",
            },
            {
              default: () => "The workflow now feels predictable.",
            },
          );
      },
    }),
  );

  assert.match(html, /^<figure/);
  assert.match(html, /role="group"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-label="Customer quote"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="blockquote"/);
  assert.match(html, /data-size="lg"/);
  assert.match(html, /data-tone="accent"/);
  assert.match(html, /The workflow now feels predictable/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /cite=/);
  assert.doesNotMatch(html, /style=/);
});
