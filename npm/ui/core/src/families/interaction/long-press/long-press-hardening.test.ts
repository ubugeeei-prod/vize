import assert from "node:assert/strict";

import { ref } from "vue";
import type { Ref } from "vue";
import { test } from "vite-plus/test";

import { elapseThreshold, mountLongPress, pointer } from "./long-press-test-utils.ts";
import { surfaceErrors } from "./long-press-internal.ts";
import type { LongPressEvent } from "./long-press.ts";

test("multiple cleanup errors remain inspectable without a native AggregateError", () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "AggregateError");
  Object.defineProperty(globalThis, "AggregateError", { configurable: true, value: undefined });
  try {
    assert.throws(
      () => surfaceErrors([new Error("release"), new Error("selection")], "cleanup failed"),
      (error: unknown) => {
        const aggregate = error as Error & { errors?: unknown[] };
        assert.equal(aggregate.name, "AggregateError");
        assert.equal(aggregate.message, "cleanup failed");
        assert.deepEqual(
          aggregate.errors?.map((reason) => (reason as Error).message),
          ["release", "selection"],
        );
        return true;
      },
    );
  } finally {
    if (descriptor) Object.defineProperty(globalThis, "AggregateError", descriptor);
  }
});

test("invalid reactive disablement cannot wedge triggered teardown", async () => {
  const disabled = ref<unknown>(false);
  const harness = mountLongPress({
    isDisabled: disabled as Ref<boolean>,
    threshold: 0,
  });
  harness.host.style.setProperty("user-select", "text", "important");
  harness.host.dispatchEvent(pointer("pointerdown", "touch"));
  await elapseThreshold();
  disabled.value = "invalid";

  let reportedError: unknown;
  const captureReportedError = (event: ErrorEvent) => {
    reportedError = event.error;
    event.preventDefault();
  };
  window.addEventListener("error", captureReportedError);
  try {
    harness.host.dispatchEvent(pointer("pointerup", "touch"));
  } catch (error) {
    // happy-dom propagates listener errors; browsers report them on Window.
    reportedError ??= error;
  } finally {
    window.removeEventListener("error", captureReportedError);
  }
  assert.match(String(reportedError), /VIZE_UI_LONG_PRESS_OPTION.*isDisabled/);
  assert.equal(harness.controller.isPressed.value, false);
  assert.equal(harness.controller.isLongPressed.value, false);
  assert.equal(harness.host.style.getPropertyValue("user-select"), "text");
  assert.equal(harness.host.style.getPropertyPriority("user-select"), "important");
  assert.equal((harness.events.at(-1) as LongPressEvent).isCanceled, true);
  harness.controller.dispose();
  harness.host.remove();
});

test("disposal finishes every teardown step and becomes terminal after a cleanup failure", async () => {
  const harness = mountLongPress({ threshold: 0 });
  harness.host.style.setProperty("user-select", "text", "important");
  harness.host.dispatchEvent(pointer("pointerdown", "touch"));
  await elapseThreshold();

  const style = harness.host.style;
  const originalRemoveProperty = style.removeProperty.bind(style);
  style.removeProperty = function removeProperty(property: string): string {
    if (property === "-webkit-user-select") throw new Error("restore failed");
    return originalRemoveProperty(property);
  };
  try {
    assert.throws(() => harness.controller.dispose(), /restore failed/);
  } finally {
    style.removeProperty = originalRemoveProperty;
  }

  assert.equal(harness.controller.isPressed.value, false);
  assert.equal(harness.controller.isLongPressed.value, false);
  assert.equal(harness.host.style.getPropertyValue("user-select"), "text");
  assert.equal(harness.host.style.getPropertyPriority("user-select"), "important");
  assert.throws(() => harness.controller.cancel(), /VIZE_UI_LONG_PRESS_DISPOSED/);
  harness.host.remove();
});
