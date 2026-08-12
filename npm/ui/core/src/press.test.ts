import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createPress } from "./press.ts";
import type { PressController, PressEvent, PressOptions, PressProps } from "./press.ts";

const eventNames: Readonly<Record<keyof PressProps, string>> = Object.freeze({
  onClick: "click",
  onDragstart: "dragstart",
  onKeydown: "keydown",
  onKeyup: "keyup",
  onMousedown: "mousedown",
  onMousemove: "mousemove",
  onMouseup: "mouseup",
  onPointercancel: "pointercancel",
  onPointerdown: "pointerdown",
  onPointermove: "pointermove",
  onPointerup: "pointerup",
  onTouchcancel: "touchcancel",
  onTouchend: "touchend",
  onTouchmove: "touchmove",
  onTouchstart: "touchstart",
});

interface PressHarness {
  readonly controller: PressController;
  readonly host: HTMLElement;
  readonly events: PressEvent[];
  dispatch: (event: Event, target?: EventTarget) => boolean;
  unmount: () => void;
}

function mountPress(options: PressOptions = {}, tag = "div"): PressHarness {
  const events: PressEvent[] = [];
  const host = document.createElement(tag);
  host.tabIndex = 0;
  document.body.append(host);
  const controller = createPress({
    ...options,
    onPressStart(event) {
      events.push(event);
      options.onPressStart?.(event);
    },
    onPressEnd(event) {
      events.push(event);
      options.onPressEnd?.(event);
    },
    onPressUp(event) {
      events.push(event);
      options.onPressUp?.(event);
    },
    onPress(event) {
      events.push(event);
      options.onPress?.(event);
    },
  });
  for (const [property, type] of Object.entries(eventNames) as Array<[keyof PressProps, string]>) {
    host.addEventListener(type, controller.pressProps[property] as EventListener);
  }
  return {
    controller,
    host,
    events,
    dispatch(event, target = host) {
      return target.dispatchEvent(event);
    },
    unmount() {
      controller.dispose();
      host.remove();
    },
  };
}

function pointer(type: string, init: PointerEventInit = {}): PointerEvent {
  return new PointerEvent(type, {
    bubbles: true,
    button: 0,
    isPrimary: true,
    pointerId: 7,
    pointerType: "mouse",
    ...init,
  });
}

test("normalizes a primary pointer lifecycle and emits immutable snapshots", () => {
  const changes: boolean[] = [];
  const harness = mountPress({ onPressChange: (value) => changes.push(value) });
  const down = pointer("pointerdown", { clientX: 12, clientY: 34, shiftKey: true });
  harness.dispatch(down);
  assert.equal(harness.controller.isPressed.value, true);

  harness.dispatch(pointer("pointerup"));
  harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 1 }));

  assert.equal(harness.controller.isPressed.value, false);
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["pressstart", "pressup", "pressend", "press"],
  );
  assert.deepEqual(changes, [true, false]);
  assert.ok(harness.events.every(Object.isFrozen));
  assert.equal(harness.events[0]?.originalEvent, down);
  assert.equal(harness.events[0]?.x, 12);
  assert.equal(harness.events[0]?.y, 34);
  assert.equal(harness.events[0]?.shiftKey, true);
  assert.equal(harness.events.at(-1)?.pointerType, "mouse");
  harness.unmount();
});

test("ignores non-primary contacts and secondary buttons", () => {
  const harness = mountPress();
  harness.dispatch(pointer("pointerdown", { button: 2 }));
  harness.dispatch(pointer("pointerdown", { isPrimary: false }));
  harness.dispatch(pointer("pointerdown", { pointerId: 8, pointerType: "touch" }));
  harness.dispatch(pointer("pointerdown", { pointerId: 9, pointerType: "touch" }));
  harness.dispatch(pointer("pointerup", { pointerId: 8, pointerType: "touch" }));
  harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 1 }));

  assert.equal(harness.events.filter(({ type }) => type === "press").length, 1);
  assert.equal(harness.events[0]?.pointerType, "touch");
  harness.unmount();
});

test("preserves pen, unknown, and non-pointing Pointer Events classifications", () => {
  const cases = [
    { pointerId: 1, pointerType: "pen", expected: "pen" },
    { pointerId: 2, pointerType: "vendor-device", expected: "pointer" },
    { pointerId: -1, pointerType: "", expected: "virtual" },
  ] as const;
  for (const { pointerId, pointerType, expected } of cases) {
    const harness = mountPress();
    harness.dispatch(pointer("pointerdown", { pointerId, pointerType }));
    harness.dispatch(pointer("pointerup", { pointerId, pointerType }));
    harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 0 }));
    assert.equal(harness.events.find(({ type }) => type === "press")?.pointerType, expected);
    harness.unmount();
  }
});

test("pauses outside, resumes inside, and activates only after an inside release", () => {
  const harness = mountPress();
  const outside = document.createElement("div");
  document.body.append(outside);
  harness.dispatch(pointer("pointerdown"));
  harness.dispatch(pointer("pointermove"), outside);
  assert.equal(harness.controller.isPressed.value, false);
  harness.dispatch(pointer("pointermove"));
  assert.equal(harness.controller.isPressed.value, true);
  harness.dispatch(pointer("pointerup"));
  harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 1 }));

  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["pressstart", "pressend", "pressstart", "pressup", "pressend", "press"],
  );
  assert.equal(harness.events[1]?.isCanceled, true);
  outside.remove();
  harness.unmount();
});

