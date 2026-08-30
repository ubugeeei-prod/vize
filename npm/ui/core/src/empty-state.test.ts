import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { EmptyStateExpose, EmptyStateSlotState } from "./empty-state.ts";
import EmptyState from "./empty-state.vue";
import { mountInteraction } from "./testing/mount.ts";

test("renders a neutral section empty state by default", async () => {
  const handle = mountInteraction(EmptyState, {
    slots: { default: "No projects yet" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "empty-state");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("data-density"), "comfortable");
  assert.equal(root.getAttribute("data-orientation"), "block");
  assert.equal(root.getAttribute("data-state"), "empty");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.textContent, "No projects yet");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders custom hooks without adding accessibility or focus policy", () => {
  const handle = mountInteraction(EmptyState, {
    props: {
      as: "article",
      density: "compact",
      orientation: "inline",
      tone: "warning",
    },
    slots: { default: "No matching filters" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "ARTICLE");
  assert.equal(root.getAttribute("data-tone"), "warning");
  assert.equal(root.getAttribute("data-density"), "compact");
  assert.equal(root.getAttribute("data-orientation"), "inline");
  assert.equal(root.getAttribute("data-state"), "empty");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.textContent, "No matching filters");
  handle.unmount();
});

test("keeps labels, roles, live-region policy, and focus attrs consumer owned", () => {
  const handle = mountInteraction(EmptyState, {
    attrs: {
      "aria-label": "No search results",
      "aria-live": "polite",
      role: "status",
      tabindex: "-1",
    },
    props: {
      density: "compact",
      orientation: "inline",
      tone: "info",
    },
    slots: { default: "Try another query" },
  });
  const root = handle.getByRole("status", { name: "No search results" });

  assert.equal(root.getAttribute("aria-live"), "polite");
  assert.equal(root.getAttribute("tabindex"), "-1");
  assert.equal(root.getAttribute("data-tone"), "info");
  assert.equal(root.getAttribute("data-density"), "compact");
  assert.equal(root.getAttribute("data-orientation"), "inline");
  assert.equal(root.textContent, "Try another query");
  handle.unmount();
});

test("passes slot state and exposes live empty-state hooks", async () => {
  const handle = mountInteraction(EmptyState, {
    props: {
      density: "compact",
      orientation: "inline",
      tone: "danger",
    },
    slots: {
      default: (state: EmptyStateSlotState) =>
        `${state.state}:${state.tone}:${state.density}:${state.orientation}`,
    },
  });
  const exposed = handle.exposes<EmptyStateExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.state, "empty");
  assert.equal(exposed.tone, "danger");
  assert.equal(exposed.density, "compact");
  assert.equal(exposed.orientation, "inline");
  assert.equal(handle.root().textContent, "empty:danger:compact:inline");

  await handle.wrapper.setProps({
    density: "comfortable",
    orientation: "block",
    tone: "success",
  });
  assert.equal(exposed.state, "empty");
  assert.equal(exposed.tone, "success");
  assert.equal(exposed.density, "comfortable");
  assert.equal(exposed.orientation, "block");
  assert.equal(handle.root().getAttribute("data-tone"), "success");
  assert.equal(handle.root().getAttribute("data-density"), "comfortable");
  assert.equal(handle.root().getAttribute("data-orientation"), "block");
  assert.equal(handle.root().textContent, "empty:success:comfortable:block");
  handle.unmount();
});
