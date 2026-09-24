import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import {
  compareTimes,
  createPlainTime,
  formatIsoTime,
  isSameTime,
  normalizePlainTime,
  parseIsoTime,
  truncateTime,
} from "./plain-time.ts";
import type { PlainTime } from "./plain-time.ts";
import TimeField from "./time-field.vue";
import type { TimeFieldExpose } from "./time-field-types.ts";

function segment(root: Element, type: string): HTMLElement {
  const element = root.querySelector(`[role='spinbutton'][data-segment='${type}']`);
  assert.ok(element instanceof HTMLElement, `missing ${type} segment`);
  return element;
}

function order(root: Element): (string | null)[] {
  return [...root.querySelectorAll("[data-segment]")].map((element) =>
    element.getAttribute("data-segment"),
  );
}

async function key(target: Element, value: string): Promise<void> {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true }),
  );
  await nextTick();
}

async function type(text: string): Promise<void> {
  for (const character of text) {
    const target = document.activeElement;
    assert.ok(target);
    await key(target, character);
  }
}

function time(value: unknown): string {
  return value === null ? "null" : formatIsoTime(value as PlainTime, "second");
}

test("renders locale clocks: 12-hour with a day period or 24-hour without", () => {
  const english = mountInteraction(TimeField, {
    props: {
      locale: "en-US",
      defaultValue: createPlainTime(21, 5),
      name: "alarm",
      ariaLabel: "Alarm",
    },
  });
  const root = english.root();
  assert.deepEqual(order(root), ["hour", "minute", "dayPeriod"]);
  assert.equal(root.getAttribute("data-hour-cycle"), "12");
  assert.equal(root.getAttribute("data-granularity"), "minute");
  assert.equal(segment(root, "hour").textContent, "9");
  assert.equal(segment(root, "hour").getAttribute("aria-valuemin"), "1");
  assert.equal(segment(root, "hour").getAttribute("aria-valuemax"), "12");
  assert.equal(segment(root, "minute").textContent, "05");
  assert.equal(segment(root, "dayPeriod").textContent, "PM");
  assert.equal(segment(root, "dayPeriod").getAttribute("inputmode"), "text");
  assert.equal(segment(root, "dayPeriod").getAttribute("aria-valuetext"), "PM");
  assert.equal(root.querySelector<HTMLInputElement>("input[type='hidden']")?.value, "21:05");
  english.unmount();

  const german = mountInteraction(TimeField, {
    props: { locale: "de-DE", defaultValue: createPlainTime(7, 30, 15), granularity: "second" },
  });
  assert.deepEqual(order(german.root()), ["hour", "minute", "second"]);
  assert.equal(segment(german.root(), "hour").textContent, "07");
  assert.equal(segment(german.root(), "second").textContent, "15");
  assert.equal(segment(german.root(), "hour").getAttribute("aria-label"), "Stunde");
  german.unmount();

  const forced = mountInteraction(TimeField, {
    props: {
      locale: "en-US",
      hourCycle: 24,
      granularity: "hour",
      defaultValue: createPlainTime(0, 45),
    },
  });
  assert.deepEqual(order(forced.root()), ["hour"]);
  assert.equal(segment(forced.root(), "hour").textContent, "00");
  forced.unmount();

  const japanese = mountInteraction(TimeField, { props: { locale: "ja-JP", hourCycle: 12 } });
  assert.equal(order(japanese.root())[0], "dayPeriod");
  japanese.unmount();
});

test("typing hours, minutes, and a day period commits a 24-hour value", async () => {
  const handle = mountInteraction(TimeField, {
    props: { locale: "en-US" },
    record: ["update:modelValue", "change"],
  });
  const root = handle.root();
  segment(root, "hour").focus();
  await type("7");
  assert.equal(document.activeElement, segment(root, "minute"));
  await type("45");
  assert.equal(document.activeElement, segment(root, "dayPeriod"));
  assert.equal(root.getAttribute("data-state"), "partial");
  await type("p");
  assert.equal(root.getAttribute("data-state"), "complete");
  const updates = handle.recorded().filter((entry) => entry.event === "update:modelValue");
  assert.deepEqual(
    updates.map((entry) => time(entry.payload[0])),
    ["19:45:00"],
  );
  segment(root, "dayPeriod").focus();
  await type("a");
  assert.equal(time(handle.recorded().at(-1)?.payload[0]), "07:45:00");
  await key(segment(root, "dayPeriod"), "ArrowUp");
  assert.equal(segment(root, "dayPeriod").textContent, "PM");
  handle.unmount();
});