test("cancel-on-exit suppresses the following compatibility click", () => {
  const harness = mountPress({ shouldCancelOnPointerExit: true });
  const outside = document.createElement("div");
  document.body.append(outside);
  harness.dispatch(pointer("pointerdown"));
  harness.dispatch(pointer("pointermove"), outside);
  const click = new MouseEvent("click", { bubbles: true, cancelable: true, detail: 1 });
  harness.dispatch(click);

  assert.equal(click.defaultPrevented, true);
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["pressstart", "pressend"],
  );
  assert.equal(harness.events[1]?.isCanceled, true);
  outside.remove();
  harness.unmount();
});

test("an outside release cancels the default resumable interaction", () => {
  const harness = mountPress();
  const outside = document.createElement("div");
  document.body.append(outside);
  harness.dispatch(pointer("pointerdown"));
  harness.dispatch(pointer("pointerup"), outside);
  const click = new MouseEvent("click", { bubbles: true, cancelable: true, detail: 1 });
  harness.dispatch(click);

  assert.equal(click.defaultPrevented, true);
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["pressstart", "pressend"],
  );
  assert.equal(harness.events[1]?.isCanceled, true);
  outside.remove();
  harness.unmount();
});

test("cancels when disabled changes during a press", () => {
  let disabled = false;
  const harness = mountPress({ isDisabled: () => disabled });
  harness.dispatch(pointer("pointerdown"));
  disabled = true;
  harness.dispatch(pointer("pointerup"));
  const click = new MouseEvent("click", { bubbles: true, cancelable: true });
  harness.dispatch(click);

  assert.equal(click.defaultPrevented, true);
  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["pressstart", "pressend"],
  );
  assert.equal(harness.events[1]?.isCanceled, true);
  harness.unmount();
});

test("a disabled click clears stale release ownership before re-enabling", () => {
  let disabled = false;
  const pointerTypes: string[] = [];
  const harness = mountPress({
    isDisabled: () => disabled,
    onPress: ({ pointerType }) => pointerTypes.push(pointerType),
  });
  harness.dispatch(pointer("pointerdown"));
  harness.dispatch(pointer("pointerup"));
  disabled = true;
  harness.dispatch(new MouseEvent("click", { bubbles: true, cancelable: true, detail: 1 }));
  disabled = false;
  harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 0 }));

  assert.deepEqual(pointerTypes, ["virtual"]);
  harness.unmount();
});

test("maps coordinate-free and click-only activation without duplication", () => {
  const harness = mountPress();
  harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 0 }));
  harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 2 }));

  assert.deepEqual(
    harness.events.map(({ type }) => type),
    ["pressstart", "pressup", "pressend", "press", "pressstart", "pressup", "pressend", "press"],
  );
  assert.deepEqual(
    harness.events.filter(({ type }) => type === "press").map(({ pointerType }) => pointerType),
    ["virtual", "mouse"],
  );
  harness.unmount();
});

test("emulates button and link keyboard semantics only for custom hosts", () => {
  const button = mountPress();
  button.host.focus();
  const spaceDown = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: " " });
  button.dispatch(spaceDown);
  button.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: " " }));
  assert.equal(spaceDown.defaultPrevented, true);
  assert.equal(button.events.filter(({ type }) => type === "press").length, 1);
  button.unmount();

  const link = mountPress({ keyboardBehavior: "link" });
  link.dispatch(new KeyboardEvent("keydown", { bubbles: true, key: " " }));
  link.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: " " }));
  link.dispatch(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));
  link.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: "Enter" }));
  assert.equal(link.events.filter(({ type }) => type === "press").length, 1);
  link.unmount();
});

test("treats href-less anchors as custom hosts while preserving href links", () => {
  const custom = mountPress({}, "a");
  custom.dispatch(new KeyboardEvent("keydown", { bubbles: true, key: " " }));
  custom.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: " " }));
  assert.equal(custom.events.filter(({ type }) => type === "press").length, 1);
  custom.unmount();

  const link = mountPress({ keyboardBehavior: "link" }, "a");
  link.dispatch(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));
  link.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: "Enter" }));
  assert.equal(link.events.filter(({ type }) => type === "press").length, 1);
  link.unmount();

  const native = mountPress({}, "a");
  native.host.setAttribute("href", "#destination");
  native.dispatch(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));
  native.dispatch(new MouseEvent("click", { bubbles: true, detail: 0 }));
  native.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: "Enter" }));
  assert.equal(native.events.filter(({ type }) => type === "press").length, 1);
  native.unmount();
});

test("preserves native keyboard click timing and delivers exactly one press", () => {
  for (const key of ["Enter", " "]) {
    const harness = mountPress({}, "button");
    harness.dispatch(new KeyboardEvent("keydown", { bubbles: true, key }));
    if (key === "Enter") harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 0 }));
    harness.dispatch(new KeyboardEvent("keyup", { bubbles: true, key }));
    if (key === " ") harness.dispatch(new MouseEvent("click", { bubbles: true, detail: 0 }));

    assert.equal(harness.events.filter(({ type }) => type === "press").length, 1);
    assert.equal(harness.events.find(({ type }) => type === "press")?.pointerType, "keyboard");
    harness.unmount();
  }
});

test("ignores IME, repeats, nested targets, and disabled keyboard input", () => {
  const harness = mountPress({ isDisabled: false });
  const child = document.createElement("span");
  harness.host.append(child);
  child.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));
  harness.dispatch(
    new KeyboardEvent("keydown", { bubbles: true, isComposing: true, key: "Enter" }),
  );
  harness.dispatch(new KeyboardEvent("keydown", { bubbles: true, key: "Enter", repeat: true }));
  harness.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: "Enter" }));

  assert.deepEqual(harness.events, []);
  harness.unmount();

  const disabled = mountPress({ isDisabled: true });
  disabled.dispatch(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));
  disabled.dispatch(new KeyboardEvent("keyup", { bubbles: true, key: "Enter" }));
  assert.deepEqual(disabled.events, []);
  disabled.unmount();
});
