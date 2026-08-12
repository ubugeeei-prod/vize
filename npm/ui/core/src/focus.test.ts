import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import {
  createFocus,
  createFocusRing,
  createFocusWithin,
  useFocus,
  useFocusRing,
  useFocusWithin,
} from "./focus.ts";
import type { FocusController } from "./focus.ts";
import { forceModalityFallback, mountFocus } from "./focus-test-utils.ts";

function keydown(key = "Tab"): void {
  document.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key }));
}

function pointerdown(): void {
  document.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, pointerId: 1, pointerType: "mouse" }),
  );
}

test("publishes immutable direct-focus snapshots and modality-aware ring state", () => {
  const harness = mountFocus();
  const restoreMatches = forceModalityFallback(harness.host);
  pointerdown();
  harness.host.focus();

  assert.equal(harness.controller.isFocused.value, true);
  assert.equal(harness.controller.isFocusVisible.value, false);
  assert.deepEqual(harness.transitions, [true]);
  assert.equal(harness.events[0]?.type, "focus");
  assert.equal(harness.events[0]?.target, harness.host);
  assert.equal(harness.events[0]?.focusedTarget, harness.host);
  assert.equal(harness.events[0]?.reason, "focus");
  assert.ok(harness.events[0]?.originalEvent instanceof globalThis.FocusEvent);
  assert.ok(Object.isFrozen(harness.events[0]));

  keydown();
  assert.equal(harness.controller.isFocusVisible.value, true);
  harness.host.blur();
  assert.equal(harness.controller.isFocused.value, false);
  assert.equal(harness.controller.isFocusVisible.value, false);
  assert.deepEqual(harness.transitions, [true, false]);
  assert.equal(harness.events.at(-1)?.type, "blur");
  restoreMatches();
  harness.unmount();
});

test("autoFocus forces a ring while focused and factory aliases preserve the contract", () => {
  for (const factory of [createFocus, createFocusRing] as const) {
    const host = document.createElement("button");
    document.body.append(host);
    const controller = factory({ autoFocus: true });
    host.addEventListener("focus", controller.focusProps.onFocus!);
    pointerdown();
    host.focus();
    assert.equal(controller.isFocused.value, true);
    assert.equal(controller.isFocusVisible.value, true);
    controller.dispose();
    host.remove();
  }
});

test("target mode ignores descendant focus while within mode owns composed descendants", () => {
  const direct = mountFocus({}, "div");
  const child = document.createElement("button");
  direct.host.append(child);
  child.focus();
  assert.equal(direct.controller.isFocused.value, false);
  direct.unmount();

  const within = mountFocus({ mode: "within" }, "div");
  const first = document.createElement("button");
  const second = document.createElement("button");
  const outside = document.createElement("button");
  within.host.append(first, second);
  document.body.append(outside);
  first.focus();
  assert.equal(within.controller.isFocused.value, true);
  assert.equal(within.events[0]?.focusedTarget, first);
  second.focus();
  assert.equal(within.controller.isFocused.value, true);
  assert.deepEqual(within.transitions, [true]);
  outside.focus();
  assert.equal(within.controller.isFocused.value, false);
  assert.equal(within.events.at(-1)?.relatedTarget, outside);
  assert.deepEqual(within.transitions, [true, false]);
  outside.remove();
  within.unmount();
});

test("focus within resolves deep active elements through an open shadow root", () => {
  const harness = mountFocus({ mode: "within" }, "div");
  const shadow = harness.host.attachShadow({ mode: "open" });
  const button = document.createElement("button");
  shadow.append(button);
  button.focus();
  harness.controller.refresh(harness.host);
  assert.equal(harness.controller.isFocused.value, true);
  assert.equal(harness.events[0]?.focusedTarget, button);
  harness.unmount();
});

