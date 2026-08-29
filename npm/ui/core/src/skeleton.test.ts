import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { SkeletonExpose, SkeletonSlotState } from "./skeleton.ts";
import Skeleton from "./skeleton.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders a decorative loading placeholder by default", async () => {
  const handle = mountInteraction(Skeleton);
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("data-vize-ui"), "skeleton");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "loading");
  assert.equal(root.getAttribute("data-loading"), "true");
  assert.equal(root.getAttribute("data-visible"), "true");
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.style.getPropertyValue("--vize-ui-skeleton-block-size"), "1em");
  assert.equal(root.style.getPropertyValue("--vize-ui-skeleton-inline-size"), "100%");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders status semantics when labelled", () => {
  const handle = mountInteraction(Skeleton, {
    props: {
      ariaLabel: "Loading profile",
      as: "section",
      blockSize: "2rem",
      inlineSize: "12rem",
    },
  });
  const root = handle.getByRole("status", { name: "Loading profile" });

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("aria-label"), "Loading profile");
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("data-aria-state"), "status");
  assert.equal(root.style.getPropertyValue("--vize-ui-skeleton-block-size"), "2rem");
  assert.equal(root.style.getPropertyValue("--vize-ui-skeleton-inline-size"), "12rem");
  handle.unmount();
});

test("lets ariaHidden override labelled status semantics", () => {
  const handle = mountInteraction(Skeleton, {
    props: {
      ariaHidden: true,
      ariaLabel: "Ignored loading label",
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(handle.queryByRole("status"), null);
  handle.unmount();
});

test("keeps hidden and loaded states observable without unmounting", async () => {
  const handle = mountInteraction(Skeleton, {
    props: {
      ariaHidden: false,
      ariaLabel: "Loading table",
      loading: false,
      visible: true,
    },
  });
  const root = handle.getByRole("status", { name: "Loading table" });

  assert.equal(root.hasAttribute("hidden"), false);
  assert.equal(root.getAttribute("data-state"), "loaded");
  assert.equal(root.getAttribute("data-loading"), "false");
  assert.equal(root.getAttribute("data-visible"), "true");

  await handle.wrapper.setProps({ loading: true, visible: false });
  assert.ok(root.hasAttribute("hidden"));
  assert.equal(root.getAttribute("data-state"), "hidden");
  assert.equal(root.getAttribute("data-loading"), "true");
  assert.equal(root.getAttribute("data-visible"), "false");
  handle.unmount();
});

test("passes slot state and exposes live element/loading state", async () => {
  const handle = mountInteraction(Skeleton, {
    props: {
      ariaLabel: "Loading metrics",
    },
    slots: {
      default: (state: SkeletonSlotState) =>
        `${state.state}:${state.loading}:${state.visible}:${state.ariaState}`,
    },
  });
  const exposed = handle.exposes<SkeletonExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.loading, true);
  assert.equal(exposed.visible, true);
  assert.equal(exposed.state, "loading");
  assert.equal(exposed.ariaState, "status");
  assert.equal(handle.root().textContent, "loading:true:true:status");

  await handle.wrapper.setProps({ loading: false, visible: false });
  assert.equal(exposed.loading, false);
  assert.equal(exposed.visible, false);
  assert.equal(exposed.state, "hidden");
  assert.equal(exposed.ariaState, "status");
  assert.equal(handle.root().textContent, "hidden:false:false:status");
  handle.unmount();
});
