import assert from "node:assert/strict";

import { h } from "vue";

import Knob from "./knob.vue";
import AnglePicker from "../angle-picker/angle-picker.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const rotaryRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "knob",
    sourceFile: "families/form/knob/knob.vue",
    render: () => h(Knob, { id: "gain", ariaLabel: "Gain", defaultValue: 50 }),
    assertServerMarkup(html) {
      assert.match(html, /^<span id="gain" role="slider"/);
      assert.match(html, /aria-valuenow="50"/);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelector("#gain")?.getAttribute("aria-valuemax"), "100");
    },
  },
  {
    name: "angle-picker",
    sourceFile: "families/form/angle-picker/angle-picker.vue",
    render: () => h(AnglePicker, { id: "hue", ariaLabel: "Hue", defaultValue: 90 }),
    assertServerMarkup(html) {
      assert.match(html, /^<span id="hue" role="slider"/);
      assert.match(html, /aria-valuetext="90 degrees"/);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelector("#hue")?.getAttribute("aria-valuenow"), "90");
    },
  },
];
