import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { focusableElements } from "./focus-scope-dom.ts";
import { surfaceErrors } from "./focus-scope-internal.ts";
import { createFocusScope, useFocusScope } from "./focus-scope.ts";
import { mountFocusScope, tab } from "./focus-scope-test-utils.ts";

test("reactive containment takes effect without rebuilding the controller", () => {
  const contain = ref(false);
  const harness = mountFocusScope({ contain });
  harness.first.focus();
  harness.outside.focus();
  assert.equal(document.activeElement, harness.outside);
  contain.value = true;
  harness.first.focus();
  harness.outside.focus();
  assert.equal(document.activeElement, harness.first);
  contain.value = false;
  harness.outside.focus();
  assert.equal(document.activeElement, harness.outside);
  harness.unmount();
});

test("a focusable root is the contained fallback for an empty scope", () => {
  const root = document.createElement("div");
  root.tabIndex = -1;
  document.body.append(root);
  const controller = createFocusScope({ contain: true, root });
  controller.activate();
  const event = tab(root);
  assert.equal(event.defaultPrevented, true);
  assert.equal(document.activeElement, root);
  controller.dispose();
  root.remove();
});

test("hidden, inert, disabled, and collapsed content is omitted from traversal", () => {
  const root = document.createElement("div");
  const visible = document.createElement("button");
  const hidden = document.createElement("button");
  hidden.hidden = true;
  const inert = document.createElement("div");
  inert.setAttribute("inert", "");
  inert.append(document.createElement("button"));
  const details = document.createElement("details");
  const summary = document.createElement("summary");
  const summaryLink = document.createElement("a");
  summaryLink.href = "#summary";
  summary.append(summaryLink);
  const collapsed = document.createElement("button");
  const secondSummary = document.createElement("summary");
  details.append(summary, collapsed, secondSummary);
  const standaloneSummary = document.createElement("summary");
  const hiddenInput = document.createElement("input");
  hiddenInput.type = "hidden";
  hiddenInput.tabIndex = 0;
  const editable = document.createElement("div");
  editable.setAttribute("contenteditable", "");
  const disabled = document.createElement("button");
  disabled.disabled = true;
  root.append(visible, hidden, inert, details, standaloneSummary, hiddenInput, editable, disabled);
  document.body.append(root);
  assert.deepEqual(focusableElements(root), [visible, summary, summaryLink, editable]);
  const inertAncestor = document.createElement("div");
  root.replaceWith(inertAncestor);
  inertAncestor.setAttribute("inert", "");
  inertAncestor.append(root);
  assert.deepEqual(focusableElements(root), []);
  inertAncestor.remove();
});

test("stylesheet visibility and disabled fieldset inheritance follow browser semantics", () => {
  const style = document.createElement("style");
  style.textContent = ".focus-scope-test-hidden { display: none; }";
  document.head.append(style);
  const root = document.createElement("div");
  const hidden = document.createElement("button");
  hidden.className = "focus-scope-test-hidden";
  const fieldset = document.createElement("fieldset");
  fieldset.disabled = true;
  const legend = document.createElement("legend");
  const legendButton = document.createElement("button");
  legend.append(legendButton);
  const disabledButton = document.createElement("button");
  fieldset.append(legend, disabledButton);
  root.append(hidden, fieldset);
  document.body.append(root);
  assert.deepEqual(focusableElements(root), [legendButton]);
  style.remove();
  root.remove();
});

test("slot order is composed-tree order and unassigned light content is omitted", () => {
  const root = document.createElement("div");
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open" });
  const before = document.createElement("button");
  const slot = document.createElement("slot");
  slot.name = "actions";
  const after = document.createElement("button");
  shadow.append(before, slot, after);
  const assigned = document.createElement("button");
  assigned.slot = "actions";
  const unassigned = document.createElement("button");
  host.append(unassigned, assigned);
  root.append(host);
  document.body.append(root);
  assert.deepEqual(focusableElements(root), [before, assigned, after]);
  root.remove();
});

