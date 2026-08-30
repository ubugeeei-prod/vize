import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Surface from "./surface.vue";

const SsrProbe = defineComponent({
  name: "SurfaceSsrProbe",
  setup() {
    return () =>
      h(
        Surface,
        {
          ariaDescribedby: "release-summary-help",
          ariaLabelledby: "release-summary-title",
          as: "article",
          elevation: "raised",
          tone: "neutral",
        },
        {
          default: () => [h("h2", { id: "release-summary-title" }, "Release"), h("p", "Ready")],
        },
      );
  },
});

test("renders byte-identical labelled surface markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<article/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="surface"/);
  assert.match(html, /aria-labelledby="release-summary-title"/);
  assert.match(html, /aria-describedby="release-summary-help"/);
  assert.match(html, /data-tone="neutral"/);
  assert.match(html, /data-elevation="raised"/);
  assert.match(html, /<h2 id="release-summary-title">Release<\/h2><p>Ready<\/p>/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /aria-live=/);
});

test("omits optional ARIA and data hooks from default SSR markup", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "SurfaceDefaultSsrProbe",
      setup() {
        return () => h(Surface, null, { default: () => "Dashboard" });
      },
    }),
  );

  assert.match(html, /^<section/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="surface"/);
  assert.match(html, /Dashboard/);
  assert.doesNotMatch(html, /aria-labelledby=/);
  assert.doesNotMatch(html, /aria-describedby=/);
  assert.doesNotMatch(html, /data-tone=/);
  assert.doesNotMatch(html, /data-elevation=/);
  assert.doesNotMatch(html, /class=|style=|role=|tabindex=|aria-hidden=|aria-live=/);
});
