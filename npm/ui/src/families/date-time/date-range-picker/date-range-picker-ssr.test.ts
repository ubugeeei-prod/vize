import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import PopoverTrigger from "../../overlays/popover/popover-trigger.vue";
import { createPlainDate } from "../calendar/plain-date.ts";
import { createDateRange } from "../calendar/plain-date.ts";
import DateRangePickerCalendar from "./date-range-picker-calendar.vue";
import DateRangePickerContent from "./date-range-picker-content.vue";
import DateRangePickerField from "./date-range-picker-field.vue";
import DateRangePickerRoot from "./date-range-picker-root.vue";

function probe(defaultOpen: boolean): Component {
  return defineComponent({
    name: "DateRangePickerSsrProbe",
    setup: () => () =>
      h(
        DateRangePickerRoot,
        {
          today: createPlainDate(2026, 9, 25),
          locale: "en-US",
          startName: "checkin",
          endName: "checkout",
          defaultValue: createDateRange(createPlainDate(2026, 9, 12), createPlainDate(2026, 9, 15)),
          defaultOpen,
        },
        () => [
          h(DateRangePickerField, { boundary: "start", ariaLabel: "Check-in" }),
          h(DateRangePickerField, { boundary: "end", ariaLabel: "Check-out" }, () =>
            h(PopoverTrigger, { ariaLabel: "Choose dates" }, () => "Pick"),
          ),
          h(DateRangePickerContent, { ariaLabel: "Calendar" }, () => h(DateRangePickerCalendar)),
        ],
      ),
  });
}

test("renders byte-identical closed and open range picker markup across SSR requests", async () => {
  for (const open of [false, true]) {
    const [left, right] = await Promise.all([
      renderToString(createSSRApp(probe(open))),
      renderToString(createSSRApp(probe(open))),
    ]);
    assert.equal(left, right);
    assert.match(left, /data-vize-ui="date-range-picker"/);
    assert.match(left, /data-vize-ui="date-field"/);
    assert.match(left, /name="checkin" value="2026-09-12"/);
    assert.match(left, /name="checkout" value="2026-09-15"/);
    assert.match(left, new RegExp(`aria-expanded="${String(open)}"`, "u"));
    if (open) {
      assert.match(left, /role="dialog"/);
      assert.match(left, /data-date="2026-09-13"[^>]*data-state="range-middle"/);
    } else {
      assert.doesNotMatch(left, /role="grid"/);
    }
    assert.doesNotMatch(left, /function|NaN/);
  }
});

test("hydrates an open range picker without mismatches", async () => {
  const Probe = probe(true);
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.deepEqual(diagnostics, []);
    assert.equal(
      host.querySelector("[data-vize-ui='date-range-picker']")?.getAttribute("data-state"),
      "open",
    );
  } finally {
    app.unmount();
    host.remove();
    document.body.replaceChildren();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
