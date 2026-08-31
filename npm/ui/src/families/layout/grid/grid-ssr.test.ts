import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Grid from "./grid.vue";

const SsrProbe = defineComponent({
  name: "GridSsrProbe",
  setup() {
    return () =>
      h(
        Grid,
        {
          align: "center",
          as: "section",
          autoFlow: "column dense",
          columnGap: "2rem",
          columns: 3,
          gap: 12,
          justify: "end",
          rowGap: 8,
        },
        {
          default: () => [h("article", "Alpha"), h("article", "Beta")],
        },
      );
  },
});

test("renders byte-identical grid markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<section/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="grid"/);
  assert.match(html, /data-columns="repeat\(3, minmax\(0, 1fr\)\)"/);
  assert.match(html, /data-auto-flow="column dense"/);
  assert.match(html, /data-align="center"/);
  assert.match(html, /data-justify="end"/);
  assert.match(html, /--vize-ui-grid-columns:repeat\(3, minmax\(0, 1fr\)\)/);
  assert.match(html, /--vize-ui-grid-gap:12px/);
  assert.match(html, /--vize-ui-grid-row-gap:8px/);
  assert.match(html, /--vize-ui-grid-column-gap:2rem/);
  assert.match(html, /--vize-ui-grid-align:center/);
  assert.match(html, /--vize-ui-grid-justify:end/);
  assert.match(html, /--vize-ui-grid-auto-flow:column dense/);
  assert.match(html, /display:grid/);
  assert.match(html, /grid-template-columns:var\(--vize-ui-grid-columns\)/);
  assert.match(html, /grid-auto-flow:var\(--vize-ui-grid-auto-flow\)/);
  assert.match(html, /gap:var\(--vize-ui-grid-gap\)/);
  assert.match(html, /row-gap:var\(--vize-ui-grid-row-gap\)/);
  assert.match(html, /column-gap:var\(--vize-ui-grid-column-gap\)/);
  assert.match(html, /align-items:var\(--vize-ui-grid-align\)/);
  assert.match(html, /justify-items:var\(--vize-ui-grid-justify\)/);
  assert.match(html, /<article>Alpha<\/article><article>Beta<\/article>/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
});
