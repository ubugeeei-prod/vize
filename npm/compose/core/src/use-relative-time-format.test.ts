import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { selectRelativeTimeUnit, useRelativeTimeFormat } from "./use-relative-time-format.ts";

const MINUTE = 60_000;
const DAY = 86_400_000;

void test("selects the best-fitting unit", () => {
  assert.deepEqual(selectRelativeTimeUnit(0), { value: 0, unit: "second" });
  assert.deepEqual(selectRelativeTimeUnit(-45_000), { value: -45, unit: "second" });
  assert.deepEqual(selectRelativeTimeUnit(5 * MINUTE), { value: 5, unit: "minute" });
  assert.deepEqual(selectRelativeTimeUnit(-3 * 60 * MINUTE), { value: -3, unit: "hour" });
  assert.deepEqual(selectRelativeTimeUnit(2 * DAY), { value: 2, unit: "day" });
  assert.deepEqual(selectRelativeTimeUnit(-14 * DAY), { value: -2, unit: "week" });
  assert.deepEqual(selectRelativeTimeUnit(95 * DAY), { value: 3, unit: "month" });
  assert.deepEqual(selectRelativeTimeUnit(-800 * DAY), { value: -2, unit: "year" });
  assert.throws(
    () => selectRelativeTimeUnit(Number.NaN),
    /VIZE_COMPOSE_RELATIVE_TIME_INVALID_DIFF/,
  );
});

void test("formats the reactive value and unit", () => {
  const value = ref<number | null>(-1);
  const unit = ref<Intl.RelativeTimeFormatUnit>("day");
  const relative = useRelativeTimeFormat(value, unit, { locale: "en-US", numeric: "auto" });

  assert.equal(relative.formatted.value, "yesterday");
  value.value = 3;
  unit.value = "week";
  assert.equal(relative.formatted.value, "in 3 weeks");
  value.value = null;
  assert.equal(relative.formatted.value, "");
});

void test("formats differences and dates against an explicit reference", () => {
  const relative = useRelativeTimeFormat(null, "second", { locale: "en-US" });
  const now = Date.UTC(2026, 8, 25);

  assert.equal(relative.formatDiff(-5 * MINUTE), "5 minutes ago");
  assert.equal(relative.formatFrom(now + 2 * DAY, { now }), "in 2 days");
  assert.equal(relative.formatFrom(new Date(now - 400 * DAY), { now }), "1 year ago");
  assert.deepEqual(
    relative.formatToParts(2, "hour").map((part) => part.type),
    ["literal", "integer", "literal"],
  );
});

void test("follows the reactive locale", () => {
  const locale = ref("en-US");
  const relative = useRelativeTimeFormat(2, "day", () => ({ locale: locale.value }));
  assert.equal(relative.formatted.value, "in 2 days");
  locale.value = "ja-JP";
  assert.equal(relative.formatted.value, "2 日後");
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => ({
    en: useRelativeTimeFormat(-1, "day", { locale: "en-US", numeric: "auto" }).formatted,
    ja: useRelativeTimeFormat(3, "hour", { locale: "ja-JP" }).formatted,
  }));
  assert.equal(state, '{"en":"yesterday","ja":"3 時間後"}');
});
