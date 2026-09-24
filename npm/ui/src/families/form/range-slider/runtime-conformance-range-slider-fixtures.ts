import assert from "node:assert/strict";

import { h } from "vue";

import RangeSlider from "./range-slider.vue";
import RangeSliderRange from "./range-slider-range.vue";
import RangeSliderThumb from "./range-slider-thumb.vue";
import RangeSliderTrack from "./range-slider-track.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    RangeSlider,
    { ariaLabel: "Price", defaultValue: [20, 80], name: "price" },
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
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /^<span role="group"/);
  assert.match(html, /data-vize-ui="range-slider"/);
  assert.match(html, /data-vize-ui="range-slider-track"/);
  assert.match(html, /data-vize-ui="range-slider-range"/);
  assert.match(html, /role="slider"/);
  assert.match(html, /aria-valuenow="80"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const sliders = host.querySelectorAll('[role="slider"]');
  assert.equal(sliders.length, 2);
  assert.equal(sliders[0]?.getAttribute("aria-label"), "Minimum");
  assert.equal(sliders[1]?.getAttribute("aria-valuemin"), "20");
  assert.equal(host.querySelectorAll('input[type="hidden"]').length, 2);
}

export const rangeSliderRuntimeFixtures: readonly RuntimeFixture[] = [
  "range-slider.vue",
  "range-slider-track.vue",
  "range-slider-range.vue",
  "range-slider-thumb.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/range-slider/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
