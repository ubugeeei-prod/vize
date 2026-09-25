import assert from "node:assert/strict";

import { h } from "vue";

import { createPlainDate } from "../calendar/plain-date.ts";
import YearPicker from "./year-picker.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const yearPickerRuntimeFixture: RuntimeFixture = {
  name: "year-picker",
  sourceFile: "families/date-time/year-picker/year-picker.vue",
  render: () =>
    h(YearPicker, {
      id: "runtime-year-picker",
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      defaultValue: 2020,
    }),
  assertServerMarkup(html) {
    assert.match(html, /role="grid"/);
    assert.match(html, /2016 – 2027/);
  },
  assertHydratedDom(host) {
    assert.equal(host.querySelector('[data-year="2020"]')?.getAttribute("tabindex"), "0");
  },
};
