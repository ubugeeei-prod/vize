import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createControllerRig, settle } from "./fullscreen-button-test-utils.ts";
import FullscreenButton from "./fullscreen-button.vue";
import type { FullscreenButtonExpose, FullscreenButtonSlotState } from "./fullscreen-button.ts";

test("supports custom labels and slot rendering", async () => {
  const rig = createControllerRig();
  const handle = mountInteraction(FullscreenButton, {
    props: {
      busyLabel: "Fullscreen is changing",
      controller: rig.controller,
      enterLabel: "Open canvas",
      errorLabel: "Canvas fullscreen failed",
      exitLabel: "Close canvas",
    },
    slots: {
      default: (state: FullscreenButtonSlotState) =>
        h(
          "span",
          { "data-slot-state": state.state, "data-slot-operation": state.operation ?? "" },
          `${state.label}:${state.state}:${state.operation}:${state.active}:${state.pending}`,
        ),
    },
  });
  const button = handle.getByRole("button", { name: "Open canvas:idle:null:false:false" });

  assert.equal(button.textContent, "Open canvas:idle:null:false:false");
  assert.equal(button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"), "idle");
  await handle.click(button);
  await settle();
  assert.equal(button.getAttribute("data-state"), "active");
  assert.equal(button.textContent, "Close canvas:active:null:true:false");
  assert.equal(
    button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"),
    "active",
  );
  handle.unmount();
});

test("exposes live state and focus", async () => {
  let resolveEnter: (() => void) | null = null;
  const rig = createControllerRig({
    request: () =>
      new Promise<void>((resolve) => {
        resolveEnter = resolve;
      }),
  });
  const handle = mountInteraction(FullscreenButton, {
    props: {
      ariaLabel: "Toggle fullscreen",
      controller: rig.controller,
      exitLabel: "Leave fullscreen",
    },
  });
  const exposed = handle.exposes<FullscreenButtonExpose>();
  const button = handle.getByRole("button", { name: "Toggle fullscreen" });

  assert.ok(exposed.element === button);
  assert.equal(exposed.active, false);
  assert.equal(exposed.disabled, false);
  assert.equal(exposed.pending, false);
  assert.equal(exposed.operation, null);
  assert.equal(exposed.unavailable, false);
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.label, "Enter fullscreen");
  exposed.focus();
  assert.ok(handle.activeElement() === button);

  void handle.click(button);
  await settle();
  assert.equal(exposed.state, "entering");
  assert.equal(exposed.pending, true);
  assert.equal(exposed.operation, "enter");
  assert.equal(exposed.unavailable, true);
  assert.equal(exposed.label, "Changing fullscreen");

  assert.ok(resolveEnter);
  resolveEnter();
  await settle();
  assert.equal(exposed.state, "active");
  assert.equal(exposed.active, true);
  assert.equal(exposed.pending, false);
  assert.equal(exposed.operation, null);
  assert.equal(exposed.label, "Leave fullscreen");
  handle.unmount();
});
