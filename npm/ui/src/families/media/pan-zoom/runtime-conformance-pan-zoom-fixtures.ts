import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  PanZoomContent,
  PanZoomFit,
  PanZoomReset,
  PanZoomRoot,
  PanZoomStatus,
  PanZoomViewport,
  PanZoomZoomIn,
  PanZoomZoomOut,
} from "./pan-zoom.ts";

function panZoom() {
  return h(PanZoomRoot, { defaultValue: { x: 4, y: 6, scale: 1.5 } }, () => [
    h(PanZoomViewport, { ariaLabel: "Diagram" }, () =>
      h(PanZoomContent, null, () =>
        h("svg", { viewBox: "0 0 10 10", role: "img", "aria-label": "Diagram" }),
      ),
    ),
    h(PanZoomZoomIn),
    h(PanZoomZoomOut),
    h(PanZoomReset),
    h(PanZoomFit),
    h(PanZoomStatus),
  ]);
}

function part(host: HTMLElement, name: string): Element {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element, `${name} must render`);
  return element;
}

function buttonFixture(file: string, name: string, label: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/media/pan-zoom/${file}`,
    render: panZoom,
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="${name}"`));
      assert.match(html, new RegExp(label));
    },
    assertHydratedDom(host) {
      const button = part(host, name);
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.type, "button");
    },
  };
}

export const panZoomRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "pan-zoom",
    sourceFile: "families/media/pan-zoom/pan-zoom-root.vue",
    render: panZoom,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="pan-zoom-root"/);
      assert.match(html, /data-state="idle"/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "pan-zoom-root").getAttribute("data-state"), "idle");
    },
  },
  {
    name: "pan-zoom-viewport",
    sourceFile: "families/media/pan-zoom/pan-zoom-viewport.vue",
    render: panZoom,
    assertServerMarkup(html) {
      assert.match(html, /role="group" tabindex="0" aria-roledescription="pan and zoom area"/);
      assert.match(html, /aria-label="Diagram"/);
    },
    assertHydratedDom(host) {
      const viewport = part(host, "pan-zoom-viewport");
      assert.ok(viewport instanceof HTMLDivElement);
      assert.equal(viewport.style.touchAction, "none");
    },
  },
  {
    name: "pan-zoom-content",
    sourceFile: "families/media/pan-zoom/pan-zoom-content.vue",
    render: panZoom,
    assertServerMarkup(html) {
      assert.match(html, /transform:translate\(4px, 6px\) scale\(1\.5\)/);
    },
    assertHydratedDom(host) {
      const content = part(host, "pan-zoom-content");
      assert.ok(content instanceof HTMLDivElement);
      assert.equal(content.style.transform, "translate(4px, 6px) scale(1.5)");
    },
  },
  buttonFixture("pan-zoom-zoom-in.vue", "pan-zoom-zoom-in", "Zoom in"),
  buttonFixture("pan-zoom-zoom-out.vue", "pan-zoom-zoom-out", "Zoom out"),
  buttonFixture("pan-zoom-reset.vue", "pan-zoom-reset", "Reset zoom"),
  buttonFixture("pan-zoom-fit.vue", "pan-zoom-fit", "Fit to view"),
  {
    name: "pan-zoom-status",
    sourceFile: "families/media/pan-zoom/pan-zoom-status.vue",
    render: panZoom,
    assertServerMarkup(html) {
      assert.match(html, /role="status" aria-live="polite"/);
      assert.match(html, /150%/);
    },
    assertHydratedDom(host) {
      assert.equal(part(host, "pan-zoom-status").textContent, "150%");
    },
  },
];
