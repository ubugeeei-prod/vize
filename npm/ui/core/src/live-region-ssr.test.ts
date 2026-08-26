import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import LiveRegion from "./live-region.vue";

const SsrProbe = defineComponent({
  name: "LiveRegionSsrProbe",
  setup() {
    return () => h(LiveRegion);
  },
});

test("renders byte-identical empty polite live regions", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /aria-live="polite"/);
  assert.match(outputs[0], /role="status"/);
  assert.match(outputs[0], /data-vize-ui="live-region"/);
});
