import assert from "node:assert/strict";

import { effectScope, nextTick, ref } from "vue";
import { test } from "vite-plus/test";

import { createFocusGuards, focusGuardPreset, useFocusGuards } from "./focus-guards.ts";
import { mountFocusGuards, pressTab } from "./focus-guards-test-utils.ts";

test("wraps forward and backward focus with immutable redirect evidence", () => {
  const harness = mountFocusGuards();
  harness.first.focus();
  pressTab(true);
  harness.before.focus();
  assert.equal(document.activeElement, harness.last);
  assert.equal(harness.events[0]?.position, "before");
  assert.equal(harness.events[0]?.direction, "backward");
  assert.equal(harness.events[0]?.reason, "wrap");
  assert.equal(harness.events[0]?.relatedTarget, harness.first);
  assert.ok(Object.isFrozen(harness.events[0]));

  pressTab();
  harness.after.focus();
  assert.equal(document.activeElement, harness.first);
  assert.equal(harness.events[1]?.position, "after");
  assert.equal(harness.events[1]?.direction, "forward");
  harness.unmount();
});

test("enters at the logical endpoint when a guard is reached from outside", () => {
  const harness = mountFocusGuards();
  harness.outsideBefore.focus();
  harness.before.focus();
  assert.equal(document.activeElement, harness.first);
  assert.equal(harness.events[0]?.reason, "enter");
  assert.equal(harness.events[0]?.direction, "forward");

  harness.outsideAfter.focus();
  harness.after.focus();
  assert.equal(document.activeElement, harness.last);
  assert.equal(harness.events[1]?.reason, "enter");
  assert.equal(harness.events[1]?.direction, "backward");
  harness.unmount();
});

test("related focus ownership overrides stale keyboard-direction evidence", () => {
  const harness = mountFocusGuards();
  pressTab();
  harness.last.focus();
  harness.before.focus();
  assert.equal(document.activeElement, harness.last);
  assert.equal(harness.events[0]?.direction, "backward");
  assert.equal(harness.events[0]?.reason, "wrap");
  harness.unmount();
});

test("orders positive tabindex across portalled and open-shadow regions", () => {
  const branch = document.createElement("div");
  const shadowHost = document.createElement("div");
  const shadow = shadowHost.attachShadow({ mode: "open" });
  const shadowButton = document.createElement("button");
  shadowButton.tabIndex = 1;
  shadow.append(shadowButton);
  branch.append(shadowHost);
  document.body.append(branch);
  const branches = ref<readonly Element[]>([branch]);
  const harness = mountFocusGuards({ branches });
  harness.first.tabIndex = 2;
  harness.last.focus();
  pressTab();
  harness.after.focus();
  assert.equal(shadow.activeElement, shadowButton);

  const ignored = document.createElement("button");
  ignored.tabIndex = -1;
  branch.append(ignored);
  harness.controller.refresh();
  harness.first.focus();
  pressTab(true);
  harness.before.focus();
  assert.notEqual(document.activeElement, ignored);
  harness.unmount();
  branch.remove();
});

test("the topmost connected owner alone exposes sequential guards", async () => {
  const parent = mountFocusGuards();
  const child = mountFocusGuards();
  assert.equal(parent.controller.isGuarding.value, false);
  assert.equal(parent.controller.beforeProps.tabindex, -1);
  assert.equal(child.controller.isGuarding.value, true);
  const replacement = document.createElement("div");
  replacement.append(document.createElement("button"));
  document.body.append(replacement);
  parent.rootRef.value = replacement;
  assert.equal(parent.controller.isGuarding.value, false);
  assert.equal(child.controller.isGuarding.value, true);
  child.controller.deactivate();
  assert.equal(parent.controller.isGuarding.value, true);

  replacement.remove();
  await nextTick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(parent.controller.isGuarding.value, false);
  document.body.append(replacement);
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(parent.controller.isGuarding.value, true);
  child.unmount();
  parent.unmount();
  replacement.remove();
});

test("reactive roots, branches, enablement, and cross-document migration recompute", () => {
  const enabled = ref(true);
  const branches = ref<readonly Element[]>([]);
  const root = ref<Element | null>(null);
  const controller = createFocusGuards({ branches, enabled, root });
  controller.activate();
  assert.equal(controller.isActive.value, true);
  assert.equal(controller.isGuarding.value, false);
  const localRoot = document.createElement("div");
  document.body.append(localRoot);
  root.value = localRoot;
  assert.equal(controller.isGuarding.value, true);
  enabled.value = false;
  assert.equal(controller.isGuarding.value, false);
  enabled.value = true;

  const frame = document.createElement("iframe");
  document.body.append(frame);
  const frameDocument = frame.contentDocument;
  assert.ok(frameDocument);
  const frameRoot = frameDocument.createElement("div");
  frameDocument.body.append(frameRoot);
  root.value = frameRoot;
  assert.equal(controller.isGuarding.value, true);
  assert.throws(() => {
    branches.value = [localRoot];
  }, /VIZE_UI_FOCUS_GUARDS_OPTION.*Document/);
  controller.dispose();
  frame.remove();
  localRoot.remove();
});

test("fallback focus and preventable redirection keep empty regions operable", () => {
  const harness = mountFocusGuards({ onRedirect: (event) => event.preventDefault() });
  harness.first.remove();
  harness.last.remove();
  harness.root.tabIndex = -1;
  harness.outsideBefore.focus();
  harness.before.focus();
  assert.equal(document.activeElement, harness.before);
  assert.equal(harness.events[0]?.defaultPrevented, true);
  harness.unmount();

  const fallback = mountFocusGuards();
  fallback.first.remove();
  fallback.last.remove();
  fallback.root.tabIndex = -1;
  fallback.before.focus();
  assert.equal(document.activeElement, fallback.root);
  fallback.unmount();
});

test("runtime diagnostics, effect ownership, disposal, and preset immutability are explicit", () => {
  assert.throws(() => createFocusGuards(null as never), /options must be an object/);
  assert.throws(
    () => createFocusGuards({ root: document.body, enabled: "yes" } as never),
    /VIZE_UI_FOCUS_GUARDS_OPTION.*enabled/,
  );
  assert.throws(
    () => createFocusGuards({ root: document.body, branches: document.body } as never),
    /VIZE_UI_FOCUS_GUARDS_OPTION.*branches/,
  );
  assert.throws(() => useFocusGuards({ root: null }), /VIZE_UI_FOCUS_GUARDS_SETUP/);
  assert.ok(Object.isFrozen(focusGuardPreset));
  assert.equal(focusGuardPreset.pointerEvents, "none");

  const scope = effectScope();
  const controller = scope.run(() => useFocusGuards({ root: document.body }))!;
  scope.stop();
  assert.equal(controller.isActive.value, false);
  assert.throws(() => controller.refresh(), /VIZE_UI_FOCUS_GUARDS_DISPOSED/);
});
