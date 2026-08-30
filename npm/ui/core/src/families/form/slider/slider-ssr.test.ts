import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Slider from "./slider.vue";

const SsrProbe = defineComponent({
  name: "SliderSsrProbe",
  setup() {
    return () =>
      h(
        Slider,
        {
          ariaLabel: "Volume",
          ariaValueText: "40 percent",
          defaultValue: 40,
          dir: "rtl",
          id: "volume-slider",
          max: 100,
          min: 0,
          name: "volume",
          orientation: "vertical",
          required: true,
          step: 5,
        },
        {
          default: () => "40%",
        },
      );
  },
});

test("renders byte-identical native slider markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<span/);
  assert.match(html, /data-vize-ui="slider"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="in-range"/);
  assert.match(html, /data-orientation="vertical"/);
  assert.match(html, /data-dir="rtl"/);
  assert.match(html, /--vize-slider-percent:40%/);
  assert.match(html, /<input/);
  assert.match(html, /type="range"/);
  assert.match(html, /id="volume-slider"/);
  assert.match(html, /name="volume"/);
  assert.match(html, /value="40"/);
  assert.match(html, /min="0"/);
  assert.match(html, /max="100"/);
  assert.match(html, /step="5"/);
  assert.match(html, /required/);
  assert.match(html, /aria-label="Volume"/);
  assert.match(html, /aria-orientation="vertical"/);
  assert.match(html, /aria-valuetext="40 percent"/);
  assert.match(html, /orient="vertical"/);
  assert.match(html, /40%/);
  assert.doesNotMatch(html, /function/);
});

test("renders repaired invalid server markup without unsafe numeric attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "SliderInvalidSsrProbe",
      setup() {
        return () =>
          h(Slider, {
            ariaInvalid: true,
            ariaLabel: "Volume",
            defaultValue: Number.NaN,
            max: 0,
            min: Number.NEGATIVE_INFINITY,
            step: Number.NaN,
          });
      },
    }),
  );

  assert.match(html, /^<span/);
  assert.match(html, /data-state="invalid"/);
  assert.match(html, /data-invalid="true"/);
  assert.match(html, /value="0"/);
  assert.match(html, /min="0"/);
  assert.match(html, /max="1"/);
  assert.match(html, /step="1"/);
  assert.doesNotMatch(html, /NaN|Infinity/);
});
