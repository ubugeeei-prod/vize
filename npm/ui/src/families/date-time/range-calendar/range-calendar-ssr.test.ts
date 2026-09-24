import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createDateRange, createPlainDate } from "../calendar/plain-date.ts";
import RangeCalendarRoot from "./range-calendar-root.vue";

const SsrProbe = defineComponent({
  name: "RangeCalendarSsrProbe",
  setup: () => () =>
    h(RangeCalendarRoot, {
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      numberOfMonths: 2,
      startName: "from",
      endName: "to",
      defaultValue: createDateRange(createPlainDate(2026, 9, 28), createPlainDate(2026, 10, 2)),
    }),
});

test("renders byte-identical range markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-mode="range"/);
  assert.match(left, /aria-multiselectable="true"/);
  assert.match(left, /data-date="2026-09-30"[^>]*data-state="range-middle"/);
  assert.match(left, /data-range-start="true"/);
  assert.match(left, /data-range-end="true"/);
  assert.match(left, /name="from" value="2026-09-28"/);
  assert.match(left, /name="to" value="2026-10-02"/);
  assert.doesNotMatch(left, /data-anchor|data-preview|function/);
});

test("hydrates a range calendar without mismatches", async () => {
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
