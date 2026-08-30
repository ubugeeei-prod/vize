import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Container from "./container.vue";

const SsrProbe = defineComponent({
  name: "ContainerSsrProbe",
  setup() {
    return () =>
      h(
        Container,
        {
          as: "main",
          maxInlineSize: 960,
          paddingInline: 24,
          size: "lg",
        },
        {
          default: () => [h("h1", "Dashboard"), h("p", "Stable shell")],
        },
      );
  },
});

test("renders byte-identical container markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<main/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="container"/);
  assert.match(html, /data-size="lg"/);
  assert.match(html, /data-centered="true"/);
  assert.match(html, /--vize-ui-container-max-inline-size:960px/);
  assert.match(html, /--vize-ui-container-padding-inline:24px/);
  assert.match(html, /max-inline-size:var\(--vize-ui-container-max-inline-size\)/);
  assert.match(html, /padding-inline:var\(--vize-ui-container-padding-inline\)/);
  assert.match(html, /margin-inline:auto/);
  assert.match(html, /<h1>Dashboard<\/h1><p>Stable shell<\/p>/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
});
