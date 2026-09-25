import assert from "node:assert/strict";

import { h } from "vue";

import { createPlainDate } from "../calendar/plain-date.ts";
import { weekOf } from "./week-picker-selection.ts";
import WeekPickerRoot from "./week-picker-root.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const weekPickerRuntimeFixture: RuntimeFixture = {
  name: "week-picker",
  sourceFile: "families/date-time/week-picker/week-picker-root.vue",
  render: () =>
    h(WeekPickerRoot, {
      id: "runtime-week-picker",
      today: createPlainDate(2026, 9, 25),
      locale: "en-GB",
      defaultValue: weekOf(createPlainDate(2026, 9, 25), 1),
    }),
  assertServerMarkup(html) {
    assert.match(html, /data-mode="week"/);
    assert.match(html, /scope="row"/);
  },
  assertHydratedDom(host) {
    assert.equal(
      host.querySelector('[data-vize-ui="week-picker"]')?.getAttribute("data-week"),
      "2026-W39",
    );
  },
};
