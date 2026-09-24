import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  ColorPickerArea,
  ColorPickerChannelSlider,
  ColorPickerEyeDropper,
  ColorPickerField,
  ColorPickerRoot,
  ColorPickerSwatch,
  ColorPickerSwatchGroup,
} from "./color-picker.ts";

const familyRoot = "families/form/color-picker/";

function renderInRoot(children: () => unknown) {
  return () => h(ColorPickerRoot, { defaultValue: "#3366cc", id: "accent" }, children);
}

export const colorPickerRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "color-picker",
    sourceFile: `${familyRoot}color-picker-root.vue`,
    render: () =>
      h(ColorPickerRoot, { defaultValue: "#3366cc", id: "accent", name: "accent" }, () => "Accent"),
    assertServerMarkup(html) {
      assert.match(html, /^<div id="accent" role="group"/);
      assert.match(html, /data-vize-ui="color-picker-root"/);
      assert.match(html, /data-value="#3366cc"/);
      assert.match(html, /type="hidden" name="accent" value="#3366cc"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="color-picker-root"]');
      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.style.getPropertyValue("--vize-ui-color-picker-color"), "rgb(51 102 204)");
      assert.equal(host.querySelector<HTMLInputElement>("input[type='hidden']")?.value, "#3366cc");
    },
  },
  {
    name: "color-picker-area",
    sourceFile: `${familyRoot}color-picker-area.vue`,
    render: renderInRoot(() => h(ColorPickerArea)),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="color-picker-area"/);
      assert.match(html, /id="accent-area-thumb"[^>]*role="slider"/);
      assert.match(html, /aria-valuetext="Saturation 75%, Brightness 80%"/);
    },
    assertHydratedDom(host) {
      const thumb = host.querySelector('[data-vize-ui="color-picker-area-thumb"]');
      assert.ok(thumb instanceof HTMLDivElement);
      assert.equal(thumb.tabIndex, 0);
      assert.equal(thumb.getAttribute("aria-valuenow"), "75");
    },
  },
  {
    name: "color-picker-channel-slider",
    sourceFile: `${familyRoot}color-picker-channel-slider.vue`,
    render: renderInRoot(() => h(ColorPickerChannelSlider, { channel: "hue" })),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="color-picker-channel-slider"/);
      assert.match(html, /id="accent-channel-hue-thumb"[^>]*role="slider"/);
      assert.match(html, /aria-valuenow="220"/);
      assert.match(html, /aria-valuetext="Hue 220°"/);
    },
    assertHydratedDom(host) {
      const thumb = host.querySelector('[data-vize-ui="color-picker-channel-thumb"]');
      assert.ok(thumb instanceof HTMLDivElement);
      assert.equal(thumb.getAttribute("aria-valuemax"), "360");
    },
  },
  {
    name: "color-picker-eye-dropper",
    sourceFile: `${familyRoot}color-picker-eye-dropper.vue`,
    render: renderInRoot(() => h(ColorPickerEyeDropper, { ariaLabel: "Pick" })),
    assertServerMarkup(html) {
      assert.match(html, /id="accent-eye-dropper" type="button" disabled/);
      assert.match(html, /data-state="unsupported"/);
    },
    assertHydratedDom(host) {
      const button = host.querySelector('[data-vize-ui="color-picker-eye-dropper"]');
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.type, "button");
    },
  },
  {
    name: "color-picker-field",
    sourceFile: `${familyRoot}color-picker-field.vue`,
    render: renderInRoot(() => h(ColorPickerField, { ariaLabel: "Hex" })),
    assertServerMarkup(html) {
      assert.match(html, /id="accent-field" type="text"/);
      assert.match(html, /value="#3366cc"/);
      assert.match(html, /aria-label="Hex"/);
    },
    assertHydratedDom(host) {
      const input = host.querySelector('[data-vize-ui="color-picker-field"]');
      assert.ok(input instanceof HTMLInputElement);
      assert.equal(input.value, "#3366cc");
    },
  },
  {
    name: "color-picker-swatch",
    sourceFile: `${familyRoot}color-picker-swatch.vue`,
    render: renderInRoot(() =>
      h(ColorPickerSwatchGroup, { ariaLabel: "Presets" }, () => [
        h(ColorPickerSwatch, { value: "#3366cc", label: "Brand" }),
        h(ColorPickerSwatch, { value: "#ff0000", label: "Red" }),
      ]),
    ),
    assertServerMarkup(html) {
      assert.match(
        html,
        /<div(?=[^>]*role="radio")(?=[^>]*aria-checked="true")(?=[^>]*aria-label="Brand")/,
      );
      assert.match(
        html,
        /<div(?=[^>]*role="radio")(?=[^>]*aria-checked="false")(?=[^>]*aria-label="Red")/,
      );
    },
    assertHydratedDom(host) {
      const swatches = host.querySelectorAll('[data-vize-ui="color-picker-swatch"]');
      assert.equal(swatches.length, 2);
      assert.equal(swatches[0]?.getAttribute("data-state"), "checked");
    },
  },
  {
    name: "color-picker-swatch-group",
    sourceFile: `${familyRoot}color-picker-swatch-group.vue`,
    render: renderInRoot(() =>
      h(ColorPickerSwatchGroup, { ariaLabel: "Presets" }, () =>
        h(ColorPickerSwatch, { value: "#00ff00", label: "Green" }),
      ),
    ),
    assertServerMarkup(html) {
      assert.match(html, /id="accent-swatches" role="radiogroup" aria-label="Presets"/);
    },
    assertHydratedDom(host) {
      const group = host.querySelector('[data-vize-ui="color-picker-swatch-group"]');
      assert.ok(group instanceof HTMLDivElement);
      assert.equal(group.getAttribute("role"), "radiogroup");
    },
  },
];
