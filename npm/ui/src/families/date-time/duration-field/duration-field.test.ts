import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import {
  durationToSeconds,
  formatIsoDuration,
  isSameDuration,
  normalizeDuration,
  parseIsoDuration,
} from "./duration.ts";
import DurationField from "./duration-field.vue";
import { durationUnitBounds, normalizeDurationFields } from "./duration-field-runtime.ts";
import type { DurationFieldExpose } from "./duration-field-types.ts";

function segment(root: Element, unit: string): HTMLElement {
  const element = root.querySelector(`[role='spinbutton'][data-unit='${unit}']`);
  assert.ok(element instanceof HTMLElement, `missing ${unit}`);
  return element;
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

test("renders unit segments with localized affixes, bounds, and an ISO input", () => {
  const handle = mountInteraction(DurationField, {
    props: {
      locale: "en-US",
      name: "estimate",
      ariaLabel: "Estimate",
      fields: ["minutes", "hours", "days"],
      defaultValue: { days: 1, hours: 2, minutes: 30 },
    },
  });
  const root = handle.root();
  assert.deepEqual(
    [...root.querySelectorAll("[role='spinbutton']")].map((element) =>
      element.getAttribute("data-unit"),
    ),
    ["days", "hours", "minutes"],
  );
  const hours = segment(root, "hours");
  assert.equal(hours.textContent, "2");
  assert.equal(hours.getAttribute("aria-label"), "hour");
  assert.equal(hours.getAttribute("aria-valuemax"), "23");
  assert.equal(hours.getAttribute("aria-valuetext"), "2 hours");
  assert.equal(segment(root, "days").getAttribute("aria-valuemax"), "9999");
  assert.equal(
    [...root.querySelectorAll("[data-vize-ui='duration-field-affix']")]
      .map((element) => element.textContent)
      .join("|"),
    " day| hr| min",
  );
  assert.equal(root.querySelector<HTMLInputElement>("input")?.value, "P1DT2H30M");
  assert.equal(root.getAttribute("data-value"), "P1DT2H30M");
  handle.unmount();
});

test("typing and stepping fill units and commit normalized durations", async () => {
  const handle = mountInteraction(DurationField, {
    props: { locale: "en-US" },
    record: ["update:modelValue", "change"],
  });
  const root = handle.root();
  segment(root, "minutes").focus();
  await type("75");
  assert.equal(segment(root, "minutes").textContent, "5");
  assert.equal(root.getAttribute("data-state"), "partial");
  await key(segment(root, "hours"), "ArrowUp");
  assert.deepEqual(handle.recorded().at(-1)?.payload[0], { minutes: 5 });
  await key(segment(root, "hours"), "ArrowUp");
  assert.deepEqual(handle.recorded().at(-1)?.payload[0], { hours: 1, minutes: 5 });
  await key(segment(root, "minutes"), "ArrowDown");
  await key(segment(root, "minutes"), "PageDown");
  assert.deepEqual(handle.recorded().at(-1)?.payload[0], { hours: 1, minutes: 54 });
  await key(segment(root, "minutes"), "Backspace");
  assert.equal(segment(root, "minutes").textContent, "5");
  await key(segment(root, "minutes"), "Delete");
  assert.equal(handle.recorded().at(-2)?.payload[0], null);
  assert.equal(root.getAttribute("data-state"), "partial");
  segment(root, "hours").focus();
  await key(segment(root, "hours"), "ArrowRight");
  assert.equal(document.activeElement, segment(root, "minutes"));
  handle.unmount();
});

test("required durations join native validation and read-only blocks edits", async () => {
  const Form = defineComponent({
    setup: () => () =>
      h("form", [h(DurationField, { locale: "en-US", required: true, fields: ["seconds"] })]),
  });
  const handle = mountInteraction(Form);
  const form = handle.root() as HTMLFormElement;
  assert.equal(form.checkValidity(), false);
  await key(segment(form, "seconds"), "End");
  assert.equal(form.checkValidity(), true);
  handle.unmount();

  const readOnly = mountInteraction(DurationField, {
    props: { locale: "en-US", readOnly: true, defaultValue: { hours: 1 } },
    record: ["update:modelValue"],
  });
  await key(segment(readOnly.root(), "hours"), "ArrowUp");
  assert.equal(readOnly.recorded().length, 0);
  readOnly.unmount();
});

test("exposes focus, setValue, and clear", async () => {
  const handle = mountInteraction(DurationField, {
    props: { locale: "en-US", fields: ["weeks", "days"] },
  });
  const api = handle.exposes<DurationFieldExpose>();
  assert.equal(api.setValue({ weeks: 2, days: 3 }), true);
  await nextTick();
  assert.equal(segment(handle.root(), "days").getAttribute("aria-valuemax"), "6");
  assert.equal(api.focus("days"), true);
  assert.equal(document.activeElement, segment(handle.root(), "days"));
  assert.equal(api.clear(), true);
  assert.equal(api.state, "empty");
  handle.unmount();
});

test("ISO 8601 durations round-trip and convert to seconds", () => {
  assert.equal(
    formatIsoDuration({ years: 1, months: 2, days: 3, hours: 4, minutes: 5, seconds: 6 }),
    "P1Y2M3DT4H5M6S",
  );
  assert.equal(formatIsoDuration({ hours: 0 }), "PT0S");
  assert.equal(formatIsoDuration({ weeks: 2 }), "P2W");
  assert.deepEqual(parseIsoDuration("pt1h30m"), { hours: 1, minutes: 30 });
  assert.deepEqual(parseIsoDuration("P1Y2M3W4DT5H6M7S"), {
    years: 1,
    months: 2,
    weeks: 3,
    days: 4,
    hours: 5,
    minutes: 6,
    seconds: 7,
  });
  assert.equal(parseIsoDuration("P"), null);
  assert.equal(parseIsoDuration("PT"), null);
  assert.equal(parseIsoDuration("PT1.5H"), null);
  assert.equal(parseIsoDuration("-PT1H"), null);
  assert.equal(normalizeDuration({ hours: -1 }), null);
  assert.equal(isSameDuration({ hours: 1 }, { hours: 1, minutes: 0 }), true);
  assert.equal(durationToSeconds({ days: 1, minutes: 1 }), 86_460);
  assert.equal(durationToSeconds({ months: 1 }), null);
  assert.deepEqual(normalizeDurationFields(["seconds", "hours", "hours"]), ["hours", "seconds"]);
  assert.deepEqual(durationUnitBounds("seconds", ["hours", "seconds"], 9_999), { min: 0, max: 59 });
  assert.deepEqual(durationUnitBounds("months", ["months"], 99), { min: 0, max: 99 });
});
