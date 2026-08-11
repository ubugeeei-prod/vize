import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue } from "vue";

import { createPressEvent, disableTextSelection } from "./press-event.ts";
import { createPress } from "./press.ts";
import type {
  LongPressController,
  LongPressEvent,
  LongPressEventType,
  LongPressOptions,
  LongPressPointerType,
  LongPressProps,
} from "./long-press-types.ts";
import type { PressEvent } from "./press-types.ts";

const defaultThreshold = 500;
const invalidOptionDiagnostic = "VIZE_UI_LONG_PRESS_OPTION";
const disposedDiagnostic = "VIZE_UI_LONG_PRESS_DISPOSED";
const setupDiagnostic = "VIZE_UI_LONG_PRESS_SETUP";
const hardwarePointers = new Set<LongPressPointerType>(["mouse", "pen", "pointer", "touch"]);

interface Attempt {
  readonly event: LongPressEvent;
  readonly pointerType: LongPressPointerType;
  readonly target: Element;
  timer: ReturnType<typeof setTimeout> | null;
}

function readBoolean(value: LongPressOptions["isDisabled"], name: string): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return false;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return resolved;
}

function readPointerType(value: LongPressOptions["pointerType"]): LongPressPointerType | null {
  const resolved = toValue(value) ?? null;
  if (resolved !== null && !hardwarePointers.has(resolved)) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: pointerType must resolve to mouse, pen, pointer, or touch`,
    );
  }
  return resolved;
}

function readThreshold(value: LongPressOptions["threshold"]): number {
  const resolved = toValue(value) ?? defaultThreshold;
  if (typeof resolved !== "number" || !Number.isFinite(resolved) || resolved < 0) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: threshold must resolve to a finite number >= 0`,
    );
  }
  return resolved;
}

