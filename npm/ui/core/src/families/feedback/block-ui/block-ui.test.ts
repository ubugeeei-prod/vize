import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { BlockUIExpose, BlockUISlotState } from "./block-ui.ts";
import BlockUI from "./block-ui.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders an idle section by default without styling or accessibility policy", async () => {
  const handle = mountInteraction(BlockUI, {
    slots: { default: "Account settings" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "block-ui");
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(root.getAttribute("data-reason"), "loading");
  assert.equal(root.getAttribute("data-interaction"), "none");
  assert.equal(root.getAttribute("data-announcement"), "off");
  assert.equal(root.getAttribute("aria-busy"), null);
  assert.equal(root.hasAttribute("inert"), false);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.textContent, "Account settings");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("marks blocked inert regions busy and announces politely when labelled", () => {
  const handle = mountInteraction(BlockUI, {
    props: {
      announce: "polite",
      as: "article",
      blocked: true,
      interaction: "inert",
      label: "Saving profile",
      reason: "saving",
    },
    slots: { default: "Profile form" },
  });
  const root = handle.getByRole("status", { name: "Saving profile" });

  assert.equal(root.tagName, "ARTICLE");
  assert.equal(root.getAttribute("aria-busy"), "true");
  assert.equal(root.hasAttribute("inert"), true);
  assert.equal(root.getAttribute("aria-live"), "polite");
  assert.equal(root.getAttribute("data-state"), "blocked");
  assert.equal(root.getAttribute("data-reason"), "saving");
  assert.equal(root.getAttribute("data-interaction"), "inert");
  assert.equal(root.getAttribute("data-announcement"), "polite");
  assert.equal(root.textContent, "Profile form");
  handle.unmount();
});

test("owns busy and inert while leaving unrelated fallthrough attrs consumer owned", async () => {
  const handle = mountInteraction(BlockUI, {
    attrs: {
      "aria-busy": "false",
      "aria-describedby": "sync-help",
      "aria-label": "Sync region",
      class: "sync-panel",
      inert: "",
      role: "region",
      tabindex: "0",
    },
    props: {
      blocked: true,
      interaction: "none",
      reason: "syncing",
    },
    slots: { default: '<p id="sync-help">Sync details</p>' },
  });
  const root = handle.getByRole("region", { name: "Sync region" });

  assert.equal(root.getAttribute("aria-busy"), "true");
  assert.equal(root.hasAttribute("inert"), false);
  assert.equal(root.getAttribute("aria-describedby"), "sync-help");
  assert.equal(root.getAttribute("class"), "sync-panel");
  assert.equal(root.getAttribute("tabindex"), "0");

  await handle.wrapper.setProps({ blocked: false, interaction: "inert" });
  assert.equal(root.getAttribute("aria-busy"), null);
  assert.equal(root.hasAttribute("inert"), false);
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(root.getAttribute("data-interaction"), "inert");
  handle.unmount();
});

test("uses assertive announcement attrs only while announce and label are present", async () => {
  const handle = mountInteraction(BlockUI, {
    props: {
      announce: "assertive",
      label: "Offline workspace",
      reason: "offline",
    },
    slots: { default: "Cached data" },
  });
  const root = handle.getByRole("alert", { name: "Offline workspace" });

  assert.equal(root.getAttribute("aria-live"), "assertive");
  assert.equal(root.getAttribute("data-announcement"), "assertive");
  assert.equal(root.getAttribute("data-reason"), "offline");

  await handle.wrapper.setProps({ announce: "polite", label: "   " });
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(handle.queryByRole("alert"), null);
  handle.unmount();
});

test("passes slot state and exposes live block-ui state", async () => {
  const handle = mountInteraction(BlockUI, {
    props: {
      announce: "polite",
      blocked: true,
      interaction: "inert",
      label: "Loading dashboard",
      reason: "loading",
    },
    slots: {
      default: (state: BlockUISlotState) =>
        `${state.state}:${state.blocked}:${state.reason}:${state.interaction}:${state.announcement}`,
    },
  });
  const exposed = handle.exposes<BlockUIExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.blocked, true);
  assert.equal(exposed.state, "blocked");
  assert.equal(exposed.reason, "loading");
  assert.equal(exposed.interaction, "inert");
  assert.equal(exposed.announcement, "polite");
  assert.equal(handle.root().textContent, "blocked:true:loading:inert:polite");

  await handle.wrapper.setProps({
    announce: "off",
    blocked: false,
    interaction: "none",
    reason: "stale",
  });
  assert.equal(exposed.blocked, false);
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.reason, "stale");
  assert.equal(exposed.interaction, "none");
  assert.equal(exposed.announcement, "off");
  assert.equal(handle.root().textContent, "idle:false:stale:none:off");
  handle.unmount();
});
