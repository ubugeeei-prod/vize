import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import PopoverTrigger from "../../overlays/popover/popover-trigger.vue";
import { createPlainDate } from "../calendar/plain-date.ts";
import DatePickerCalendar from "./date-picker-calendar.vue";
import DatePickerContent from "./date-picker-content.vue";
import DatePickerField from "./date-picker-field.vue";
import DatePickerRoot from "./date-picker-root.vue";

function probe(defaultOpen: boolean): Component {
  return defineComponent({
    name: "DatePickerSsrProbe",
    setup: () => () =>
      h(
        DatePickerRoot,
        {
          today: createPlainDate(2026, 9, 25),
          locale: "en-US",
          name: "departure",
          defaultValue: createPlainDate(2026, 9, 12),
          defaultOpen,
        },
        () => [
          h(DatePickerField, { ariaLabel: "Departure" }, () =>
            h(PopoverTrigger, { ariaLabel: "Choose date" }, () => "Pick"),
          ),
          h(DatePickerContent, { ariaLabel: "Calendar" }, () => h(DatePickerCalendar)),
        ],
      ),
  });
}

test("renders byte-identical closed and open picker markup across SSR requests", async () => {
  for (const open of [false, true]) {
    const [left, right] = await Promise.all([
      renderToString(createSSRApp(probe(open))),
      renderToString(createSSRApp(probe(open))),
    ]);
    assert.equal(left, right);
    assert.match(left, /data-vize-ui="date-picker"/);
    assert.match(left, /data-vize-ui="date-field"/);
    assert.match(left, /type="hidden" name="departure" value="2026-09-12"/);
    assert.match(left, new RegExp(`aria-expanded="${String(open)}"`, "u"));
    if (open) {
      assert.match(left, /role="dialog"/);
      assert.match(left, /data-date="2026-09-12"[^>]*data-state="selected"/);
    } else {
      assert.doesNotMatch(left, /role="grid"/);
    }
    assert.doesNotMatch(left, /function|NaN/);
  }
});

test("hydrates an open picker without mismatches", async () => {
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
      host.querySelector("[data-vize-ui='date-picker']")?.getAttribute("data-state"),
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
