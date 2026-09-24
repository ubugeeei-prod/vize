import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import RangeSlider from "./range-slider.vue";
import RangeSliderRange from "./range-slider-range.vue";
import RangeSliderThumb from "./range-slider-thumb.vue";
import RangeSliderTrack from "./range-slider-track.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const Probe = defineComponent({
  name: "RangeSliderSsrProbe",
  setup: () => () =>
    h(
      RangeSlider,
      {
        ariaLabel: "Price",
        defaultValue: [25, 75],
        dir: "rtl",
        getValueText: (value: number) => `${value} dollars`,
        name: "price",
        orientation: "vertical",
      },
      {
        default: () =>
          h(RangeSliderTrack, null, {
            default: () => [
              h(RangeSliderRange),
              h(RangeSliderThumb, { index: 0, ariaLabel: "Minimum" }),
              h(RangeSliderThumb, { index: 1, ariaLabel: "Maximum" }),
            ],
          }),
      },
    ),
});

test("renders byte-identical multi-thumb markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(Probe);
  try {
    assert.match(html, /^<span role="group" aria-label="Price" part="root"/);
    assert.match(html, /--vize-range-slider-range-start:25%/);
    assert.match(html, /--vize-range-slider-range-end:75%/);
    assert.match(html, /type="hidden" name="price" value="25"/);
    assert.match(html, /type="hidden" name="price" value="75"/);
    assert.match(html, /role="slider" tabindex="0" aria-label="Minimum"/);
    assert.match(html, /aria-valuetext="75 dollars"/);
    assert.match(html, /aria-orientation="vertical"/);
    assert.match(html, /data-dir="rtl"/);
    assert.doesNotMatch(html, /NaN|Infinity/);

    const [low] = host.querySelectorAll<HTMLElement>('[role="slider"]');
    assert.ok(low);
    low.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true }));
    await nextTick();
    assert.equal(low.getAttribute("aria-valuenow"), "26", "hydrated thumbs are interactive");
  } finally {
    dispose();
  }
});
