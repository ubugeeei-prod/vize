import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainDateTime } from "./plain-date-time.ts";
import DateTimeField from "./datetime-field.vue";

const SsrProbe = defineComponent({
  name: "DateTimeFieldSsrProbe",
  setup: () => () =>
    h(DateTimeField, {
      locale: "en-US",
      name: "alarm",
      ariaLabel: "Alarm",
      granularity: "second",
      defaultValue: createPlainDateTime(2026, 9, 25, 21, 5, 9),
    }),
});

test("renders byte-identical locale-ordered date-time segments across SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="datetime-field"/);
  assert.match(left, /role="group"/);
  assert.match(
    left,
    /data-segment="year"[\s\S]*data-segment="hour"[\s\S]*data-segment="minute"[\s\S]*data-segment="second"[\s\S]*data-segment="dayPeriod"/,
  );
  assert.match(left, /role="spinbutton"/);
  assert.match(left, /contenteditable="true"/);
  assert.match(left, /aria-valuetext="PM"/);
  assert.match(left, /data-hour-cycle="12"/);
  assert.match(left, /type="text" name="alarm" value="2026-09-25T21:05:09"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates date-time segments with generated ids and no mismatches", async () => {
  const Probe = defineComponent({
    name: "DateTimeFieldHydrationProbe",
    setup: () => () => h(DateTimeField, { locale: "en-GB", ariaLabel: "Start" }),
  });
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const ids = [...host.querySelectorAll("[role='spinbutton']")].map((element) => element.id);
  assert.match(ids[0] ?? "", /^vize-v-\d+-datetime-field-day$/u);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.deepEqual(
      [...host.querySelectorAll("[role='spinbutton']")].map((element) => element.id),
      ids,
    );
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
