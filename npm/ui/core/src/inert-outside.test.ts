import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createInertOutside, useInertOutside } from "./inert-outside.ts";
import { isIsolated, mountInertOutside } from "./inert-outside-test-utils.ts";

test("isolates the smallest sibling subtrees and restores them exactly", () => {
  const harness = mountInertOutside({}, false);
  harness.before.setAttribute("aria-hidden", "false");
  harness.after.setAttribute("inert", "");
  harness.controller.activate();
  assert.deepEqual(harness.controller.affectedElements.value, [
    harness.before,
    harness.beforeInside,
    harness.afterInside,
    harness.after,
  ]);
  assert.equal(harness.app.hasAttribute("inert"), false);
  assert.equal(harness.root.hasAttribute("aria-hidden"), false);
  for (const element of harness.controller.affectedElements.value) {
    assert.equal(isIsolated(element), true);
  }
  harness.controller.deactivate();
  assert.equal(harness.before.getAttribute("aria-hidden"), "false");
  assert.equal(harness.before.hasAttribute("inert"), false);
  assert.equal(harness.after.hasAttribute("inert"), true);
  assert.equal(harness.after.hasAttribute("aria-hidden"), false);
  harness.unmount();
});

test("branches preserve portalled content without exposing its siblings", () => {
  const portal = document.createElement("div");
  const portalSibling = document.createElement("div");
  document.body.append(portal, portalSibling);
  const harness = mountInertOutside({ branches: [portal] });
  assert.equal(portal.hasAttribute("inert"), false);
  assert.equal(isIsolated(portalSibling), true);
  harness.unmount();
  portal.remove();
  portalSibling.remove();
});

test("a nested portalled layer temporarily isolates its parent layer", () => {
  const parent = mountInertOutside();
  const childRoot = document.createElement("div");
  document.body.append(childRoot);
  const child = createInertOutside({ root: childRoot });
  child.activate();
  assert.equal(isIsolated(parent.app), true);
  assert.equal(childRoot.hasAttribute("inert"), false);
  child.deactivate();
  assert.equal(parent.root.hasAttribute("inert"), false);
  assert.equal(isIsolated(parent.before), true);
  child.dispose();
  childRoot.remove();
  parent.unmount();
});

test("nested layers merge independent masks and unwind them in stack order", () => {
  const parent = mountInertOutside({ mode: "aria-hidden" });
  const childRoot = document.createElement("div");
  document.body.append(childRoot);
  const child = createInertOutside({ mode: "inert", root: childRoot });
  child.activate();
  assert.equal(parent.before.getAttribute("aria-hidden"), "true");
  assert.equal(parent.before.hasAttribute("inert"), true);
  assert.equal(parent.app.getAttribute("aria-hidden"), null);
  assert.equal(parent.app.hasAttribute("inert"), true);
  assert.equal(childRoot.hasAttribute("aria-hidden"), false);
  assert.equal(childRoot.hasAttribute("inert"), false);
  child.deactivate();
  assert.equal(parent.before.getAttribute("aria-hidden"), "true");
  assert.equal(parent.before.hasAttribute("inert"), false);
  assert.equal(parent.app.hasAttribute("inert"), false);
  child.dispose();
  childRoot.remove();
  parent.unmount();
});

test("reactive portal branches are acquired and released without exposing siblings", () => {
  const portal = document.createElement("div");
  const portalSibling = document.createElement("div");
  document.body.append(portal, portalSibling);
  const branches = ref<readonly Element[]>([]);
  const harness = mountInertOutside({ branches });
  assert.equal(isIsolated(portal), true);
  branches.value = [portal];
  assert.equal(portal.hasAttribute("inert"), false);
  assert.equal(isIsolated(portalSibling), true);
  branches.value = [];
  assert.equal(isIsolated(portal), true);
  harness.unmount();
  assert.equal(portal.hasAttribute("inert"), false);
  portal.remove();
  portalSibling.remove();
});

test("reactive mode and enablement recompute without losing original attributes", () => {
  const enabled = ref(true);
  const mode = ref<"aria-hidden" | "inert">("aria-hidden");
  const harness = mountInertOutside({ enabled, mode });
  assert.equal(harness.before.getAttribute("aria-hidden"), "true");
  assert.equal(harness.before.hasAttribute("inert"), false);
  mode.value = "inert";
  assert.equal(harness.before.hasAttribute("aria-hidden"), false);
  assert.equal(harness.before.hasAttribute("inert"), true);
  enabled.value = false;
  assert.equal(harness.before.hasAttribute("inert"), false);
  enabled.value = true;
  assert.equal(harness.before.hasAttribute("inert"), true);
  harness.unmount();
});

test("new rendered siblings are isolated in a batched mutation turn", async () => {
  const harness = mountInertOutside();
  const added = document.createElement("div");
  document.body.append(added);
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  assert.equal(isIsolated(added), true);
  harness.unmount();
  assert.equal(added.hasAttribute("inert"), false);
  added.remove();
});