test("radio groups expose the checked item or the first item when none is checked", () => {
  const root = document.createElement("div");
  const first = document.createElement("input");
  first.type = "radio";
  first.name = "choice";
  const second = first.cloneNode() as HTMLInputElement;
  second.checked = true;
  root.append(first, second);
  document.body.append(root);
  assert.deepEqual(focusableElements(root), [second]);
  second.checked = false;
  assert.deepEqual(focusableElements(root), [first]);
  root.remove();
});

test("same-name radio groups in separate shadow roots remain independently tabbable", () => {
  const root = document.createElement("div");
  const radios = [document.createElement("div"), document.createElement("div")].map((host) => {
    const shadow = host.attachShadow({ mode: "open" });
    const radio = document.createElement("input");
    radio.type = "radio";
    radio.name = "choice";
    shadow.append(radio);
    root.append(host);
    return radio;
  });
  document.body.append(root);
  assert.deepEqual(focusableElements(root), radios);
  root.remove();
});

test("removed restoration targets fall forward past the closing subtree", () => {
  const harness = mountFocusScope({ autoFocus: true, restoreFocus: true });
  harness.trigger.remove();
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.outside);
  harness.unmount();
});

test("non-contained scopes do not steal focus that deliberately moved elsewhere", () => {
  const harness = mountFocusScope({ restoreFocus: true });
  harness.first.focus();
  harness.outside.focus();
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.outside);
  harness.unmount();
});

test("same-document root replacement preserves nested portal ownership", () => {
  const parent = mountFocusScope({ contain: true });
  const childRoot = document.createElement("div");
  const childButton = document.createElement("button");
  childRoot.append(childButton);
  const nextRoot = document.createElement("div");
  nextRoot.append(document.createElement("button"));
  document.body.append(childRoot, nextRoot);
  const child = createFocusScope({ root: childRoot });
  child.activate();
  childButton.focus();
  parent.rootRef.value = nextRoot;
  parent.outside.focus();
  assert.equal(document.activeElement, childButton);
  child.dispose();
  childRoot.remove();
  parent.unmount();
  nextRoot.remove();
});

test("cross-document root replacement transfers containment and restores its invoker", () => {
  const harness = mountFocusScope({ contain: true, restoreFocus: true });
  const frame = document.createElement("iframe");
  document.body.append(frame);
  const frameDocument = frame.contentDocument;
  assert.ok(frameDocument);
  const frameRoot = frameDocument.createElement("div");
  const frameButton = frameDocument.createElement("button");
  const frameOutside = frameDocument.createElement("button");
  frameRoot.append(frameButton);
  frameDocument.body.append(frameRoot, frameOutside);
  harness.rootRef.value = frameRoot;
  assert.equal(harness.controller.focusFirst(), frameButton);
  frameOutside.focus();
  assert.equal(frameDocument.activeElement, frameButton);
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.trigger);
  harness.unmount();
  frame.remove();
});

test("reactivation captures fresh entry and restoration state", () => {
  const harness = mountFocusScope({ autoFocus: true, restoreFocus: true }, false);
  harness.controller.activate();
  assert.equal(document.activeElement, harness.first);
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.trigger);
  harness.outside.focus();
  harness.controller.activate();
  assert.equal(document.activeElement, harness.first);
  harness.controller.deactivate();
  assert.equal(document.activeElement, harness.outside);
  harness.unmount();
});

test("entry callback and focus failures aggregate and roll activation back", () => {
  const callbackFailure = new Error("callback failed");
  const focusFailure = new Error("focus failed");
  const harness = mountFocusScope(
    {
      autoFocus: true,
      onMountAutoFocus: () => {
        throw callbackFailure;
      },
    },
    false,
  );
  harness.first.focus = () => {
    throw focusFailure;
  };
  assert.throws(
    () => harness.controller.activate(),
    (error) => {
      assert.equal((error as Error).name, "AggregateError");
      assert.deepEqual((error as AggregateError).errors, [callbackFailure, focusFailure]);
      return true;
    },
  );
  assert.equal(harness.controller.isActive.value, false);
  harness.unmount();
});

