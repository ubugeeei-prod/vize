import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Stack from "./stack.vue";

const SsrProbe = defineComponent({
  name: "StackSsrProbe",
  setup() {
    return () =>
      h(
        Stack,
        {
          align: "center",
          as: "section",
          axis: "inline",
          dir: "rtl",
          gap: "2rem",
          justify: "space-between",
          reversed: true,
        },
        {
          default: () => [h("span", "Alpha"), h("span", "Beta")],
        },
      );
  },
});

test("renders byte-identical logical stack markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<section/);
  assert.match(html, /dir="rtl"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="stack"/);
  assert.match(html, /data-state="stacked"/);
  assert.match(html, /data-axis="inline"/);
  assert.match(html, /data-reversed="true"/);
  assert.match(html, /data-vize-stack-direction="row-reverse"/);
  assert.match(html, /data-vize-stack-gap="2rem"/);
  assert.match(html, /data-vize-stack-align="center"/);
  assert.match(html, /data-vize-stack-justify="space-between"/);
  assert.match(html, /--vize-ui-stack-gap:2rem/);
  assert.match(html, /--vize-ui-stack-align:center/);
  assert.match(html, /--vize-ui-stack-justify:space-between/);
  assert.match(html, /display:flex/);
  assert.match(html, /flex-direction:row-reverse/);
  assert.match(html, /gap:var\(--vize-ui-stack-gap\)/);
  assert.match(html, /align-items:var\(--vize-ui-stack-align\)/);
  assert.match(html, /justify-content:var\(--vize-ui-stack-justify\)/);
  assert.match(html, /<span>Alpha<\/span><span>Beta<\/span>/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
});
