import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { isValidAspectRatio, normalizeAspectRatio } from "./aspect-ratio-runtime.ts";
import type { AspectRatioSlotState } from "./aspect-ratio.ts";
import AspectRatio from "./aspect-ratio.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("normalizes only positive finite aspect ratios", () => {
  assert.equal(isValidAspectRatio(undefined), true);
  assert.equal(isValidAspectRatio(1), true);
  assert.equal(isValidAspectRatio(16 / 9), true);
  assert.equal(isValidAspectRatio(0), false);
  assert.equal(isValidAspectRatio(-1), false);
  assert.equal(isValidAspectRatio(Number.NaN), false);
  assert.equal(isValidAspectRatio(Number.POSITIVE_INFINITY), false);
  assert.equal(normalizeAspectRatio(undefined), 1);
  assert.equal(normalizeAspectRatio(4 / 3), 4 / 3);
  assert.equal(normalizeAspectRatio(0), 1);
});

test("renders a square headless host by default", () => {
  const handle = mountInteraction(AspectRatio, { slots: { default: "Preview" } });
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("data-vize-ui"), "aspect-ratio");
  assert.equal(root.getAttribute("data-state"), "valid");
  assert.equal(root.getAttribute("data-vize-aspect-ratio"), "1");
  assert.equal(root.style.getPropertyValue("--vize-ui-aspect-ratio"), "1");
  assert.equal(root.style.aspectRatio, "var(--vize-ui-aspect-ratio)");
  assert.equal(root.textContent, "Preview");
  handle.unmount();
});

test("publishes the requested ratio through stable data and style hooks", () => {
  const ratio = 16 / 9;
  const handle = mountInteraction(AspectRatio, { props: { ratio } });
  const root = handle.root();

  assert.equal(root.getAttribute("data-state"), "valid");
  assert.equal(root.getAttribute("data-vize-aspect-ratio"), String(ratio));
  assert.equal(root.style.getPropertyValue("--vize-ui-aspect-ratio"), String(ratio));
  assert.equal(root.style.aspectRatio, "var(--vize-ui-aspect-ratio)");
  handle.unmount();
});

test("falls back deliberately for non-positive and non-finite ratios", async () => {
  const handle = mountInteraction(AspectRatio);
  const root = handle.root();

  for (const value of [0, -1, Number.NaN, Number.POSITIVE_INFINITY]) {
    await handle.wrapper.setProps({ ratio: value });
    assert.equal(root.getAttribute("data-state"), "fallback");
    assert.equal(root.getAttribute("data-vize-aspect-ratio"), "1");
    assert.equal(root.style.getPropertyValue("--vize-ui-aspect-ratio"), "1");
  }

  handle.unmount();
});

test("renders a semantic host and exposes normalized slot state", () => {
  const handle = mountInteraction(AspectRatio, {
    props: { as: "section", ratio: 4 / 3 },
    slots: {
      default: ({ invalid, ratio }: AspectRatioSlotState) => `ratio:${ratio} invalid:${invalid}`,
    },
  });

  assert.equal(handle.root().tagName, "SECTION");
  assert.equal(handle.root().textContent, "ratio:1.3333333333333333 invalid:false");
  handle.unmount();
});

test("exposes the rendered element and live normalized ratio state", async () => {
  const handle = mountInteraction(AspectRatio, { props: { ratio: 2 } });
  const exposed = handle.exposes<{ element: Element | null; invalid: boolean; ratio: number }>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.ratio, 2);
  assert.equal(exposed.invalid, false);

  await handle.wrapper.setProps({ ratio: -1 });
  assert.equal(exposed.ratio, 1);
  assert.equal(exposed.invalid, true);
  handle.unmount();
});
