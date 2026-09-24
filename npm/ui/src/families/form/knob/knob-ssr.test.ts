import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import Knob from "./knob.vue";
import AnglePicker from "../angle-picker/angle-picker.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical rotary markup and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "RotarySsrProbe",
      setup: () => () =>
        h("div", [
          h(Knob, { ariaLabel: "Gain", defaultValue: 25, name: "gain" }),
          h(AnglePicker, { ariaLabel: "Hue", defaultValue: 120, name: "hue" }),
        ]),
    }),
  );
  try {
    assert.match(html, /id="vize-v-\d+-knob" role="slider" tabindex="0" aria-valuenow="25"/);
    assert.match(html, /--vize-knob-angle:-67\.5deg/);
    assert.match(html, /id="vize-v-\d+-angle" role="slider"[^>]*aria-valuetext="120 degrees"/);
    assert.match(html, /--vize-angle-picker-angle:120deg/);
    assert.doesNotMatch(html, /NaN|Infinity/);
  } finally {
    dispose();
  }
});