test("12 AM and 12 PM map to midnight and noon; hours wrap within the clock", async () => {
  const handle = mountInteraction(TimeField, {
    props: { locale: "en-US", hourCycle: 12, defaultValue: createPlainTime(0, 0) },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  assert.equal(segment(root, "hour").textContent, "12");
  assert.equal(segment(root, "dayPeriod").textContent, "AM");
  await key(segment(root, "dayPeriod"), "ArrowDown");
  assert.equal(time(handle.recorded().at(-1)?.payload[0]), "12:00:00");
  await key(segment(root, "hour"), "ArrowUp");
  assert.equal(segment(root, "hour").textContent, "1");
  assert.equal(time(handle.recorded().at(-1)?.payload[0]), "13:00:00");
  await key(segment(root, "minute"), "ArrowDown");
  assert.equal(segment(root, "minute").textContent, "59");
  await key(segment(root, "minute"), "PageUp");
  assert.equal(segment(root, "minute").textContent, "14");
  handle.unmount();
});

test("24-hour typing accepts two-digit hours and seconds granularity requires seconds", async () => {
  const handle = mountInteraction(TimeField, {
    props: { locale: "en-GB", granularity: "second" },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  segment(root, "hour").focus();
  await type("2");
  assert.equal(document.activeElement, segment(root, "hour"));
  await type("3");
  assert.equal(document.activeElement, segment(root, "minute"));
  await type("59");
  assert.equal(handle.recorded().length, 0);
  await type("58");
  assert.equal(time(handle.recorded().at(-1)?.payload[0]), "23:59:58");
  assert.equal(root.querySelector("input"), null);
  handle.unmount();
});

test("min and max mark times invalid; placeholderValue seeds empty stepping", async () => {
  const handle = mountInteraction(TimeField, {
    props: {
      locale: "en-GB",
      min: createPlainTime(9, 0),
      max: createPlainTime(17, 30),
      placeholderValue: createPlainTime(8, 0),
    },
  });
  const root = handle.root();
  await key(segment(root, "hour"), "ArrowUp");
  await key(segment(root, "minute"), "ArrowUp");
  assert.equal(root.getAttribute("data-state"), "invalid");
  assert.equal(segment(root, "hour").getAttribute("aria-invalid"), "true");
  await key(segment(root, "hour"), "ArrowUp");
  assert.equal(root.getAttribute("data-state"), "complete");
  handle.unmount();
});

test("disabled and read-only time fields block edits", async () => {
  const handle = mountInteraction(TimeField, {
    props: { locale: "en-GB", readOnly: true, defaultValue: createPlainTime(10, 10) },
    record: ["update:modelValue"],
  });
  await key(segment(handle.root(), "hour"), "ArrowUp");
  assert.equal(handle.recorded().length, 0);
  await handle.wrapper.setProps({ readOnly: false, disabled: true });
  assert.equal(segment(handle.root(), "hour").getAttribute("aria-disabled"), "true");
  await key(segment(handle.root(), "hour"), "ArrowUp");
  assert.equal(handle.recorded().length, 0);
  handle.unmount();
});

test("exposes hour cycle, granularity, and imperative editing", async () => {
  const handle = mountInteraction(TimeField, { props: { locale: "en-US", hourCycle: 24 } });
  const api = handle.exposes<TimeFieldExpose>();
  assert.equal(api.hourCycle, 24);
  assert.equal(api.granularity, "minute");
  assert.equal(api.setValue(createPlainTime(6, 7)), true);
  await nextTick();
  assert.equal(segment(handle.root(), "hour").textContent, "06");
  await handle.wrapper.setProps({ hourCycle: 12 });
  assert.equal(segment(handle.root(), "hour").textContent, "6");
  assert.equal(segment(handle.root(), "dayPeriod").textContent, "AM");
  assert.equal(api.focus("minute"), true);
  assert.equal(api.clear(), true);
  assert.equal(api.value, null);
  handle.unmount();
});

test("plain time helpers validate, compare, truncate, and round-trip ISO text", () => {
  assert.throws(() => createPlainTime(24, 0), /VIZE_UI_PLAIN_TIME_INVALID/);
  assert.equal(normalizePlainTime({ hour: 1, minute: 60 }), null);
  assert.deepEqual(normalizePlainTime({ hour: 1, minute: 2 }), { hour: 1, minute: 2, second: 0 });
  assert.equal(compareTimes(createPlainTime(1, 2, 3), createPlainTime(1, 2, 4)), -1);
  assert.equal(isSameTime(null, undefined), true);
  assert.deepEqual(truncateTime(createPlainTime(9, 8, 7), "hour"), {
    hour: 9,
    minute: 0,
    second: 0,
  });
  assert.equal(formatIsoTime(createPlainTime(9, 8, 7)), "09:08");
  assert.deepEqual(parseIsoTime("23:59:58.123"), { hour: 23, minute: 59, second: 58 });
  assert.equal(parseIsoTime("7:00"), null);
});
