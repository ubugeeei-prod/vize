import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import FloatingWindow from "./floating-window.vue";
import type { WindowLayout } from "./window-manager-model.ts";
import type { FloatingWindowSlotState } from "./window-manager-types.ts";
import WindowDock from "./window-dock.vue";
import WindowManager from "./window-manager.vue";

const defaultLayout: WindowLayout = {
  windows: { notes: { x: 40, y: 30, width: 240, height: 180, mode: "minimized" } },
  order: ["notes", "editor"],
};

const Probe = defineComponent({
  name: "WindowManagerSsrProbe",
  setup: () => () =>
    h(WindowManager, { defaultLayout }, () => [
      h(
        FloatingWindow,
        { id: "editor", title: "Editor", defaultRect: { x: 10, y: 20, width: 300, height: 200 } },
        {
          default: ({ handleProps, active }: FloatingWindowSlotState) =>
            h("header", { ...handleProps }, `Editor ${String(active)}`),
        },
      ),
      h(FloatingWindow, { id: "notes", title: "Notes" }, () => "Notes body"),
      h(WindowDock),
    ]),
});

test("renders byte-identical window geometry, stacking, and dock markup", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(
    html,
    /role="dialog" aria-modal="false" aria-label="Editor"[^>]*data-mode="normal"[^>]*style="position:absolute;z-index:2;left:10px;top:20px;width:300px;height:200px;"/,
  );
  assert.match(html, /aria-label="Notes"[^>]*data-mode="minimized" hidden/);
  assert.match(html, /<nav aria-label="Windows" data-vize-ui="window-dock"/);
  assert.match(html, /data-part="drag-handle" tabindex="0"/);
});

test("hydrates windows and dock without mismatch warnings", async () => {
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
    assert.equal(host.querySelectorAll('[role="dialog"]').length, 2);
    assert.equal(host.querySelectorAll('[data-part="dock-item"]').length, 2);
  } finally {
    app.unmount();
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
  }
  assert.deepEqual(diagnostics, []);
});
