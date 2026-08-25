import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, nextTick, ref } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import { createPresence, usePresence } from "./presence.ts";
import Presence from "./presence.vue";

test("keeps unmounted content out of the tree", () => {
  const handle = mountInteraction(Presence, {
    props: { present: false },
    slots: { default: "Hidden overlay" },
  });

  assert.equal(handle.wrapper.element.nodeType, Node.COMMENT_NODE);
  handle.unmount();
});

test("enters through an explicit completion step", async () => {
  const handle = mountInteraction(Presence, {
    props: { present: false },
    slots: { default: "Dialog" },
  });

  await handle.wrapper.setProps({ present: true });
  const exposed = handle.exposes<{
    completeAnimation: () => void;
    status: { readonly value: string };
  }>();
  assert.equal(handle.root().getAttribute("data-vize-presence"), "entering");
  assert.equal(handle.root().textContent, "Dialog");
  exposed.completeAnimation();
  await nextTick();
  assert.equal(handle.root().getAttribute("data-vize-presence"), "present");
  handle.unmount();
});

test("exits through an explicit completion step", async () => {
  const handle = mountInteraction(Presence, {
    props: { present: true },
    slots: { default: "Dialog" },
  });

  assert.equal(handle.root().getAttribute("data-vize-presence"), "present");
  await handle.wrapper.setProps({ present: false });
  assert.equal(handle.root().getAttribute("data-vize-presence"), "exiting");
  handle.exposes<{ completeAnimation: () => void }>().completeAnimation();
  await nextTick();
  assert.equal(handle.wrapper.element.nodeType, Node.COMMENT_NODE);
  handle.unmount();
});

test("cancels an in-flight enter", async () => {
  const present = ref(false);
  const controller = createPresence({ present });
  present.value = true;
  assert.equal(controller.status.value, "entering");
  present.value = false;
  assert.equal(controller.status.value, "unmounted");
  assert.equal(controller.isPresent.value, false);
  controller.dispose();
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
    const controller = createPresence({ present, respectReducedMotion: true });
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
  const handle = mountInteraction(Presence, {
    props: { present: false, forceMount: true },
    slots: { default: "Pre-rendered" },
  });

  assert.equal(handle.root().getAttribute("data-vize-presence"), "unmounted");
  assert.equal(handle.root().textContent, "Pre-rendered");
  handle.unmount();
});

test("exposes the rendered element for composition", () => {
  const handle = mountInteraction(Presence, {
    props: { present: true },
    slots: { default: "Visible" },
  });
  const exposed = handle.exposes<{ element: HTMLElement | null }>();
  assert.ok(exposed.element === handle.root());
  handle.unmount();
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => usePresence(), /VIZE_UI_PRESENCE_SETUP/);
});

test("disposes with the current effect scope", () => {
  const scope = effectScope();
  const controller = scope.run(() => usePresence({ present: true }));
  assert.equal(controller?.status.value, "present");
  scope.stop();
  assert.throws(() => controller?.completeAnimation(), /VIZE_UI_PRESENCE_DISPOSED/);
});
