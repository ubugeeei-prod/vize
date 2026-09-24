import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import PopoverTrigger from "../../overlays/popover/popover-trigger.vue";
import { createDateRange, createPlainDate, formatIsoDate } from "../calendar/plain-date.ts";
import type { DateRange } from "../calendar/plain-date.ts";
import DateRangePickerCalendar from "./date-range-picker-calendar.vue";
import DateRangePickerContent from "./date-range-picker-content.vue";
import DateRangePickerField from "./date-range-picker-field.vue";
import DateRangePickerRoot from "./date-range-picker-root.vue";
import type { DateRangePickerRootExpose } from "./date-range-picker-types.ts";

const today = createPlainDate(2026, 9, 25);

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await nextTick();
}

function rangeText(value: unknown): string {
  const range = value as DateRange | null;
  return range ? `${formatIsoDate(range.start)}/${formatIsoDate(range.end)}` : "null";
}

function mountRangePicker(props: Record<string, unknown> = {}, record: readonly string[] = []) {
  const Picker = defineComponent({
    props: {
      pickerProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    },
    emits: ["update:modelValue", "change"],
    setup(componentProps, { emit, expose }) {
      const exposed: { root: DateRangePickerRootExpose | null } = { root: null };
      expose(exposed);
      return () =>
        h(
          DateRangePickerRoot,
          {
            ...componentProps.pickerProps,
            ref: (value: unknown) => {
              exposed.root = value as DateRangePickerRootExpose | null;
            },
            "onUpdate:modelValue": (value: DateRange | null) => emit("update:modelValue", value),
            onChange: (...args: unknown[]) => emit("change", ...args),
          },
          () => [
            h(DateRangePickerField, { boundary: "start", ariaLabel: "Check-in" }),
            h(DateRangePickerField, { boundary: "end", ariaLabel: "Check-out" }, () =>
              h(PopoverTrigger, { ariaLabel: "Choose dates" }, () => "📅"),
            ),
            h(DateRangePickerContent, { portalDisabled: true }, () =>
              h(DateRangePickerCalendar, { numberOfMonths: 2 }),
            ),
          ],
        );
    },
  });
  return mountInteraction(Picker, {
    props: { pickerProps: { today, locale: "en-US", ...props } },
    record,
  });
}

function field(root: Element, boundary: "start" | "end"): HTMLElement {
  const element = root.querySelector(`[data-vize-ui='date-field'][data-boundary='${boundary}']`);
  assert.ok(element instanceof HTMLElement);
  return element;
}

function day(iso: string): HTMLButtonElement {
  const element = document.querySelector(
    `[data-vize-ui="calendar-day"][data-date="${iso}"]:not([data-outside-month])`,
  );
  assert.ok(element instanceof HTMLButtonElement, `missing day ${iso}`);
  return element;
}

async function key(target: Element, value: string): Promise<void> {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true }),
  );
  await settle();
}

test("renders start and end fields and commits a two-click calendar range", async () => {
  const handle = mountRangePicker({ startName: "checkin", endName: "checkout" }, [
    "update:modelValue",
    "change",
  ]);
  const root = handle.root();
  assert.equal(root.getAttribute("data-vize-ui"), "date-range-picker");
  assert.equal(field(root, "start").getAttribute("aria-label"), "Check-in");
  assert.match(field(root, "end").id, /-end$/u);

  await handle.click(root.querySelector("[data-vize-ui='popover-trigger']") as Element);
  await settle();
  assert.equal(document.activeElement, day("2026-09-25"));
  assert.equal(document.querySelectorAll("[data-vize-ui='calendar-grid']").length, 2);
  await handle.click(day("2026-10-02"));
  await handle.click(day("2026-09-28"));
  await settle();
  assert.equal(rangeText(handle.recorded()[0]?.payload[0]), "2026-09-28/2026-10-02");
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.getAttribute("data-start"), "2026-09-28");
  assert.equal(field(root, "start").querySelector<HTMLInputElement>("input")?.value, "2026-09-28");
  assert.equal(field(root, "end").querySelector<HTMLInputElement>("input")?.value, "2026-10-02");
  assert.equal(field(root, "end").querySelector<HTMLInputElement>("input")?.name, "checkout");
  handle.unmount();
});

test("editing one field keeps a draft endpoint until both are known and orders the range", async () => {
  const handle = mountRangePicker(
    { defaultValue: createDateRange(createPlainDate(2026, 9, 10), createPlainDate(2026, 9, 12)) },
    ["update:modelValue"],
  );
  const root = handle.root();
  await key(field(root, "end").querySelector("[data-segment='day']") as Element, "Delete");
  assert.equal(rangeText(handle.recorded().at(-1)?.payload[0]), "null");
  assert.equal(field(root, "start").querySelector("[data-segment='day']")?.textContent, "10");
  const endDay = field(root, "end").querySelector("[data-segment='day']") as HTMLElement;
  endDay.focus();
  await key(endDay, "0");
  await key(document.activeElement as Element, "5");
  assert.equal(rangeText(handle.recorded().at(-1)?.payload[0]), "2026-09-05/2026-09-10");
  assert.equal(field(root, "start").querySelector("[data-segment='day']")?.textContent, "05");
  handle.unmount();
});

test("exposes imperative range and open control and respects disabled", async () => {
  const handle = mountRangePicker();
  const api = (handle.wrapper.vm as unknown as { root: DateRangePickerRootExpose }).root;
  assert.equal(
    api.setValue(createDateRange(createPlainDate(2026, 1, 1), createPlainDate(2026, 1, 3))),
    true,
  );
  await settle();
  assert.equal(
    field(handle.root(), "end").querySelector("[data-segment='day']")?.textContent,
    "03",
  );
  assert.equal(api.setOpen(true), true);
  await settle();
  assert.equal(day("2026-01-02").getAttribute("data-in-range"), "true");
  assert.equal(api.setValue(null), true);
  await settle();
  assert.equal(field(handle.root(), "start").getAttribute("data-state"), "empty");
  handle.unmount();

  const disabled = mountRangePicker({ disabled: true });
  const trigger = disabled
    .root()
    .querySelector<HTMLButtonElement>("[data-vize-ui='popover-trigger']");
  assert.equal(trigger?.disabled, true);
  disabled.unmount();
});