test("reactive disablement settles first and refresh explicitly reacquires DOM focus", () => {
  const disabled = ref(false);
  const harness = mountFocus({ isDisabled: disabled });
  harness.host.focus();
  disabled.value = true;
  assert.equal(harness.controller.isFocused.value, false);
  assert.equal(document.activeElement, harness.host);
  assert.equal(harness.events.at(-1)?.reason, "disabled");

  disabled.value = false;
  assert.equal(harness.controller.refresh(harness.host), true);
  assert.equal(harness.controller.isFocused.value, true);
  assert.equal(harness.events.at(-1)?.reason, "refresh");
  assert.equal(harness.controller.refresh(harness.host), false);
  harness.unmount();
});

test("manual cancellation keeps DOM focus and disposal is terminal without callbacks", () => {
  const harness = mountFocus();
  harness.host.focus();
  assert.equal(harness.controller.cancel(), true);
  assert.equal(document.activeElement, harness.host);
  assert.equal(harness.events.at(-1)?.reason, "manual");
  assert.equal(harness.controller.cancel(), false);
  const transitionCount = harness.transitions.length;
  harness.controller.refresh(harness.host);
  harness.controller.dispose();
  assert.equal(harness.controller.isFocused.value, false);
  assert.equal(harness.transitions.length, transitionCount + 1);
  assert.throws(() => harness.controller.cancel(), /VIZE_UI_FOCUS_DISPOSED/);
  assert.throws(() => harness.controller.refresh(harness.host), /VIZE_UI_FOCUS_DISPOSED/);
  harness.host.remove();
});

test("reentrant changes cannot publish a stale phase callback", () => {
  const transitions: boolean[] = [];
  const phases: string[] = [];
  let controller!: FocusController;
  const host = document.createElement("button");
  document.body.append(host);
  controller = createFocus({
    onFocusChange(value) {
      transitions.push(value);
      if (value) controller.cancel();
    },
    onFocus: () => phases.push("focus"),
    onBlur: () => phases.push("blur"),
  });
  host.addEventListener("focus", controller.focusProps.onFocus!);
  host.focus();
  assert.deepEqual(transitions, [true, false]);
  assert.deepEqual(phases, ["blur"]);
  assert.equal(controller.isFocused.value, false);
  controller.dispose();
  host.remove();
});

test("callback failures preserve settled state and aggregate independent errors", () => {
  const harness = mountFocus({
    onFocusChange: () => {
      throw new Error("change failed");
    },
    onFocus: () => {
      throw new Error("focus failed");
    },
  });
  assert.throws(() => harness.host.focus(), AggregateError);
  assert.equal(harness.controller.isFocused.value, true);
  harness.controller.dispose();
  harness.host.remove();
});

test("rejects invalid runtime options with stable diagnostics", () => {
  assert.throws(
    () => createFocus({ mode: "descendants" as "within" }),
    /VIZE_UI_FOCUS_OPTION.*mode/,
  );
  assert.throws(
    () => createFocus({ autoFocus: "yes" as never }),
    /VIZE_UI_FOCUS_OPTION.*autoFocus/,
  );
  assert.throws(
    () => createFocus({ isDisabled: "false" as never }),
    /VIZE_UI_FOCUS_OPTION.*isDisabled/,
  );
  assert.throws(
    () => createFocus({ onFocus: "callback" as never }),
    /VIZE_UI_FOCUS_OPTION.*onFocus/,
  );
  const controller = createFocus();
  assert.throws(() => controller.refresh({} as Element), /VIZE_UI_FOCUS_TARGET/);
  controller.dispose();
});

test("Vue effect scopes own all focus convenience hooks", () => {
  for (const hook of [useFocus, useFocusWithin, useFocusRing] as const) {
    assert.throws(() => hook(), /VIZE_UI_FOCUS_SETUP/);
    const scope = effectScope();
    const controller = scope.run(() => hook())!;
    scope.stop();
    assert.throws(() => controller.cancel(), /VIZE_UI_FOCUS_DISPOSED/);
  }
  assert.equal(createFocusWithin().focusProps.onFocusin instanceof Function, true);
});
