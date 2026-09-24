import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainDate } from "../calendar/plain-date.ts";
import DateField from "./date-field.vue";

const SsrProbe = defineComponent({
  name: "DateFieldSsrProbe",
  setup: () => () =>
    h(DateField, {
      locale: "de-DE",
      name: "birthday",
      ariaLabel: "Geburtstag",
      defaultValue: createPlainDate(2026, 9, 5),
    }),
});

test("renders byte-identical locale-ordered date segments across SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="date-field"/);
  assert.match(left, /role="group"/);
  assert.match(left, /data-segment="day"[\s\S]*data-segment="month"[\s\S]*data-segment="year"/);
  assert.match(left, /role="spinbutton"/);
  assert.match(left, /contenteditable="true"/);
  assert.match(left, /aria-valuetext="9 – September"/);
  assert.match(left, /type="hidden" name="birthday" value="2026-09-05"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates date segments with generated ids and no mismatches", async () => {
  const Probe = defineComponent({
    name: "DateFieldHydrationProbe",
    setup: () => () => h(DateField, { locale: "en-US", ariaLabel: "Start" }),
  });
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const ids = [...host.querySelectorAll("[role='spinbutton']")].map((element) => element.id);
  assert.match(ids[0] ?? "", /^vize-v-\d+-date-field-month$/u);
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
