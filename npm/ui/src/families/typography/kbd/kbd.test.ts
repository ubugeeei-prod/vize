import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import type { KbdExpose, KbdSlotState } from "./kbd.ts";
import Kbd from "./kbd.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders native keyboard input by default without styling or focus policy", async () => {
  const handle = mountInteraction(Kbd, {
    slots: { default: "Esc" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "KBD");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "kbd");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-variant"), "key");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Esc");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("mirrors shortcut presentation hooks on a custom host", () => {
  const handle = mountInteraction(Kbd, {
    props: {
      as: "span",
      size: "lg",
      tone: "accent",
      variant: "shortcut",
    },
    slots: { default: "Ctrl K" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("data-size"), "lg");
  assert.equal(root.getAttribute("data-variant"), "shortcut");
  assert.equal(root.getAttribute("data-tone"), "accent");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.textContent, "Ctrl K");
  handle.unmount();
});

test("keeps custom semantics and focus policy consumer owned through attrs", async () => {
  const handle = mountInteraction(Kbd, {
    attrs: {
      "aria-label": "Keyboard shortcut",
      role: "term",
      tabindex: "0",
    },
    props: {
      tone: "success",
      variant: "sequence",
    },
    slots: { default: "G then I" },
  });
  const root = handle.getByRole("term", { name: "Keyboard shortcut" });

  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("data-vize-ui"), "kbd");
  assert.equal(root.getAttribute("data-variant"), "sequence");
  assert.equal(root.getAttribute("data-tone"), "success");
  assert.equal(root.textContent, "G then I");
  assert.equal(await handle.tab(), root);
  handle.unmount();
});

test("passes slot state and exposes live kbd state", async () => {
  const handle = mountInteraction(Kbd, {
    props: {
      size: "sm",
      tone: "warning",
      variant: "shortcut",
    },
    slots: {
      default: (state: KbdSlotState) => `${state.size}:${state.variant}:${state.tone}`,
    },
  });
  const exposed = handle.exposes<KbdExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.size, "sm");
  assert.equal(exposed.variant, "shortcut");
  assert.equal(exposed.tone, "warning");
  assert.equal(handle.root().textContent, "sm:shortcut:warning");

  await handle.wrapper.setProps({
    size: "lg",
    tone: "danger",
    variant: "sequence",
  });
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.variant, "sequence");
  assert.equal(exposed.tone, "danger");
  assert.equal(handle.root().getAttribute("data-size"), "lg");
  assert.equal(handle.root().getAttribute("data-variant"), "sequence");
  assert.equal(handle.root().getAttribute("data-tone"), "danger");
  assert.equal(handle.root().textContent, "lg:sequence:danger");
  handle.unmount();
});
