import assert from "node:assert/strict";

import { h } from "vue";

import PopoverTrigger from "../../overlays/popover/popover-trigger.vue";
import { createDateRange, createPlainDate } from "../calendar/plain-date.ts";
import DateRangePickerCalendar from "./date-range-picker-calendar.vue";
import DateRangePickerContent from "./date-range-picker-content.vue";
import DateRangePickerField from "./date-range-picker-field.vue";
import DateRangePickerRoot from "./date-range-picker-root.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

function picker(): ReturnType<typeof h> {
  return h(
    DateRangePickerRoot,
    {
      id: "runtime-date-range-picker",
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      defaultOpen: true,
      defaultValue: createDateRange(createPlainDate(2026, 9, 12), createPlainDate(2026, 9, 14)),
    },
    () => [
      h(DateRangePickerField, { boundary: "start", ariaLabel: "Check-in" }),
      h(DateRangePickerField, { boundary: "end", ariaLabel: "Check-out" }, () =>
        h(PopoverTrigger, { ariaLabel: "Choose dates" }, () => "Pick"),
      ),
      h(DateRangePickerContent, { ariaLabel: "Calendar" }, () => h(DateRangePickerCalendar)),
    ],
  );
}

function fixture(name: string, file: string, server: RegExp, selector: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/date-time/date-range-picker/${file}`,
    render: picker,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="date-range-picker"/);
      assert.match(html, server);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector(selector), `${name} must hydrate ${selector}`);
    },
  };
}

export const dateRangePickerRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture(
    "date-range-picker-root",
    "date-range-picker-root.vue",
    /data-start="2026-09-12"/,
    '[data-vize-ui="date-range-picker"]',
  ),
  fixture(
    "date-range-picker-field",
    "date-range-picker-field.vue",
    /id="runtime-date-range-picker-end"/,
    '[data-boundary="end"]',
  ),
  fixture(
    "date-range-picker-content",
    "date-range-picker-content.vue",
    /role="dialog"/,
    '[data-vize-ui="popover-content-host"]',
  ),
  fixture(
    "date-range-picker-calendar",
    "date-range-picker-calendar.vue",
    /data-mode="range"/,
    '[data-mode="range"]',
  ),
];
