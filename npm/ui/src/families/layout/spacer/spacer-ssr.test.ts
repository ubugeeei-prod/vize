import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Spacer from "./spacer.vue";

const SsrProbe = defineComponent({
  name: "SpacerSsrProbe",
  setup() {
    return () => h(Spacer, { as: "div", blockSize: "2rem", inlineSize: "100%" });
  },
});

test("renders byte-identical logical spacer markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<div/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="spacer"/);
  assert.match(html, /data-state="sized"/);
  assert.match(html, /data-axis="block"/);
  assert.match(html, /data-vize-spacer-inline-size="100%"/);
  assert.match(html, /data-vize-spacer-block-size="2rem"/);
  assert.match(html, /--vize-ui-spacer-inline-size:100%/);
  assert.match(html, /--vize-ui-spacer-block-size:2rem/);
  assert.match(html, /inline-size:var\(--vize-ui-spacer-inline-size\)/);
  assert.match(html, /block-size:var\(--vize-ui-spacer-block-size\)/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
});

test("renders an SVG spacer without server accessibility semantics", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "SpacerSvgSsrProbe",
      setup() {
        return () => h(Spacer, { as: "svg", axis: "both", size: "24px" });
      },
    }),
  );

  assert.match(html, /^<svg/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /data-axis="both"/);
  assert.match(html, /data-vize-spacer-inline-size="24px"/);
  assert.match(html, /data-vize-spacer-block-size="24px"/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /aria-label=/);
  assert.doesNotMatch(html, /tabindex=/);
});
