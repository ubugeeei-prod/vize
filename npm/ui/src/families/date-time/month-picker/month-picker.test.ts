import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createPlainDate } from "../calendar/plain-date.ts";
import type { PlainYearMonth } from "../calendar/plain-date.ts";
import MonthPicker from "./month-picker.vue";
import {
  formatIsoYearMonth,
  fromMonthUnit,
  parseIsoYearMonth,
  toMonthUnit,
} from "./month-picker-runtime.ts";
import type { MonthPickerExpose } from "./month-picker-types.ts";
import { periodGridKeyOffset, periodPageStart } from "./period-grid-runtime.ts";

const today = createPlainDate(2026, 9, 25);

function month(root: Element, value: number): HTMLButtonElement {
  const element = root.querySelector(`[data-vize-ui='month-picker-month'][data-month='${value}']`);
  assert.ok(element instanceof HTMLButtonElement, `missing month ${value}`);
  return element;
}

async function key(target: Element, value: string, init: KeyboardEventInit = {}): Promise<void> {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true, ...init }),
  );
  await nextTick();
  await nextTick();
}

function focusedMonth(): string | null {
  return document.activeElement?.getAttribute("data-month") ?? null;
}

test("renders a labelled month grid with roving focus and the current month", () => {
  const handle = mountInteraction(MonthPicker, {
    props: { today, locale: "en-US", defaultValue: { year: 2026, month: 4 }, name: "billing" },
  });
  const root = handle.root();
  const grid = handle.getByRole("grid");
  const heading = root.querySelector("[data-vize-ui='month-picker-heading']");
  assert.equal(heading?.textContent?.trim(), "2026");
  assert.equal(root.getAttribute("aria-labelledby"), heading?.id);
  assert.equal(grid.getAttribute("aria-labelledby"), heading?.id);
  assert.equal(root.querySelectorAll("tr").length, 4);
  assert.equal(month(root, 4).getAttribute("tabindex"), "0");
  assert.equal(month(root, 4).getAttribute("aria-label"), "April 2026");
  assert.equal(month(root, 4).textContent?.trim(), "Apr");
  assert.equal(month(root, 4).closest("td")?.getAttribute("aria-selected"), "true");
  assert.equal(month(root, 9).getAttribute("aria-current"), "date");
  assert.equal(root.querySelector<HTMLInputElement>("input")?.value, "2026-04");
  handle.unmount();
});

test("keyboard moves by month, row, row edge, year, and decade; activation selects", async () => {
  const handle = mountInteraction(MonthPicker, {
    props: { today, locale: "en-US", columns: 4 },
    record: ["update:modelValue", "change", "update:focusedMonth"],
  });
  const root = handle.root();
  month(root, 9).focus();
  await key(month(root, 9), "ArrowRight");
  assert.equal(focusedMonth(), "10");
  await key(month(root, 10), "ArrowUp");
  assert.equal(focusedMonth(), "6");
  await key(month(root, 6), "Home");
  assert.equal(focusedMonth(), "5");
  await key(month(root, 5), "End");
  assert.equal(focusedMonth(), "8");
  await key(month(root, 8), "PageDown");
  assert.equal(root.getAttribute("data-year"), "2027");
  assert.equal(focusedMonth(), "8");
  await key(month(root, 8), "PageUp", { shiftKey: true });
  assert.equal(root.getAttribute("data-year"), "2017");
  await handle.press(month(root, 8), "Enter");
  const updates = handle.recorded().filter((entry) => entry.event === "update:modelValue");
  assert.deepEqual(updates[0]?.payload[0], { year: 2017, month: 8 });
  handle.unmount();
});

test("RTL, min/max bounds, unavailability, and year paging controls", async () => {
  const handle = mountInteraction(MonthPicker, {
    props: {
      today,
      locale: "en-US",
      dir: "rtl",
      min: { year: 2026, month: 3 },
      max: { year: 2027, month: 2 },
      isMonthUnavailable: (value: PlainYearMonth) => value.month === 6,
    },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  const previous = root.querySelector<HTMLButtonElement>("[data-vize-ui='month-picker-previous']");
  const next = root.querySelector<HTMLButtonElement>("[data-vize-ui='month-picker-next']");
  assert.equal(previous?.disabled, true);
  assert.equal(previous?.getAttribute("aria-label"), "Previous year");
  assert.equal(month(root, 2).disabled, true);
  assert.equal(month(root, 6).getAttribute("aria-disabled"), "true");
  await handle.click(month(root, 6));
  assert.equal(handle.recorded().length, 0);
  month(root, 6).focus();
  await key(month(root, 6), "ArrowLeft");
  assert.equal(focusedMonth(), "7");
  await key(month(root, 7), "Home");
  await key(month(root, 7), "ArrowUp");
  await key(month(root, 4), "ArrowUp");
  assert.equal(focusedMonth(), "3");
  await handle.click(next as HTMLButtonElement);
  assert.equal(root.getAttribute("data-year"), "2027");
  assert.equal(next?.disabled, true);
  assert.equal(month(root, 3).disabled, true);
  handle.unmount();
});

test("pending until mount without a clock, exposed API, and unit helpers", async () => {
  const handle = mountInteraction(MonthPicker, { props: { locale: "en-US", timeZone: "UTC" } });
  await nextTick();
  assert.notEqual(handle.root().getAttribute("data-state"), "pending");
  assert.equal(handle.root().querySelectorAll("[data-current='true']").length, 1);
  const api = handle.exposes<MonthPickerExpose>();
  assert.equal(api.setValue({ year: 2030, month: 1 }), true);
  await nextTick();
  assert.equal(api.year, 2030);
  assert.equal(api.navigate(-1), true);
  await nextTick();
  assert.equal(api.year, 2029);
  assert.equal(api.focus(), true);
  handle.unmount();

  assert.equal(toMonthUnit({ year: 2026, month: 1 }), 24_312);
  assert.deepEqual(fromMonthUnit(24_311), { year: 2025, month: 12 });
  assert.equal(formatIsoYearMonth({ year: 987, month: 3 }), "0987-03");
  assert.deepEqual(parseIsoYearMonth("2026-12"), { year: 2026, month: 12 });
  assert.equal(parseIsoYearMonth("2026-13"), null);
  assert.equal(periodPageStart(-1, 12), -12);
  assert.equal(
    periodGridKeyOffset("End", {
      index: 4,
      columns: 3,
      pageSize: 12,
      bigStep: 120,
      shiftKey: false,
      rtl: false,
    }),
    1,
  );
});
