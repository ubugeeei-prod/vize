import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createLongPress, useLongPress } from "./long-press.ts";
import type { LongPressEvent } from "./long-press.ts";
import { elapseThreshold, mountLongPress, pointer } from "./long-press-test-utils.ts";

test("reports a stable long-press lifecycle and suppresses the compatibility click", async () => {
  const harness = mountLongPress({ threshold: 0 });
  harness.host.dispatchEvent(pointer("pointerdown"));
  assert.equal(harness.controller.isPressed.value, true);
  assert.equal(harness.controller.isLongPressed.value, false);

  await elapseThreshold();
  assert.equal(harness.controller.isPressed.value, true);
  assert.equal(harness.controller.isLongPressed.value, true);
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["longpressstart", "longpress"],
  );
  assert.ok(harness.events.every(Object.isFrozen));
  assert.deepEqual([harness.events[1]?.x, harness.events[1]?.y], [12, 18]);

  harness.host.dispatchEvent(pointer("pointerup"));
  const click = new MouseEvent("click", { bubbles: true, cancelable: true, detail: 1 });
  harness.host.dispatchEvent(click);
  assert.equal(click.defaultPrevented, true);
  assert.equal(harness.controller.isPressed.value, false);
  assert.equal(harness.controller.isLongPressed.value, false);
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["longpressstart", "longpress", "longpressend"],
  );
  assert.equal((harness.events[2] as LongPressEvent).isCanceled, false);
  harness.unmount();
});

test("delivers short, keyboard, and virtual alternatives without a long activation", () => {
  const harness = mountLongPress({ threshold: 60_000 });
  harness.host.dispatchEvent(pointer("pointerdown"));
  harness.host.dispatchEvent(pointer("pointerup"));
  harness.host.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1 }));
  harness.host.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));
  harness.host.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 }));
  harness.host.dispatchEvent(new KeyboardEvent("keyup", { bubbles: true, key: "Enter" }));

  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["longpressstart", "longpressend", "press", "press"],
  );
  assert.deepEqual(
    harness.events.filter(({ type }) => type === "press").map(({ pointerType }) => pointerType),
    ["mouse", "keyboard"],
  );
  harness.unmount();
});

test("cancels on pointer exit, manual cancellation, disablement, and disposal", async () => {
  for (const finish of ["exit", "cancel", "disable"] as const) {
    const disabled = ref(false);
    const harness = mountLongPress({ isDisabled: disabled, threshold: 60_000 });
    harness.host.dispatchEvent(pointer("pointerdown"));
    if (finish === "exit") {
      document.body.dispatchEvent(pointer("pointermove"));
    } else if (finish === "cancel") {
      assert.equal(harness.controller.cancel(), true);
    } else {
      disabled.value = true;
      harness.host.dispatchEvent(pointer("pointerup"));
    }
    assert.equal(harness.controller.isPressed.value, false);
    assert.equal((harness.events.at(-1) as LongPressEvent).isCanceled, true);
    await elapseThreshold();
    assert.equal(
      harness.events.some(({ type }) => type === "longpress"),
      false,
    );
    harness.unmount();
  }

  const disabled = ref(false);
  const triggered = mountLongPress({ isDisabled: disabled, threshold: 0 });
  triggered.host.dispatchEvent(pointer("pointerdown"));
  await elapseThreshold();
  disabled.value = true;
  triggered.host.dispatchEvent(pointer("pointerup"));
  assert.equal((triggered.events.at(-1) as LongPressEvent).isCanceled, true);
  triggered.unmount();

  const harness = mountLongPress({ threshold: 0 });
  harness.host.dispatchEvent(pointer("pointerdown"));
  await elapseThreshold();
  harness.controller.dispose();
  assert.equal(harness.controller.isPressed.value, false);
  assert.equal(harness.controller.isLongPressed.value, false);
  assert.throws(() => harness.controller.cancel(), /VIZE_UI_LONG_PRESS_DISPOSED/);
  harness.host.remove();
});

test("filters pointer families and ignores secondary or non-primary contacts", async () => {
  for (const [filter, actual, expected] of [
    ["touch", "touch", true],
    ["pen", "mouse", false],
    [undefined, "pen", true],
    [undefined, "trackpad", true],
  ] as const) {
    const harness = mountLongPress({ pointerType: filter, threshold: 0 });
    harness.host.dispatchEvent(pointer("pointerdown", actual));
    await elapseThreshold();
    assert.equal(
      harness.events.some(({ type }) => type === "longpress"),
      expected,
    );
    harness.unmount();
  }

  for (const values of [
    { button: 2, isPrimary: true },
    { button: 0, isPrimary: false },
  ]) {
    const harness = mountLongPress({ threshold: 0 });
    harness.host.dispatchEvent(pointer("pointerdown", "mouse", values));
    await elapseThreshold();
    assert.deepEqual(harness.events, []);
    harness.unmount();
  }
});

