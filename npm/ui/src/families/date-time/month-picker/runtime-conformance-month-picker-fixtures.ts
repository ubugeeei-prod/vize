import assert from "node:assert/strict";

import { h } from "vue";

import { createPlainDate } from "../calendar/plain-date.ts";
import MonthPicker from "./month-picker.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const monthPickerRuntimeFixture: RuntimeFixture = {
  name: "month-picker",
  sourceFile: "families/date-time/month-picker/month-picker.vue",
  render: () =>
    h(MonthPicker, {
      id: "runtime-month-picker",
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      defaultValue: { year: 2026, month: 4 },
    }),
  assertServerMarkup(html) {
    assert.match(html, /role="grid"/);
    assert.match(html, /aria-current="date"/);
  },
  assertHydratedDom(host) {
    assert.equal(host.querySelector('[data-month="4"]')?.getAttribute("tabindex"), "0");
  },
};
