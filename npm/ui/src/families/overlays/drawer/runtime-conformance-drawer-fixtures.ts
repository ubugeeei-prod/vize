import assert from "node:assert/strict";

import { h } from "vue";

import { DrawerContent, DrawerHandle, DrawerRoot, DrawerTitle } from "./drawer.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

function drawer(children: () => unknown): ReturnType<typeof h> {
  return h(
    DrawerRoot,
    { defaultOpen: true, id: "runtime-drawer", snapPoints: [0.5, 1], side: "bottom" },
    children,
  );
}

export const drawerRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "drawer-root",
    sourceFile: "families/overlays/drawer/drawer-root.vue",
    render: () => drawer(() => "Drawer"),
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-drawer"/);
      assert.match(html, /data-vize-ui="drawer-root"/);
      assert.match(html, /data-side="bottom"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="drawer-root"]');
      assert.ok(root instanceof HTMLElement);
      assert.equal(root.getAttribute("data-state"), "open");
    },
  },
  {
    name: "drawer-content",
    sourceFile: "families/overlays/drawer/drawer-content.vue",
    render: () =>
      drawer(() =>
        h(DrawerContent, { lockScroll: false }, () => h(DrawerTitle, null, () => "Cart")),
      ),
    assertServerMarkup(html) {
      assert.match(html, /<dialog id="runtime-drawer-content"/);
      assert.match(html, /aria-labelledby="runtime-drawer-title"/);
      assert.match(html, /data-snap-point="0.5"/);
      assert.doesNotMatch(html, /<dialog[^>]*\sopen[\s>=]/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="drawer-content"]');
      assert.ok(content instanceof HTMLDialogElement);
      assert.equal(content.getAttribute("data-side"), "bottom");
    },
  },
  {
    name: "drawer-handle",
    sourceFile: "families/overlays/drawer/drawer-handle.vue",
    render: () => drawer(() => h(DrawerContent, { lockScroll: false }, () => h(DrawerHandle))),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="drawer-handle"/);
      assert.match(html, /aria-label="Resize drawer"/);
      assert.match(html, /aria-controls="runtime-drawer-content"/);
    },
    assertHydratedDom(host) {
      const handle = host.querySelector('[data-vize-ui="drawer-handle"]');
      assert.ok(handle instanceof HTMLButtonElement);
      assert.equal(handle.type, "button");
    },
  },
];
