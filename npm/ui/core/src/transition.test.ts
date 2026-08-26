import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, nextTick, ref } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import { createTransition, useTransition } from "./transition.ts";
import Transition from "./transition.vue";

/** The conditional transition element inside the always-present host. */
function renderedTransition(handle: { root(): HTMLElement }): HTMLElement {
  const element = handle.root().querySelector('[data-vize-ui="transition"]');
  assert.ok(element instanceof HTMLElement, "transition content must be rendered");
  return element;
}

test("keeps unmounted content out of the tree", () => {
  const handle = mountInteraction(Transition, {
    props: { present: false },
    slots: { default: "Hidden overlay" },
  });
  assert.equal(handle.root().getAttribute("data-vize-ui"), "transition-host");
  assert.equal(handle.root().querySelector('[data-vize-ui="transition"]'), null);
  handle.unmount();
});

test("enters through an explicit completion step", async () => {
  const handle = mountInteraction(Transition, {
    props: { present: false, timeoutPadding: 50_000 },
    slots: { default: "Dialog" },
  });

  await handle.wrapper.setProps({ present: true });
  const exposed = handle.exposes<{
    completeAnimation: () => void;
    status: { readonly value: string };
  }>();
  assert.equal(renderedTransition(handle).getAttribute("data-vize-transition"), "entering");
  assert.equal(renderedTransition(handle).textContent, "Dialog");
  exposed.completeAnimation();
  await nextTick();
  assert.equal(renderedTransition(handle).getAttribute("data-vize-transition"), "present");
  handle.unmount();
});

test("auto-completes when CSS motion duration is 0", async () => {
  const present = ref(false);
  const controller = createTransition({ present });
  present.value = true;
  assert.equal(controller.status.value, "entering");
  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.equal(controller.status.value, "present");
  controller.dispose();
});

test("exits through an explicit completion step", async () => {
  const handle = mountInteraction(Transition, {
    props: { present: true, timeoutPadding: 50_000 },
    slots: { default: "Dialog" },
  });

  assert.equal(renderedTransition(handle).getAttribute("data-vize-transition"), "present");
  await handle.wrapper.setProps({ present: false });
  assert.equal(renderedTransition(handle).getAttribute("data-vize-transition"), "exiting");
  handle.exposes<{ completeAnimation: () => void }>().completeAnimation();
  await nextTick();
  assert.equal(handle.root().querySelector('[data-vize-ui="transition"]'), null);
  handle.unmount();
});

test("skips motion when the user prefers it", () => {
  const original = globalThis.matchMedia;
  globalThis.matchMedia = ((query: string) => ({
    matches: query.includes("prefers-reduced-motion"),
    media: query,
    addEventListener() {},
    removeEventListener() {},
  })) as typeof matchMedia;

  try {
    const present = ref(false);
    const controller = createTransition({ present, respectReducedMotion: true });
    present.value = true;
    assert.equal(controller.status.value, "present");
    present.value = false;
    assert.equal(controller.status.value, "unmounted");
    controller.dispose();
  } finally {
    globalThis.matchMedia = original;
  }
});

test("force-mounts hidden content", () => {
  const handle = mountInteraction(Transition, {
    props: { present: false, forceMount: true },
    slots: { default: "Pre-rendered" },
  });

  assert.equal(renderedTransition(handle).getAttribute("data-vize-transition"), "unmounted");
  assert.equal(renderedTransition(handle).textContent, "Pre-rendered");
  handle.unmount();
});

test("exposes the rendered element for composition", () => {
  const handle = mountInteraction(Transition, {
    props: { present: true },
    slots: { default: "Visible" },
  });
  const exposed = handle.exposes<{ element: HTMLElement | null }>();
  assert.ok(exposed.element === renderedTransition(handle));
  handle.unmount();
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => useTransition(), /VIZE_UI_TRANSITION_SETUP/);
});

test("disposes with the current effect scope", () => {
  const scope = effectScope();
  const controller = scope.run(() => useTransition({ present: true }));
  assert.equal(controller?.status.value, "present");
  scope.stop();
  assert.throws(() => controller?.completeAnimation(), /VIZE_UI_TRANSITION_DISPOSED/);
});
