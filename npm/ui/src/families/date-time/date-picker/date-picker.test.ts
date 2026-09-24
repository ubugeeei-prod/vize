import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import PopoverTrigger from "../../overlays/popover/popover-trigger.vue";
import { createPlainDate, formatIsoDate } from "../calendar/plain-date.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import DatePickerCalendar from "./date-picker-calendar.vue";
import DatePickerContent from "./date-picker-content.vue";
import DatePickerField from "./date-picker-field.vue";
import DatePickerRoot from "./date-picker-root.vue";
import type { DatePickerRootExpose } from "./date-picker-types.ts";

const today = createPlainDate(2026, 9, 25);

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await nextTick();
}

function mountPicker(props: Record<string, unknown> = {}, record: readonly string[] = []) {
  const Picker = defineComponent({
    props: {
      pickerProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    },
    emits: ["update:modelValue", "change", "update:open", "open-change"],
    setup(componentProps, { emit, expose }) {
      const exposed: { root: DatePickerRootExpose | null } = { root: null };
      expose(exposed);
      return () =>
        h(
          DatePickerRoot,
          {
            ...componentProps.pickerProps,
            ref: (value: unknown) => {
              exposed.root = value as DatePickerRootExpose | null;
            },
            "onUpdate:modelValue": (value: PlainDate | null) => emit("update:modelValue", value),
            onChange: (...args: unknown[]) => emit("change", ...args),
            "onUpdate:open": (value: boolean) => emit("update:open", value),
            "onOpen-change": (...args: unknown[]) => emit("open-change", ...args),
          },
          () => [
            h(DatePickerField, { ariaLabel: "Departure" }, () =>
              h(PopoverTrigger, { ariaLabel: "Choose date" }, () => "📅"),
            ),
            h(DatePickerContent, { portalDisabled: true, ariaLabel: "Calendar" }, () =>
              h(DatePickerCalendar),
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

function trigger(root: Element): HTMLButtonElement {
  const element = root.querySelector("[data-vize-ui='popover-trigger']");
  assert.ok(element instanceof HTMLButtonElement);
  return element;
}

function day(iso: string): HTMLButtonElement {
  const element = document.querySelector(
    `[data-vize-ui="calendar-day"][data-date="${iso}"]:not([data-outside-month])`,
  );
  assert.ok(element instanceof HTMLButtonElement, `missing day ${iso}`);
  return element;
}

test("renders a date field with a trigger that opens a calendar dialog focused on the value", async () => {
  const handle = mountPicker({ defaultValue: createPlainDate(2026, 10, 12), name: "departure" }, [
    "update:open",
    "open-change",
  ]);
  const root = handle.root();
  assert.equal(root.getAttribute("data-vize-ui"), "date-picker");
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.getAttribute("data-value"), "2026-10-12");
  const field = root.querySelector("[data-vize-ui='date-field']");
  assert.equal(field?.getAttribute("aria-label"), "Departure");
  assert.equal(field?.querySelector<HTMLInputElement>("input")?.value, "2026-10-12");
  assert.equal(trigger(root).getAttribute("aria-expanded"), "false");
  assert.equal(trigger(root).getAttribute("aria-haspopup"), "dialog");

  await handle.click(trigger(root));
  await settle();
  assert.equal(root.getAttribute("data-state"), "open");
  assert.equal(trigger(root).getAttribute("aria-expanded"), "true");
  const dialog = document.querySelector("[data-vize-ui='popover-content']");
  assert.equal(dialog?.getAttribute("role"), "dialog");
  assert.equal(dialog?.getAttribute("aria-label"), "Calendar");
  assert.equal(document.activeElement, day("2026-10-12"));
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload[0]]),
    [
      ["update:open", true],
      ["open-change", true],
    ],
  );
  handle.unmount();
});

test("selecting a day commits the value, closes the popover, and restores focus to the trigger", async () => {
  const handle = mountPicker({}, ["update:modelValue", "change", "update:open"]);
  const root = handle.root();
  await handle.click(trigger(root));
  await settle();
  assert.equal(document.activeElement, day("2026-09-25"));
  await handle.click(day("2026-09-18"));
  await settle();
  const events = handle.recorded().map((entry) => entry.event);
  assert.deepEqual(events, ["update:open", "update:modelValue", "change", "update:open"]);
  assert.equal(formatIsoDate(handle.recorded()[1]?.payload[0] as PlainDate), "2026-09-18");
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.querySelector("[data-segment='day']")?.textContent, "18");
  assert.equal(document.activeElement, trigger(root));
  handle.unmount();
});

test("typing in the field moves the open calendar and keyboard Escape dismisses", async () => {
  const handle = mountPicker({ defaultOpen: true, defaultValue: createPlainDate(2026, 9, 1) });
  const root = handle.root();
  await settle();
  const month = root.querySelector<HTMLElement>("[data-segment='month']");
  assert.ok(month);
  month.dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true, cancelable: true }),
  );
  await settle();
  assert.equal(
    document.querySelector("[data-vize-ui='calendar-grid']")?.getAttribute("aria-label"),
    "October 2026",
  );
  assert.equal(day("2026-10-01").getAttribute("data-selected"), "true");
  day("2026-10-01").focus();
  document.activeElement?.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
  );
  await settle();
  assert.equal(root.getAttribute("data-state"), "closed");
  handle.unmount();
});

test("closeOnSelect=false keeps the popover open and read-only blocks value changes", async () => {
  const open = mountPicker({ defaultOpen: true, closeOnSelect: false }, ["update:modelValue"]);
  await settle();
  await open.click(day("2026-09-03"));
  await settle();
  assert.equal(open.root().getAttribute("data-state"), "open");
  assert.equal(open.recorded().length, 1);
  open.unmount();

  const readOnly = mountPicker({ defaultOpen: true, readOnly: true }, ["update:modelValue"]);
  await settle();
  await readOnly.click(day("2026-09-03"));
  assert.equal(readOnly.recorded().length, 0);
  assert.equal(readOnly.root().getAttribute("data-readonly"), "true");
  readOnly.unmount();
});

test("disabled pickers cannot open and expose imperative value and open control", async () => {
  const disabled = mountPicker({ disabled: true });
  assert.equal(trigger(disabled.root()).disabled, true);
  assert.equal(
    disabled.root().querySelector("[role='spinbutton']")?.getAttribute("aria-disabled"),
    "true",
  );
  disabled.unmount();

  const handle = mountPicker({ min: createPlainDate(2026, 9, 10) });
  const api = (handle.wrapper.vm as unknown as { root: DatePickerRootExpose }).root;
  assert.equal(api.setOpen(true), true);
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "open");
  assert.equal(day("2026-09-09").disabled, true);
  assert.equal(api.setValue(createPlainDate(2026, 9, 12)), true);
  await settle();
  assert.equal(handle.root().getAttribute("data-value"), "2026-09-12");
  assert.equal(api.setOpen(false), true);
  assert.equal(api.open, false);
  handle.unmount();
});
