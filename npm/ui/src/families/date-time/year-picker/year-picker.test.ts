import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createPlainDate } from "../calendar/plain-date.ts";
import YearPicker from "./year-picker.vue";
import type { YearPickerExpose } from "./year-picker-types.ts";

const today = createPlainDate(2026, 9, 25);

function year(root: Element, value: number): HTMLButtonElement {
  const element = root.querySelector(`[data-vize-ui='year-picker-year'][data-year='${value}']`);
  assert.ok(element instanceof HTMLButtonElement, `missing year ${value}`);
  return element;
}

async function key(target: Element, value: string, init: KeyboardEventInit = {}): Promise<void> {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true, ...init }),
  );
  await nextTick();
  await nextTick();
}

test("renders an aligned page of years with the current and selected year", () => {
  const handle = mountInteraction(YearPicker, {
    props: { today, locale: "en-US", defaultValue: 2020, name: "vintage" },
  });
  const root = handle.root();
  assert.equal(
    root.querySelector("[data-vize-ui='year-picker-heading']")?.textContent?.trim(),
    "2016 – 2027",
  );
  assert.equal(root.querySelectorAll("[data-vize-ui='year-picker-year']").length, 12);
  assert.equal(year(root, 2020).getAttribute("tabindex"), "0");
  assert.equal(year(root, 2020).closest("td")?.getAttribute("aria-selected"), "true");
  assert.equal(year(root, 2026).getAttribute("aria-current"), "date");
  assert.equal(root.querySelector<HTMLInputElement>("input")?.value, "2020");
  handle.unmount();
});

test("keyboard pages by row and page, selection emits, and bounds disable paging", async () => {
  const handle = mountInteraction(YearPicker, {
    props: { today, locale: "en-US", pageSize: 20, columns: 5, min: 1990, max: 2045 },
    record: ["update:modelValue", "change"],
  });
  const root = handle.root();
  assert.equal(root.getAttribute("data-first-year"), "2020");
  year(root, 2026).focus();
  await key(year(root, 2026), "ArrowDown");
  assert.equal(document.activeElement, year(root, 2031));
  await key(year(root, 2031), "PageDown");
  assert.equal(root.getAttribute("data-first-year"), "2040");
  assert.equal(document.activeElement, year(root, 2045));
  assert.equal(year(root, 2046).disabled, true);
  assert.equal(
    root.querySelector<HTMLButtonElement>("[data-vize-ui='year-picker-next']")?.disabled,
    true,
  );
  await handle.click(year(root, 2044));
  assert.equal(handle.recorded()[0]?.payload[0], 2044);
  await key(year(root, 2044), "PageUp", { shiftKey: true });
  assert.equal(document.activeElement, year(root, 1990));
  handle.unmount();
});

test("exposes value and paging", async () => {
  const handle = mountInteraction(YearPicker, {
    props: { today, locale: "en-US", isYearUnavailable: (value: number) => value === 2025 },
  });
  const api = handle.exposes<YearPickerExpose>();
  assert.equal(api.firstYear, 2016);
  assert.equal(api.lastYear, 2027);
  assert.equal(year(handle.root(), 2025).getAttribute("data-unavailable"), "true");
  assert.equal(api.navigate(1), true);
  await nextTick();
  assert.equal(api.firstYear, 2028);
  assert.equal(api.setValue(1999), true);
  await nextTick();
  assert.equal(api.firstYear, 1992);
  assert.equal(api.state, "selected");
  handle.unmount();
});
