import { toValue } from "vue";

import { createPressEvent } from "../press/press-event.ts";
import type {
  LongPressEvent,
  LongPressEventType,
  LongPressOptions,
  LongPressPointerType,
} from "./long-press-types.ts";
import type { PressEvent } from "../press/press-types.ts";

const defaultThreshold = 500;
const invalidOptionDiagnostic = "VIZE_UI_LONG_PRESS_OPTION";
const hardwarePointers = new Set<LongPressPointerType>(["mouse", "pen", "pointer", "touch"]);

export interface Attempt {
  readonly event: LongPressEvent;
  readonly owner: AttemptOwner;
  readonly pointerType: LongPressPointerType;
  readonly target: Element;
  readonly touchIdentifier: number | null;
  timer: ReturnType<typeof setTimeout> | null;
}

type AttemptOwner =
  | { readonly source: "mouse" }
  | { readonly id: number; readonly source: "pointer" | "touch" };

export function isHardwarePointer(value: string): value is LongPressPointerType {
  return hardwarePointers.has(value as LongPressPointerType);
}

export function ownerOf(event: Omit<PressEvent, "type">): AttemptOwner {
  const original = event.originalEvent;
  if (original && "pointerId" in original) {
    return { id: Number((original as PointerEvent).pointerId), source: "pointer" };
  }
  if (original && "changedTouches" in original) {
    const id = (original as TouchEvent).changedTouches.item(0)?.identifier;
    if (id !== undefined) return { id, source: "touch" };
  }
  return { source: "mouse" };
}

export function ownerMatches(owner: AttemptOwner, event: Omit<PressEvent, "type">): boolean {
  const original = event.originalEvent;
  if (owner.source === "pointer") {
    return Boolean(
      original &&
      "pointerId" in original &&
      Number((original as PointerEvent).pointerId) === owner.id,
    );
  }
  if (owner.source === "touch") {
    return Boolean(
      original &&
      "changedTouches" in original &&
      Array.from((original as TouchEvent).changedTouches).some(
        ({ identifier }) => identifier === owner.id,
      ),
    );
  }
  return Boolean(original && !("pointerId" in original) && !("changedTouches" in original));
}

export function captureError(errors: unknown[], callback: () => void): void {
  try {
    callback();
  } catch (error) {
    errors.push(error);
  }
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

export function readBoolean(value: LongPressOptions["isDisabled"], name: string): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return false;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return resolved;
}

export function readPointerType(
  value: LongPressOptions["pointerType"],
): LongPressPointerType | null {
  const resolved = toValue(value) ?? null;
  if (resolved !== null && !hardwarePointers.has(resolved)) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: pointerType must resolve to mouse, pen, pointer, or touch`,
    );
  }
  return resolved;
}

export function readThreshold(value: LongPressOptions["threshold"]): number {
  const resolved = toValue(value) ?? defaultThreshold;
  if (typeof resolved !== "number" || !Number.isFinite(resolved) || resolved < 0) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: threshold must resolve to a finite number >= 0`,
    );
  }
  return resolved;
}

export function readText(
  value: LongPressOptions["accessibilityDescription"],
  name: string,
): string | undefined {
  const resolved = toValue(value);
  if (resolved === undefined || resolved === "") return undefined;
  if (typeof resolved !== "string") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must resolve to a string`);
  }
  return resolved;
}

export function toLongPressEvent(
  type: LongPressEventType,
  event: Omit<PressEvent, "type">,
  originalEvent = event.originalEvent,
  isCanceled = event.isCanceled,
  touchIdentifier: number | null = null,
): LongPressEvent {
  const snapshot = createPressEvent(
    "pressend",
    event.target,
    event.pointerType,
    originalEvent,
    isCanceled,
    touchIdentifier,
  );
  return Object.freeze({ ...snapshot, type }) as LongPressEvent;
}

export function validateOptions(options: LongPressOptions): void {
  for (const name of ["onLongPress", "onLongPressEnd", "onLongPressStart", "onPress"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.threshold !== "function") readThreshold(options.threshold);
  if (typeof options.pointerType !== "function") readPointerType(options.pointerType);
}

/** Install physical-release listeners for an already triggered attempt. */
export function installTriggeredRelease(
  current: Attempt,
  finish: (event: Event | null, canceled?: boolean) => boolean,
): () => void {
  const removals: Array<() => void> = [];
  const document = current.target.ownerDocument;
  const listen = (
    owner: Document | Window,
    type: string,
    callback: EventListener,
    capture = true,
  ) => {
    owner.addEventListener(type, callback, capture);
    removals.push(() => owner.removeEventListener(type, callback, capture));
  };
  try {
    if (current.owner.source === "pointer") {
      const id = current.owner.id;
      listen(document, "pointerup", ((event: PointerEvent) => {
        if (event.pointerId === id) finish(event);
      }) as EventListener);
      listen(document, "pointercancel", ((event: PointerEvent) => {
        if (event.pointerId === id) finish(event, true);
      }) as EventListener);
    } else if (current.owner.source === "touch") {
      const id = current.owner.id;
      const ownsTouch = (event: TouchEvent) =>
        Array.from(event.changedTouches).some((touch) => touch.identifier === id);
      listen(document, "touchend", ((event: TouchEvent) => {
        if (ownsTouch(event)) finish(event);
      }) as EventListener);
      listen(document, "touchcancel", ((event: TouchEvent) => {
        if (ownsTouch(event)) finish(event, true);
      }) as EventListener);
    } else {
      listen(document, "mouseup", ((event: MouseEvent) => {
        if (event.button === 0) finish(event);
      }) as EventListener);
    }
    listen(document, "dragstart", (event) => finish(event, true));
    listen(document, "visibilitychange", (() => {
      if (document.visibilityState === "hidden") finish(null, true);
    }) as EventListener);
    if (document.defaultView) {
      listen(document.defaultView, "blur", (event) => finish(event, true), false);
    }
  } catch (error) {
    const errors: unknown[] = [error];
    for (const remove of removals.reverse()) captureError(errors, remove);
    surfaceErrors(errors, "Long-press listener setup failed");
    throw error;
  }
  return () => {
    const errors: unknown[] = [];
    for (const remove of removals.splice(0)) captureError(errors, remove);
    surfaceErrors(errors, "Long-press listener cleanup failed");
  };
}
