import assert from "node:assert/strict";

import { h } from "vue";

import { ResizableHandle, ResizableRoot } from "./resizable.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const resizableFamilyRoot = "families/layout/resizable/";

function resizable(children: () => unknown): ReturnType<typeof h> {
  return h(ResizableRoot, { id: "runtime-resizable", maxWidth: 480 }, children);
}

export const resizableRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "resizable-root",
    sourceFile: `${resizableFamilyRoot}resizable-root.vue`,
    render: () => resizable(() => "Panel"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-resizable"/);
      assert.match(html, /data-vize-ui="resizable-root"/);
      assert.match(html, /width:320px/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="resizable-root"]');
      assert.ok(root instanceof HTMLElement);
      assert.equal(root.style.width, "320px");
    },
  },
  {
    name: "resizable-handle",
    sourceFile: `${resizableFamilyRoot}resizable-handle.vue`,
    render: () => resizable(() => h(ResizableHandle, { edge: "e" })),
    assertServerMarkup(html) {
      assert.match(html, /role="separator"/);
      assert.match(html, /aria-controls="runtime-resizable"/);
      assert.match(html, /aria-valuemax="480"/);
    },
    assertHydratedDom(host) {
      const handle = host.querySelector('[data-vize-ui="resizable-handle"]');
      assert.ok(handle instanceof HTMLElement);
      assert.equal(handle.getAttribute("aria-orientation"), "vertical");
    },
  },
];