function readText(
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

function toLongPressEvent(
  type: LongPressEventType,
  event: Omit<PressEvent, "type">,
  originalEvent = event.originalEvent,
  isCanceled = event.isCanceled,
): LongPressEvent {
  const snapshot = createPressEvent(
    "pressend",
    event.target,
    event.pointerType,
    originalEvent,
    isCanceled,
  );
  return Object.freeze({ ...snapshot, type }) as LongPressEvent;
}

function validateOptions(options: LongPressOptions): void {
  for (const name of ["onLongPress", "onLongPressEnd", "onLongPressStart", "onPress"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.threshold !== "function") readThreshold(options.threshold);
  if (typeof options.pointerType !== "function") readPointerType(options.pointerType);
}

/** Create an SSR-safe long-press recognizer for one host element. */
export function createLongPress(options: LongPressOptions = {}): LongPressController {
  validateOptions(options);
  const isPressed = shallowRef(false);
  const isLongPressed = shallowRef(false);
  let attempt: Attempt | null = null;
  let releaseTriggered: (() => void) | null = null;
  let restoreTriggeredSelection: (() => void) | null = null;
  let contextMenuPointer: LongPressPointerType | null = null;
  let contextMenuTimer: ReturnType<typeof setTimeout> | null = null;
  let endingAtThreshold = false;
  let disposed = false;

  const clearContextMenuTimer = () => {
    if (contextMenuTimer !== null) clearTimeout(contextMenuTimer);
    contextMenuTimer = null;
  };
  const lingerContextMenuSuppression = () => {
    clearContextMenuTimer();
    contextMenuTimer = setTimeout(() => {
      contextMenuPointer = null;
      contextMenuTimer = null;
    }, 50);
  };
  const clearRelease = () => {
    releaseTriggered?.();
    releaseTriggered = null;
    restoreTriggeredSelection?.();
    restoreTriggeredSelection = null;
  };
  const clearAttempt = () => {
    const timer = attempt?.timer;
    if (timer != null) clearTimeout(timer);
    attempt = null;
  };

  const finishTriggered = (originalEvent: Event | null, isCanceled: boolean): boolean => {
    if (!isLongPressed.value || !attempt) return false;
    const current = attempt;
    const canceled = isCanceled || readBoolean(options.isDisabled, "isDisabled");
    clearRelease();
    clearAttempt();
    isPressed.value = false;
    isLongPressed.value = false;
    lingerContextMenuSuppression();
    options.onLongPressEnd?.(
      toLongPressEvent("longpressend", current.event, originalEvent, canceled),
    );
    return true;
  };

  const installTriggeredRelease = (current: Attempt): (() => void) => {
    const removals: Array<() => void> = [];
    const document = current.target.ownerDocument;
    const start = current.event.originalEvent;
    const listen = (
      owner: Document | Window,
      type: string,
      callback: EventListener,
      capture = true,
    ) => {
      owner.addEventListener(type, callback, capture);
      removals.push(() => owner.removeEventListener(type, callback, capture));
    };
    const finish = (event: Event | null, canceled = false) => finishTriggered(event, canceled);
    if (start && "pointerId" in start) {
      const id = Number((start as PointerEvent).pointerId);
      listen(document, "pointerup", ((event: PointerEvent) => {
        if (event.pointerId === id) finish(event);
      }) as EventListener);
      listen(document, "pointercancel", ((event: PointerEvent) => {
        if (event.pointerId === id) finish(event, true);
      }) as EventListener);
    } else if (start && "changedTouches" in start) {
      const id = (start as TouchEvent).changedTouches.item(0)?.identifier;
      const ownsTouch = (event: TouchEvent) =>
        id !== undefined &&
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
    return () => {
      for (const remove of removals.splice(0)) remove();
    };
  };

  let press!: ReturnType<typeof createPress>;
  const trigger = (current: Attempt) => {
    if (disposed || attempt !== current) return;
    current.timer = null;
    if (readBoolean(options.isDisabled, "isDisabled")) {
      press.cancel();
      return;
    }
    isLongPressed.value = true;
    releaseTriggered = installTriggeredRelease(current);
    endingAtThreshold = true;
    try {
      press.cancel();
    } finally {
      endingAtThreshold = false;
    }
    const focusable = current.target as HTMLElement;
    if (
      (current.pointerType === "touch" || current.pointerType === "pen") &&
      current.target.ownerDocument.activeElement !== current.target &&
      typeof focusable.focus === "function"
    ) {
      try {
        focusable.focus({ preventScroll: true });
      } catch {
        focusable.focus();
      }
    }
    if (!readBoolean(options.allowTextSelectionOnPress, "allowTextSelectionOnPress")) {
      restoreTriggeredSelection = disableTextSelection(current.target);
    }
    options.onLongPress?.(toLongPressEvent("longpress", current.event));
  };

  press = createPress({
    ...(options.isDisabled === undefined ? {} : { isDisabled: options.isDisabled }),
    ...(options.allowTextSelectionOnPress === undefined
      ? {}
      : { allowTextSelectionOnPress: options.allowTextSelectionOnPress }),
    ...(options.preventFocusOnPress === undefined
      ? {}
      : { preventFocusOnPress: options.preventFocusOnPress }),
    shouldCancelOnPointerExit: true,
    onPressStart(event) {
      if (!hardwarePointers.has(event.pointerType as LongPressPointerType)) return;
      const pointerType = event.pointerType as LongPressPointerType;
      const filter = readPointerType(options.pointerType);
      if (filter && filter !== pointerType) return;
      clearAttempt();
      contextMenuPointer = pointerType;
      clearContextMenuTimer();
      const start = toLongPressEvent("longpressstart", event);
      const current: Attempt = { event: start, pointerType, target: event.target, timer: null };
      attempt = current;
      isPressed.value = true;
      current.timer = setTimeout(() => trigger(current), readThreshold(options.threshold));
      options.onLongPressStart?.(start);
    },
    onPressEnd(event) {
      if (!attempt || endingAtThreshold) return;
      const current = attempt;
      clearAttempt();
      isPressed.value = false;
      if (current.pointerType === "touch" || current.pointerType === "pen") {
        lingerContextMenuSuppression();
      } else {
        contextMenuPointer = null;
      }
      options.onLongPressEnd?.(
        toLongPressEvent("longpressend", current.event, event.originalEvent, event.isCanceled),
      );
    },
    ...(options.onPress ? { onPress: options.onPress } : {}),
  });

  const attributes = {
    ...press.pressProps,
    get "aria-describedby"() {
      if (readBoolean(options.isDisabled, "isDisabled") || !options.onLongPress) return undefined;
      return readText(options.accessibilityDescriptionId, "accessibilityDescriptionId");
    },
    get "aria-description"() {
      if (readBoolean(options.isDisabled, "isDisabled") || !options.onLongPress) return undefined;
      if (readText(options.accessibilityDescriptionId, "accessibilityDescriptionId"))
        return undefined;
      return readText(options.accessibilityDescription, "accessibilityDescription");
    },
    onContextmenu(event: MouseEvent) {
      if (contextMenuPointer === "touch" || contextMenuPointer === "pen") event.preventDefault();
    },
  } satisfies LongPressProps;

  const controller: LongPressController = Object.freeze({
    isPressed: shallowReadonly(isPressed),
    isLongPressed: shallowReadonly(isLongPressed),
    longPressProps: Object.freeze(attributes),
    cancel: () => {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      if (finishTriggered(null, true)) return true;
      return press.cancel();
    },
    dispose: () => {
      if (disposed) return;
      clearRelease();
      clearAttempt();
      clearContextMenuTimer();
      contextMenuPointer = null;
      isPressed.value = false;
      isLongPressed.value = false;
      press.dispose();
      disposed = true;
    },
  });
  return controller;
}

/** Create a long-press recognizer disposed with the current Vue effect scope. */
export function useLongPress(options: LongPressOptions = {}): LongPressController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createLongPress(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  LongPressController,
  LongPressEvent,
  LongPressEventType,
  LongPressOptions,
  LongPressPointerType,
  LongPressProps,
} from "./long-press-types.ts";
