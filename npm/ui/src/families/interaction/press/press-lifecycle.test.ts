import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import { createPress, usePress } from "./press.ts";
import type { PressController } from "./press.ts";

function currentTarget<EventType extends Event>(event: EventType, target: Element): EventType {
  Object.defineProperty(event, "currentTarget", { configurable: true, value: target });
  Object.defineProperty(event, "target", { configurable: true, value: target });
  return event;
}

function pointer(type: string, target: Element, init: PointerEventInit = {}): PointerEvent {
  return currentTarget(
    new PointerEvent(type, {
      bubbles: true,
      button: 0,
      isPrimary: true,
      pointerId: 13,
      pointerType: "pen",
      ...init,
    }),
    target,
  );
}

test("installs listeners only while active and releases every listener exactly once", () => {
  const isolated = document.implementation.createHTMLDocument("press listeners");
  const host = isolated.createElement("button");
  isolated.body.append(host);
  const additions: string[] = [];
  const removals: string[] = [];
  const addEventListener = isolated.addEventListener.bind(isolated);
  const removeEventListener = isolated.removeEventListener.bind(isolated);
  isolated.addEventListener = ((
    type: string,
    listener: EventListenerOrEventListenerObject,
    options,
  ) => {
    additions.push(type);
    addEventListener(type, listener, options);
  }) as typeof isolated.addEventListener;
  isolated.removeEventListener = ((
    type: string,
    listener: EventListenerOrEventListenerObject,
    options,
  ) => {
    removals.push(type);
    removeEventListener(type, listener, options);
  }) as typeof isolated.removeEventListener;
  const controller = createPress();

  controller.pressProps.onPointerdown(pointer("pointerdown", host));
  assert.deepEqual(additions.sort(), [
    "pointercancel",
    "pointermove",
    "pointerup",
    "visibilitychange",
  ]);
  controller.pressProps.onPointercancel(pointer("pointercancel", host));
  assert.deepEqual(removals.sort(), additions);

  controller.dispose();
  controller.dispose();
  assert.deepEqual(removals.sort(), additions);
});

test("manual cancellation is idempotent and disposal rejects later mutation", () => {
  const host = document.createElement("button");
  const endings: boolean[] = [];
  const controller = createPress({ onPressEnd: (event) => endings.push(event.isCanceled) });
  controller.pressProps.onPointerdown(pointer("pointerdown", host));

  assert.equal(controller.cancel(), true);
  assert.equal(controller.cancel(), false);
  assert.deepEqual(endings, [true]);
  assert.equal(controller.isPressed.value, false);
  controller.dispose();
  assert.throws(() => controller.cancel(), /VIZE_UI_PRESS_DISPOSED/);
});

test("the composable requires and follows a Vue effect scope", () => {
  assert.throws(() => usePress(), /VIZE_UI_PRESS_SETUP/);
  const scope = effectScope();
  const controller = scope.run(() => usePress())!;
  const host = document.createElement("button");
  controller.pressProps.onPointerdown(pointer("pointerdown", host));
  assert.equal(controller.isPressed.value, true);

  scope.stop();
  assert.equal(controller.isPressed.value, false);
  assert.throws(() => controller.cancel(), /VIZE_UI_PRESS_DISPOSED/);
});

test("reactive options are read at event time and invalid values have stable diagnostics", () => {
  const disabled = ref(false);
  const controller = createPress({ isDisabled: disabled });
  const host = document.createElement("button");
  disabled.value = true;
  controller.pressProps.onPointerdown(pointer("pointerdown", host));
  assert.equal(controller.isPressed.value, false);
  controller.dispose();

  const invalid = createPress({ isDisabled: "false" as unknown as boolean });
  assert.throws(
    () => invalid.pressProps.onPointerdown(pointer("pointerdown", host)),
    /VIZE_UI_PRESS_OPTION: isDisabled/,
  );
  invalid.dispose();
  assert.throws(
    () => createPress({ onPress: "activate" as unknown as () => void }),
    /VIZE_UI_PRESS_OPTION: onPress/,
  );
});

test("synthetic activation completes every phase before callback errors surface", () => {
  const calls: string[] = [];
  const failure = new Error("consumer failed");
  const controller = createPress({
    onPressStart: () => {
      calls.push("start");
      throw failure;
    },
    onPressChange: (value) => calls.push(`change:${value}`),
    onPressUp: () => {
      calls.push("up");
      throw failure;
    },
    onPressEnd: () => {
      calls.push("end");
      throw failure;
    },
    onPress: () => {
      calls.push("press");
      throw failure;
    },
  });
  const host = document.createElement("button");
  const click = currentTarget(new MouseEvent("click", { detail: 0 }), host);

  assert.throws(() => controller.pressProps.onClick(click), /Press callbacks failed/);
  assert.deepEqual(calls, ["start", "change:true", "up", "end", "change:false", "press"]);
  assert.equal(controller.isPressed.value, false);
  controller.dispose();
});

test("a reentrant cancel cannot publish a stale pressed=true change", () => {
  const changes: boolean[] = [];
  let controller!: PressController;
  controller = createPress({
    onPressStart: () => controller.cancel(),
    onPressChange: (value) => changes.push(value),
  });
  const host = document.createElement("button");

  controller.pressProps.onPointerdown(pointer("pointerdown", host));
  assert.equal(controller.isPressed.value, false);
  assert.deepEqual(changes, [false]);
  controller.dispose();
});

