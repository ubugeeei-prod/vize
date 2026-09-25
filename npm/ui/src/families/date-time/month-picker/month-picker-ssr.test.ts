import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainDate } from "../calendar/plain-date.ts";
import MonthPicker from "./month-picker.vue";

const SsrProbe = defineComponent({
  name: "MonthPickerSsrProbe",
  setup: () => () =>
    h(MonthPicker, {
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      name: "month",
      defaultValue: { year: 2026, month: 4 },
    }),
});

test("renders byte-identical month picker markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="month-picker"/);
  assert.match(left, /role="grid"/);
  assert.match(left, /aria-label="April 2026"/);
  assert.match(left, /aria-current="date"/);
  assert.match(left, /value="2026-04"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates month picker markup without mismatches", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  try {
    app.mount(host);
    await nextTick();
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
