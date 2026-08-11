import { toValue } from "vue";

import type { MoveEvent, MoveEventType, MoveOptions, MovePointerType } from "./move-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_MOVE_OPTION";

export type PointerSource = "mouse" | "pointer" | "touch";

export interface Position {
  readonly x: number;
  readonly y: number;
}

export interface ActiveMove {
  readonly id: number | null;
  readonly pointerType: Exclude<MovePointerType, "keyboard">;
  readonly releaseListeners: () => void;
  readonly restoreSelection: () => void;
  readonly source: PointerSource;
  readonly target: Element;
  didMove: boolean;
  lastEvent: Event;
  position: Position;
}

export interface MoveListenerContext {
  readonly emitDelta: (current: ActiveMove, event: Event, position: Position) => void;
  readonly finish: (event: Event | null, canceled: boolean) => boolean;
  readonly getActive: () => ActiveMove | null;
  readonly readDisabled: (event: Event) => boolean;
  readonly rememberTouch: (timeStamp: number) => void;
}

export function readBoolean(value: MoveOptions["isDisabled"]): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return false;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: isDisabled must resolve to a boolean`);
  }
  return resolved;
}

export function readKeyboardStep(value: MoveOptions["keyboardStep"]): number {
  const resolved = toValue(value) ?? 1;
  if (typeof resolved !== "number" || !Number.isFinite(resolved) || resolved <= 0) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: keyboardStep must resolve to a finite number > 0`,
    );
  }
  return resolved;
}

function modifier(event: Event | null, key: "altKey" | "ctrlKey" | "metaKey" | "shiftKey") {
  return event && key in event ? Boolean((event as KeyboardEvent)[key]) : false;
}

export function positionOf(event: Event, touchId: number | null = null): Position | null {
  if ("pageX" in event && "pageY" in event) {
    const x = Number(event.pageX);
    const y = Number(event.pageY);
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
  }
  if ("clientX" in event && "clientY" in event) {
    const x = Number(event.clientX);
    const y = Number(event.clientY);
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
  }
  if ("changedTouches" in event) {
    const touch = Array.from((event as TouchEvent).changedTouches).find(
      ({ identifier }) => touchId === null || identifier === touchId,
    );
    if (touch) return { x: touch.pageX, y: touch.pageY };
  }
  return null;
}

export function pointerTypeOf(event: PointerEvent): Exclude<MovePointerType, "keyboard"> {
  if (event.pointerType === "mouse" || event.pointerType === "pen") return event.pointerType;
  if (event.pointerType === "touch") return "touch";
  return "pointer";
}

export function createMoveEvent(
  type: MoveEventType,
  target: Element,
  pointerType: MovePointerType,
  originalEvent: Event | null,
  deltaX = 0,
  deltaY = 0,
  isCanceled = false,
  touchId: number | null = null,
): MoveEvent {
  const position = originalEvent ? positionOf(originalEvent, touchId) : null;
  return Object.freeze({
    type,
    pointerType,
    target,
    originalEvent,
    x: position?.x ?? null,
    y: position?.y ?? null,
    deltaX,
    deltaY,
    altKey: modifier(originalEvent, "altKey"),
    ctrlKey: modifier(originalEvent, "ctrlKey"),
    metaKey: modifier(originalEvent, "metaKey"),
    shiftKey: modifier(originalEvent, "shiftKey"),
    isCanceled,
  }) as MoveEvent;
}

export function surfaceErrors(errors: readonly unknown[], message: string): void {
  if (errors.length === 1) throw errors[0];
  if (errors.length < 2) return;
  const Aggregate = globalThis.AggregateError as typeof AggregateError | undefined;
  if (typeof Aggregate === "function") throw new Aggregate(errors, message);
  const fallback = Object.assign(new Error(message), { errors: [...errors] });
  fallback.name = "AggregateError";
  throw fallback;
}

export function capture(errors: unknown[], callback: () => void): void {
  try {
    callback();
  } catch (error) {
    errors.push(error);
  }
}

