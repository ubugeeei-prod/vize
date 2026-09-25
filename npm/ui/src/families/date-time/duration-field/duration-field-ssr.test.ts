import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import DurationField from "./duration-field.vue";

const SsrProbe = defineComponent({
  name: "DurationFieldSsrProbe",
  setup: () => () =>
    h(DurationField, {
      locale: "en-US",
      name: "estimate",
      ariaLabel: "Estimate",
      defaultValue: { hours: 1, minutes: 30 },
    }),
});

test("renders byte-identical duration field markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="duration-field"/);
  assert.match(left, /role="spinbutton"/);
  assert.match(left, /aria-valuetext="30 minutes"/);
  assert.match(left, /value="PT1H30M"/);
  assert.doesNotMatch(left, /function|NaN/);
});

test("hydrates duration field markup without mismatches", async () => {
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
