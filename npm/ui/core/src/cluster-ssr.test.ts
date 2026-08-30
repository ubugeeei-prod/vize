import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Cluster from "./cluster.vue";

const SsrProbe = defineComponent({
  name: "ClusterSsrProbe",
  setup() {
    return () =>
      h(
        Cluster,
        {
          align: "center",
          as: "section",
          gap: 12,
          justify: "space-between",
          reversed: true,
          wrap: false,
        },
        {
          default: () => [h("button", { type: "button" }, "Alpha"), h("a", { href: "/" }, "Beta")],
        },
      );
  },
});

test("renders byte-identical cluster markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<section/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="cluster"/);
  assert.match(html, /data-state="clustered"/);
  assert.match(html, /data-wrap="false"/);
  assert.match(html, /data-reversed="true"/);
  assert.match(html, /data-align="center"/);
  assert.match(html, /data-justify="space-between"/);
  assert.match(html, /data-vize-cluster-direction="row-reverse"/);
  assert.match(html, /data-vize-cluster-gap="12px"/);
  assert.match(html, /--vize-ui-cluster-gap:12px/);
  assert.match(html, /--vize-ui-cluster-align:center/);
  assert.match(html, /--vize-ui-cluster-justify:space-between/);
  assert.match(html, /display:flex/);
  assert.match(html, /flex-direction:row-reverse/);
  assert.match(html, /flex-wrap:nowrap/);
  assert.match(html, /gap:var\(--vize-ui-cluster-gap\)/);
  assert.match(html, /align-items:var\(--vize-ui-cluster-align\)/);
  assert.match(html, /justify-content:var\(--vize-ui-cluster-justify\)/);
  assert.match(html, /<button type="button">Alpha<\/button><a href="\/">Beta<\/a>/);
  assert.doesNotMatch(html, /aria-hidden=/);
  assert.doesNotMatch(html, /role=/);
  assert.doesNotMatch(html, /tabindex=/);
});
