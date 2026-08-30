import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";

import { disableTextSelection, eventElement, isPrimaryPointer } from "../press/press-event.ts";
import {
  capture,
  createMoveEvent,
  installMoveListeners,
  pointerTypeOf,
  positionOf,
  readBoolean,
  readKeyboardStep,
  surfaceErrors,
  validateOptions,
} from "./move-internal.ts";
import type { ActiveMove, PointerSource, Position } from "./move-internal.ts";
import type {
  MoveController,
  MoveEndEvent,
  MovePointerType,
  MoveOptions,
  MoveProps,
  MoveStartEvent,
  MoveUpdateEvent,
} from "./move-types.ts";

const disposedDiagnostic = "VIZE_UI_MOVE_DISPOSED";
const setupDiagnostic = "VIZE_UI_MOVE_SETUP";

interface KeyboardMove {
  readonly target: Element;
}

/** Create an SSR-safe mouse, pen, touch, and keyboard move normalizer. */
export function createMove(options: MoveOptions = {}): MoveController {
  validateOptions(options);
  const moving = shallowRef(false);
  let active: ActiveMove | null = null;
  let keyboard: KeyboardMove | null = null;
  let disposed = false;
  let lastTouchTime = Number.NEGATIVE_INFINITY;

  const finish = (originalEvent: Event | null, isCanceled: boolean): boolean => {
    const current = active;
    if (!current) return false;
    active = null;
    moving.value = false;
    const errors: unknown[] = [];
    capture(errors, current.releaseListeners);
    capture(errors, current.restoreSelection);
    if (current.didMove) {
      capture(errors, () =>
        options.onMoveEnd?.(
          createMoveEvent(
            "moveend",
            current.target,
            current.pointerType,
            originalEvent,
            0,
            0,
            isCanceled,
            current.source === "touch" ? current.id : null,
          ) as MoveEndEvent,
        ),
      );
    }
    surfaceErrors(errors, "Move completion failed");
    return true;
  };

  const readDisabledDuringActive = (event: Event): boolean => {
    try {
      return readBoolean(options.isDisabled);
    } catch (error) {
      const errors: unknown[] = [error];
      capture(errors, () => finish(event, true));
      surfaceErrors(errors, "Move option validation failed during teardown");
      throw error;
    }
  };

  const finishKeyboard = (originalEvent: Event | null, isCanceled: boolean): boolean => {
    const current = keyboard;
    if (!current) return false;
    keyboard = null;
    moving.value = false;
    options.onMoveEnd?.(
      createMoveEvent(
        "moveend",
        current.target,
        "keyboard",
        originalEvent,
        0,
        0,
        isCanceled,
      ) as MoveEndEvent,
    );
    return true;
  };

  const emitDelta = (current: ActiveMove, event: Event, position: Position): void => {
    if (active !== current) return;
    current.lastEvent = event;
    if (readDisabledDuringActive(event) || !current.target.isConnected) {
      finish(event, true);
      return;
    }
    const deltaX = position.x - current.position.x;
    const deltaY = position.y - current.position.y;
    current.position = position;
    if (deltaX === 0 && deltaY === 0) return;
    event.preventDefault();
    const errors: unknown[] = [];
    if (!current.didMove) {
      current.didMove = true;
      moving.value = true;
      capture(errors, () =>
        options.onMoveStart?.(
          createMoveEvent(
            "movestart",
            current.target,
            current.pointerType,
            event,
            0,
            0,
            false,
            current.source === "touch" ? current.id : null,
          ) as MoveStartEvent,
        ),
      );
    }
    if (active === current && !disposed) {
      capture(errors, () =>
        options.onMove?.(
          createMoveEvent(
            "move",
            current.target,
            current.pointerType,
            event,
            deltaX,
            deltaY,
            false,
            current.source === "touch" ? current.id : null,
          ) as MoveUpdateEvent,
        ),
      );
    }
    surfaceErrors(errors, "Move callbacks failed");
  };

  const matches = (source: PointerSource, id: number | null): ActiveMove | null => {
    const current = active;
    return current?.source === source && current.id === id ? current : null;
  };

  const start = (
    event: Event,
    source: PointerSource,
    pointerType: Exclude<MovePointerType, "keyboard">,
    id: number | null,
    position: Position,
  ): void => {
    if (disposed || active || readBoolean(options.isDisabled)) return;
    const target = eventElement(event);
    if (!target) return;
    event.preventDefault();
    event.stopPropagation();
    let releaseListeners: () => void = () => undefined;
    let restoreSelection: () => void = () => undefined;
    try {
      releaseListeners = installMoveListeners(target.ownerDocument, source, {
        emitDelta,
        finish,
        getActive: () => active,
        readDisabled: readDisabledDuringActive,
        rememberTouch: (timeStamp) => {
          lastTouchTime = timeStamp;
        },
      });
      restoreSelection = disableTextSelection(target);
    } catch (error) {
      const errors: unknown[] = [error];
      capture(errors, releaseListeners);
      capture(errors, restoreSelection);
      surfaceErrors(errors, "Move start cleanup failed");
      throw error;
    }
    active = {
      id,
      pointerType,
      releaseListeners,
      restoreSelection,
      source,
      target,
      didMove: false,
      lastEvent: event,
      position,
    };
  };

  const keyboardMove = (event: KeyboardEvent, deltaX: number, deltaY: number): void => {
    if (disposed || active || keyboard || readBoolean(options.isDisabled)) return;
    const target = eventElement(event);
    if (!target || event.target !== target) return;
    event.preventDefault();
    event.stopPropagation();
    const step = readKeyboardStep(options.keyboardStep);
    const errors: unknown[] = [];
    const current: KeyboardMove = { target };
    keyboard = current;
    moving.value = true;
    capture(errors, () =>
      options.onMoveStart?.(
        createMoveEvent("movestart", target, "keyboard", event) as MoveStartEvent,
      ),
    );
    if (keyboard === current && !disposed) {
      capture(errors, () =>
        options.onMove?.(
          createMoveEvent(
            "move",
            target,
            "keyboard",
            event,
            deltaX * step,
            deltaY * step,
          ) as MoveUpdateEvent,
        ),
      );
    }
    if (keyboard === current && !disposed) capture(errors, () => finishKeyboard(event, false));
    surfaceErrors(errors, "Keyboard move callbacks failed");
  };

  const moveProps: Readonly<MoveProps> = Object.freeze({
    onDragstart(event: DragEvent) {
      if (active?.target === eventElement(event)) finish(event, true);
    },
    onKeydown(event: KeyboardEvent) {
      if (event.isComposing || event.altKey || event.ctrlKey || event.metaKey) return;
      switch (event.key) {
        case "Left":
        case "ArrowLeft":
          keyboardMove(event, -1, 0);
          break;
        case "Right":
        case "ArrowRight":
          keyboardMove(event, 1, 0);
          break;
        case "Up":
        case "ArrowUp":
          keyboardMove(event, 0, -1);
          break;
        case "Down":
        case "ArrowDown":
          keyboardMove(event, 0, 1);
          break;
      }
    },
    onMousedown(event: MouseEvent) {
      if (event.button !== 0 || active || (event.view && "PointerEvent" in event.view)) return;
      const elapsed = event.timeStamp - lastTouchTime;
      if (elapsed >= 0 && elapsed < 800) return;
      const position = positionOf(event);
      if (position) start(event, "mouse", "mouse", null, position);
    },
    onPointercancel(event: PointerEvent) {
      if (matches("pointer", event.pointerId)) finish(event, true);
    },
    onPointerdown(event: PointerEvent) {
      if (!isPrimaryPointer(event)) return;
      const position = positionOf(event);
      if (position) start(event, "pointer", pointerTypeOf(event), event.pointerId, position);
    },
    onTouchcancel(event: TouchEvent) {
      const current = active?.source === "touch" ? active : null;
      if (current && positionOf(event, current.id)) finish(event, true);
    },
    onTouchend(event: TouchEvent) {
      const current = active?.source === "touch" ? active : null;
      if (current && positionOf(event, current.id)) finish(event, false);
    },
    onTouchmove(event: TouchEvent) {
      const current = active?.source === "touch" ? active : null;
      const position = current ? positionOf(event, current.id) : null;
      if (current && position) emitDelta(current, event, position);
    },
    onTouchstart(event: TouchEvent) {
      lastTouchTime = event.timeStamp;
      if (
        (event.view && "PointerEvent" in event.view) ||
        active ||
        event.changedTouches.length !== 1
      )
        return;
      const touch = event.changedTouches.item(0);
      if (touch) {
        start(event, "touch", "touch", touch.identifier, { x: touch.pageX, y: touch.pageY });
      }
    },
  });

  return Object.freeze({
    isMoving: shallowReadonly(moving),
    moveProps,
    cancel: () => {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      return finish(null, true) || finishKeyboard(null, true);
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      const current = active;
      active = null;
      keyboard = null;
      moving.value = false;
      if (!current) return;
      const errors: unknown[] = [];
      capture(errors, current.releaseListeners);
      capture(errors, current.restoreSelection);
      surfaceErrors(errors, "Move disposal failed");
    },
  });
}

/** Create a move normalizer disposed with the current Vue effect scope. */
export function useMove(options: MoveOptions = {}): MoveController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createMove(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  MoveController,
  MoveEndEvent,
  MoveEvent,
  MoveEventType,
  MoveOptions,
  MovePointerType,
  MoveProps,
  MoveStartEvent,
  MoveUpdateEvent,
} from "./move-types.ts";
