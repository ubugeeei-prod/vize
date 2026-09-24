import assert from "node:assert/strict";

import { h } from "vue";

import PopoverTrigger from "../../overlays/popover/popover-trigger.vue";
import { createPlainDate } from "../calendar/plain-date.ts";
import DatePickerCalendar from "./date-picker-calendar.vue";
import DatePickerContent from "./date-picker-content.vue";
import DatePickerField from "./date-picker-field.vue";
import DatePickerRoot from "./date-picker-root.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

function picker(): ReturnType<typeof h> {
  return h(
    DatePickerRoot,
    {
      id: "runtime-date-picker",
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      defaultOpen: true,
      defaultValue: createPlainDate(2026, 9, 12),
    },
    () => [
      h(DatePickerField, { ariaLabel: "Departure" }, () =>
        h(PopoverTrigger, { ariaLabel: "Choose date" }, () => "Pick"),
      ),
      h(DatePickerContent, { ariaLabel: "Calendar" }, () => h(DatePickerCalendar)),
    ],
  );
}

function fixture(name: string, file: string, server: RegExp, selector: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/date-time/date-picker/${file}`,
    render: picker,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="date-picker"/);
      assert.match(html, server);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector(selector), `${name} must hydrate ${selector}`);
    },
  };
}

export const datePickerRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture(
    "date-picker-root",
    "date-picker-root.vue",
    /data-state="open"/,
    '[data-vize-ui="date-picker"]',
  ),
  fixture(
    "date-picker-field",
    "date-picker-field.vue",
    /id="runtime-date-picker-field"/,
    '[data-vize-ui="date-field"]',
  ),
  fixture(
    "date-picker-content",
    "date-picker-content.vue",
    /role="dialog"/,
    '[data-vize-ui="popover-content-host"]',
  ),
  fixture(
    "date-picker-calendar",
    "date-picker-calendar.vue",
    /data-picker-part="calendar"/,
    '[data-vize-ui="date-picker"]',
  ),
];
