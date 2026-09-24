import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createCalendarFormatters, normalizeWeekday, resolveWeekStart } from "./calendar-locale.ts";
import {
  addDays,
  addMonths,
  addYears,
  clampDate,
  compareDates,
  createDateRange,
  createPlainDate,
  dayOfWeek,
  daysBetween,
  daysInMonth,
  endOfWeek,
  formatIsoDate,
  fromEpochDay,
  fromEpochMilliseconds,
  fromLocalDate,
  fromUtcDate,
  isDateInRange,
  isSameDay,
  isSameRange,
  isValidPlainDate,
  monthsBetween,
  normalizeDateRange,
  normalizePlainDate,
  parseIsoDate,
  shiftYearMonth,
  startOfWeek,
  toEpochDay,
  toLocalDate,
  toUtcDate,
} from "./plain-date.ts";

test("epoch-day arithmetic matches UTC Date math across eras and leap rules", () => {
  for (let epochDay = -800_000; epochDay <= 800_000; epochDay += 997) {
    const date = fromEpochDay(epochDay);
    const reference = new Date(epochDay * 86_400_000);
    assert.equal(date.year, reference.getUTCFullYear());
    assert.equal(date.month, reference.getUTCMonth() + 1);
    assert.equal(date.day, reference.getUTCDate());
    assert.equal(toEpochDay(date), epochDay);
    assert.equal(dayOfWeek(date), reference.getUTCDay());
  }
  assert.equal(toEpochDay(createPlainDate(1970, 1, 1)), 0);
  assert.equal(daysInMonth(2024, 2), 29);
  assert.equal(daysInMonth(1900, 2), 28);
  assert.equal(daysInMonth(2000, 2), 29);
  assert.equal(daysInMonth(2026, 13), 0);
  assert.ok(Object.isFrozen(fromEpochDay(10)));
});

test("validation rejects impossible dates and copies Temporal-like records", () => {
  assert.equal(isValidPlainDate({ year: 2026, month: 2, day: 29 }), false);
  assert.equal(isValidPlainDate({ year: 2026.5, month: 1, day: 1 }), false);
  assert.equal(isValidPlainDate(null), false);
  assert.throws(() => createPlainDate(2026, 4, 31), /VIZE_UI_PLAIN_DATE_INVALID/);
  const temporalLike = Object.freeze(
    Object.create(
      {},
      {
        year: { get: () => 2026, enumerable: false },
        month: { get: () => 9, enumerable: false },
        day: { get: () => 25, enumerable: false },
        calendarId: { value: "iso8601" },
      },
    ) as { readonly year: number; readonly month: number; readonly day: number },
  );
  const copied = normalizePlainDate(temporalLike);
  assert.deepEqual(copied, { year: 2026, month: 9, day: 25 });
  assert.ok(Object.isFrozen(copied));
  assert.equal(normalizePlainDate(undefined), null);
});

test("month, year, and week arithmetic clamps to real calendar days", () => {
  assert.deepEqual(addMonths(createPlainDate(2026, 1, 31), 1), { year: 2026, month: 2, day: 28 });
  assert.deepEqual(addMonths(createPlainDate(2026, 3, 31), -13), { year: 2025, month: 2, day: 28 });
  assert.deepEqual(addYears(createPlainDate(2024, 2, 29), 1), { year: 2025, month: 2, day: 28 });
  assert.deepEqual(addDays(createPlainDate(2026, 12, 31), 1), { year: 2027, month: 1, day: 1 });
  assert.deepEqual(shiftYearMonth({ year: 2026, month: 1 }, -1), { year: 2025, month: 12 });
  assert.equal(monthsBetween({ year: 2025, month: 11 }, { year: 2026, month: 2 }), 3);
  assert.equal(daysBetween(createPlainDate(2026, 3, 1), createPlainDate(2026, 2, 1)), -28);
  assert.deepEqual(startOfWeek(createPlainDate(2026, 9, 25), 1), { year: 2026, month: 9, day: 21 });
  assert.deepEqual(endOfWeek(createPlainDate(2026, 9, 25), 0), { year: 2026, month: 9, day: 26 });
  assert.equal(compareDates(createPlainDate(2026, 1, 2), createPlainDate(2026, 1, 1)), 1);
  const min = createPlainDate(2026, 1, 10);
  const max = createPlainDate(2026, 1, 20);
  assert.equal(clampDate(createPlainDate(2026, 1, 1), min, max), min);
  assert.equal(clampDate(createPlainDate(2026, 2, 1), min, max), max);
  assert.equal(isSameDay(null, undefined), true);
  assert.equal(isSameDay(min, null), false);
});

