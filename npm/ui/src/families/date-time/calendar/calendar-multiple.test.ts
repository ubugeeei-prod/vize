import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import CalendarMultipleRoot from "./calendar-multiple-root.vue";
import { normalizeDateList } from "./calendar-selection.ts";
import type { CalendarMultipleRootExpose } from "./calendar-types.ts";
import { createPlainDate, formatIsoDate } from "./plain-date.ts";
import type { PlainDate } from "./plain-date.ts";

const today = createPlainDate(2026, 9, 25);

function day(root: Element, iso: string): HTMLButtonElement {
  const element = root.querySelector(
    `[data-vize-ui="calendar-day"][data-date="${iso}"]:not([data-outside-month])`,
  );
  assert.ok(element instanceof HTMLButtonElement, `missing day ${iso}`);
  return element;
}

function isoList(value: unknown): string {
  return (value as readonly PlainDate[]).map((date) => formatIsoDate(date)).join(",");
}

test("multiple mode toggles dates, keeps them sorted, and submits one input per date", async () => {
  const Form = defineComponent({
    setup: () => () =>
      h("form", [h(CalendarMultipleRoot, { today, locale: "en-US", name: "days" })]),
  });
  const handle = mountInteraction(Form);
  const form = handle.root() as HTMLFormElement;
  const root = form.querySelector("[data-vize-ui='calendar']") as HTMLElement;
  assert.equal(root.getAttribute("data-mode"), "multiple");
  assert.equal(root.querySelector("table")?.getAttribute("aria-multiselectable"), "true");
  await handle.click(day(root, "2026-09-20"));
  await handle.click(day(root, "2026-09-02"));
  await handle.click(day(root, "2026-09-11"));
  assert.deepEqual(new FormData(form).getAll("days"), ["2026-09-02", "2026-09-11", "2026-09-20"]);
  assert.equal(root.getAttribute("data-count"), "3");
  assert.equal(day(root, "2026-09-11").closest("td")?.getAttribute("aria-selected"), "true");
  await handle.click(day(root, "2026-09-11"));
  assert.deepEqual(new FormData(form).getAll("days"), ["2026-09-02", "2026-09-20"]);
  assert.equal(day(root, "2026-09-11").getAttribute("data-selected"), null);
  handle.unmount();
});

test("multiple mode emits update, change, and toggle and honors maxSelections", async () => {
  const handle = mountInteraction(CalendarMultipleRoot, {
    props: {
      today,
      locale: "en-US",
      maxSelections: 2,
      defaultValue: [createPlainDate(2026, 9, 3)],
      isDateUnavailable: (date: PlainDate) => date.day === 13,
    },
    record: ["update:modelValue", "change", "toggle"],
  });
  const root = handle.root();
  await handle.click(day(root, "2026-09-05"));
  assert.deepEqual(
    handle
      .recorded()
      .map((entry) => [
        entry.event,
        entry.event === "toggle" ? entry.payload[1] : isoList(entry.payload[0]),
      ]),
    [
      ["update:modelValue", "2026-09-03,2026-09-05"],
      ["change", "2026-09-03,2026-09-05"],
      ["toggle", true],
    ],
  );
  await handle.click(day(root, "2026-09-07"));
  await handle.click(day(root, "2026-09-13"));
  assert.equal(handle.recorded().length, 3);
  await handle.click(day(root, "2026-09-03"));
  assert.equal(isoList(handle.recorded().at(-2)?.payload[0]), "2026-09-05");
  assert.equal(handle.recorded().at(-1)?.payload[1], false);
  handle.unmount();
});

test("controlled multiple selection, keyboard toggling, and the exposed API", async () => {
  const handle = mountInteraction(CalendarMultipleRoot, {
    props: {
      today,
      locale: "en-US",
      modelValue: [createPlainDate(2026, 9, 25), createPlainDate(2026, 9, 1)],
    },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  assert.equal(day(root, "2026-09-01").getAttribute("data-state"), "selected");
  day(root, "2026-09-25").focus();
  await handle.press(day(root, "2026-09-25"), "Enter");
  assert.equal(isoList(handle.recorded()[0]?.payload[0]), "2026-09-01");
  assert.equal(day(root, "2026-09-25").getAttribute("data-selected"), "true");
  const api = handle.exposes<CalendarMultipleRootExpose>();
  assert.equal(api.value.length, 2);
  assert.equal(api.toggle(createPlainDate(2026, 12, 24)), true);
  assert.equal(isoList(handle.recorded().at(-1)?.payload[0]), "2026-09-01,2026-09-25,2026-12-24");
  await handle.wrapper.setProps({ modelValue: [createPlainDate(2026, 12, 24)] });
  await nextTick();
  assert.equal(handle.getByRole("grid").getAttribute("aria-label"), "December 2026");
  assert.deepEqual(
    normalizeDateList([
      createPlainDate(2026, 1, 2),
      createPlainDate(2026, 1, 1),
      createPlainDate(2026, 1, 2),
    ]).map((date) => formatIsoDate(date)),
    ["2026-01-01", "2026-01-02"],
  );
  handle.unmount();
});