test("a triggered attempt retains ownership when another primary contact starts", async () => {
  for (const releaseOrder of ["other-first", "owner-first"] as const) {
    const harness = mountLongPress({ threshold: 0 });
    harness.host.dispatchEvent(pointer("pointerdown", "touch", { pointerId: 31 }));
    await elapseThreshold();

    harness.host.dispatchEvent(pointer("pointerdown", "touch", { pointerId: 47 }));
    assert.equal(harness.controller.isLongPressed.value, true);
    if (releaseOrder === "other-first") {
      harness.host.dispatchEvent(pointer("pointerup", "touch", { pointerId: 47 }));
      assert.equal(harness.controller.isLongPressed.value, true);
      harness.host.dispatchEvent(pointer("pointerup", "touch", { pointerId: 31 }));
    } else {
      harness.host.dispatchEvent(pointer("pointerup", "touch", { pointerId: 31 }));
      harness.host.dispatchEvent(pointer("pointerup", "touch", { pointerId: 47 }));
    }

    assert.equal(harness.controller.isLongPressed.value, false);
    assert.deepEqual(
      harness.events.map(({ type }) => type),
      ["longpressstart", "longpress", "longpressend"],
    );
    const end = harness.events.at(-1) as LongPressEvent;
    assert.equal(end.originalEvent?.type, "pointerup");
    assert.equal((end.originalEvent as PointerEvent).pointerId, 31);
    harness.unmount();
  }
});

test("guards selection and touch context menus until physical release", async () => {
  const touch = mountLongPress({ threshold: 0 });
  const previous = document.createElement("input");
  document.body.append(previous);
  previous.focus();
  touch.host.style.setProperty("user-select", "text", "important");
  touch.host.dispatchEvent(pointer("pointerdown", "touch"));
  await elapseThreshold();
  assert.equal(document.activeElement, touch.host);
  assert.equal(touch.host.style.userSelect, "none");
  const activeMenu = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
  touch.host.dispatchEvent(activeMenu);
  assert.equal(activeMenu.defaultPrevented, true);
  touch.host.dispatchEvent(pointer("pointerup", "touch"));
  assert.equal(touch.host.style.getPropertyValue("user-select"), "text");
  assert.equal(touch.host.style.getPropertyPriority("user-select"), "important");
  const trailingMenu = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
  touch.host.dispatchEvent(trailingMenu);
  assert.equal(trailingMenu.defaultPrevented, true);
  touch.unmount();
  previous.remove();

  const shortTouch = mountLongPress({ threshold: 60_000 });
  shortTouch.host.dispatchEvent(pointer("pointerdown", "touch"));
  shortTouch.host.dispatchEvent(pointer("pointerup", "touch"));
  const shortTrailingMenu = new MouseEvent("contextmenu", {
    bubbles: true,
    cancelable: true,
  });
  shortTouch.host.dispatchEvent(shortTrailingMenu);
  assert.equal(shortTrailingMenu.defaultPrevented, true);
  shortTouch.unmount();

  const mouse = mountLongPress({ threshold: 60_000 });
  mouse.host.dispatchEvent(pointer("pointerdown", "mouse"));
  const nativeMenu = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });
  mouse.host.dispatchEvent(nativeMenu);
  assert.equal(nativeMenu.defaultPrevented, false);
  mouse.unmount();
});

test("reactively resolves threshold, pointer filter, disabled state, and descriptions", async () => {
  const threshold = ref(60_000);
  const pointerType = ref<"mouse" | "touch">("touch");
  const disabled = ref(false);
  const description = ref("Hold for actions");
  const descriptionId = ref<string>();
  const harness = mountLongPress({
    accessibilityDescription: description,
    accessibilityDescriptionId: descriptionId,
    isDisabled: disabled,
    pointerType,
    threshold,
  });
  assert.equal(harness.controller.longPressProps["aria-description"], "Hold for actions");
  assert.equal(harness.controller.longPressProps["aria-describedby"], undefined);
  descriptionId.value = "long-help";
  assert.equal(harness.controller.longPressProps["aria-description"], undefined);
  assert.equal(harness.controller.longPressProps["aria-describedby"], "long-help");

  harness.host.dispatchEvent(pointer("pointerdown", "mouse"));
  assert.deepEqual(harness.events, []);
  assert.equal(harness.controller.cancel(), true);
  pointerType.value = "mouse";
  threshold.value = 0;
  harness.host.dispatchEvent(pointer("pointerdown", "mouse", { pointerId: 20 }));
  await elapseThreshold();
  assert.ok(harness.events.some(({ type }) => type === "longpress"));
  harness.host.dispatchEvent(pointer("pointerup", "mouse", { pointerId: 20 }));

  disabled.value = true;
  assert.equal(harness.controller.longPressProps["aria-description"], undefined);
  assert.equal(harness.controller.longPressProps["aria-describedby"], undefined);
  harness.unmount();
});

test("rejects invalid runtime options with stable diagnostics", () => {
  assert.throws(() => createLongPress({ threshold: -1 }), /VIZE_UI_LONG_PRESS_OPTION.*threshold/);
  assert.throws(
    () => createLongPress({ threshold: Number.POSITIVE_INFINITY }),
    /VIZE_UI_LONG_PRESS_OPTION.*threshold/,
  );
  assert.throws(
    () => createLongPress({ pointerType: "keyboard" as "mouse" }),
    /VIZE_UI_LONG_PRESS_OPTION.*pointerType/,
  );
  assert.throws(
    () => createLongPress({ onLongPress: "callback" as never }),
    /VIZE_UI_LONG_PRESS_OPTION.*onLongPress/,
  );
});

test("useLongPress requires and follows a Vue effect scope", () => {
  assert.throws(() => useLongPress(), /VIZE_UI_LONG_PRESS_SETUP/);
  const scope = effectScope();
  const controller = scope.run(() => useLongPress())!;
  scope.stop();
  assert.throws(() => controller.cancel(), /VIZE_UI_LONG_PRESS_DISPOSED/);
});
