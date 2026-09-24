import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useDateTimeFormat } from "./use-date-time-format.ts";

const moment = Date.UTC(2026, 8, 25, 12, 30);

void test("formats the reactive value with explicit locale and zone", async () => {
  const value = ref<Date | number | null>(moment);
  const date = useDateTimeFormat(value, { locale: "en-US", dateStyle: "medium", timeZone: "UTC" });

  assert.equal(date.formatted.value, "Sep 25, 2026");
  value.value = new Date(Date.UTC(2027, 0, 1));
  await nextTick();
  assert.equal(date.formatted.value, "Jan 1, 2027");
  value.value = null;
  assert.equal(date.formatted.value, "");
  value.value = new Date(Number.NaN);
  assert.equal(date.formatted.value, "", "invalid dates render empty");
});

void test("follows reactive locale changes", () => {
  const locale = ref("en-US");
  const date = useDateTimeFormat(moment, () => ({
    locale: locale.value,
    dateStyle: "long",
    timeZone: "UTC",
  }));
  assert.equal(date.formatted.value, "September 25, 2026");
  locale.value = "ja-JP";
  assert.equal(date.formatted.value, "2026年9月25日");
  assert.equal(date.locale.value, "ja-JP");
});

void test("formats parts and ranges", () => {
  const date = useDateTimeFormat(undefined, {
    locale: "en-US",
    month: "short",
    day: "numeric",
    timeZone: "UTC",
  });
  assert.equal(date.formatted.value, "");
  assert.equal(date.format(moment), "Sep 25");
  assert.deepEqual(
    date.formatToParts(moment).map((part) => part.type),
    ["month", "literal", "day"],
  );
  assert.equal(date.formatRange(moment, moment + 2 * 86_400_000), "Sep 25\u2009–\u200927");
});

void test("surfaces invalid options as the platform RangeError", () => {
  const date = useDateTimeFormat(moment, { locale: "en-US", timeZone: "Mars/Olympus" });
  assert.throws(() => date.formatted.value, RangeError);
});

void test("server rendering is deterministic for explicit locale and zone", async () => {
  const state = await renderComposableOnServer(() => ({
    us: useDateTimeFormat(moment, { locale: "en-US", dateStyle: "short", timeZone: "UTC" })
      .formatted,
    ja: useDateTimeFormat(moment, {
      locale: "ja-JP",
      dateStyle: "full",
      timeZone: "Asia/Tokyo",
    }).formatted,
  }));
  assert.equal(state, '{"us":"9/25/26","ja":"2026年9月25日金曜日"}');
});
