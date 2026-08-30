import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Code from "./code.vue";

const SsrProbe = defineComponent({
  name: "CodeSsrProbe",
  setup() {
    return () =>
      h(
        Code,
        {
          size: "md",
          tone: "neutral",
          variant: "inline",
        },
        {
          default: () => "const value = 1;",
        },
      );
  },
});

test("renders byte-identical native code markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<code/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="code"/);
  assert.match(html, /data-size="md"/);
  assert.match(html, /data-variant="inline"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /const value = 1;/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});

test("renders consumer-owned server semantics without implicit accessibility attrs", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "CodeCustomSsrProbe",
      setup() {
        return () =>
          h(
            Code,
            {
              "aria-label": "Build command",
              as: "pre",
              role: "region",
              size: "lg",
              tabindex: "0",
              tone: "accent",
              variant: "block",
            },
            {
              default: () => "vp pack",
            },
          );
      },
    }),
  );

  assert.match(html, /^<pre/);
  assert.match(html, /role="region"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-label="Build command"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="code"/);
  assert.match(html, /data-size="lg"/);
  assert.match(html, /data-variant="block"/);
  assert.match(html, /data-tone="accent"/);
  assert.match(html, /vp pack/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});