export function validateOptions(options: MoveOptions): void {
  for (const name of ["onMove", "onMoveEnd", "onMoveStart"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.keyboardStep !== "function") readKeyboardStep(options.keyboardStep);
}

/** Install and exhaustively clean up the document listeners for one pointer family. */
export function installMoveListeners(
  document: Document,
  source: PointerSource,
  context: MoveListenerContext,
): () => void {
  const removals: Array<() => void> = [];
  const listen = (
    owner: Document | Window,
    type: string,
    callback: EventListener,
    capture = true,
  ) => {
    owner.addEventListener(type, callback, capture);
    removals.push(() => owner.removeEventListener(type, callback, capture));
  };
  const matches = (family: PointerSource, id: number | null): ActiveMove | null => {
    const current = context.getActive();
    return current?.source === family && current.id === id ? current : null;
  };
  const pointerMove = (event: PointerEvent) => {
    const current = matches("pointer", event.pointerId);
    const position = positionOf(event);
    if (current && position) context.emitDelta(current, event, position);
  };
  const pointerEnd = (event: PointerEvent, canceled: boolean) => {
    const current = matches("pointer", event.pointerId);
    if (!current) return;
    const disabled = context.readDisabled(event);
    context.finish(event, canceled || disabled || !current.target.isConnected);
  };
  const mouseMove = (event: MouseEvent) => {
    const current = matches("mouse", null);
    const position = positionOf(event);
    if (current && position) context.emitDelta(current, event, position);
  };
  const mouseUp = (event: MouseEvent) => {
    const current = matches("mouse", null);
    if (event.button === 0 && current) {
      context.finish(event, context.readDisabled(event) || !current.target.isConnected);
    }
  };
  const touchMove = (event: TouchEvent) => {
    const candidate = context.getActive();
    const current = candidate?.source === "touch" ? candidate : null;
    const position = current ? positionOf(event, current.id) : null;
    if (current && position) context.emitDelta(current, event, position);
  };
  const touchEnd = (event: TouchEvent, canceled: boolean) => {
    const candidate = context.getActive();
    const current = candidate?.source === "touch" ? candidate : null;
    if (!current || !positionOf(event, current.id)) return;
    context.rememberTouch(event.timeStamp);
    context.finish(event, canceled || context.readDisabled(event) || !current.target.isConnected);
  };
  try {
    if (source === "pointer") {
      listen(document, "pointermove", pointerMove as EventListener);
      listen(document, "pointerup", ((event: PointerEvent) =>
        pointerEnd(event, false)) as EventListener);
      listen(document, "pointercancel", ((event: PointerEvent) =>
        pointerEnd(event, true)) as EventListener);
    } else if (source === "mouse") {
      listen(document, "mousemove", mouseMove as EventListener);
      listen(document, "mouseup", mouseUp as EventListener);
    } else {
      listen(document, "touchmove", touchMove as EventListener);
      listen(document, "touchend", ((event: TouchEvent) =>
        touchEnd(event, false)) as EventListener);
      listen(document, "touchcancel", ((event: TouchEvent) =>
        touchEnd(event, true)) as EventListener);
    }
    listen(document, "dragstart", (event) => context.finish(event, true));
    listen(document, "visibilitychange", (() => {
      if (document.visibilityState === "hidden") context.finish(null, true);
    }) as EventListener);
    if (document.defaultView) {
      listen(document.defaultView, "blur", (event) => context.finish(event, true), false);
    }
  } catch (error) {
    const errors: unknown[] = [error];
    for (const remove of removals.reverse()) capture(errors, remove);
    surfaceErrors(errors, "Move listener setup failed");
    throw error;
  }
  let released = false;
  return () => {
    if (released) return;
    released = true;
    const errors: unknown[] = [];
    for (const remove of removals) capture(errors, remove);
    surfaceErrors(errors, "Move listener cleanup failed");
  };
}
