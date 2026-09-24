import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { FakeClock } from "./testing/fake-clock.ts";
import { formatTimeAgo, useTimeAgo } from "./use-time-ago.ts";

const day = 86_400_000;
const now = Date.UTC(2026, 0, 2);

void test("picks the largest fitting unit and the phrase direction", () => {
  assert.equal(formatTimeAgo(now - day, now), "yesterday");
  assert.equal(formatTimeAgo(now, now), "now");
  assert.equal(formatTimeAgo(now + 3 * 3_600_000, now), "in 3 hours");
  assert.equal(formatTimeAgo(now - 90_000, now, { numeric: "always" }), "2 minutes ago");
  assert.equal(formatTimeAgo(now - 14 * day, now), "2 weeks ago");
  assert.equal(formatTimeAgo(now - 400 * day, now), "last year");
});

void test("honours locale, style, units, rounding, and thresholds", () => {
  assert.equal(formatTimeAgo(now - 3 * day, now, { locale: "ja" }), "3 日前");
  assert.equal(
    formatTimeAgo(now - 10 * day, now, { units: ["day"], style: "short" }),
    "10 days ago",
  );
  assert.equal(
    formatTimeAgo(now - 90_000, now, { rounding: "floor", numeric: "always" }),
    "1 minute ago",
  );
  assert.equal(formatTimeAgo(now - 30_000, now, { justNowMs: 60_000 }), "now");
  assert.equal(formatTimeAgo(now - 400 * day, now, { maxMs: 365 * day }), "Nov 28, 2024");
  assert.equal(
    formatTimeAgo(now - 400 * day, now, {
      maxMs: 365 * day,
      fullDateFormatter: (date) => date.toISOString(),
    }),
    "2024-11-28T00:00:00.000Z",
  );
  assert.equal(
    formatTimeAgo("2026-01-01T00:00:00Z", now, { units: ["hour", "minute"] }),
    "24 hours ago",
  );
});

void test("rejects invalid dates with a tagged error", () => {
  assert.throws(
    () => formatTimeAgo("soon", now),
    (error: unknown) =>
      error instanceof RangeError &&
      error.message.startsWith("[VIZE_COMPOSE_TIME_AGO_INVALID_DATE]"),
  );
});

void test("useTimeAgo refreshes on its interval and follows reactive inputs", () => {
  const clock = new FakeClock();
  clock.now = now;
  const time = shallowRef<number>(now - 30_000);
  const locale = shallowRef("en");
  const phrase = useTimeAgo(time, {
    locale,
    numeric: "always",
    updateIntervalMs: 60_000,
    now: () => clock.now,
    runOnServer: true,
    scheduler: clock.interval,
  });

  assert.equal(phrase.value, "30 seconds ago");
  clock.advance(60_000);
  assert.equal(phrase.value, "2 minutes ago");
  locale.value = "de";
  assert.equal(phrase.value, "vor 2 Minuten");
  time.value = clock.now;
  assert.equal(phrase.value, "vor 0 Sekunden");
});

void test("useTimeAgo is deterministic on the server with an injected initial now", () => {
  const clock = new FakeClock();
  const {
    timeAgo,
    now: reference,
    isActive,
    pause,
  } = useTimeAgo(now - day, {
    controls: true,
    initialNow: now,
    now: () => now + 5 * day,
    scheduler: clock.interval,
  });

  assert.equal(timeAgo.value, "yesterday");
  assert.equal(reference.value, now);
  assert.equal(clock.size, 0);
  assert.equal(isActive.value, true);
  pause();
  assert.equal(isActive.value, false);
});
