import assert from "node:assert/strict";
import { h, ref } from "vue";
import { test } from "vitest";

import { updateLastRenderLayouts } from "../layoutMetrics.js";
import { dispatchKey, findNodes, firstChild, mountFresco, renderTui } from "../testing/index.js";
import Slider from "./FrescoSlider.vue";
import { normalizeSliderRange, normalizeSliderValue, sliderValueAtRatio } from "./slider-types.js";

test("slider range normalization clamps malformed and fractional values", () => {
  const range = normalizeSliderRange(0.1, 1, 0.1);

  assert.equal(normalizeSliderValue(Number.NaN, range), 0.1);
  assert.equal(normalizeSliderValue(-1, range), 0.1);
  assert.equal(normalizeSliderValue(0.30000000000000004, range), 0.3);
  assert.equal(normalizeSliderValue(2, range), 1);
  assert.equal(sliderValueAtRatio(0.5, range), 0.6);
  assert.deepEqual(normalizeSliderRange(5, 2, 0), { min: 5, max: 5, step: 1 });
});

test("Slider owns bounded keyboard changes while focused", async () => {
  const value = ref(0);
  const changes: number[] = [];
  const mounted = mountFresco(() =>
    h(Slider, {
      modelValue: value.value,
      min: 0,
      max: 10,
      step: 0.25,
      pageStep: 2.5,
      length: 5,
      focused: true,
      "onUpdate:modelValue": (next: number) => {
        value.value = next;
      },
      onChange: (next: number) => changes.push(next),
    }),
  );

  await dispatchKey({ key: "right" });
  await dispatchKey({ key: "pageup" });
  await dispatchKey({ key: "end" });
  await dispatchKey({ key: "right" });
  await dispatchKey({ key: "home" });

  assert.equal(value.value, 0);
  assert.deepEqual(changes, [0.25, 2.75, 10, 0]);
  const root = firstChild(mounted);
  assert.equal(root.props["aria-role"], "slider");
  assert.deepEqual(root.props["aria-state"], {
    orientation: "horizontal",
    valueMin: 0,
    valueMax: 10,
    valueNow: 0,
  });
  assert.equal(
    findNodes(mounted.root, (node) => node.type === "text").filter(
      (node) => node.props["aria-hidden"] === true,
    ).length,
    5,
  );
  mounted.unmount();
});

test("Slider maps click, drag, and wheel input through measured bounds", async () => {
  const value = ref(0);
  const rendered = renderTui(() =>
    h(Slider, {
      modelValue: value.value,
      min: 0,
      max: 100,
      step: 10,
      length: 5,
      focused: true,
      "onUpdate:modelValue": (next: number) => {
        value.value = next;
      },
    }),
  );
  const root = firstChild(rendered);
  updateLastRenderLayouts([{ id: root.id, x: 10, y: 4, width: 5, height: 1 }]);

  await rendered.input.mouse({ kind: "down", x: 12, y: 4 });
  assert.equal(value.value, 50);
  await rendered.input.mouse({ kind: "drag", x: 14, y: 4 });
  assert.equal(value.value, 100);
  await rendered.input.mouse({ kind: "scroll-down", x: 14, y: 4 });
  assert.equal(value.value, 90);
  await rendered.input.mouse({ kind: "down", x: 2, y: 2 });
  assert.equal(value.value, 90, "events outside the measured track are ignored");

  rendered.unmount();
  updateLastRenderLayouts([]);
});
