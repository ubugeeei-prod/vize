import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { formatDate, useDateFormat } from "./use-date-format.ts";

const instant = Date.UTC(2026, 0, 4, 15, 5, 9, 42);

void test("formats numeric tokens in UTC by default", () => {
  assert.equal(formatDate(0), "00:00:00");
  assert.equal(
    formatDate(instant, "YYYY-MM-DD HH:mm:ss.SSS Z ZZ"),
    "2026-01-04 15:05:09.042 +00:00 +0000",
  );
  assert.equal(formatDate(instant, "YY M D H m s h hh"), "26 1 4 15 5 9 3 03");
});

void test("renders calendar fields in the requested time zone", () => {
  assert.equal(
    formatDate(instant, "dddd, MMMM D [at] h:mm A (Z) d", { timeZone: "Asia/Tokyo" }),
    "Monday, January 5 at 12:05 AM (+09:00) 1",
  );
  assert.equal(formatDate(Date.UTC(2026, 6, 1), "Z", { timeZone: "America/St_Johns" }), "-02:30");
});

void test("localizes names and meridiem through Intl", () => {
  assert.equal(
    formatDate(instant, "YYYY/M/D dddd MMM A", { locale: "ja", timeZone: "America/New_York" }),
    "2026/1/4 日曜日 1月 午前",
  );
  assert.equal(formatDate(instant, "a dd ddd", { locale: ["en-US"] }), "pm S Sun");
});

void test("keeps bracketed text literal", () => {
  assert.equal(formatDate(0, "[YYYY is] YYYY [MM]"), "YYYY is 1970 MM");
});

void test("accepts Intl.DateTimeFormat options", () => {
  assert.equal(formatDate(0, { dateStyle: "long" }, { locale: "ja" }), "1970年1月1日");
  assert.equal(formatDate("1970-01-01T12:00:00Z", { hour: "numeric" }), "12 PM");
});

void test("rejects invalid dates with a tagged error", () => {
  for (const invalid of ["not a date", Number.NaN, new Date(Number.NaN)]) {
    assert.throws(
      () => formatDate(invalid),
      (error: unknown) =>
        error instanceof RangeError &&
        error.message.startsWith("[VIZE_COMPOSE_DATE_FORMAT_INVALID_DATE]"),
    );
  }
});

void test("useDateFormat reacts to the date, pattern, locale, and zone", () => {
  const date = shallowRef<number | Date>(0);
  const pattern = shallowRef("MMMM");
  const locale = shallowRef("en");
  const timeZone = shallowRef("UTC");
  const label = useDateFormat(date, pattern, { locale, timeZone });

  assert.equal(label.value, "January");
  locale.value = "fr";
  assert.equal(label.value, "janvier");
  pattern.value = "YYYY-MM-DD HH:mm";
  timeZone.value = "Asia/Tokyo";
  assert.equal(label.value, "1970-01-01 09:00");
  date.value = new Date(instant);
  assert.equal(label.value, "2026-01-05 00:05");
});
