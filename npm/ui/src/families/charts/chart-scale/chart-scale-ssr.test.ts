import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { scaleBand, scaleLinear, scaleTime } from "./chart-scale.ts";

const start = Date.UTC(2024, 2, 9, 18);
const stop = Date.UTC(2024, 2, 11, 6);

const AxisProbe = defineComponent({
  name: "ChartScaleSsrProbe",
  setup() {
    const time = scaleTime({
      domain: [start, stop],
      range: [0, 600],
      timeZone: "America/New_York",
    });
    const value = scaleLinear({ domain: [0, 0.83], range: [300, 0], nice: true });
    const band = scaleBand({ domain: ["q1", "q2", "q3"], range: [0, 300], paddingInner: 0.2 });
    const timeLabel = time.tickFormat({ locale: "en-US" });
    const valueLabel = value.tickFormat(5, { locale: "en-US" });
    return () =>
      h("svg", { viewBox: "0 0 600 300" }, [
        ...time
          .ticks(6)
          .map((tick) =>
            h("text", { x: time(tick), "data-time": tick.toISOString() }, timeLabel(tick)),
          ),
        ...value.ticks(5).map((tick) => h("text", { y: value(tick) }, valueLabel(tick))),
        ...band.domain.map((category) =>
          h("rect", { x: band(category), width: band.bandwidth, "data-category": category }),
        ),
      ]);
  },
});

async function render(): Promise<string> {
  return renderToString(createSSRApp(AxisProbe));
}

test("scale-driven markup is byte-identical across requests and host time zones", async () => {
  const originalTimeZone = process.env.TZ;
  try {
    process.env.TZ = "UTC";
    const utcHost = await render();
    process.env.TZ = "Asia/Kolkata";
    const indiaHost = await render();
    process.env.TZ = "America/Los_Angeles";
    const pacificHost = await render();
    assert.equal(utcHost, indiaHost);
    assert.equal(utcHost, pacificHost);
    assert.match(utcHost, /data-time="2024-03-10T05:00:00.000Z"[^>]*>Mar 10</);
    assert.match(utcHost, />0.8</);
    assert.match(
      utcHost,
      /data-time="2024-03-10T10:00:00.000Z"[^>]*>6 AM</,
      "ticks follow the DST jump",
    );
    assert.match(utcHost, /data-category="q2"/);
  } finally {
    if (originalTimeZone === undefined) delete process.env.TZ;
    else process.env.TZ = originalTimeZone;
  }
});

test("hydrates scale-driven markup without mismatch warnings", async () => {
  const html = await render();
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(AxisProbe);
  try {
    app.mount(host);
    assert.equal(host.querySelectorAll("text").length > 6, true);
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
