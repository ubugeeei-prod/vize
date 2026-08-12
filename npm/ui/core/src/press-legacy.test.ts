import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createPress } from "./press.ts";
import type { PressController, PressEvent } from "./press.ts";

function pointer(type: string): PointerEvent {
  return new PointerEvent(type, {
    bubbles: true,
    button: 0,
    isPrimary: true,
    pointerId: 7,
    pointerType: "mouse",
  });
}

function bindPointerHost(host: HTMLElement, controller: PressController): void {
  host.addEventListener("click", controller.pressProps.onClick);
  host.addEventListener("mousedown", controller.pressProps.onMousedown);
  host.addEventListener("pointercancel", controller.pressProps.onPointercancel);
  host.addEventListener("pointerdown", controller.pressProps.onPointerdown);
}

test("restores exact selection styles and scopes pointer-focus prevention", () => {
  const host = document.createElement("button");
  document.body.append(host);
  const controller = createPress({ preventFocusOnPress: true });
  bindPointerHost(host, controller);
  host.style.setProperty("user-select", "text", "important");
  host.dispatchEvent(pointer("pointerdown"));
  assert.equal(host.style.userSelect, "none");
  host.dispatchEvent(pointer("pointercancel"));
  assert.equal(host.style.getPropertyValue("user-select"), "text");
  assert.equal(host.style.getPropertyPriority("user-select"), "important");

  const primary = new MouseEvent("mousedown", { bubbles: true, cancelable: true });
  host.dispatchEvent(primary);
  assert.equal(primary.defaultPrevented, true);
  controller.dispose();
  host.remove();

  for (const options of [
    { button: 2, isDisabled: false },
    { button: 0, isDisabled: true },
  ]) {
    const guardedHost = document.createElement("button");
    document.body.append(guardedHost);
    const guarded = createPress({
      isDisabled: options.isDisabled,
      preventFocusOnPress: true,
    });
    guardedHost.addEventListener("mousedown", guarded.pressProps.onMousedown);
    const down = new MouseEvent("mousedown", {
      bubbles: true,
      button: options.button,
      cancelable: true,
    });
    guardedHost.dispatchEvent(down);
    assert.equal(down.defaultPrevented, false);
    guarded.dispose();
    guardedHost.remove();
  }
});

test("normalizes legacy touch and suppresses its compatibility mouse sequence", () => {
  const isolated = document.implementation.createHTMLDocument("legacy touch");
  const host = isolated.createElement("button");
  isolated.body.append(host);
  const events: PressEvent[] = [];
  const controller = createPress({
    onPressStart: (event) => events.push(event),
    onPressEnd: (event) => events.push(event),
    onPress: (event) => events.push(event),
  });
  host.addEventListener("click", controller.pressProps.onClick);
  host.addEventListener("mousedown", controller.pressProps.onMousedown);
  host.addEventListener("mouseup", controller.pressProps.onMouseup);
  host.addEventListener("touchend", controller.pressProps.onTouchend);
  host.addEventListener("touchstart", controller.pressProps.onTouchstart);
  const touchEvent = (
    type: string,
    values: Array<{ clientX: number; clientY: number; identifier: number }>,
  ) => {
    const event = new Event(type, { bubbles: true }) as TouchEvent;
    const touches = Object.assign(values, {
      item: (index: number) => touches[index] ?? null,
    }) as unknown as TouchList;
    Object.defineProperty(event, "changedTouches", { value: touches });
    Object.defineProperty(event, "view", { value: null });
    return event;
  };

  host.dispatchEvent(touchEvent("touchstart", [{ identifier: 31, clientX: 4, clientY: 8 }]));
  host.dispatchEvent(
    touchEvent("touchend", [
      { identifier: 99, clientX: 100, clientY: 200 },
      { identifier: 31, clientX: 9, clientY: 12 },
    ]),
  );
  host.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  host.dispatchEvent(new MouseEvent("mouseup", { bubbles: true }));
  host.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1 }));

  assert.deepEqual(
    events.map(({ type }) => type),
    ["pressstart", "pressend", "press"],
  );
  assert.ok(events.every(({ pointerType }) => pointerType === "touch"));
  assert.deepEqual([events[0]?.x, events[0]?.y], [4, 8]);
  assert.deepEqual([events[1]?.x, events[1]?.y], [9, 12]);
  controller.dispose();
});
