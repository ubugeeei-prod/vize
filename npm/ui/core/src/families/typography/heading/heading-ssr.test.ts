import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Heading from "./heading.vue";

const SsrProbe = defineComponent({
  name: "HeadingSsrProbe",
  setup() {
    return () =>
      h(
        Heading,
        {
          level: 2,
          size: "md",
          tone: "neutral",
          weight: "semibold",
        },
        {
          default: () => "Release notes",
        },
      );
  },
});

test("renders byte-identical semantic heading markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<h2/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="heading"/);
  assert.match(html, /data-level="2"/);
  assert.match(html, /data-size="md"/);
  assert.match(html, /data-weight="semibold"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /data-truncate="false"/);
  assert.match(html, /Release notes/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-level=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /style=/);
});

test("renders consumer-owned custom heading semantics without implicit aria", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "HeadingCustomSsrProbe",
      setup() {
        return () =>
          h(
            Heading,
            {
              "aria-level": "3",
              as: "div",
              level: 3,
              role: "heading",
              size: "xl",
              tabindex: "0",
              tone: "muted",
              truncate: true,
              weight: "bold",
            },
            {
              default: () => "Results",
            },
          );
      },
    }),
  );

  assert.match(html, /^<div/);
  assert.match(html, /role="heading"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /aria-level="3"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="heading"/);
  assert.match(html, /data-level="3"/);
  assert.match(html, /data-size="xl"/);
  assert.match(html, /data-weight="bold"/);
  assert.match(html, /data-tone="muted"/);
  assert.match(html, /data-truncate="true"/);
  assert.match(html, /Results/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /style=/);
});
