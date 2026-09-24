import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { arcPath, areaPath, linePath, pieLayout, stackLayout } from "./chart-shape.ts";

interface Sample {
  readonly day: number;
  readonly visits: number;
  readonly signups: number;
}

const samples: readonly Sample[] = [
  { day: 0, visits: 12, signups: 3 },
  { day: 1, visits: 18, signups: 5 },
  { day: 2, visits: 9, signups: 2 },
  { day: 3, visits: 22, signups: 7 },
];

const ShapeProbe = defineComponent({
  name: "ChartShapeSsrProbe",
  setup() {
    const series = stackLayout(samples, {
      keys: ["visits", "signups"],
      value: (sample, key) => sample[key],
    });
    const slices = pieLayout(samples, { value: (sample) => sample.visits, padAngle: 0.02 });
    return () =>
      h("svg", { viewBox: "0 0 300 200" }, [
        h("path", {
          d: linePath(samples, {
            x: (s) => s.day * 100,
            y: (s) => 200 - s.visits * 5,
            curve: "monotoneX",
          }),
        }),
        ...series.map((entry) =>
          h("path", {
            "data-key": entry.key,
            d: areaPath(entry, {
              x: (p) => p.data.day * 100,
              y0: (p) => 200 - p[0] * 5,
              y1: (p) => 200 - p[1] * 5,
            }),
          }),
        ),
        ...slices.map((slice) =>
          h("path", {
            d: arcPath({ ...slice, innerRadius: 30, outerRadius: 60, cornerRadius: 4 }),
          }),
        ),
      ]);
  },
});

test("path generators render byte-identical SSR markup", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(ShapeProbe)),
    renderToString(createSSRApp(ShapeProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /<path d="M0,140C/);
  assert.match(html, /data-key="signups"/);
  assert.equal((html.match(/<path/g) ?? []).length, 7);
});

test("hydrates generated paths without mismatch warnings", async () => {
  const html = await renderToString(createSSRApp(ShapeProbe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(ShapeProbe);
  try {
    app.mount(host);
    assert.equal(host.querySelectorAll("path").length, 7);
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
