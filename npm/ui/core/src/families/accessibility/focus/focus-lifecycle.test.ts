import assert from "node:assert/strict";

import { nextTick, ref } from "vue";
import { test } from "vite-plus/test";

import { createFocus } from "./focus.ts";
import { surfaceErrors } from "./focus-internal.ts";
import { mountFocus } from "./focus-test-utils.ts";

test("removing a focused host settles ownership through the mutation observer", async () => {
  const harness = mountFocus();
  harness.host.focus();
  harness.host.remove();
  await nextTick();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(harness.controller.isFocused.value, false);
  assert.deepEqual(harness.transitions, [true, false]);
  harness.controller.dispose();
});

test("a document focusin safety net settles ownership when host blur is unavailable", () => {
  const harness = mountFocus();
  harness.host.focus();
  const outside = document.createElement("button");
  document.body.append(outside);
  harness.host.removeEventListener("blur", harness.controller.focusProps.onBlur!);
  outside.focus();
  assert.equal(harness.controller.isFocused.value, false);
  outside.remove();
  harness.unmount();
});

test("invalid reactive disablement settles before surfacing its diagnostic", () => {
  const disabled = ref<boolean | string>(false);
  const harness = mountFocus({ isDisabled: disabled as never });
  harness.host.focus();
  assert.throws(() => {
    disabled.value = "invalid";
  }, /VIZE_UI_FOCUS_OPTION.*isDisabled/);
  assert.equal(harness.controller.isFocused.value, false);
  assert.equal(harness.events.at(-1)?.reason, "disabled");
  harness.unmount();
});

test("a nonreactive disabled getter is revalidated on the next focus event", () => {
  let disabled = false;
  const harness = mountFocus({ isDisabled: () => disabled });
  harness.host.focus();
  disabled = true;
  harness.host.dispatchEvent(new globalThis.FocusEvent("focus"));
  assert.equal(harness.controller.isFocused.value, false);
  assert.equal(harness.events.at(-1)?.reason, "disabled");
  harness.unmount();
});

test("listener setup failure rolls back modality ownership", () => {
  const host = document.createElement("button");
  document.body.append(host);
  const controller = createFocus();
  const OriginalObserver = window.MutationObserver;
  class FailingObserver extends OriginalObserver {
    override observe(): void {
      throw new Error("observe failed");
    }
  }
  window.MutationObserver = FailingObserver;
  host.addEventListener("focus", controller.focusProps.onFocus!);
  try {
    assert.throws(() => host.focus(), /observe failed/);
    assert.equal(controller.isFocused.value, false);
  } finally {
    window.MutationObserver = OriginalObserver;
    controller.dispose();
    host.remove();
  }
});

test("observer construction failure also rolls back the document listener", () => {
  const host = document.createElement("button");
  document.body.append(host);
  const controller = createFocus();
  const OriginalObserver = window.MutationObserver;
  class FailingObserver {
    constructor(_callback: MutationCallback) {
      throw new Error("constructor failed");
    }
  }
  window.MutationObserver = FailingObserver as unknown as typeof MutationObserver;
  host.addEventListener("focus", controller.focusProps.onFocus!);
  try {
    assert.throws(() => host.focus(), /constructor failed/);
    assert.equal(controller.isFocused.value, false);
  } finally {
    window.MutationObserver = OriginalObserver;
    controller.dispose();
    host.remove();
  }
});

test("cross-realm documents are accepted without instanceof assumptions", () => {
  const isolated = document.implementation.createHTMLDocument("focus realm");
  const host = isolated.createElement("button");
  isolated.body.append(host);
  const controller = createFocus({ autoFocus: true });
  host.addEventListener("focus", controller.focusProps.onFocus!);
  host.focus();
  controller.refresh(host);
  assert.equal(controller.isFocused.value, true);
  assert.equal(controller.isFocusVisible.value, true);
  controller.dispose();
});

test("cleanup failures do not prevent the remaining owned resources from releasing", () => {
  const harness = mountFocus();
  harness.host.focus();
  const originalRemove = document.removeEventListener.bind(document);
  let focusRemovalFailed = false;
  document.removeEventListener = function removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject | null,
    options?: boolean | EventListenerOptions,
  ): void {
    originalRemove(type, listener, options);
    if (type === "focusin" && !focusRemovalFailed) {
      focusRemovalFailed = true;
      throw new Error("focus removal failed");
    }
  };
  try {
    assert.throws(() => harness.controller.cancel(), /focus removal failed/);
    assert.equal(harness.controller.isFocused.value, false);
  } finally {
    document.removeEventListener = originalRemove;
    harness.controller.dispose();
    harness.host.remove();
  }
});

test("multiple cleanup errors remain inspectable without native AggregateError", () => {
  const OriginalAggregateError = globalThis.AggregateError;
  Object.defineProperty(globalThis, "AggregateError", { configurable: true, value: undefined });
  try {
    assert.throws(
      () => surfaceErrors([new Error("observer"), new Error("modality")], "focus failed"),
      (error: unknown) => {
        const aggregate = error as Error & { errors: unknown[] };
        assert.equal(aggregate.name, "AggregateError");
        assert.equal(aggregate.message, "focus failed");
        assert.equal(aggregate.errors.length, 2);
        return true;
      },
    );
  } finally {
    Object.defineProperty(globalThis, "AggregateError", {
      configurable: true,
      value: OriginalAggregateError,
    });
  }
});