test("restoration callback and focus failures aggregate after cleanup", () => {
  const callbackFailure = new Error("callback failed");
  const focusFailure = new Error("focus failed");
  const harness = mountFocusScope({
    autoFocus: true,
    restoreFocus: true,
    onUnmountAutoFocus: () => {
      throw callbackFailure;
    },
  });
  harness.trigger.focus = () => {
    throw focusFailure;
  };
  assert.throws(
    () => harness.controller.deactivate(),
    (error) => {
      assert.equal((error as Error).name, "AggregateError");
      assert.deepEqual((error as AggregateError).errors, [callbackFailure, focusFailure]);
      return true;
    },
  );
  assert.equal(harness.controller.isActive.value, false);
  harness.outside.focus();
  assert.equal(document.activeElement, harness.outside);
  harness.unmount();
});

test("invalid runtime options are diagnostic and disposal still detaches containment", () => {
  assert.throws(() => createFocusScope(null as never), /options must be an object/);
  assert.throws(
    () => createFocusScope({ root: "not-an-element" } as never),
    /VIZE_UI_FOCUS_SCOPE_ROOT/,
  );
  const invalidTarget = mountFocusScope(
    { autoFocus: true, initialFocus: () => "not-an-element" as never },
    false,
  );
  assert.throws(
    () => invalidTarget.controller.activate(),
    /VIZE_UI_FOCUS_SCOPE_OPTION.*initialFocus/,
  );
  assert.equal(invalidTarget.controller.isActive.value, false);
  invalidTarget.unmount();
  const contain = ref<boolean | string>(true);
  const harness = mountFocusScope({ contain: contain as never });
  contain.value = "invalid";
  assert.throws(() => harness.controller.deactivate(), /VIZE_UI_FOCUS_SCOPE_OPTION.*contain/);
  assert.equal(harness.controller.isActive.value, false);
  harness.outside.focus();
  assert.equal(document.activeElement, harness.outside);
  harness.unmount();
});

test("movement validation and disposed diagnostics are explicit", () => {
  const harness = mountFocusScope();
  assert.throws(
    () => harness.controller.focusFirst({ wrap: "yes" } as never),
    /VIZE_UI_FOCUS_SCOPE_OPTION.*wrap/,
  );
  harness.controller.dispose();
  harness.controller.dispose();
  assert.throws(() => harness.controller.focusFirst(), /VIZE_UI_FOCUS_SCOPE_DISPOSED/);
  harness.trigger.remove();
  harness.root.remove();
  harness.outside.remove();
});

test("useFocusScope activates in an effect scope and disposes with it", () => {
  const root = document.createElement("div");
  root.append(document.createElement("button"));
  document.body.append(root);
  assert.throws(() => useFocusScope({ root }), /VIZE_UI_FOCUS_SCOPE_SETUP/);
  const scope = effectScope();
  let controller!: ReturnType<typeof useFocusScope>;
  scope.run(() => {
    controller = useFocusScope({ root });
  });
  assert.equal(controller.isActive.value, true);
  scope.stop();
  assert.throws(() => controller.refresh(), /VIZE_UI_FOCUS_SCOPE_DISPOSED/);
  root.remove();
});

test("fallback aggregation works without native AggregateError", () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "AggregateError");
  const failures = [new Error("first"), new Error("second")];
  Object.defineProperty(globalThis, "AggregateError", { configurable: true, value: undefined });
  try {
    assert.throws(
      () => surfaceErrors(failures, "failed"),
      (error) => {
        assert.equal((error as Error).name, "AggregateError");
        assert.deepEqual((error as Error & { errors: unknown[] }).errors, failures);
        return true;
      },
    );
  } finally {
    if (descriptor) Object.defineProperty(globalThis, "AggregateError", descriptor);
  }
});