test("ISO strings round-trip including extended years", () => {
  assert.equal(formatIsoDate(createPlainDate(2026, 9, 5)), "2026-09-05");
  assert.equal(formatIsoDate(createPlainDate(12, 1, 1)), "0012-01-01");
  assert.equal(formatIsoDate(createPlainDate(-5, 3, 1)), "-000005-03-01");
  assert.equal(formatIsoDate(createPlainDate(10_000, 1, 1)), "+010000-01-01");
  assert.deepEqual(parseIsoDate(" 2024-02-29 "), { year: 2024, month: 2, day: 29 });
  assert.deepEqual(parseIsoDate("-000005-03-01"), { year: -5, month: 3, day: 1 });
  assert.equal(parseIsoDate("2023-02-29"), null);
  assert.equal(parseIsoDate("-000000-01-01"), null);
  assert.equal(parseIsoDate("2026-9-5"), null);
});

test("Date adapters read local or UTC fields and never shift days", () => {
  const local = toLocalDate(createPlainDate(2026, 3, 29));
  assert.equal(local.getFullYear(), 2026);
  assert.equal(local.getMonth(), 2);
  assert.equal(local.getDate(), 29);
  assert.deepEqual(fromLocalDate(local), { year: 2026, month: 3, day: 29 });
  assert.equal(toLocalDate(createPlainDate(50, 1, 1)).getFullYear(), 50);
  assert.equal(toUtcDate(createPlainDate(2026, 9, 25)).toISOString(), "2026-09-25T00:00:00.000Z");
  assert.deepEqual(fromUtcDate(new Date("2026-09-25T23:59:59Z")), {
    year: 2026,
    month: 9,
    day: 25,
  });
  assert.equal(fromUtcDate(new Date(Number.NaN)), null);
  assert.equal(fromLocalDate(new Date(Number.NaN)), null);
});

test("instants resolve to calendar dates per IANA time zone", () => {
  const instant = Date.UTC(2026, 8, 24, 16, 30);
  assert.deepEqual(fromEpochMilliseconds(instant, "UTC"), { year: 2026, month: 9, day: 24 });
  assert.deepEqual(fromEpochMilliseconds(instant, "Asia/Tokyo"), { year: 2026, month: 9, day: 25 });
  assert.deepEqual(fromEpochMilliseconds(Date.UTC(2026, 2, 8, 7, 30), "America/Los_Angeles"), {
    year: 2026,
    month: 3,
    day: 7,
  });
  assert.deepEqual(fromEpochMilliseconds(Date.UTC(-5, 0, 1, 12), "UTC")?.year, -5);
  assert.equal(fromEpochMilliseconds(instant, "Not/AZone"), null);
  assert.equal(fromEpochMilliseconds(Number.NaN, "UTC"), null);
});

test("ranges are ordered, normalized, and compared by day", () => {
  const range = createDateRange(createPlainDate(2026, 9, 30), createPlainDate(2026, 9, 1));
  assert.deepEqual(range, {
    start: { year: 2026, month: 9, day: 1 },
    end: { year: 2026, month: 9, day: 30 },
  });
  assert.equal(isDateInRange(createPlainDate(2026, 9, 15), range), true);
  assert.equal(isDateInRange(createPlainDate(2026, 10, 1), range), false);
  assert.equal(isDateInRange(createPlainDate(2026, 10, 1), null), false);
  assert.deepEqual(normalizeDateRange({ start: range.end, end: range.start }), range);
  assert.equal(
    normalizeDateRange({ start: range.start, end: { year: 2026, month: 2, day: 30 } }),
    null,
  );
  assert.equal(isSameRange(range, normalizeDateRange(range)), true);
  assert.equal(isSameRange(null, undefined), true);
});

test("week start follows CLDR regions, the fw extension, and explicit overrides", () => {
  assert.equal(resolveWeekStart("en-US"), 0);
  assert.equal(resolveWeekStart("en-GB"), 1);
  assert.equal(resolveWeekStart("ja"), 0);
  assert.equal(resolveWeekStart("fa-IR"), 6);
  assert.equal(resolveWeekStart("dv-MV"), 5);
  assert.equal(resolveWeekStart("en-US-u-fw-mon"), 1);
  assert.equal(resolveWeekStart("not a locale"), 1);
  assert.equal(normalizeWeekday(4, "en-US"), 4);
  assert.equal(normalizeWeekday(9, "en-US"), 0);
});

test("formatters render UTC-anchored labels with calendar and numbering overrides", () => {
  const english = createCalendarFormatters({ locale: "en-US" });
  const date = createPlainDate(2026, 9, 25);
  assert.equal(english.day(date), "25");
  assert.equal(english.fullDate(date), "Friday, September 25, 2026");
  assert.equal(english.monthName(2), "February");
  assert.equal(english.year(2026), "2026");
  assert.deepEqual(
    english.weekdays(1, "narrow").map((weekday) => weekday.label),
    ["M", "T", "W", "T", "F", "S", "S"],
  );
  const arabic = createCalendarFormatters({ locale: "ar-EG", numberingSystem: "arab" });
  assert.equal(arabic.day(date), "٢٥");
  const buddhist = createCalendarFormatters({
    locale: "th-TH",
    calendar: "buddhist",
    numberingSystem: "latn",
  });
  assert.match(buddhist.year(2026), /2569/u);
  const fallback = createCalendarFormatters({ locale: "en-US", calendar: "not-a-calendar!" });
  assert.equal(fallback.day(date), "25");
});