test("a virtual start callback can cancel the synchronous activation", () => {
  const calls: string[] = [];
  let controller!: PressController;
  controller = createPress({
    onPressStart: () => {
      calls.push("start");
      controller.cancel();
    },
    onPressEnd: (event) => calls.push(`end:${event.isCanceled}`),
    onPress: () => calls.push("press"),
  });
  const host = document.createElement("button");

  controller.pressProps.onClick(currentTarget(new MouseEvent("click"), host));
  assert.deepEqual(calls, ["start", "end:true"]);
  assert.equal(controller.isPressed.value, false);
  controller.dispose();
});

test("a reentrant press-up cancellation cannot activate on keyboard release", () => {
  const calls: string[] = [];
  let controller!: PressController;
  controller = createPress({
    onPressUp: () => {
      calls.push("up");
      controller.cancel();
    },
    onPress: () => calls.push("press"),
  });
  const host = document.createElement("div");
  document.body.append(host);
  controller.pressProps.onKeydown(
    currentTarget(new KeyboardEvent("keydown", { key: "Enter" }), host),
  );
  controller.pressProps.onKeyup(currentTarget(new KeyboardEvent("keyup", { key: "Enter" }), host));

  assert.deepEqual(calls, ["up"]);
  assert.equal(controller.isPressed.value, false);
  controller.dispose();
  host.remove();
});

test("keyboard ownership cancels when focus moves before keyup", () => {
  const host = document.createElement("div");
  const next = document.createElement("button");
  host.tabIndex = 0;
  document.body.append(host, next);
  let presses = 0;
  const controller = createPress({ onPress: () => presses++ });
  host.addEventListener("keydown", controller.pressProps.onKeydown);
  host.focus();
  host.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: " " }));
  next.focus();
  document.dispatchEvent(new KeyboardEvent("keyup", { bubbles: true, key: " " }));

  assert.equal(controller.isPressed.value, false);
  assert.equal(presses, 0);
  controller.dispose();
  host.remove();
  next.remove();
});

test("pointer-driven focus movement does not cancel or suppress its click", () => {
  const previous = document.createElement("input");
  const host = document.createElement("button");
  document.body.append(previous, host);
  let presses = 0;
  const controller = createPress({
    preventFocusOnPress: false,
    onPress: () => presses++,
  });
  host.addEventListener("pointerdown", controller.pressProps.onPointerdown);
  host.addEventListener("pointerup", controller.pressProps.onPointerup);
  host.addEventListener("click", controller.pressProps.onClick);
  previous.focus();

  host.dispatchEvent(pointer("pointerdown", host));
  host.focus();
  host.dispatchEvent(pointer("pointerup", host));
  const click = new MouseEvent("click", { bubbles: true, cancelable: true, detail: 1 });
  host.dispatchEvent(click);

  assert.equal(click.defaultPrevented, false);
  assert.equal(presses, 1);
  controller.dispose();
  previous.remove();
  host.remove();
});

test("drag, window blur, and host removal terminate pointer ownership", () => {
  const host = document.createElement("button");
  document.body.append(host);
  const controller = createPress();
  controller.pressProps.onPointerdown(pointer("pointerdown", host));
  controller.pressProps.onDragstart(currentTarget(new DragEvent("dragstart"), host));
  assert.equal(controller.isPressed.value, false);

  controller.pressProps.onPointerdown(pointer("pointerdown", host));
  window.dispatchEvent(new Event("blur"));
  assert.equal(controller.isPressed.value, false);

  controller.pressProps.onPointerdown(pointer("pointerdown", host));
  host.remove();
  controller.pressProps.onPointerup(pointer("pointerup", host));
  assert.equal(controller.isPressed.value, false);
  controller.dispose();
});

test("listener setup failure rolls back earlier registrations and styling", () => {
  const isolated = document.implementation.createHTMLDocument("listener failure");
  const host = isolated.createElement("button");
  host.style.setProperty("user-select", "text", "important");
  isolated.body.append(host);
  const removals: string[] = [];
  const addEventListener = isolated.addEventListener.bind(isolated);
  const removeEventListener = isolated.removeEventListener.bind(isolated);
  isolated.addEventListener = ((
    type: string,
    listener: EventListenerOrEventListenerObject,
    options,
  ) => {
    if (type === "pointerup") throw new Error("listener rejected");
    addEventListener(type, listener, options);
  }) as typeof isolated.addEventListener;
  isolated.removeEventListener = ((
    type: string,
    listener: EventListenerOrEventListenerObject,
    options,
  ) => {
    removals.push(type);
    removeEventListener(type, listener, options);
  }) as typeof isolated.removeEventListener;
  const controller = createPress();

  assert.throws(
    () => controller.pressProps.onPointerdown(pointer("pointerdown", host)),
    /listener rejected/,
  );
  assert.deepEqual(removals, ["pointermove"]);
  assert.equal(host.style.getPropertyValue("user-select"), "text");
  assert.equal(host.style.getPropertyPriority("user-select"), "important");
  assert.equal(controller.cancel(), false);
  controller.dispose();
});

test("callback failure leaves an active interaction cancelable and leak-free", () => {
  const host = document.createElement("button");
  const failure = new Error("start failed");
  const controller = createPress({
    onPressStart: () => {
      throw failure;
    },
  });

  assert.throws(() => controller.pressProps.onPointerdown(pointer("pointerdown", host)), failure);
  assert.equal(controller.isPressed.value, true);
  assert.equal(controller.cancel(), true);
  assert.equal(controller.isPressed.value, false);
  controller.dispose();
});

test("disposed bound handlers remain inert for late renderer events", () => {
  const host = document.createElement("button");
  const controller = createPress();
  controller.dispose();

  assert.doesNotThrow(() => {
    controller.pressProps.onPointerdown(pointer("pointerdown", host));
    controller.pressProps.onClick(currentTarget(new MouseEvent("click"), host));
  });
  assert.equal(controller.isPressed.value, false);
});