test("external attempts to clear owned isolation are repaired without changing restoration", async () => {
  const harness = mountInertOutside();
  harness.before.removeAttribute("inert");
  harness.before.setAttribute("aria-hidden", "false");
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  assert.equal(isIsolated(harness.before), true);
  harness.controller.deactivate();
  assert.equal(harness.before.hasAttribute("inert"), false);
  assert.equal(harness.before.hasAttribute("aria-hidden"), false);
  harness.unmount();
});

test("a nullable root is inert until attached and root replacement is reactive", () => {
  const harness = mountInertOutside({}, false);
  const original = harness.root;
  harness.rootRef.value = null;
  harness.controller.activate();
  assert.deepEqual(harness.controller.affectedElements.value, []);
  harness.rootRef.value = original;
  assert.equal(isIsolated(harness.before), true);
  const replacement = document.createElement("div");
  harness.app.append(replacement);
  harness.rootRef.value = replacement;
  assert.equal(isIsolated(original), true);
  assert.equal(replacement.hasAttribute("inert"), false);
  harness.unmount();
});

test("disconnecting and reinserting the same root releases and reacquires isolation", async () => {
  const harness = mountInertOutside();
  harness.root.remove();
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  assert.deepEqual(harness.controller.affectedElements.value, []);
  assert.equal(harness.before.hasAttribute("inert"), false);
  harness.app.append(harness.root);
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  assert.equal(isIsolated(harness.before), true);
  assert.equal(harness.root.hasAttribute("inert"), false);
  harness.unmount();
});

test("open shadow roots and assigned branches follow rendered-tree paths", () => {
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open" });
  const before = document.createElement("div");
  const slot = document.createElement("slot");
  slot.name = "modal";
  const after = document.createElement("div");
  shadow.append(before, slot, after);
  const root = document.createElement("div");
  root.slot = "modal";
  host.append(root);
  document.body.append(host);
  const controller = createInertOutside({ root });
  controller.activate();
  assert.equal(isIsolated(before), true);
  assert.equal(isIsolated(after), true);
  assert.equal(host.hasAttribute("inert"), false);
  assert.equal(root.hasAttribute("inert"), false);
  controller.dispose();
  host.remove();
});

test("open shadow-root mutations and external unmasking are observed", async () => {
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open" });
  const root = document.createElement("div");
  const sibling = document.createElement("div");
  shadow.append(root, sibling);
  document.body.append(host);
  const controller = createInertOutside({ root });
  controller.activate();
  assert.equal(isIsolated(sibling), true);
  sibling.removeAttribute("inert");
  sibling.removeAttribute("aria-hidden");
  const added = document.createElement("div");
  shadow.append(added);
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  assert.equal(isIsolated(sibling), true);
  assert.equal(isIsolated(added), true);
  controller.dispose();
  assert.equal(sibling.hasAttribute("inert"), false);
  assert.equal(added.hasAttribute("aria-hidden"), false);
  host.remove();
});

test("cross-document root replacement transfers all attribute ownership", () => {
  const harness = mountInertOutside();
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const frameDocument = frame.contentDocument;
  assert.ok(frameDocument);
  const frameRoot = frameDocument.createElement("div");
  const frameOutside = frameDocument.createElement("div");
  frameDocument.body.append(frameRoot, frameOutside);
  harness.rootRef.value = frameRoot;
  assert.equal(harness.before.hasAttribute("inert"), false);
  assert.equal(isIsolated(frameOutside), true);
  harness.unmount();
  assert.equal(frameOutside.hasAttribute("inert"), false);
  frame.remove();
});

test("runtime diagnostics and disposal are explicit", () => {
  assert.throws(() => createInertOutside(null as never), /options must be an object/);
  assert.throws(
    () => createInertOutside({ root: "#modal" } as never),
    /VIZE_UI_INERT_OUTSIDE_ROOT/,
  );
  assert.throws(
    () => createInertOutside({ mode: "automatic", root: null } as never),
    /VIZE_UI_INERT_OUTSIDE_OPTION.*mode/,
  );
  const harness = mountInertOutside();
  harness.controller.dispose();
  harness.controller.dispose();
  assert.throws(() => harness.controller.refresh(), /VIZE_UI_INERT_OUTSIDE_DISPOSED/);
  harness.before.remove();
  harness.app.remove();
  harness.after.remove();
});

test("useInertOutside activates in an effect scope and disposes with it", () => {
  const root = document.createElement("div");
  const outside = document.createElement("div");
  document.body.append(root, outside);
  assert.throws(() => useInertOutside({ root }), /VIZE_UI_INERT_OUTSIDE_SETUP/);
  const scope = effectScope();
  let controller!: ReturnType<typeof useInertOutside>;
  scope.run(() => {
    controller = useInertOutside({ root });
  });
  assert.equal(isIsolated(outside), true);
  scope.stop();
  assert.equal(outside.hasAttribute("inert"), false);
  assert.throws(() => controller.activate(), /VIZE_UI_INERT_OUTSIDE_DISPOSED/);
  root.remove();
  outside.remove();
});
