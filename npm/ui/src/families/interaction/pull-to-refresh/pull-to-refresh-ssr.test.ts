import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import PullToRefresh from "./pull-to-refresh.vue";
import PullToRefreshTrigger from "./pull-to-refresh-trigger.vue";
import type { PullToRefreshSlotState } from "./pull-to-refresh-types.ts";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical idle markup and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "PullToRefreshSsrProbe",
      setup: () => () =>
        h(PullToRefresh, null, {
          default: (state: PullToRefreshSlotState) => [
            h("span", state.state),
            h(PullToRefreshTrigger, null, { default: () => "Refresh" }),
          ],
        }),
    }),
  );
  try {
    assert.match(html, /id="vize-v-\d+-pull-to-refresh"/);
    assert.match(html, /data-state="idle"/);
    assert.match(html, /--vize-pull-distance:0px/);
    assert.match(html, /aria-controls="vize-v-\d+-pull-to-refresh"/);
    assert.doesNotMatch(html, /aria-busy|data-reduced-motion/);
  } finally {
    dispose();
  }
});
