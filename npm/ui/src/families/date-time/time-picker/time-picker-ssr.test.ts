import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainTime } from "../time-field/plain-time.ts";
import TimePicker from "./time-picker.vue";

const SsrProbe = defineComponent({
  name: "TimePickerSsrProbe",
  setup: () => () =>
    h(TimePicker, {
      locale: "en-US",
      ariaLabel: "Start",
      step: 60,
      defaultValue: createPlainTime(9, 0),
      name: "start",
    }),
});

test("renders byte-identical time picker markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="time-picker"/);
  assert.match(left, /role="listbox"/);
  assert.match(left, /9:00 AM/);
  assert.match(left, /value="09:00"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates time picker markup without mismatches", async () => {
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
