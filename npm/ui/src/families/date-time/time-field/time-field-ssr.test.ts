import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainTime } from "./plain-time.ts";
import TimeField from "./time-field.vue";

const SsrProbe = defineComponent({
  name: "TimeFieldSsrProbe",
  setup: () => () =>
    h(TimeField, {
      locale: "en-US",
      name: "alarm",
      ariaLabel: "Alarm",
      granularity: "second",
      defaultValue: createPlainTime(21, 5, 9),
    }),
});

test("renders byte-identical locale-ordered time segments across SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="time-field"/);
  assert.match(left, /role="group"/);
  assert.match(
    left,
    /data-segment="hour"[\s\S]*data-segment="minute"[\s\S]*data-segment="second"[\s\S]*data-segment="dayPeriod"/,
  );
  assert.match(left, /role="spinbutton"/);
  assert.match(left, /contenteditable="true"/);
  assert.match(left, /aria-valuetext="PM"/);
  assert.match(left, /data-hour-cycle="12"/);
  assert.match(left, /type="hidden" name="alarm" value="21:05:09"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates time segments with generated ids and no mismatches", async () => {
  const Probe = defineComponent({
    name: "TimeFieldHydrationProbe",
    setup: () => () => h(TimeField, { locale: "en-GB", ariaLabel: "Start" }),
  });
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const ids = [...host.querySelectorAll("[role='spinbutton']")].map((element) => element.id);
  assert.match(ids[0] ?? "", /^vize-v-\d+-time-field-hour$/u);
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
