import assert from "node:assert/strict";

import { h } from "vue";

import { createPlainTime } from "./plain-time.ts";
import TimeField from "./time-field.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const timeFieldRuntimeFixture: RuntimeFixture = {
  name: "time-field",
  sourceFile: "families/date-time/time-field/time-field.vue",
  render: () =>
    h(TimeField, {
      id: "runtime-time-field",
      locale: "en-US",
      name: "runtime-time",
      ariaLabel: "Time",
      defaultValue: createPlainTime(21, 5),
    }),
  assertServerMarkup(html) {
    assert.match(html, /role="group"/);
    assert.match(html, /data-segment="dayPeriod"/);
    assert.match(html, /value="21:05"/);
  },
  assertHydratedDom(host) {
    const hour = host.querySelector('[data-segment="hour"]');
    assert.equal(hour?.getAttribute("aria-valuenow"), "9");
  },
};
