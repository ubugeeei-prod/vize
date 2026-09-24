import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent } from "vue";
import { renderToString } from "vue/server-renderer";

import { renderFixtureChart } from "./chart-render-fixture.ts";

const Probe = defineComponent({ name: "ChartSsrProbe", setup: () => renderFixtureChart });

test("renders byte-identical chart markup across requests and host time zones", async () => {
  const originalTimeZone = process.env.TZ;
  try {
    process.env.TZ = "UTC";
    const first = await renderToString(createSSRApp(Probe));
    process.env.TZ = "Pacific/Auckland";
    const second = await renderToString(createSSRApp(Probe));
    assert.equal(first, second);
    assert.match(first, /^<figure/);
    assert.match(first, /role="group"/);
    assert.match(first, /aria-roledescription="chart"/);
    assert.match(first, /<title id="readings-title"[^>]*>Monthly readings<\/title>/);
    assert.match(first, /tabindex="0"[^>]*data-vize-ui="chart-point"/);
    assert.match(first, /aria-live="polite"/);
    assert.match(first, /<caption[^>]*>Monthly readings/);
    assert.match(first, />Feb</);
    assert.match(first, /data-state="closed"/, "tooltips render closed on the server");
  } finally {
    if (originalTimeZone === undefined) delete process.env.TZ;
    else process.env.TZ = originalTimeZone;
  }
});

test("hydrates the full chart without mismatch warnings", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(host.querySelectorAll('[data-vize-ui="chart-point"]').length, 3);
    assert.equal(host.querySelectorAll('[data-vize-ui="chart-legend-item"]').length, 4);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
