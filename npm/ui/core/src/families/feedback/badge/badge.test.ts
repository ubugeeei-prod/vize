import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { BadgeExpose, BadgeSlotState } from "./badge.ts";
import Badge from "./badge.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders an inline neutral label badge by default", async () => {
  const handle = mountInteraction(Badge, {
    slots: { default: "Beta" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "badge");
  assert.equal(root.getAttribute("data-variant"), "label");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.textContent, "Beta");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders status and count variants without adding ARIA or focus policy", () => {
  const status = mountInteraction(Badge, {
    props: {
      as: "strong",
      tone: "success",
      variant: "status",
    },
    slots: { default: "Online" },
  });
  const statusRoot = status.root();

  assert.equal(statusRoot.tagName, "STRONG");
  assert.equal(statusRoot.getAttribute("data-variant"), "status");
  assert.equal(statusRoot.getAttribute("data-tone"), "success");
  assert.equal(statusRoot.getAttribute("role"), null);
  assert.equal(statusRoot.getAttribute("tabindex"), null);
  assert.equal(statusRoot.getAttribute("aria-hidden"), null);
  assert.equal(statusRoot.textContent, "Online");
  status.unmount();

  const count = mountInteraction(Badge, {
    props: {
      as: "sup",
      tone: "danger",
      variant: "count",
    },
    slots: { default: "12" },
  });
  const countRoot = count.root();

  assert.equal(countRoot.tagName, "SUP");
  assert.equal(countRoot.getAttribute("data-variant"), "count");
  assert.equal(countRoot.getAttribute("data-tone"), "danger");
  assert.equal(countRoot.getAttribute("role"), null);
  assert.equal(countRoot.getAttribute("tabindex"), null);
  assert.equal(countRoot.getAttribute("aria-hidden"), null);
  assert.equal(countRoot.textContent, "12");
  count.unmount();
});

test("keeps ARIA and live-region semantics consumer owned through attrs", () => {
  const handle = mountInteraction(Badge, {
    attrs: {
      "aria-atomic": "true",
      "aria-label": "12 unread notifications",
      "aria-live": "polite",
      role: "status",
    },
    props: {
      tone: "info",
      variant: "count",
    },
    slots: { default: "12" },
  });
  const root = handle.getByRole("status", { name: "12 unread notifications" });

  assert.equal(root.getAttribute("aria-live"), "polite");
  assert.equal(root.getAttribute("aria-atomic"), "true");
  assert.equal(root.getAttribute("data-variant"), "count");
  assert.equal(root.getAttribute("data-tone"), "info");
  assert.equal(root.textContent, "12");
  handle.unmount();
});

test("passes slot state and exposes live badge state", async () => {
  const handle = mountInteraction(Badge, {
    props: {
      tone: "warning",
      variant: "status",
    },
    slots: {
      default: (state: BadgeSlotState) => `${state.variant}:${state.tone}`,
    },
  });
  const exposed = handle.exposes<BadgeExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.variant, "status");
  assert.equal(exposed.tone, "warning");
  assert.equal(handle.root().textContent, "status:warning");

  await handle.wrapper.setProps({ tone: "accent", variant: "label" });
  assert.equal(exposed.variant, "label");
  assert.equal(exposed.tone, "accent");
  assert.equal(handle.root().getAttribute("data-variant"), "label");
  assert.equal(handle.root().getAttribute("data-tone"), "accent");
  assert.equal(handle.root().textContent, "label:accent");
  handle.unmount();
});
