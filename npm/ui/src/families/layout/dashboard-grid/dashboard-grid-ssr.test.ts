import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import DashboardGridItem from "./dashboard-grid-item.vue";
import type { DashboardGridItemSlotState } from "./dashboard-grid-types.ts";
import DashboardGrid from "./dashboard-grid.vue";

const Probe = defineComponent({
  name: "DashboardGridSsrProbe",
  setup: () => () =>
    h(
      DashboardGrid,
      { defaultLayout: [{ id: "sales", x: 1, y: 0, w: 2, h: 2 }], columns: 6, label: "Metrics" },
      () => [
        h(
          DashboardGridItem,
          { id: "sales", label: "Sales" },
          {
            default: ({ handleProps }: DashboardGridItemSlotState) =>
              h("h3", { ...handleProps }, "Sales"),
          },
        ),
      ],
    ),
});

test("renders byte-identical grid placement across SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(
    html,
    /role="region" aria-label="Metrics" data-vize-ui="dashboard-grid" data-columns="6"/,
  );
  assert.match(html, /grid-column:2 \/ span 2;grid-row:1 \/ span 2;/);
  assert.match(html, /data-part="drag-handle" tabindex="0"/);
});

test("hydrates the grid without mismatch warnings", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.querySelectorAll('[data-vize-ui="dashboard-grid-item"]').length, 1);
  } finally {
    app.unmount();
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
  }
  assert.deepEqual(diagnostics, []);
});
