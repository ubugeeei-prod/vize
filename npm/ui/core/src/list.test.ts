import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import type { ListExpose, ListSlotState } from "./list.ts";
import List from "./list.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders a native unordered list by default without adding semantics or styling", async () => {
  const handle = mountInteraction(List, {
    slots: { default: () => h("li", "Ship the primitive") },
  });
  const root = handle.root();

  assert.equal(root.tagName, "UL");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "list");
  assert.equal(root.getAttribute("data-marker"), "disc");
  assert.equal(root.getAttribute("data-spacing"), "normal");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Ship the primitive");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("mirrors list presentation hooks on an ordered host", () => {
  const handle = mountInteraction(List, {
    props: {
      as: "ol",
      marker: "decimal",
      spacing: "loose",
      tone: "accent",
    },
    slots: { default: () => h("li", "Publish the subpath") },
  });
  const root = handle.root();

  assert.equal(root.tagName, "OL");
  assert.equal(root.getAttribute("data-marker"), "decimal");
  assert.equal(root.getAttribute("data-spacing"), "loose");
  assert.equal(root.getAttribute("data-tone"), "accent");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Publish the subpath");
  handle.unmount();
});

test("keeps custom semantics and focus policy consumer owned through attrs", async () => {
  const handle = mountInteraction(List, {
    attrs: {
      "aria-label": "Release checklist",
      role: "group",
      tabindex: "0",
    },
    props: {
      as: "nav",
      marker: "none",
      tone: "success",
    },
    slots: { default: () => h("li", "Verify before release") },
  });
  const root = handle.getByRole("group", { name: "Release checklist" });

  assert.equal(root.tagName, "NAV");
  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("data-vize-ui"), "list");
  assert.equal(root.getAttribute("data-marker"), "none");
  assert.equal(root.getAttribute("data-spacing"), "normal");
  assert.equal(root.getAttribute("data-tone"), "success");
  assert.equal(root.textContent, "Verify before release");
  assert.equal(await handle.tab(), root);
  handle.unmount();
});

test("passes slot state and exposes live list state", async () => {
  const handle = mountInteraction(List, {
    props: {
      marker: "none",
      spacing: "compact",
      tone: "warning",
    },
    slots: {
      default: (state: ListSlotState) => h("li", `${state.marker}:${state.spacing}:${state.tone}`),
    },
  });
  const exposed = handle.exposes<ListExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.marker, "none");
  assert.equal(exposed.spacing, "compact");
  assert.equal(exposed.tone, "warning");
  assert.equal(handle.root().textContent, "none:compact:warning");

  await handle.wrapper.setProps({
    marker: "decimal",
    spacing: "loose",
    tone: "danger",
  });
  assert.equal(exposed.marker, "decimal");
  assert.equal(exposed.spacing, "loose");
  assert.equal(exposed.tone, "danger");
  assert.equal(handle.root().getAttribute("data-marker"), "decimal");
  assert.equal(handle.root().getAttribute("data-spacing"), "loose");
  assert.equal(handle.root().getAttribute("data-tone"), "danger");
  assert.equal(handle.root().textContent, "decimal:loose:danger");
  handle.unmount();
});
