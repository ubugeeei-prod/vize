import assert from "node:assert/strict";

import { h } from "vue";

import { createPlainDate } from "../calendar/plain-date.ts";
import { createPlainDateTime } from "../datetime-field/plain-date-time.ts";
import SchedulerRoot from "./scheduler-root.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const events = [
  {
    id: "review",
    title: "Review",
    data: null,
    start: createPlainDateTime(2026, 9, 25, 9, 0),
    end: createPlainDateTime(2026, 9, 25, 10, 0),
  },
];

function scheduler(view: "week" | "month", children?: () => unknown): ReturnType<typeof h> {
  return h(
    SchedulerRoot,
    {
      id: `runtime-scheduler-${view}`,
      events,
      view,
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      dayStartHour: 8,
      dayEndHour: 11,
    },
    children,
  );
}

function fixture(
  name: string,
  file: string,
  view: "week" | "month",
  server: RegExp,
  selector: string,
): RuntimeFixture {
  return {
    name,
    sourceFile: `families/date-time/scheduler/${file}`,
    render: () => scheduler(view),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="scheduler"/);
      assert.match(html, server);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector(selector), `${name} must hydrate ${selector}`);
    },
  };
}

export const schedulerRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture(
    "scheduler-root",
    "scheduler-root.vue",
    "week",
    /role="region"/,
    '[data-vize-ui="scheduler"]',
  ),
  fixture(
    "scheduler-heading",
    "scheduler-heading.vue",
    "week",
    /aria-live="polite"/,
    '[data-vize-ui="scheduler-heading"]',
  ),
  fixture(
    "scheduler-nav",
    "scheduler-nav.vue",
    "week",
    /data-action="today"/,
    '[data-vize-ui="scheduler-nav"]',
  ),
  fixture(
    "scheduler-time-grid",
    "scheduler-time-grid.vue",
    "week",
    /data-vize-ui="scheduler-slot"/,
    '[data-vize-ui="scheduler-time-grid"]',
  ),
  fixture(
    "scheduler-day-column",
    "scheduler-day-column.vue",
    "week",
    /data-vize-ui="scheduler-day-column"/,
    '[data-vize-ui="scheduler-day-column"]',
  ),
  fixture(
    "scheduler-event",
    "scheduler-event.vue",
    "week",
    /data-event-id="review"/,
    '[data-vize-ui="scheduler-event"]',
  ),
  fixture(
    "scheduler-month-grid",
    "scheduler-month-grid.vue",
    "month",
    /data-vize-ui="scheduler-month-grid"/,
    '[data-vize-ui="scheduler-month-grid"]',
  ),
  fixture(
    "scheduler-month-day",
    "scheduler-month-day.vue",
    "month",
    /data-vize-ui="scheduler-month-cell"/,
    '[data-vize-ui="scheduler-day"]',
  ),
];
