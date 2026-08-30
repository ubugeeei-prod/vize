import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import List from "./list.vue";

const SsrProbe = defineComponent({
  name: "ListSsrProbe",
  setup() {
    return () =>
      h(
        List,
        {
          marker: "disc",
          spacing: "normal",
          tone: "neutral",
        },
        {
          default: () => h("li", "Ship the primitive"),
        },
      );
  },
});

test("renders byte-identical native list markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<ul/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="list"/);
  assert.match(html, /data-marker="disc"/);
  assert.match(html, /data-spacing="normal"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /<li>Ship the primitive<\/li>/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});

test("renders consumer-owned server semantics without implicit accessibility attrs", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "ListCustomSsrProbe",
      setup() {
        return () =>
          h(
            List,
            {
              "aria-label": "Install steps",
              as: "ol",
              marker: "decimal",
              role: "list",
              spacing: "loose",
              tabindex: "0",
              tone: "accent",
            },
            {
              default: () => h("li", "Run the checks"),
            },
          );
      },
    }),
  );

  assert.match(html, /^<ol/);
  assert.match(html, /role="list"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-label="Install steps"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="list"/);
  assert.match(html, /data-marker="decimal"/);
  assert.match(html, /data-spacing="loose"/);
  assert.match(html, /data-tone="accent"/);
  assert.match(html, /<li>Run the checks<\/li>/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
  assert.doesNotMatch(html, /style=/);
});
