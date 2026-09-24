import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import SafeArea from "./safe-area.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical safe-area markup and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "SafeAreaSsrProbe",
      setup: () => () =>
        h(SafeArea, { edges: ["bottom"], apply: "padding" }, { default: () => "Tab bar" }),
    }),
  );
  try {
    assert.match(html, /^<div part="root" data-vize-ui="safe-area" data-edges="bottom"/);
    assert.match(html, /--vize-safe-area-inset-bottom:env\(safe-area-inset-bottom, 0px\)/);
    assert.match(html, /padding-bottom:var\(--vize-safe-area-inset-bottom\)/);
    assert.doesNotMatch(html, /data-insets/);
  } finally {
    dispose();
  }
});
