import assert from "node:assert/strict";

import { h } from "vue";

import { createPlainTime } from "../time-field/plain-time.ts";
import TimePicker from "./time-picker.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const timePickerRuntimeFixture: RuntimeFixture = {
  name: "time-picker",
  sourceFile: "families/date-time/time-picker/time-picker.vue",
  render: () =>
    h(TimePicker, {
      id: "runtime-time-picker",
      locale: "en-US",
      ariaLabel: "Time",
      step: 120,
      defaultValue: createPlainTime(8, 0),
    }),
  assertServerMarkup(html) {
    assert.match(html, /role="listbox"/);
    assert.match(html, /role="option"/);
  },
  assertHydratedDom(host) {
    assert.equal(host.querySelectorAll('[role="option"]').length, 12);
  },
};
