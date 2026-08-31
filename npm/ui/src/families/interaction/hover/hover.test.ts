import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createHover, useHover } from "./hover.ts";
import type { HoverController, HoverEvent, HoverOptions, HoverProps } from "./hover.ts";

const eventNames: Readonly<Record<keyof HoverProps, string>> = {
  onMouseenter: "mouseenter",
  onMouseleave: "mouseleave",
  onMousemove: "mousemove",
  onPointercancel: "pointercancel",
  onPointerenter: "pointerenter",
  onPointerleave: "pointerleave",
  onPointermove: "pointermove",
  onTouchstart: "touchstart",
};

interface Harness {
  readonly controller: HoverController;
  readonly events: HoverEvent[];
  readonly host: HTMLDivElement;
  readonly unmount: () => void;
}

function mountHover(options: HoverOptions = {}): Harness {
  const host = document.createElement("div");
  document.body.append(host);
  const events: HoverEvent[] = [];
  const controller = createHover({
    ...options,
    onHoverStart: (event) => {
      events.push(event);
      options.onHoverStart?.(event);
    },
    onHoverEnd: (event) => {
      events.push(event);
      options.onHoverEnd?.(event);
    },
  });
  for (const [property, type] of Object.entries(eventNames) as Array<[keyof HoverProps, string]>) {
    host.addEventListener(type, controller.hoverProps[property] as EventListener);
  }
  return {
    controller,
    events,
    host,
    unmount: () => {
      controller.dispose();
      host.remove();
    },
  };
}

function pointer(
  type: string,
  pointerType = "mouse",
  values: Partial<PointerEventInit> = {},
): PointerEvent {
  return new PointerEvent(type, {
    bubbles: false,
    clientX: 7,
    clientY: 13,
    isPrimary: true,
    pointerId: 4,
    pointerType,
    ...values,
  });
}

test("normalizes mouse and pen boundary transitions into immutable snapshots", () => {
  for (const pointerType of ["mouse", "pen"] as const) {
    const harness = mountHover();
    const child = document.createElement("span");
    const outside = document.createElement("div");
    harness.host.append(child);
    document.body.append(outside);

    harness.host.dispatchEvent(pointer("pointerenter", pointerType));
    assert.equal(harness.controller.isHovered.value, true);
    assert.deepEqual(
      harness.events.map(({ type }) => type),
      ["hoverstart"],
    );
    assert.ok(Object.isFrozen(harness.events[0]));
    assert.deepEqual([harness.events[0]?.x, harness.events[0]?.y], [7, 13]);

    harness.host.dispatchEvent(pointer("pointerleave", pointerType, { relatedTarget: child }));
    assert.equal(harness.controller.isHovered.value, true);
    harness.host.dispatchEvent(
      pointer("pointerleave", pointerType, { clientX: 20, clientY: 30, relatedTarget: outside }),
    );
    assert.equal(harness.controller.isHovered.value, false);
    assert.deepEqual(
      harness.events.map(({ type }) => type),
      ["hoverstart", "hoverend"],
    );
    assert.equal(harness.events[1]?.isCanceled, false);
    assert.deepEqual([harness.events[1]?.x, harness.events[1]?.y], [20, 30]);
    outside.remove();
    harness.unmount();
  }
});

test("ignores touch and unknown pointers, disabled hosts, and filtered families", () => {
  for (const [actual, filter, disabled, expected] of [
    ["touch", undefined, false, false],
    ["trackpad", undefined, false, false],
    ["mouse", "pen", false, false],
    ["pen", "pen", false, true],
    ["mouse", undefined, true, false],
  ] as const) {
    const harness = mountHover({ isDisabled: disabled, pointerType: filter });
    harness.host.dispatchEvent(pointer("pointerenter", actual));
    assert.equal(harness.controller.isHovered.value, expected);
    harness.unmount();
  }
});

test("reactive disablement and pointer filters cancel on the next native movement", () => {
  const disabled = ref(false);
  const pointerType = ref<"mouse" | "pen">("mouse");
  const harness = mountHover({ isDisabled: disabled, pointerType });
  harness.host.dispatchEvent(pointer("pointerenter"));
  disabled.value = true;
  document.dispatchEvent(pointer("pointermove"));
  assert.equal(harness.controller.isHovered.value, false);
  assert.equal(harness.events.at(-1)?.isCanceled, true);

  disabled.value = false;
  harness.host.dispatchEvent(pointer("pointerenter"));
  pointerType.value = "pen";
  document.dispatchEvent(pointer("pointermove"));
  assert.equal(harness.controller.isHovered.value, false);
  assert.equal(harness.events.at(-1)?.isCanceled, true);
  harness.unmount();
});

