import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";

import { disableTextSelection } from "../press/press-event.ts";
import { createPress } from "../press/press.ts";
import {
  captureError,
  installTriggeredRelease,
  isHardwarePointer,
  ownerMatches,
  ownerOf,
  readBoolean,
  readPointerType,
  readText,
  readThreshold,
  surfaceErrors,
  toLongPressEvent,
  validateOptions,
} from "./long-press-internal.ts";
import type { Attempt } from "./long-press-internal.ts";
import type {
  LongPressController,
  LongPressOptions,
  LongPressPointerType,
  LongPressProps,
} from "./long-press-types.ts";
const contextMenuLingerMs = 50;
const disposedDiagnostic = "VIZE_UI_LONG_PRESS_DISPOSED";
const setupDiagnostic = "VIZE_UI_LONG_PRESS_SETUP";

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
    }, contextMenuLingerMs);
  };
  const clearRelease = () => {
    const release = releaseTriggered;
    const restore = restoreTriggeredSelection;
    releaseTriggered = null;
    restoreTriggeredSelection = null;
    const errors: unknown[] = [];
    try {
      captureError(errors, () => release?.());
    } finally {
      captureError(errors, () => restore?.());
    }
    surfaceErrors(errors, "Long-press release cleanup failed");
  };
  const clearAttempt = (owner: Attempt | null = attempt): boolean => {
    if (!owner || attempt !== owner) return false;
    const timer = owner.timer;
    if (timer != null) clearTimeout(timer);
    attempt = null;
    return true;
  };

  const finishTriggered = (
    owner: Attempt,
    originalEvent: Event | null,
    isCanceled: boolean,
  ): boolean => {
    if (!isLongPressed.value || attempt !== owner) return false;
    const errors: unknown[] = [];
    captureError(errors, clearRelease);
    clearAttempt(owner);
    isPressed.value = false;
    isLongPressed.value = false;
    captureError(errors, lingerContextMenuSuppression);
    let canceled = isCanceled;
    try {
      canceled ||= readBoolean(options.isDisabled, "isDisabled");
    } catch (error) {
      canceled = true;
      errors.push(error);
    }
    captureError(errors, () =>
      options.onLongPressEnd?.(
        toLongPressEvent(
          "longpressend",
          owner.event,
          originalEvent,
          canceled,
          owner.touchIdentifier,
        ),
      ),
    );
    surfaceErrors(errors, "Long-press completion failed");
    return true;
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
    try {
      releaseTriggered = installTriggeredRelease(current, (event, canceled = false) =>
        finishTriggered(current, event, canceled),
      );
    } catch (error) {
      const errors: unknown[] = [error];
      clearAttempt(current);
      isPressed.value = false;
      isLongPressed.value = false;
      contextMenuPointer = null;
      endingAtThreshold = true;
      try {
        captureError(errors, press.cancel);
      } finally {
        endingAtThreshold = false;
      }
      surfaceErrors(errors, "Long-press trigger setup failed");
      return;
    }
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
    options.onLongPress?.(
      toLongPressEvent(
        "longpress",
        current.event,
        current.event.originalEvent,
        current.event.isCanceled,
        current.touchIdentifier,
      ),
    );
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
      if (!isHardwarePointer(event.pointerType)) return;
      const pointerType = event.pointerType;
      const filter = readPointerType(options.pointerType);
      if (filter && filter !== pointerType) return;
      if (attempt) {
        press.cancel();
        return;
      }
      contextMenuPointer = pointerType;
      clearContextMenuTimer();
      const owner = ownerOf(event);
      const touchIdentifier = owner.source === "touch" ? owner.id : null;
      const start = toLongPressEvent(
        "longpressstart",
        event,
        event.originalEvent,
        event.isCanceled,
        touchIdentifier,
      );
      const current: Attempt = {
        event: start,
        owner,
        pointerType,
        target: event.target,
        touchIdentifier,
        timer: null,
      };
      attempt = current;
      isPressed.value = true;
      current.timer = setTimeout(() => trigger(current), readThreshold(options.threshold));
      options.onLongPressStart?.(start);
    },
    onPressEnd(event) {
      if (!attempt || endingAtThreshold) return;
      const current = attempt;
      if (!ownerMatches(current.owner, event)) return;
      clearAttempt(current);
      isPressed.value = false;
      if (current.pointerType === "touch" || current.pointerType === "pen") {
        lingerContextMenuSuppression();
      } else {
        contextMenuPointer = null;
      }
      options.onLongPressEnd?.(
        toLongPressEvent(
          "longpressend",
          current.event,
          event.originalEvent,
          event.isCanceled,
          current.touchIdentifier,
        ),
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
      if (attempt && finishTriggered(attempt, null, true)) return true;
      return press.cancel();
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      const errors: unknown[] = [];
      captureError(errors, clearRelease);
      captureError(errors, () => clearAttempt());
      captureError(errors, clearContextMenuTimer);
      contextMenuPointer = null;
      isPressed.value = false;
      isLongPressed.value = false;
      captureError(errors, press.dispose);
      surfaceErrors(errors, "Long-press disposal failed");
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
