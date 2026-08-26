import assert from "node:assert/strict";
// source-contract: computed CSS is not observable without a real style pipeline; see below.
import { readFile } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";
import { h } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import VisuallyHidden from "./visually-hidden.vue";

test("keeps slotted content queryable in the accessibility tree", async () => {
  const handle = mountInteraction(VisuallyHidden, {
    slots: { default: () => h("button", "Dismiss notification") },
  });

  assert.equal(handle.root().getAttribute("data-vize-ui"), "visually-hidden");

  const control = handle.getByRole("button", { name: "Dismiss notification" });
  assert.ok((await handle.tab()) === control, "hidden content must stay keyboard reachable");
  assert.ok(handle.activeElement() === control);
  handle.unmount();
});

test("exposes the rendered element for composition", () => {
  const handle = mountInteraction(VisuallyHidden, { slots: { default: "Saving" } });
  const exposed = handle.exposes<{ element: HTMLElement | null }>();

  assert.ok(exposed.element === handle.root());
  assert.equal(handle.root().textContent, "Saving");
  handle.unmount();
});

test("hides content with a recoverable clipping technique, never display:none", async () => {
  // source-contract: the scoped <style> ships via the packaged stylesheet, so the
  // computed clip-path cannot be observed on a mounted node without a CSS pipeline.
  const source = await readFile(path.resolve("src/visually-hidden.vue"), "utf8");

  // source-contract: assert the hiding technique directly on the style block.
  assert.match(source, /position: absolute/);
  // source-contract: clip-path keeps the node in the accessibility tree.
  assert.match(source, /clip-path: inset\(50%\)/);
  // source-contract: display:none or visibility:hidden would remove it from the tree.
  assert.doesNotMatch(source, /display:\s*none|visibility:\s*hidden/);
});