test("touch switching, pointer cancellation, lost visibility, and window blur terminate hover", () => {
  for (const finish of ["touch", "pointercancel", "hidden", "blur"] as const) {
    const harness = mountHover();
    harness.host.dispatchEvent(pointer("pointerenter"));
    if (finish === "touch") {
      document.dispatchEvent(pointer("pointerdown", "touch", { bubbles: true }));
    } else if (finish === "pointercancel") {
      harness.host.dispatchEvent(pointer("pointercancel"));
    } else if (finish === "hidden") {
      const descriptor = Object.getOwnPropertyDescriptor(document, "visibilityState");
      Object.defineProperty(document, "visibilityState", {
        configurable: true,
        value: "hidden",
      });
      try {
        document.dispatchEvent(new Event("visibilitychange"));
      } finally {
        if (descriptor) Object.defineProperty(document, "visibilityState", descriptor);
        else delete (document as { visibilityState?: DocumentVisibilityState }).visibilityState;
      }
    } else {
      window.dispatchEvent(new Event("blur"));
    }
    assert.equal(harness.controller.isHovered.value, false);
    assert.equal(harness.events.at(-1)?.isCanceled, true);
    harness.unmount();
  }
});

test("legacy mouse fallback suppresses compatibility events after touch", () => {
  const harness = mountHover();
  const touch = new Event("touchstart", { bubbles: true });
  harness.host.dispatchEvent(touch);
  const emulated = new MouseEvent("mouseenter");
  Object.defineProperty(emulated, "timeStamp", { value: touch.timeStamp + 1 });
  harness.host.dispatchEvent(emulated);
  assert.equal(harness.controller.isHovered.value, false);

  const genuine = new MouseEvent("mouseenter", { clientX: 3, clientY: 5 });
  Object.defineProperty(genuine, "timeStamp", { value: touch.timeStamp + 801 });
  harness.host.dispatchEvent(genuine);
  assert.equal(harness.controller.isHovered.value, true);
  harness.host.dispatchEvent(new MouseEvent("mouseleave"));
  assert.equal(harness.controller.isHovered.value, false);

  const modern = new MouseEvent("mouseenter", { view: window });
  harness.host.dispatchEvent(modern);
  assert.equal(harness.controller.isHovered.value, false);
  harness.unmount();
});

test("manual cancel, disposal, and Vue scope ownership settle state", () => {
  const harness = mountHover();
  harness.host.dispatchEvent(pointer("pointerenter"));
  assert.equal(harness.controller.cancel(), true);
  assert.equal(harness.events.at(-1)?.isCanceled, true);
  assert.equal(harness.controller.cancel(), false);
  harness.controller.dispose();
  assert.throws(() => harness.controller.cancel(), /VIZE_UI_HOVER_DISPOSED/);
  harness.host.remove();

  assert.throws(() => useHover(), /VIZE_UI_HOVER_SETUP/);
  const scope = effectScope();
  const scoped = scope.run(() => useHover())!;
  scope.stop();
  assert.throws(() => scoped.cancel(), /VIZE_UI_HOVER_DISPOSED/);
});

test("reentrant cancellation cannot publish a stale hovered change", () => {
  const changes: boolean[] = [];
  let controller!: HoverController;
  const host = document.createElement("div");
  document.body.append(host);
  controller = createHover({
    onHoverStart: () => controller.cancel(),
    onHoverChange: (hovered) => changes.push(hovered),
  });
  host.addEventListener("pointerenter", controller.hoverProps.onPointerenter);
  host.dispatchEvent(pointer("pointerenter"));

  assert.equal(controller.isHovered.value, false);
  assert.deepEqual(changes, [false]);
  controller.dispose();
  host.remove();
});

test("callback failures settle transitions and aggregate multiple errors", () => {
  const harness = mountHover({
    onHoverStart: () => {
      throw new Error("start failed");
    },
    onHoverChange: () => {
      throw new Error("change failed");
    },
  });
  assert.throws(() => harness.host.dispatchEvent(pointer("pointerenter")), AggregateError);
  assert.equal(harness.controller.isHovered.value, true);
  harness.controller.dispose();
  harness.host.remove();
});

test("rejects invalid runtime options with stable diagnostics", () => {
  assert.throws(
    () => createHover({ pointerType: "touch" as "mouse" }),
    /VIZE_UI_HOVER_OPTION.*pointerType/,
  );
  assert.throws(
    () => createHover({ onHoverStart: "callback" as never }),
    /VIZE_UI_HOVER_OPTION.*onHoverStart/,
  );
});
