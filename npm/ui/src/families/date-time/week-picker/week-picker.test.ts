import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import {
  createPlainDate,
  formatIsoDate,
  formatIsoWeek,
  isoWeekOf,
  parseIsoWeek,
  startOfIsoWeek,
} from "../calendar/plain-date.ts";
import type { DateRange } from "../calendar/plain-date.ts";
import WeekPickerRoot from "./week-picker-root.vue";
import { weekOf } from "./week-picker-selection.ts";
import type { WeekPickerRootExpose } from "./week-picker-types.ts";

const today = createPlainDate(2026, 9, 25);

function day(root: Element, iso: string): HTMLButtonElement {
  const element = root.querySelector(
    `[data-vize-ui="calendar-day"][data-date="${iso}"]:not([data-outside-month])`,
  );
  assert.ok(element instanceof HTMLButtonElement, `missing day ${iso}`);
  return element;
}

function rangeText(value: unknown): string {
  const range = value as DateRange;
  return `${formatIsoDate(range.start)}/${formatIsoDate(range.end)}`;
}

test("activating any day selects its locale week and shows ISO week numbers", async () => {
  const handle = mountInteraction(WeekPickerRoot, {
    props: { today, locale: "en-GB", name: "week" },
    record: ["update:modelValue", "change", "select"],
  });
  const root = handle.root();
  assert.equal(root.getAttribute("data-vize-ui"), "week-picker");
  assert.equal(root.getAttribute("data-mode"), "week");
  assert.equal(
    root.querySelector("[data-vize-ui='calendar-week-number-header']")?.getAttribute("abbr"),
    "Week",
  );
  const weekNumbers = [...root.querySelectorAll("[data-vize-ui='calendar-week-number']")].map(
    (cell) => cell.textContent?.trim(),
  );
  assert.deepEqual(weekNumbers, ["36", "37", "38", "39", "40"]);
  assert.equal(
    root.querySelector("[data-vize-ui='calendar-week-number']")?.getAttribute("scope"),
    "row",
  );

  await handle.click(day(root, "2026-09-24"));
  assert.equal(rangeText(handle.recorded()[0]?.payload[0]), "2026-09-21/2026-09-27");
  assert.deepEqual(
    handle.recorded().map((entry) => entry.event),
    ["update:modelValue", "change", "select"],
  );
  assert.equal(root.getAttribute("data-week"), "2026-W39");
  assert.equal(root.querySelector<HTMLInputElement>("input")?.value, "2026-W39");
  assert.equal(day(root, "2026-09-21").getAttribute("data-range-start"), "true");
  assert.equal(day(root, "2026-09-27").getAttribute("data-range-end"), "true");
  assert.equal(day(root, "2026-09-23").closest("td")?.getAttribute("aria-selected"), "true");
  handle.unmount();
});

test("pointer and keyboard focus preview the week; US locales start on Sunday", async () => {
  const handle = mountInteraction(WeekPickerRoot, {
    props: { today, locale: "en-US", hideWeekNumbers: true },
  });
  const root = handle.root();
  assert.equal(root.querySelector("[data-vize-ui='calendar-week-number']"), null);
  day(root, "2026-09-09").dispatchEvent(new PointerEvent("pointerenter"));
  await nextTick();
  assert.equal(day(root, "2026-09-06").getAttribute("data-preview"), "true");
  assert.equal(day(root, "2026-09-12").getAttribute("data-range-end"), "true");
  day(root, "2026-09-25").focus();
  day(root, "2026-09-25").dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true }),
  );
  await nextTick();
  await nextTick();
  assert.equal(day(root, "2026-10-01").getAttribute("data-preview"), "true");
  await handle.press(document.activeElement as HTMLElement, "Enter");
  const api = handle.exposes<WeekPickerRootExpose>();
  assert.equal(rangeText(api.value), "2026-09-27/2026-10-03");
  assert.deepEqual(api.isoWeek, { year: 2026, week: 40 });
  assert.equal(api.setValue(weekOf(createPlainDate(2027, 1, 1), 0)), true);
  handle.unmount();
});

test("ISO week helpers cover year boundaries and week 53", () => {
  assert.deepEqual(isoWeekOf(createPlainDate(2021, 1, 3)), { year: 2020, week: 53 });
  assert.deepEqual(isoWeekOf(createPlainDate(2024, 12, 30)), { year: 2025, week: 1 });
  assert.deepEqual(isoWeekOf(createPlainDate(2026, 9, 25)), { year: 2026, week: 39 });
  assert.equal(formatIsoWeek({ year: 2026, week: 5 }), "2026-W05");
  assert.deepEqual(parseIsoWeek("2020-W53"), { year: 2020, week: 53 });
  assert.equal(parseIsoWeek("2021-W53"), null);
  assert.equal(formatIsoDate(startOfIsoWeek({ year: 2025, week: 1 }) ?? today), "2024-12-30");
  assert.equal(rangeText(weekOf(createPlainDate(2026, 9, 25), 6)), "2026-09-19/2026-09-25");
});
