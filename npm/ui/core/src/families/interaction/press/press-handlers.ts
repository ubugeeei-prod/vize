import {
  eventElement,
  isPrimaryPointer,
  keyboardActivation,
  pointerTypeOf,
  readBooleanOption,
  readKeyboardBehavior,
} from "./press-event.ts";
import type { ActivePress, PressSource } from "./press-lifecycle.ts";
import { PressLifecycle } from "./press-lifecycle.ts";
import type { PressProps } from "./press-types.ts";

export interface PressHandlers extends Readonly<PressProps> {
  readonly installListeners: (document: Document, source: PressSource) => () => void;
}

/** Adapt Pointer Events plus legacy mouse/touch and keyboard events to one lifecycle. */
export function createPressHandlers(lifecycle: PressLifecycle): PressHandlers {
  let lastTouchTime = Number.NEGATIVE_INFINITY;

  function onPointerDown(event: PointerEvent): void {
    if (lifecycle.disposed || lifecycle.active || !isPrimaryPointer(event)) return;
    const target = eventElement(event);
    if (!target || readBooleanOption(lifecycle.options.isDisabled, "isDisabled")) return;
    lifecycle.start(event, target, "pointer", pointerTypeOf(event), event.pointerId, null, false);
  }
  function onPointerMove(event: PointerEvent): void {
    const current = lifecycle.active;
    if (!matches(current, "pointer", event.pointerId)) return;
    lifecycle.updatePointerBoundary(current, event);
  }
  function onPointerUp(event: PointerEvent): void {
    const current = lifecycle.active;
    if (!matches(current, "pointer", event.pointerId)) return;
    lifecycle.finishPointer(current, event);
  }
  function onPointerCancel(event: PointerEvent): void {
    const current = lifecycle.active;
    if (matches(current, "pointer", event.pointerId)) lifecycle.cancelActive(event);
  }
  function onMouseDown(event: MouseEvent): void {
    if (lifecycle.disposed || event.button !== 0) return;
    const target = eventElement(event);
    if (!target || readBooleanOption(lifecycle.options.isDisabled, "isDisabled")) return;
    const elapsed = event.timeStamp - lastTouchTime;
    if (elapsed >= 0 && elapsed < 800) return;
    if (readBooleanOption(lifecycle.options.preventFocusOnPress, "preventFocusOnPress")) {
      event.preventDefault();
    }
    if ((event.view && "PointerEvent" in event.view) || lifecycle.active) return;
    lifecycle.start(event, target, "mouse", "mouse", null, null, false);
  }
  function onMouseMove(event: MouseEvent): void {
    const current = lifecycle.active;
    if (current?.source === "mouse") lifecycle.updatePointerBoundary(current, event);
  }
  function onMouseUp(event: MouseEvent): void {
    const current = lifecycle.active;
    if (current?.source === "mouse" && event.button === 0) lifecycle.finishPointer(current, event);
  }
  function onTouchStart(event: TouchEvent): void {
    if (
      lifecycle.disposed ||
      (event.view && "PointerEvent" in event.view) ||
      lifecycle.active ||
      event.changedTouches.length !== 1
    ) {
      return;
    }
    const target = eventElement(event);
    if (!target || readBooleanOption(lifecycle.options.isDisabled, "isDisabled")) return;
    const touch = event.changedTouches.item(0)!;
    lastTouchTime = event.timeStamp;
    lifecycle.start(event, target, "touch", "touch", touch.identifier, null, false);
  }
  function onTouchMove(event: TouchEvent): void {
    const current = lifecycle.active;
    if (current?.source === "touch" && touchMatches(event, current)) {
      lifecycle.updatePointerBoundary(current, event);
    }
  }
  function onTouchEnd(event: TouchEvent): void {
    const current = lifecycle.active;
    if (current?.source === "touch" && touchMatches(event, current)) {
      lifecycle.finishPointer(current, event);
    }
  }
  function onTouchCancel(event: TouchEvent): void {
    const current = lifecycle.active;
    if (current?.source === "touch" && touchMatches(event, current)) {
      lifecycle.cancelActive(event);
    }
  }
  function onKeyDown(event: KeyboardEvent): void {
    if (
      lifecycle.disposed ||
      lifecycle.active ||
      event.isComposing ||
      event.repeat ||
      event.target !== event.currentTarget
    ) {
      return;
    }
    const target = eventElement(event);
    if (!target || readBooleanOption(lifecycle.options.isDisabled, "isDisabled")) return;
    const activation = keyboardActivation(
      target,
      event.key,
      readKeyboardBehavior(lifecycle.options.keyboardBehavior),
    );
    if (!activation) return;
    if (event.key === " " && activation === "custom") event.preventDefault();
    lifecycle.start(
      event,
      target,
      "keyboard",
      "keyboard",
      null,
      event.key,
      activation === "native",
    );
  }
  function onKeyUp(event: KeyboardEvent): void {
    const current = lifecycle.active;
    if (current?.source === "keyboard" && event.key === current.key) {
      lifecycle.finishKeyboard(current, event);
    }
  }
  function onClick(event: MouseEvent): void {
    if (lifecycle.disposed) return;
    const target = eventElement(event);
    if (target) lifecycle.activateClick(target, event);
  }
  function onDragStart(event: DragEvent): void {
    if (lifecycle.active?.target === eventElement(event)) lifecycle.cancelActive(event);
  }
  function onWindowBlur(event: Event): void {
    lifecycle.cancelActive(event);
  }
  function onVisibilityChange(event: Event): void {
    if (lifecycle.active?.document.visibilityState === "hidden") lifecycle.cancelActive(event);
  }
  function onFocusIn(event: Event): void {
    const current = lifecycle.active;
    if (current?.source === "keyboard" && event.target !== current.target) {
      lifecycle.cancelActive(event, false);
    }
  }

  const pressProps: PressProps = Object.freeze({
    onClick,
    onDragstart: onDragStart,
    onKeydown: onKeyDown,
    onKeyup: onKeyUp,
    onMousedown: onMouseDown,
    onMousemove: onMouseMove,
    onMouseup: onMouseUp,
    onPointercancel: onPointerCancel,
    onPointerdown: onPointerDown,
    onPointermove: onPointerMove,
    onPointerup: onPointerUp,
    onTouchcancel: onTouchCancel,
    onTouchend: onTouchEnd,
    onTouchmove: onTouchMove,
    onTouchstart: onTouchStart,
  });

  return Object.freeze({
    ...pressProps,
    installListeners(document: Document, source: PressSource) {
      const removals: Array<() => void> = [];
      const listen = (
        owner: Document | Window,
        type: string,
        listener: EventListener,
        capture = true,
      ) => {
        owner.addEventListener(type, listener, capture);
        removals.push(() => owner.removeEventListener(type, listener, capture));
      };
      try {
        if (source === "pointer") {
          listen(document, "pointermove", onPointerMove as EventListener);
          listen(document, "pointerup", onPointerUp as EventListener);
          listen(document, "pointercancel", onPointerCancel as EventListener);
        } else if (source === "mouse") {
          listen(document, "mousemove", onMouseMove as EventListener);
          listen(document, "mouseup", onMouseUp as EventListener);
        } else if (source === "touch") {
          listen(document, "touchmove", onTouchMove as EventListener);
          listen(document, "touchend", onTouchEnd as EventListener);
          listen(document, "touchcancel", onTouchCancel as EventListener);
        } else {
          listen(document, "keyup", onKeyUp as EventListener);
          listen(document, "focusin", onFocusIn as EventListener);
        }
        if (document.defaultView) {
          listen(document.defaultView, "blur", onWindowBlur as EventListener, false);
        }
        listen(document, "visibilitychange", onVisibilityChange as EventListener);
      } catch (error) {
        for (const remove of removals.reverse()) remove();
        throw error;
      }
      let released = false;
      return () => {
        if (released) return;
        released = true;
        for (const remove of removals) remove();
      };
    },
  });
}

function matches(
  active: ActivePress | null,
  source: PressSource,
  id: number,
): active is ActivePress {
  return active?.source === source && active.id === id;
}

function touchMatches(event: TouchEvent, active: ActivePress): boolean {
  return Array.from(event.changedTouches).some((touch) => touch.identifier === active.id);
}
