import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Kbd from "./kbd.vue";

const SsrProbe = defineComponent({
  name: "KbdSsrProbe",
  setup() {
    return () =>
      h(
        Kbd,
        {
          size: "md",
          tone: "neutral",
          variant: "key",
        },
        {
          default: () => "Esc",
        },
      );
  },
});

test("renders byte-identical native kbd markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<kbd/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="kbd"/);
  assert.match(html, /data-size="md"/);
  assert.match(html, /data-variant="key"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /Esc/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /style=/);
});

test("renders consumer-owned server semantics without implicit accessibility attrs", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "KbdCustomSsrProbe",
      setup() {
        return () =>
          h(
            Kbd,
            {
              "aria-label": "Command palette shortcut",
              as: "span",
              role: "term",
              size: "lg",
              tabindex: "0",
              tone: "accent",
              variant: "shortcut",
            },
            {
              default: () => "Ctrl K",
            },
          );
      },
    }),
  );

  assert.match(html, /^<span/);
  assert.match(html, /role="term"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-label="Command palette shortcut"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="kbd"/);
  assert.match(html, /data-size="lg"/);
  assert.match(html, /data-variant="shortcut"/);
  assert.match(html, /data-tone="accent"/);
  assert.match(html, /Ctrl K/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /style=/);
});
