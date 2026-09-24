import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createDateRange, createPlainDate, formatIsoDate } from "../calendar/plain-date.ts";
import type { DateRange, PlainDate } from "../calendar/plain-date.ts";
import RangeCalendarRoot from "./range-calendar-root.vue";
import type { RangeCalendarRootExpose } from "./range-calendar-types.ts";

const today = createPlainDate(2026, 9, 25);

function day(root: Element, iso: string): HTMLButtonElement {
  const element = root.querySelector(
    `[data-vize-ui="calendar-day"][data-date="${iso}"]:not([data-outside-month])`,
  );
  assert.ok(element instanceof HTMLButtonElement, `missing day ${iso}`);
  return element;
}

function rangeText(value: unknown): string {
  const range = value as DateRange | null;
  return range ? `${formatIsoDate(range.start)}/${formatIsoDate(range.end)}` : "null";
}

async function key(target: Element, value: string): Promise<boolean> {
  const event = new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  await nextTick();
  await nextTick();
  return event.defaultPrevented;
}

test("two activations anchor, preview, and commit an ordered range", async () => {
  const handle = mountInteraction(RangeCalendarRoot, {
    props: { today, locale: "en-US", startName: "from", endName: "to" },
    record: ["update:modelValue", "change", "anchor-change"],
  });
  const root = handle.root();
  assert.equal(root.getAttribute("data-mode"), "range");
  assert.equal(handle.getByRole("grid").getAttribute("aria-multiselectable"), "true");

  await handle.click(day(root, "2026-09-20"));
  assert.equal(root.getAttribute("data-anchor"), "2026-09-20");
  assert.equal(day(root, "2026-09-20").getAttribute("data-preview"), "true");
  day(root, "2026-09-14").dispatchEvent(new PointerEvent("pointerenter"));
  await nextTick();
  assert.equal(day(root, "2026-09-16").getAttribute("data-in-range"), "true");
  assert.equal(day(root, "2026-09-14").getAttribute("data-range-start"), "true");
  assert.equal(day(root, "2026-09-20").getAttribute("data-range-end"), "true");
  assert.equal(day(root, "2026-09-16").closest("td")?.getAttribute("aria-selected"), "false");

  await handle.click(day(root, "2026-09-14"));
  assert.deepEqual(
    handle
      .recorded()
      .map((entry) => [
        entry.event,
        entry.event === "anchor-change"
          ? entry.payload[0] === null
            ? "null"
            : formatIsoDate(entry.payload[0] as PlainDate)
          : rangeText(entry.payload[0]),
      ]),
    [
      ["anchor-change", "2026-09-20"],
      ["anchor-change", "null"],
      ["update:modelValue", "2026-09-14/2026-09-20"],
      ["change", "2026-09-14/2026-09-20"],
    ],
  );
  assert.equal(root.getAttribute("data-anchor"), null);
  assert.equal(root.getAttribute("data-start"), "2026-09-14");
  assert.equal(root.getAttribute("data-end"), "2026-09-20");
  assert.equal(day(root, "2026-09-17").getAttribute("data-state"), "range-middle");
  assert.equal(day(root, "2026-09-17").closest("td")?.getAttribute("aria-selected"), "true");
  assert.equal(day(root, "2026-09-14").getAttribute("data-state"), "selected");
  const inputs = [...root.querySelectorAll<HTMLInputElement>("input[type='hidden']")];
  assert.deepEqual(
    inputs.map((input) => [input.name, input.value]),
    [
      ["from", "2026-09-14"],
      ["to", "2026-09-20"],
    ],
  );
  handle.unmount();
});

test("keyboard focus previews the range and Escape cancels the anchor", async () => {
  const handle = mountInteraction(RangeCalendarRoot, {
    props: { today, locale: "en-US" },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  day(root, "2026-09-25").focus();
  await handle.press(day(root, "2026-09-25"), "Enter");
  assert.equal(root.getAttribute("data-anchor"), "2026-09-25");
  await key(day(root, "2026-09-25"), "ArrowRight");
  await key(day(root, "2026-09-26"), "ArrowDown");
  assert.equal(day(root, "2026-10-01").getAttribute("data-in-range"), "true");
  assert.equal(await key(day(root, "2026-10-03"), "Escape"), true);
  assert.equal(root.getAttribute("data-anchor"), null);
  assert.equal(day(root, "2026-10-01").getAttribute("data-in-range"), null);
  assert.equal(await key(day(root, "2026-10-03"), "Escape"), false);
  assert.equal(handle.recorded().length, 0);
  handle.unmount();
});

test("unavailable dates break ranges unless non-contiguous ranges are allowed", async () => {
  const blocked = mountInteraction(RangeCalendarRoot, {
    props: { today, locale: "en-US", isDateUnavailable: (date: PlainDate) => date.day === 15 },
    record: ["update:modelValue"],
  });
  const root = blocked.root();
  await blocked.click(day(root, "2026-09-10"));
  await blocked.click(day(root, "2026-09-20"));
  assert.equal(blocked.recorded().length, 0);
  assert.equal(root.getAttribute("data-anchor"), "2026-09-20");
  await blocked.click(day(root, "2026-09-15"));
  assert.equal(root.getAttribute("data-anchor"), "2026-09-20");
  await blocked.click(day(root, "2026-09-18"));
  assert.equal(rangeText(blocked.recorded()[0]?.payload[0]), "2026-09-18/2026-09-20");
  blocked.unmount();

  const allowed = mountInteraction(RangeCalendarRoot, {
    props: {
      today,
      locale: "en-US",
      allowNonContiguousRanges: true,
      isDateUnavailable: (date: PlainDate) => date.day === 15,
    },
    record: ["update:modelValue"],
  });
  await allowed.click(day(allowed.root(), "2026-09-10"));
  await allowed.click(day(allowed.root(), "2026-09-20"));
  assert.equal(rangeText(allowed.recorded()[0]?.payload[0]), "2026-09-10/2026-09-20");
  assert.equal(day(allowed.root(), "2026-09-15").getAttribute("data-state"), "unavailable");
  allowed.unmount();
});

test("controlled ranges, read-only mode, and the exposed API", async () => {
  const handle = mountInteraction(RangeCalendarRoot, {
    props: {
      today,
      locale: "en-US",
      numberOfMonths: 2,
      modelValue: createDateRange(createPlainDate(2026, 9, 28), createPlainDate(2026, 10, 3)),
    },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  assert.equal(day(root, "2026-10-01").getAttribute("data-in-range"), "true");
  const outside = root.querySelector("[data-month-index='0'] [data-date='2026-10-01']");
  assert.equal(outside?.getAttribute("data-outside-month"), "true");
  assert.equal(outside?.getAttribute("data-in-range"), "true");

  const api = handle.exposes<RangeCalendarRootExpose>();
  assert.equal(rangeText(api.value), "2026-09-28/2026-10-03");
  assert.equal(
    api.setValue(createDateRange(createPlainDate(2027, 1, 1), createPlainDate(2027, 1, 2))),
    true,
  );
  assert.equal(rangeText(handle.recorded()[0]?.payload[0]), "2027-01-01/2027-01-02");
  assert.equal(api.cancel(), false);
  assert.equal(api.anchor, null);

  await handle.wrapper.setProps({ readOnly: true });
  await handle.click(day(root, "2026-09-10"));
  assert.equal(root.getAttribute("data-anchor"), null);
  handle.unmount();
});
