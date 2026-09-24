import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import CalendarMultipleRoot from "./calendar-multiple-root.vue";
import { createPlainDate } from "./plain-date.ts";

const SsrProbe = defineComponent({
  name: "CalendarMultipleSsrProbe",
  setup: () => () =>
    h(CalendarMultipleRoot, {
      today: createPlainDate(2026, 9, 25),
      locale: "en-US",
      name: "days",
      defaultValue: [createPlainDate(2026, 9, 3), createPlainDate(2026, 9, 9)],
    }),
});

test("renders byte-identical multiple-date calendar markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-mode="multiple"/);
  assert.match(left, /aria-multiselectable="true"/);
  assert.match(left, /name="days" value="2026-09-03"/);
  assert.match(left, /name="days" value="2026-09-09"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates multiple-date calendar markup without mismatches", async () => {
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
