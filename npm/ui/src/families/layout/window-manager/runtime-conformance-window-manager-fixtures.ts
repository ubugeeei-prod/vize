import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import FloatingWindow from "./floating-window.vue";
import type { FloatingWindowSlotState } from "./window-manager-types.ts";
import WindowDock from "./window-dock.vue";
import WindowManager from "./window-manager.vue";

const render = () =>
  h(WindowManager, null, () => [
    h(
      FloatingWindow,
      { id: "editor", title: "Editor", defaultRect: { x: 16, y: 16, width: 320, height: 200 } },
      {
        default: ({ handleProps, minimize }: FloatingWindowSlotState) => [
          h("header", { ...handleProps }, "Editor"),
          h("button", { type: "button", onClick: minimize }, "Minimize"),
        ],
      },
    ),
    h(WindowDock, { label: "Taskbar" }),
  ]);

function assertServer(html: string): void {
  assert.match(html, /data-vize-ui="window-manager"/);
  assert.match(html, /role="dialog" aria-modal="false" aria-label="Editor"/);
  assert.match(html, /data-part="drag-handle" tabindex="0" aria-keyshortcuts="ArrowUp/);
  assert.match(html, /<nav aria-label="Taskbar" data-vize-ui="window-dock"/);
  assert.match(
    html,
    /<button type="button" data-part="dock-item" data-mode="normal" aria-pressed="true"/,
  );
}

function assertHydrated(host: HTMLElement): void {
  const dialog = host.querySelector('[role="dialog"]');
  assert.ok(dialog instanceof HTMLElement);
  assert.equal(dialog.getAttribute("aria-label"), "Editor");
  assert.equal(host.querySelector('[data-part="drag-handle"]')?.getAttribute("tabindex"), "0");
  assert.equal(host.querySelector("nav")?.getAttribute("aria-label"), "Taskbar");
}

export const windowManagerRuntimeFixtures: readonly RuntimeFixture[] = (
  ["window-manager", "floating-window", "window-dock"] as const
).map((name) => ({
  name,
  sourceFile: `families/layout/window-manager/${name}.vue`,
  render,
  assertServerMarkup: assertServer,
  assertHydratedDom: assertHydrated,
}));
