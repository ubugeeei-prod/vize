import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainDate } from "../calendar/plain-date.ts";
import YearPicker from "./year-picker.vue";

const SsrProbe = defineComponent({
  name: "YearPickerSsrProbe",
  setup: () => () =>
    h(YearPicker, { today: createPlainDate(2026, 9, 25), locale: "en-US", defaultValue: 2020 }),
});

test("renders byte-identical year picker markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="year-picker"/);
  assert.match(left, /2016 – 2027/);
  assert.match(left, /data-year="2020"[^>]*data-selected="true"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates year picker markup without mismatches", async () => {
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
