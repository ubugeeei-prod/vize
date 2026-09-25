import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import DateTimeField from "./datetime-field.vue";
import type { DateTimeFieldExpose } from "./datetime-field-types.ts";
import {
  addMinutes,
  compareDateTimes,
  createPlainDateTime,
  formatIsoDateTime,
  fromEpochMillisecondsDateTime,
  minutesBetween,
  normalizePlainDateTime,
  parseIsoDateTime,
} from "./plain-date-time.ts";
import type { PlainDateTime } from "./plain-date-time.ts";

function order(root: Element): (string | null)[] {
  return [...root.querySelectorAll("[role='spinbutton']")].map((element) =>
    element.getAttribute("data-segment"),
  );
}

function segment(root: Element, type: string): HTMLElement {
  const element = root.querySelector(`[role='spinbutton'][data-segment='${type}']`);
  assert.ok(element instanceof HTMLElement, `missing ${type}`);
  return element;
}

async function type(text: string): Promise<void> {
  for (const character of text) {
    document.activeElement?.dispatchEvent(
      new KeyboardEvent("keydown", { key: character, bubbles: true, cancelable: true }),
    );
    await nextTick();
  }
}

function iso(value: unknown): string {
  return value === null ? "null" : formatIsoDateTime(value as PlainDateTime, "second");
}

test("renders date and time segments in one locale order with a named ISO input", () => {
  const handle = mountInteraction(DateTimeField, {
    props: {
      locale: "en-US",
      name: "start",
      ariaLabel: "Start",
      defaultValue: createPlainDateTime(2026, 9, 25, 21, 5),
    },
  });
  const root = handle.root();
  assert.deepEqual(order(root), ["month", "day", "year", "hour", "minute", "dayPeriod"]);
  assert.equal(root.getAttribute("data-vize-ui"), "datetime-field");
  assert.equal(segment(root, "hour").textContent, "9");
  assert.equal(segment(root, "month").getAttribute("aria-valuetext"), "9 – September");
  assert.equal(
    root.querySelector<HTMLInputElement>("[data-vize-ui='datetime-field-input']")?.value,
    "2026-09-25T21:05",
  );
  handle.unmount();

  const japanese = mountInteraction(DateTimeField, {
    props: { locale: "ja-JP", hourCycle: 24, granularity: "second" },
  });
  assert.deepEqual(order(japanese.root()), ["year", "month", "day", "hour", "minute", "second"]);
  japanese.unmount();
});

test("typing fills every segment and commits a PlainDateTime", async () => {
  const handle = mountInteraction(DateTimeField, {
    props: { locale: "en-GB", hourCycle: 24 },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  segment(root, "day").focus();
  await type("24122026");
  assert.equal(document.activeElement, segment(root, "hour"));
  assert.equal(handle.recorded().length, 0);
  await type("1830");
  assert.equal(iso(handle.recorded().at(-1)?.payload[0]), "2026-12-24T18:30:00");
  assert.equal(root.getAttribute("data-state"), "complete");
  handle.unmount();
});

test("min, max, and unavailable dates invalidate; placeholderValue seeds stepping", async () => {
  const handle = mountInteraction(DateTimeField, {
    props: {
      locale: "en-GB",
      hourCycle: 24,
      min: createPlainDateTime(2026, 1, 1, 9),
      isDateUnavailable: (date: PlainDateTime) => date.day === 13,
      placeholderValue: createPlainDateTime(2026, 1, 1, 8, 45),
    },
  });
  const root = handle.root();
  for (const name of ["year", "month", "day", "hour", "minute"]) {
    segment(root, name).dispatchEvent(
      new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true, cancelable: true }),
    );
    await nextTick();
  }
  assert.equal(segment(root, "hour").textContent, "08");
  assert.equal(segment(root, "minute").textContent, "00");
  assert.equal(root.getAttribute("data-state"), "invalid");
  segment(root, "hour").dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "complete");
  handle.unmount();
});

test("exposes the date-time value and imperative editing", async () => {
  const handle = mountInteraction(DateTimeField, { props: { locale: "en-US" } });
  const api = handle.exposes<DateTimeFieldExpose>();
  assert.equal(api.setValue(createPlainDateTime(2027, 1, 2, 0, 15)), true);
  await nextTick();
  assert.equal(iso(api.value), "2027-01-02T00:15:00");
  assert.equal(segment(handle.root(), "hour").textContent, "12");
  assert.equal(segment(handle.root(), "dayPeriod").textContent, "AM");
  assert.equal(api.clear(), true);
  assert.equal(api.value, null);
  handle.unmount();
});

test("plain date-time helpers compare, shift, parse, and resolve zoned instants", () => {
  const start = createPlainDateTime(2026, 12, 31, 23, 30);
  assert.equal(iso(addMinutes(start, 45)), "2027-01-01T00:15:00");
  assert.equal(iso(addMinutes(start, -1_440)), "2026-12-30T23:30:00");
  assert.equal(minutesBetween(start, addMinutes(start, 3_000)), 3_000);
  assert.equal(compareDateTimes(start, addMinutes(start, 1)), -1);
  assert.equal(iso(parseIsoDateTime("2026-09-25T07:08:09")), "2026-09-25T07:08:09");
  assert.equal(iso(parseIsoDateTime("2026-09-25 07:08")), "2026-09-25T07:08:00");
  assert.equal(parseIsoDateTime("2026-09-25"), null);
  assert.equal(normalizePlainDateTime({ year: 2026, month: 2, day: 30, hour: 1, minute: 0 }), null);
  assert.throws(() => createPlainDateTime(2026, 1, 1, 24), /VIZE_UI_PLAIN_DATE_TIME_INVALID/);
  assert.equal(
    iso(fromEpochMillisecondsDateTime(Date.UTC(2026, 8, 24, 16, 30, 5), "Asia/Tokyo")),
    "2026-09-25T01:30:05",
  );
  assert.equal(fromEpochMillisecondsDateTime(0, "No/Zone"), null);
});
