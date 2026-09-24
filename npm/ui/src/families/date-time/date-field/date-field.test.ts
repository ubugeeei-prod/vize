import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createPlainDate, formatIsoDate } from "../calendar/plain-date.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import DateField from "./date-field.vue";
import type { DateFieldExpose } from "./date-field-types.ts";
import {
  resolveDateSegmentLayout,
  segmentBounds,
  stepSegmentValue,
  typeSegmentDigit,
  emptySegmentValues,
} from "./field-segments.ts";

function segment(root: Element, type: string): HTMLElement {
  const element = root.querySelector(`[role='spinbutton'][data-segment='${type}']`);
  assert.ok(element instanceof HTMLElement, `missing ${type} segment`);
  return element;
}

async function key(target: Element, value: string, init: KeyboardEventInit = {}): Promise<boolean> {
  const event = new KeyboardEvent("keydown", {
    key: value,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  await nextTick();
  return event.defaultPrevented;
}

async function type(target: Element, text: string): Promise<void> {
  for (const character of text) {
    await key(document.activeElement ?? target, character);
  }
}

function iso(value: unknown): string {
  return value === null ? "null" : formatIsoDate(value as PlainDate);
}

test("renders locale-ordered spinbutton segments with literals and a hidden ISO input", () => {
  const handle = mountInteraction(DateField, {
    props: {
      id: "birthday",
      name: "birthday",
      locale: "en-US",
      defaultValue: createPlainDate(2026, 9, 5),
      ariaLabel: "Birthday",
      ariaDescribedby: "birthday-help",
      required: true,
    },
  });
  const root = handle.root();
  const segments = [...root.children].filter((element) => element.tagName === "SPAN");
  assert.deepEqual(
    segments.map((element) => element.getAttribute("data-segment") ?? element.textContent),
    ["month", "/", "day", "/", "year"],
  );
  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("aria-label"), "Birthday");
  assert.equal(root.getAttribute("data-state"), "complete");
  assert.equal(root.getAttribute("data-value"), "2026-09-05");
  const month = segment(root, "month");
  assert.equal(month.id, "birthday-month");
  assert.equal(month.textContent, "09");
  assert.equal(month.getAttribute("tabindex"), "0");
  assert.equal(month.getAttribute("contenteditable"), "true");
  assert.equal(month.getAttribute("inputmode"), "numeric");
  assert.equal(month.getAttribute("aria-label"), "month");
  assert.equal(month.getAttribute("aria-valuenow"), "9");
  assert.equal(month.getAttribute("aria-valuemin"), "1");
  assert.equal(month.getAttribute("aria-valuemax"), "12");
  assert.equal(month.getAttribute("aria-valuetext"), "9 – September");
  assert.equal(month.getAttribute("aria-describedby"), "birthday-help");
  assert.equal(month.getAttribute("aria-required"), "true");
  assert.equal(segment(root, "day").getAttribute("aria-valuemax"), "30");
  assert.equal(segment(root, "year").textContent, "2026");
  assert.equal(
    root.querySelector("[data-vize-ui='date-field-literal']")?.getAttribute("aria-hidden"),
    "true",
  );
  const input = root.querySelector<HTMLInputElement>("input[type='hidden']");
  assert.equal(input?.name, "birthday");
  assert.equal(input?.value, "2026-09-05");
  handle.unmount();

  const german = mountInteraction(DateField, { props: { locale: "de-DE" } });
  const order = [...german.root().querySelectorAll("[data-segment]")].map((element) =>
    element.getAttribute("data-segment"),
  );
  assert.deepEqual(order, ["day", "month", "year"]);
  assert.equal(
    german.root().querySelector("[data-vize-ui='date-field-literal']")?.textContent,
    ".",
  );
  assert.equal(segment(german.root(), "day").getAttribute("aria-label"), "Tag");
  assert.equal(segment(german.root(), "year").textContent, "yyyy");
  assert.equal(segment(german.root(), "year").getAttribute("aria-valuetext"), "Empty");
  assert.equal(german.root().getAttribute("data-state"), "empty");
  german.unmount();
});

test("typing digits fills segments, auto-advances, and commits a complete date", async () => {
  const handle = mountInteraction(DateField, {
    props: { locale: "en-US" },
    record: ["update:modelValue", "change"],
  });
  const root = handle.root();
  segment(root, "month").focus();
  await type(root, "9");
  assert.equal(document.activeElement, segment(root, "day"));
  assert.equal(segment(root, "month").textContent, "09");
  await type(root, "1");
  assert.equal(document.activeElement, segment(root, "day"));
  assert.equal(segment(root, "day").textContent, "01");
  await type(root, "5");
  assert.equal(document.activeElement, segment(root, "year"));
  assert.equal(root.getAttribute("data-state"), "partial");
  await type(root, "2026");
  assert.equal(root.getAttribute("data-state"), "complete");
  const updates = handle
    .recorded()
    .filter((entry) => entry.event === "update:modelValue")
    .map((entry) => iso(entry.payload[0]));
  assert.deepEqual(updates, ["0002-09-15", "0020-09-15", "0202-09-15", "2026-09-15"]);
  const change = handle.recorded().at(-1);
  assert.equal(change?.event, "change");
  assert.equal(iso(change?.payload[1]), "0202-09-15");
  handle.unmount();
});

test("a leading zero waits for a second digit and out-of-range digits restart the buffer", async () => {
  const handle = mountInteraction(DateField, { props: { locale: "en-US" } });
  const root = handle.root();
  segment(root, "month").focus();
  await type(root, "0");
  assert.equal(document.activeElement, segment(root, "month"));
  assert.equal(segment(root, "month").textContent, "0");
  await type(root, "2");
  assert.equal(document.activeElement, segment(root, "day"));
  await type(root, "3");
  assert.equal(segment(root, "day").textContent, "03");
  segment(root, "day").focus();
  await type(root, "1");
  await type(root, "9");
  assert.equal(segment(root, "day").textContent, "19");
  handle.unmount();
});

test("arrow, page, home, and end keys step segments with wrapping and day clamping", async () => {
  const handle = mountInteraction(DateField, {
    props: { locale: "en-US", defaultValue: createPlainDate(2024, 1, 31) },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  const month = segment(root, "month");
  month.focus();
  assert.equal(await key(month, "ArrowUp"), true);
  assert.equal(month.textContent, "02");
  assert.equal(segment(root, "day").textContent, "29");
  await key(month, "ArrowDown");
  await key(month, "ArrowDown");
  assert.equal(month.textContent, "12");
  await key(month, "PageUp");
  assert.equal(month.textContent, "03");
  await key(month, "Home");
  assert.equal(month.textContent, "01");
  await key(month, "End");
  assert.equal(month.textContent, "12");
  const year = segment(root, "year");
  await key(year, "PageDown");
  assert.equal(year.textContent, "2014");
  assert.equal(iso(handle.recorded().at(-1)?.payload[0]), "2014-12-29");
  handle.unmount();
});

test("stepping an empty segment starts from placeholderValue", async () => {
  const handle = mountInteraction(DateField, {
    props: { locale: "en-US", placeholderValue: createPlainDate(2030, 6, 15) },
  });
  const root = handle.root();
  await key(segment(root, "year"), "ArrowUp");
  await key(segment(root, "month"), "ArrowDown");
  await key(segment(root, "day"), "ArrowUp");
  assert.equal(segment(root, "year").textContent, "2030");
  assert.equal(segment(root, "month").textContent, "06");
  assert.equal(segment(root, "day").textContent, "15");
  handle.unmount();
});

test("Backspace removes digits then moves back, Delete clears, and clearing commits null", async () => {
  const handle = mountInteraction(DateField, {
    props: { locale: "en-US", defaultValue: createPlainDate(2026, 12, 25) },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  const day = segment(root, "day");
  day.focus();
  await key(day, "Backspace");
  assert.equal(day.textContent, "02");
  await key(day, "Backspace");
  assert.equal(day.textContent, "dd");
  assert.equal(root.getAttribute("data-state"), "partial");
  assert.equal(iso(handle.recorded().at(-1)?.payload[0]), "null");
  await key(day, "Backspace");
  assert.equal(document.activeElement, segment(root, "month"));
  await key(segment(root, "month"), "Delete");
  assert.equal(segment(root, "month").textContent, "mm");
  handle.unmount();
});

test("horizontal arrows move between segments and honor RTL", async () => {
  const handle = mountInteraction(DateField, { props: { locale: "en-US" } });
  const root = handle.root();
  segment(root, "month").focus();
  await key(segment(root, "month"), "ArrowRight");
  assert.equal(document.activeElement, segment(root, "day"));
  await key(segment(root, "day"), "ArrowLeft");
  assert.equal(document.activeElement, segment(root, "month"));
  assert.equal(await key(segment(root, "month"), "x"), true);
  handle.unmount();

  const rtl = mountInteraction(DateField, { props: { locale: "en-US", dir: "rtl" } });
  segment(rtl.root(), "month").focus();
  await key(segment(rtl.root(), "month"), "ArrowLeft");
  assert.equal(document.activeElement, segment(rtl.root(), "day"));
  assert.equal(rtl.root().getAttribute("dir"), "rtl");
  rtl.unmount();
});

test("beforeinput supports virtual keyboards without mutating the DOM", async () => {
  const handle = mountInteraction(DateField, { props: { locale: "en-US" } });
  const root = handle.root();
  const month = segment(root, "month");
  month.focus();
  const insert = new InputEvent("beforeinput", {
    inputType: "insertText",
    data: "11",
    bubbles: true,
    cancelable: true,
  });
  month.dispatchEvent(insert);
  await nextTick();
  assert.equal(insert.defaultPrevented, true);
  assert.equal(month.textContent, "11");
  assert.equal(document.activeElement, segment(root, "day"));
  segment(root, "day").dispatchEvent(
    new InputEvent("beforeinput", {
      inputType: "deleteContentBackward",
      bubbles: true,
      cancelable: true,
    }),
  );
  await nextTick();
  assert.equal(document.activeElement, month);
  handle.unmount();
});

test("min, max, unavailable dates, and ariaInvalid mark the field invalid", async () => {
  const handle = mountInteraction(DateField, {
    props: {
      locale: "en-US",
      defaultValue: createPlainDate(2026, 1, 1),
      min: createPlainDate(2026, 1, 5),
      max: createPlainDate(2026, 12, 31),
      isDateUnavailable: (date: PlainDate) => date.day === 13,
      ariaErrormessage: "date-error",
    },
  });
  const root = handle.root();
  assert.equal(root.getAttribute("data-state"), "invalid");
  assert.equal(segment(root, "day").getAttribute("aria-invalid"), "true");
  assert.equal(segment(root, "day").getAttribute("aria-errormessage"), "date-error");
  await key(segment(root, "day"), "End");
  assert.equal(root.getAttribute("data-state"), "complete");
  assert.equal(segment(root, "day").getAttribute("aria-invalid"), null);
  segment(root, "day").focus();
  await type(root, "13");
  assert.equal(root.getAttribute("data-state"), "invalid");
  await handle.wrapper.setProps({ isDateUnavailable: undefined, ariaInvalid: true });
  assert.equal(root.getAttribute("data-invalid"), "true");
  handle.unmount();
});

test("disabled and read-only fields keep availability and form semantics", async () => {
  const disabled = mountInteraction(DateField, {
    props: {
      locale: "en-US",
      disabled: true,
      name: "d",
      defaultValue: createPlainDate(2026, 1, 1),
    },
  });
  const month = segment(disabled.root(), "month");
  assert.equal(month.getAttribute("tabindex"), "0");
  assert.equal(month.getAttribute("contenteditable"), null);
  assert.equal(month.getAttribute("aria-disabled"), "true");
  assert.equal(disabled.root().querySelector<HTMLInputElement>("input")?.disabled, true);
  disabled.unmount();

  const readOnly = mountInteraction(DateField, {
    props: { locale: "en-US", readOnly: true, defaultValue: createPlainDate(2026, 1, 1) },
    record: ["update:modelValue"],
  });
  const readOnlyMonth = segment(readOnly.root(), "month");
  readOnlyMonth.focus();
  await key(readOnlyMonth, "ArrowUp");
  await key(readOnlyMonth, "5");
  assert.equal(readOnlyMonth.textContent, "01");
  assert.equal(readOnlyMonth.getAttribute("aria-readonly"), "true");
  await key(readOnlyMonth, "ArrowRight");
  assert.equal(document.activeElement, segment(readOnly.root(), "day"));
  assert.equal(readOnly.recorded().length, 0);
  readOnly.unmount();
});

test("controlled values win, external values replace segments, and form reset restores defaults", async () => {
  const handle = mountInteraction(DateField, {
    props: { locale: "en-US", modelValue: createPlainDate(2026, 3, 3) },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  await key(segment(root, "day"), "ArrowUp");
  await nextTick();
  assert.equal(iso(handle.recorded()[0]?.payload[0]), "2026-03-04");
  assert.equal(segment(root, "day").textContent, "03");
  await handle.wrapper.setProps({ modelValue: createPlainDate(2027, 7, 7) });
  assert.equal(segment(root, "year").textContent, "2027");
  await handle.wrapper.setProps({ modelValue: null });
  assert.equal(segment(root, "year").textContent, "yyyy");
  handle.unmount();

  const Form = defineComponent({
    setup: () => () =>
      h("form", [
        h(DateField, { locale: "en-US", name: "d", defaultValue: createPlainDate(2026, 5, 5) }),
      ]),
  });
  const form = mountInteraction(Form);
  const field = form.root().querySelector("[data-vize-ui='date-field']") as HTMLElement;
  await key(segment(field, "month"), "ArrowUp");
  assert.equal(field.querySelector<HTMLInputElement>("input")?.value, "2026-06-05");
  (form.root() as HTMLFormElement).reset();
  await nextTick();
  assert.equal(field.querySelector<HTMLInputElement>("input")?.value, "2026-05-05");
  assert.equal(segment(field, "month").textContent, "05");
  form.unmount();
});

test("exposes focus, setValue, clear, reset, and normalized state", async () => {
  const handle = mountInteraction(DateField, {
    props: {
      locale: "en-US",
      defaultValue: createPlainDate(2026, 2, 2),
      placeholders: { year: "YYYY" },
    },
  });
  const api = handle.exposes<DateFieldExpose>();
  assert.equal(api.state, "complete");
  assert.equal(api.clear(), true);
  await nextTick();
  assert.equal(segment(handle.root(), "year").textContent, "YYYY");
  assert.equal(api.focus(), true);
  assert.equal(document.activeElement, segment(handle.root(), "month"));
  assert.equal(api.focus("year"), true);
  assert.equal(document.activeElement, segment(handle.root(), "year"));
  assert.equal(api.setValue(createPlainDate(2031, 8, 9)), true);
  await nextTick();
  assert.equal(iso(api.value), "2031-08-09");
  assert.equal(api.reset(), true);
  await nextTick();
  assert.equal(iso(api.value), "2026-02-02");
  assert.equal(api.segments.length, 5);
  assert.equal(api.locale, "en-US");
  handle.unmount();
});

test("segment helpers expose pure bounds, stepping, typing, and layouts", () => {
  assert.deepEqual(segmentBounds("day", { ...emptySegmentValues, month: 2, year: 2023 }), {
    min: 1,
    max: 28,
  });
  assert.deepEqual(segmentBounds("hour", emptySegmentValues, "h12"), { min: 1, max: 12 });
  assert.equal(stepSegmentValue(12, 1, { min: 1, max: 12 }, 1), 1);
  assert.equal(stepSegmentValue(null, -1, { min: 1, max: 12 }, 20), 12);
  assert.deepEqual(typeSegmentDigit("1", "3", { min: 1, max: 12 }, 2), {
    value: 3,
    buffer: "3",
    advance: true,
  });
  assert.deepEqual(typeSegmentDigit("", "1", { min: 1, max: 12 }, 2), {
    value: 1,
    buffer: "1",
    advance: false,
  });
  assert.deepEqual(
    resolveDateSegmentLayout("ja-JP").map((part) => part.type),
    ["year", "literal", "month", "literal", "day"],
  );
});
