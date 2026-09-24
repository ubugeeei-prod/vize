import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainDate } from "../calendar/plain-date.ts";
import { weekOf } from "./week-picker-selection.ts";
import WeekPickerRoot from "./week-picker-root.vue";

const SsrProbe = defineComponent({
  name: "WeekPickerSsrProbe",
  setup: () => () =>
    h(WeekPickerRoot, {
      today: createPlainDate(2026, 9, 25),
      locale: "en-GB",
      name: "week",
      defaultValue: weekOf(createPlainDate(2026, 9, 25), 1),
    }),
});

test("renders byte-identical week picker markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="week-picker"/);
  assert.match(left, /data-week="2026-W39"/);
  assert.match(left, /scope="row"/);
  assert.match(left, /value="2026-W39"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates week picker markup without mismatches", async () => {
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
