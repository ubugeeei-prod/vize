import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Portal from "./portal.vue";

const SsrProbe = defineComponent({
  name: "PortalSsrProbe",
  setup() {
    return () => h(Portal, null, { default: () => "Portalled" });
  },
});

test("renders byte-identical in-place portal SSR output", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /data-vize-ui="portal-host"/);
  assert.match(outputs[0], /data-vize-ui="portal"/);
  assert.match(outputs[0], /data-vize-portal-depth="0"/);
  assert.match(outputs[0], /Portalled/);
  assert.doesNotMatch(outputs[0], /<body/i);
});

const NestedSsrProbe = defineComponent({
  name: "NestedPortalSsrProbe",
  setup() {
    return () =>
      h(Portal, null, {
        default: () => h(Portal, null, { default: () => "Nested" }),
      });
  },
});

test("renders deterministic nesting depth without touching the stack", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(NestedSsrProbe)),
    renderToString(createSSRApp(NestedSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /data-vize-portal-depth="0"/);
  assert.match(outputs[0], /data-vize-portal-depth="1"/);
});
