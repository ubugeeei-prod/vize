import assert from "node:assert/strict";

import { h } from "vue";

import CalendarGrid from "./calendar-grid.vue";
import CalendarHeading from "./calendar-heading.vue";
import CalendarMonthSelect from "./calendar-month-select.vue";
import CalendarNext from "./calendar-next.vue";
import CalendarPrev from "./calendar-prev.vue";
import CalendarRoot from "./calendar-root.vue";
import CalendarYearSelect from "./calendar-year-select.vue";
import { createPlainDate } from "./plain-date.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const today = createPlainDate(2026, 9, 25);

function calendar(children?: () => unknown): ReturnType<typeof h> {
  return h(
    CalendarRoot,
    {
      id: "runtime-calendar",
      today,
      locale: "en-US",
      defaultValue: createPlainDate(2026, 9, 10),
      name: "runtime-date",
    },
    children,
  );
}

function part(
  name: string,
  file: string,
  child: () => unknown,
  server: RegExp,
  selector: string,
): RuntimeFixture {
  return {
    name,
    sourceFile: `families/date-time/calendar/${file}`,
    render: () => calendar(child),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="calendar"/);
      assert.match(html, server);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector(selector), `${name} must hydrate ${selector}`);
    },
  };
}

export const calendarRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "calendar-root",
    sourceFile: "families/date-time/calendar/calendar-root.vue",
    render: () => calendar(),
    assertServerMarkup(html) {
      assert.match(html, /^<div/);
      assert.match(html, /id="runtime-calendar"/);
      assert.match(html, /role="group"/);
      assert.match(html, /role="grid"/);
      assert.match(html, /aria-current="date"/);
      assert.match(html, /name="runtime-date" value="2026-09-10"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="calendar"]');
      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.getAttribute("data-state"), "selected");
      assert.equal(
        host
          .querySelector('[data-date="2026-09-10"][data-vize-ui="calendar-day"]')
          ?.getAttribute("tabindex"),
        "0",
      );
    },
  },
  part(
    "calendar-grid",
    "calendar-grid.vue",
    () => h(CalendarGrid),
    /aria-label="September 2026"/,
    'table[role="grid"]',
  ),
  part(
    "calendar-heading",
    "calendar-heading.vue",
    () => h(CalendarHeading),
    /aria-live="polite"/,
    '[data-vize-ui="calendar-heading"]',
  ),
  part(
    "calendar-prev",
    "calendar-prev.vue",
    () => h(CalendarPrev),
    /aria-label="Previous month"/,
    '[data-vize-ui="calendar-prev"]',
  ),
  part(
    "calendar-next",
    "calendar-next.vue",
    () => h(CalendarNext),
    /aria-label="Next month"/,
    '[data-vize-ui="calendar-next"]',
  ),
  part(
    "calendar-month-select",
    "calendar-month-select.vue",
    () => h(CalendarMonthSelect),
    /<option[^>]*value="9"[^>]*selected/,
    '[data-vize-ui="calendar-month-select"]',
  ),
  part(
    "calendar-year-select",
    "calendar-year-select.vue",
    () => h(CalendarYearSelect),
    /<option[^>]*value="2026"[^>]*selected/,
    '[data-vize-ui="calendar-year-select"]',
  ),
];
