import assert from "node:assert/strict";

import { h } from "vue";

import Slider from "./slider.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const sliderRuntimeFixture: RuntimeFixture = {
  name: "slider",
  sourceFile: "families/form/slider/slider.vue",
  render: () =>
    h(
      Slider,
      {
        ariaLabel: "Volume",
        ariaValueText: "40 percent",
        defaultValue: 40,
        id: "volume-slider",
        max: 100,
        min: 0,
        name: "volume",
        step: 5,
      },
      {
        default: () => "40%",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<span/);
    assert.match(html, /data-vize-ui="slider"/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-state="in-range"/);
    assert.match(html, /<input/);
    assert.match(html, /type="range"/);
    assert.match(html, /id="volume-slider"/);
    assert.match(html, /name="volume"/);
    assert.match(html, /value="40"/);
    assert.match(html, /aria-label="Volume"/);
    assert.match(html, /aria-valuetext="40 percent"/);
  },
  assertHydratedDom(host) {
    const root = host.querySelector('[data-vize-ui="slider"]');
    const input = host.querySelector('[data-vize-ui="slider-input"]');
    assert.ok(root instanceof HTMLSpanElement);
    assert.ok(input instanceof HTMLInputElement);
    assert.equal(input.type, "range");
    assert.equal(input.id, "volume-slider");
    assert.equal(input.name, "volume");
    assert.equal(input.value, "40");
    assert.equal(input.step, "5");
    assert.equal(input.getAttribute("aria-label"), "Volume");
    assert.equal(root.textContent, "40%");
  },
};
