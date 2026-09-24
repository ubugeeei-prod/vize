import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createPlainDate, formatIsoDate } from "../calendar/plain-date.ts";
import { createPlainDateTime } from "../datetime-field/plain-date-time.ts";
import {
  eventTouchesDay,
  layoutRow,
  layoutTimeGrid,
  minuteAtPoint,
  monthWeeks,
  shiftAnchor,
  snapMinute,
  visibleDays,
} from "./scheduler-layout.ts";
import type { SchedulerEvent } from "./scheduler-layout.ts";

const day = createPlainDate(2026, 9, 25);

function event(
  id: string,
  start: [number, number],
  end: [number, number],
  extra: Partial<SchedulerEvent<null>> = {},
): SchedulerEvent<null> {
  return {
    id,
    title: id,
    data: null,
    start: createPlainDateTime(2026, 9, 25, start[0], start[1]),
    end: createPlainDateTime(2026, 9, 25, end[0], end[1]),
    ...extra,
  };
}

function summary(placements: ReturnType<typeof layoutTimeGrid<null>>): string[] {
  return placements.map(
    (placement) =>
      `${placement.event.id}:${placement.column}/${placement.columns}+${placement.span}`,
  );
}

test("overlapping events split into columns and widen into free space", () => {
  const placements = layoutTimeGrid(
    [
      event("a", [9, 0], [11, 0]),
      event("b", [9, 30], [10, 0]),
      event("c", [10, 0], [10, 30]),
      event("d", [12, 0], [13, 0]),
      event("e", [10, 45], [11, 30]),
    ],
    day,
  );
  assert.deepEqual(summary(placements), ["a:0/2+1", "b:1/2+1", "c:1/2+1", "e:1/2+1", "d:0/1+1"]);
  const d = placements.find((placement) => placement.event.id === "d");
  assert.equal(d?.top, 720 / 1_440);
  assert.equal(d?.height, 60 / 1_440);
});

test("three-way overlaps widen short events into free columns", () => {
  const placements = layoutTimeGrid(
    [
      event("long", [9, 0], [12, 0]),
      event("mid", [9, 0], [10, 0]),
      event("late", [9, 30], [10, 30]),
      event("tail", [10, 0], [11, 0]),
    ],
    day,
  );
  assert.deepEqual(summary(placements), ["long:0/3+1", "mid:1/3+1", "late:2/3+1", "tail:1/3+1"]);
});

test("the day window clips events, enforces a minimum length, and skips all-day events", () => {
  const placements = layoutTimeGrid(
    [
      event("early", [6, 0], [9, 0]),
      event("zero", [10, 0], [10, 0]),
      event("allday", [0, 0], [0, 0], { allDay: true }),
      {
        id: "overnight",
        title: "overnight",
        data: null,
        start: createPlainDateTime(2026, 9, 24, 22, 0),
        end: createPlainDateTime(2026, 9, 25, 8, 30),
      },
    ],
    day,
    { startMinute: 480, endMinute: 1_080, minimumMinutes: 30 },
  );
  const byId = new Map(placements.map((placement) => [placement.event.id, placement]));
  assert.equal(byId.has("allday"), false);
  assert.equal(byId.get("early")?.startMinute, 480);
  assert.equal(byId.get("early")?.continuesBefore, true);
  assert.equal(byId.get("zero")?.endMinute, 630);
  assert.equal(byId.get("overnight")?.continuesBefore, true);
  assert.equal(byId.get("overnight")?.endMinute, 510);
  assert.equal(byId.get("early")?.top, 0);
});

test("row layout stacks multi-day events into lanes with continuation flags", () => {
  const days = visibleDays("week", day, 1);
  assert.deepEqual(
    days.map((date) => formatIsoDate(date)),
    [
      "2026-09-21",
      "2026-09-22",
      "2026-09-23",
      "2026-09-24",
      "2026-09-25",
      "2026-09-26",
      "2026-09-27",
    ],
  );
  const multi = (id: string, from: number, to: number): SchedulerEvent<null> => ({
    id,
    title: id,
    data: null,
    allDay: true,
    start: createPlainDateTime(2026, 9, from, 0, 0),
    end: createPlainDateTime(2026, 9, to, 0, 0),
  });
  const placements = layoutRow(
    [multi("trip", 19, 24), multi("conf", 23, 26), multi("day", 25, 26)],
    days,
  );
  assert.deepEqual(
    placements.map((placement) => [
      placement.event.id,
      placement.startIndex,
      placement.endIndex,
      placement.lane,
      placement.continuesBefore,
    ]),
    [
      ["trip", 0, 2, 0, true],
      ["conf", 2, 4, 1, false],
      ["day", 4, 4, 0, false],
    ],
  );
  assert.equal(eventTouchesDay(multi("x", 25, 26), createPlainDate(2026, 9, 26)), false);
  assert.deepEqual(layoutRow([], []), []);
});

test("views, month rows, anchors, snapping, and pointer mapping are deterministic", () => {
  assert.equal(visibleDays("day", day, 0).length, 1);
  const month = visibleDays("month", day, 0);
  assert.equal(month.length, 35);
  const weeks = monthWeeks(month, day);
  assert.equal(weeks.length, 5);
  assert.equal(weeks[0]?.[0]?.outsideMonth, true);
  assert.equal(formatIsoDate(shiftAnchor("month", createPlainDate(2026, 1, 31), 1)), "2026-02-28");
  assert.equal(formatIsoDate(shiftAnchor("month", createPlainDate(2026, 1, 15), -1)), "2025-12-15");
  assert.equal(formatIsoDate(shiftAnchor("week", day, -1)), "2026-09-18");
  assert.equal(formatIsoDate(shiftAnchor("day", day, 1)), "2026-09-26");
  assert.equal(snapMinute(22, 15, 0, 1_440), 15);
  assert.equal(snapMinute(1_500, 30, 0, 1_410), 1_410);
  assert.equal(
    minuteAtPoint(150, { top: 100, bottom: 580 }, { startMinute: 480, endMinute: 1_440, step: 30 }),
    570,
  );
  assert.equal(
    minuteAtPoint(9_999, { top: 0, bottom: 100 }, { startMinute: 0, endMinute: 1_440, step: 30 }),
    1_410,
  );
  assert.equal(
    minuteAtPoint(5, { top: 0, bottom: 0 }, { startMinute: 60, endMinute: 120, step: 30 }),
    60,
  );
});
