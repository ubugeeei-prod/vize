import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { resolveSpacerLayout } from "./spacer-runtime.ts";
import type { SpacerExpose } from "./spacer.ts";
import Spacer from "./spacer.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("resolves inline and both-axis logical sizes without authored CSS classes", () => {
  assert.deepEqual(resolveSpacerLayout({ axis: "inline", size: "2ch" }), {
    axis: "inline",
    blockSize: "auto",
    display: "inline-block",
    inlineSize: "2ch",
    state: "sized",
    style: {
      "--vize-ui-spacer-block-size": "auto",
      "--vize-ui-spacer-inline-size": "2ch",
      blockSize: "var(--vize-ui-spacer-block-size)",
      display: "inline-block",
      inlineSize: "var(--vize-ui-spacer-inline-size)",
    },
  });
  assert.deepEqual(resolveSpacerLayout({ axis: "both", size: "1lh" }), {
    axis: "both",
    blockSize: "1lh",
    display: "inline-block",
    inlineSize: "1lh",
    state: "sized",
    style: {
      "--vize-ui-spacer-block-size": "1lh",
      "--vize-ui-spacer-inline-size": "1lh",
      blockSize: "var(--vize-ui-spacer-block-size)",
      display: "inline-block",
      inlineSize: "var(--vize-ui-spacer-inline-size)",
    },
  });
});

test("renders a decorative block spacer by default", async () => {
  const handle = mountInteraction(Spacer);
  const root = handle.root();

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "spacer");
  assert.equal(root.getAttribute("data-state"), "sized");
  assert.equal(root.getAttribute("data-axis"), "block");
  assert.equal(root.getAttribute("data-display"), "block");
  assert.equal(root.getAttribute("data-vize-spacer-inline-size"), "auto");
  assert.equal(root.getAttribute("data-vize-spacer-block-size"), "1rem");
  assert.equal(root.style.getPropertyValue("--vize-ui-spacer-inline-size"), "auto");
  assert.equal(root.style.getPropertyValue("--vize-ui-spacer-block-size"), "1rem");
  assert.equal(root.style.display, "block");
  assert.equal(root.style.inlineSize, "var(--vize-ui-spacer-inline-size)");
  assert.equal(root.style.blockSize, "var(--vize-ui-spacer-block-size)");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders explicit logical sizes on a custom host", () => {
  const handle = mountInteraction(Spacer, {
    props: {
      as: "div",
      axis: "inline",
      blockSize: "1lh",
      display: "inline-grid",
      inlineSize: "clamp(1rem, 2vi, 3rem)",
      size: "2rem",
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("data-axis"), "inline");
  assert.equal(root.getAttribute("data-display"), "inline-grid");
  assert.equal(root.getAttribute("data-vize-spacer-inline-size"), "clamp(1rem, 2vi, 3rem)");
  assert.equal(root.getAttribute("data-vize-spacer-block-size"), "1lh");
  assert.equal(
    root.style.getPropertyValue("--vize-ui-spacer-inline-size"),
    "clamp(1rem, 2vi, 3rem)",
  );
  assert.equal(root.style.getPropertyValue("--vize-ui-spacer-block-size"), "1lh");
  assert.equal(root.style.display, "inline-grid");
  handle.unmount();
});

test("supports an SVG host without accessible content", () => {
  const handle = mountInteraction(Spacer, {
    props: {
      as: "svg",
      axis: "both",
      size: "24px",
    },
  });
  const root = handle.wrapper.element;

  assert.equal(root.tagName.toLowerCase(), "svg");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("data-axis"), "both");
  assert.equal(root.getAttribute("data-vize-spacer-inline-size"), "24px");
  assert.equal(root.getAttribute("data-vize-spacer-block-size"), "24px");
  handle.unmount();
});

test("exposes the rendered element and live resolved layout state", async () => {
  const handle = mountInteraction(Spacer, { props: { size: "2rem" } });
  const exposed = handle.exposes<SpacerExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.axis, "block");
  assert.equal(exposed.inlineSize, "auto");
  assert.equal(exposed.blockSize, "2rem");
  assert.equal(exposed.display, "block");

  await handle.wrapper.setProps({ axis: "inline", display: "inline-flex", size: "3ch" });
  assert.equal(exposed.axis, "inline");
  assert.equal(exposed.inlineSize, "3ch");
  assert.equal(exposed.blockSize, "auto");
  assert.equal(exposed.display, "inline-flex");
  handle.unmount();
});
