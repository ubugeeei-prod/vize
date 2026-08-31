import assert from "node:assert/strict";

import { nextTick } from "vue";
import { test } from "vite-plus/test";

import { createFocusScope } from "./focus-scope.ts";
import type { FocusScopeAutoFocusEvent } from "./focus-scope.ts";
import { mountFocusScope, tab } from "./focus-scope-test-utils.ts";

test("auto focuses on activation and restores the invoking element", () => {
  const events: FocusScopeAutoFocusEvent[] = [];
  const harness = mountFocusScope(
    {
      autoFocus: true,
      restoreFocus: true,
      onMountAutoFocus: (event) => events.push(event),
      onUnmountAutoFocus: (event) => events.push(event),
    },
    false,
  );
  harness.controller.activate();
  assert.equal(harness.controller.isActive.value, true);
  assert.equal(document.activeElement, harness.first);
  assert.equal(events[0]?.type, "mount");
  assert.equal(events[0]?.target, harness.first);
  assert.equal(events[0]?.defaultPrevented, false);
  assert.ok(Object.isFrozen(events[0]));
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.trigger);
  assert.equal(events[1]?.type, "unmount");
  assert.equal(events[1]?.target, harness.trigger);
  assert.equal(harness.controller.isActive.value, false);
  harness.unmount();
});

test("preventable entry and restoration events retain consumer-owned focus", () => {
  const harness = mountFocusScope(
    {
      autoFocus: true,
      restoreFocus: true,
      onMountAutoFocus: (event) => event.preventDefault(),
      onUnmountAutoFocus: (event) => event.preventDefault(),
    },
    false,
  );
  harness.controller.activate();
  assert.equal(document.activeElement, harness.trigger);
  harness.first.focus();
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.first);
  harness.unmount();
});

test("explicit initial and restoration targets override discovery", () => {
  const harness = mountFocusScope(
    {
      autoFocus: true,
      restoreFocus: true,
      initialFocus: () => harness.last,
      restoreTarget: () => harness.outside,
    },
    false,
  );
  harness.controller.activate();
  assert.equal(document.activeElement, harness.last);
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.outside);
  harness.unmount();
});

test("an unusable preferred target falls back to the first eligible descendant", () => {
  const harness = mountFocusScope({ autoFocus: true, initialFocus: () => harness.last }, false);
  harness.last.disabled = true;
  harness.controller.activate();
  assert.equal(document.activeElement, harness.first);
  harness.unmount();
});

test("containment wraps Tab and Shift+Tab at sequential boundaries", () => {
  const harness = mountFocusScope({ contain: true });
  harness.last.focus();
  const forward = tab(harness.last);
  assert.equal(forward.defaultPrevented, true);
  assert.equal(document.activeElement, harness.first);
  const backward = tab(harness.first, true);
  assert.equal(backward.defaultPrevented, true);
  assert.equal(document.activeElement, harness.last);
  harness.first.focus();
  assert.equal(tab(harness.first).defaultPrevented, false);
  harness.unmount();
});

test("programmatic focus escape is synchronously recovered to the last in-scope target", () => {
  const harness = mountFocusScope({ contain: true });
  harness.last.focus();
  harness.outside.focus();
  assert.equal(document.activeElement, harness.last);
  harness.unmount();
});

test("focus manager follows tab order and optionally includes programmatic targets", () => {
  const harness = mountFocusScope();
  const programmaticButton = document.createElement("button");
  programmaticButton.tabIndex = -1;
  harness.root.insertBefore(programmaticButton, harness.last);
  harness.last.tabIndex = 1;
  harness.first.tabIndex = 2;
  assert.equal(harness.controller.focusFirst(), harness.last);
  assert.equal(harness.controller.focusNext(), harness.first);
  assert.equal(harness.controller.focusNext(), null);
  assert.equal(harness.controller.focusNext({ wrap: true }), harness.last);
  assert.equal(
    harness.controller.focusNext({ from: harness.first, includeProgrammatic: true }),
    harness.programmatic,
  );
  assert.equal(
    harness.controller.focusNext({ from: harness.programmatic, includeProgrammatic: true }),
    programmaticButton,
  );
  assert.equal(harness.controller.focusPrevious({ wrap: true }), harness.first);
  harness.unmount();
});

test("scope and per-movement filters compose without exposing disabled content", () => {
  const harness = mountFocusScope({ accept: (element) => element !== harness.first });
  assert.equal(harness.controller.focusFirst(), harness.last);
  assert.equal(
    harness.controller.focusFirst({ accept: (element) => element !== harness.last }),
    null,
  );
  harness.unmount();
});

test("open shadow roots participate in traversal and deep focus containment", () => {
  const harness = mountFocusScope({ contain: true });
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open" });
  const shadowButton = document.createElement("button");
  shadowButton.textContent = "shadow";
  shadow.append(shadowButton);
  harness.root.insertBefore(host, harness.last);
  assert.equal(harness.controller.focusNext({ from: harness.first }), shadowButton);
  assert.equal(harness.root.getRootNode(), document);
  harness.outside.focus();
  assert.equal(shadow.activeElement, shadowButton);
  harness.unmount();
});

test("nested and portalled scopes preserve parent containment and restoration", () => {
  const parent = mountFocusScope({ contain: true, restoreFocus: true });
  parent.first.focus();
  const childRoot = document.createElement("div");
  const childButton = document.createElement("button");
  childRoot.append(childButton);
  document.body.append(childRoot);
  const child = createFocusScope({ root: childRoot, restoreFocus: true });
  child.activate();
  childButton.focus();
  assert.equal(document.activeElement, childButton);
  parent.outside.focus();
  assert.equal(document.activeElement, childButton);
  child.deactivate();
  assert.equal(document.activeElement, parent.first);
  child.dispose();
  childRoot.remove();
  parent.unmount();
});

test("a nested containing scope temporarily owns containment", () => {
  const parent = mountFocusScope({ contain: true });
  parent.first.focus();
  const childRoot = document.createElement("div");
  const childButton = document.createElement("button");
  childRoot.append(childButton);
  document.body.append(childRoot);
  const child = createFocusScope({ contain: true, root: childRoot });
  child.activate();
  childButton.focus();
  parent.last.focus();
  assert.equal(document.activeElement, childButton);
  child.dispose();
  childRoot.remove();
  parent.unmount();
});

test("a null SSR root activates and attaches exactly once after hydration", () => {
  const harness = mountFocusScope({ autoFocus: true, restoreFocus: true }, false);
  const root = harness.root;
  harness.rootRef.value = null;
  const controller = harness.controller;
  controller.activate();
  assert.equal(controller.isActive.value, true);
  assert.equal(document.activeElement, harness.trigger);
  harness.rootRef.value = root;
  assert.equal(document.activeElement, harness.first);
  controller.deactivate();
  assert.equal(document.activeElement, harness.trigger);
  harness.unmount();
});

test("removing the last focused target recovers to the first remaining control", async () => {
  const harness = mountFocusScope({ contain: true });
  harness.last.focus();
  harness.last.remove();
  await nextTick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(document.activeElement, harness.first);
  harness.unmount();
});

test("class-based visibility changes recover containment", async () => {
  const style = document.createElement("style");
  style.textContent = ".focus-scope-current-hidden { display: none; }";
  document.head.append(style);
  const harness = mountFocusScope({ contain: true });
  harness.last.focus();
  harness.last.className = "focus-scope-current-hidden";
  await nextTick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(document.activeElement, harness.first);
  harness.unmount();
  style.remove();
});
