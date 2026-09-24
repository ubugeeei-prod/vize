import assert from "node:assert/strict";

import { h } from "vue";

import { createDateRange, createPlainDate } from "../calendar/plain-date.ts";
import RangeCalendarRoot from "./range-calendar-root.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const rangeCalendarRuntimeFixture: RuntimeFixture = {
  name: "range-calendar-root",
  sourceFile: "families/date-time/range-calendar/range-calendar-root.vue",
  render: () =>
    h(RangeCalendarRoot, {
      id: "runtime-range",
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      defaultValue: createDateRange(createPlainDate(2026, 9, 8), createPlainDate(2026, 9, 11)),
    }),
  assertServerMarkup(html) {
    assert.match(html, /data-mode="range"/);
    assert.match(html, /aria-multiselectable="true"/);
    assert.match(html, /data-state="range-middle"/);
  },
  assertHydratedDom(host) {
    const root = host.querySelector('[data-vize-ui="calendar"]');
    assert.equal(root?.getAttribute("data-start"), "2026-09-08");
    assert.equal(root?.getAttribute("data-end"), "2026-09-11");
  },
};
