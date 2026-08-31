import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { SeparatorExpose } from "./separator.ts";
import Separator from "./separator.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders a native horizontal separator by default", async () => {
  const handle = mountInteraction(Separator);
  const root = handle.getByRole("separator");

  assert.equal(root.tagName, "HR");
  assert.equal(root.getAttribute("role"), "separator");
  assert.equal(root.getAttribute("aria-orientation"), "horizontal");
  assert.equal(root.getAttribute("data-vize-ui"), "separator");
  assert.equal(root.getAttribute("data-state"), "semantic");
  assert.equal(root.getAttribute("data-orientation"), "horizontal");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders a labelled vertical separator on a custom host", () => {
  const handle = mountInteraction(Separator, {
    props: {
      ariaLabel: "Pane boundary",
      as: "div",
      orientation: "vertical",
    },
  });
  const root = handle.getByRole("separator", { name: "Pane boundary" });

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("aria-orientation"), "vertical");
  assert.equal(root.getAttribute("aria-label"), "Pane boundary");
  assert.equal(root.getAttribute("data-orientation"), "vertical");
  handle.unmount();
});

test("decorative separators opt out of accessibility semantics", () => {
  const handle = mountInteraction(Separator, {
    props: {
      ariaLabel: "Ignored label",
      decorative: true,
      orientation: "vertical",
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("role"), "presentation");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("aria-orientation"), null);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("data-state"), "decorative");
  assert.equal(root.getAttribute("data-orientation"), "vertical");
  assert.equal(handle.queryByRole("separator"), null);
  handle.unmount();
});

test("exposes the rendered element and live separator state", async () => {
  const handle = mountInteraction(Separator);
  const exposed = handle.exposes<SeparatorExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.orientation, "horizontal");
  assert.equal(exposed.decorative, false);

  await handle.wrapper.setProps({ decorative: true, orientation: "vertical" });
  assert.equal(exposed.orientation, "vertical");
  assert.equal(exposed.decorative, true);
  handle.unmount();
});
