import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createPlainTime, formatIsoTime } from "../time-field/plain-time.ts";
import type { PlainTime } from "../time-field/plain-time.ts";
import TimePicker from "./time-picker.vue";
import { createTimeSlots } from "./time-picker-runtime.ts";
import type { TimePickerExpose } from "./time-picker-types.ts";

function options(root: Element): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>("[role='option']")];
}

test("renders a listbox of localized slots between min and max every step minutes", () => {
  const handle = mountInteraction(TimePicker, {
    props: {
      locale: "en-US",
      ariaLabel: "Start time",
      min: createPlainTime(9, 0),
      max: createPlainTime(11, 0),
      step: 45,
      defaultValue: createPlainTime(10, 30),
      name: "start",
    },
  });
  const root = handle.root();
  const listbox = handle.getByRole("listbox", { name: "Start time" });
  assert.equal(root.getAttribute("data-vize-ui"), "time-picker");
  assert.equal(root.getAttribute("data-hour-cycle"), "12");
  assert.deepEqual(
    options(listbox).map((option) => option.textContent?.trim()),
    ["9:00 AM", "9:45 AM", "10:30 AM"],
  );
  assert.equal(options(listbox)[2]?.getAttribute("aria-selected"), "true");
  assert.equal(options(listbox)[2]?.getAttribute("data-time"), "10:30");
  assert.equal(root.querySelector<HTMLInputElement>("input")?.value, "10:30");
  handle.unmount();
});

test("selecting slots emits PlainTime values and honors unavailable slots and read-only", async () => {
  const handle = mountInteraction(TimePicker, {
    props: {
      locale: "en-GB",
      ariaLabel: "Slot",
      step: 60,
      max: createPlainTime(3, 0),
      isTimeUnavailable: (time: PlainTime) => time.hour === 2,
    },
    record: ["update:modelValue", "change"],
  });
  const listbox = handle.getByRole("listbox");
  assert.deepEqual(
    options(listbox).map((option) => option.textContent?.trim()),
    ["00:00", "01:00", "02:00", "03:00"],
  );
  assert.equal(options(listbox)[2]?.getAttribute("aria-disabled"), "true");
  await handle.click(options(listbox)[1] as HTMLElement);
  const update = handle.recorded().find((entry) => entry.event === "update:modelValue");
  assert.equal(formatIsoTime(update?.payload[0] as PlainTime), "01:00");
  const change = handle.recorded().find((entry) => entry.event === "change");
  assert.equal(change?.payload[1], null);
  await handle.click(options(listbox)[2] as HTMLElement);
  assert.equal(handle.recorded().filter((entry) => entry.event === "change").length, 1);

  await handle.wrapper.setProps({ readOnly: true });
  await handle.click(options(listbox)[3] as HTMLElement);
  await nextTick();
  assert.equal(handle.recorded().filter((entry) => entry.event === "change").length, 1);
  assert.equal(options(listbox)[1]?.getAttribute("aria-selected"), "true");
  handle.unmount();
});

test("exposes value, slots, focus, and setValue; slot generation clamps inputs", async () => {
  const handle = mountInteraction(TimePicker, {
    props: { locale: "en-US", hourCycle: 24, step: 360 },
  });
  const api = handle.exposes<TimePickerExpose>();
  assert.equal(api.slots.length, 4);
  assert.equal(api.hourCycle, 24);
  assert.equal(api.setValue(createPlainTime(12, 0)), true);
  await nextTick();
  assert.equal(options(handle.root())[2]?.getAttribute("aria-selected"), "true");
  api.focus();
  assert.equal(document.activeElement, handle.getByRole("listbox"));
  handle.unmount();

  assert.equal(createTimeSlots({ step: 0 }).length, 48);
  assert.equal(createTimeSlots({ step: 5_000 }).length, 1);
  assert.deepEqual(
    createTimeSlots({ min: createPlainTime(8, 10, 30), max: createPlainTime(8, 40), step: 15 }).map(
      (time) => formatIsoTime(time),
    ),
    ["08:11", "08:26"],
  );
});
