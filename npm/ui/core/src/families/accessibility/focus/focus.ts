import { getCurrentScope, isRef, onScopeDispose, shallowReadonly, shallowRef, watch } from "vue";

import {
  activeElementOf,
  capture,
  composedContains,
  createFocusEvent,
  eventElement,
  ownsFocus,
  readBoolean,
  surfaceErrors,
  validateBoolean,
  validateElement,
  validateOptions,
} from "./focus-internal.ts";
import {
  createInteractionModalityTracker,
  isElementFocusVisible,
} from "../interaction-modality/interaction-modality.ts";
import type {
  FocusChangeReason,
  FocusController,
  FocusEvent,
  FocusMode,
  FocusOptions,
  FocusProps,
  FocusRingController,
  FocusRingOptions,
  FocusWithinOptions,
} from "./focus-types.ts";

const disposedDiagnostic = "VIZE_UI_FOCUS_DISPOSED";
const setupDiagnostic = "VIZE_UI_FOCUS_SETUP";

interface ActiveFocus {
  focusedTarget: Element;
  readonly host: Element;
  readonly release: () => void;
}

function installSettlementObserver(mode: FocusMode, host: Element, settle: () => void): () => void {
  const document = host.ownerDocument;
  const onFocusIn = () => {
    if (!ownsFocus(mode, host, activeElementOf(host))) settle();
  };
  document.addEventListener("focusin", onFocusIn, true);
  const Observer = document.defaultView?.MutationObserver ?? globalThis.MutationObserver;
  let observer: MutationObserver | null = null;
  try {
    observer =
      typeof Observer === "function"
        ? new Observer(() => {
            if (!host.isConnected || !ownsFocus(mode, host, activeElementOf(host))) settle();
          })
        : null;
    observer?.observe(document, { childList: true, subtree: true });
  } catch (error) {
    const errors: unknown[] = [error];
    capture(errors, () => document.removeEventListener("focusin", onFocusIn, true));
    capture(errors, () => observer?.disconnect());
    surfaceErrors(errors, "Focus observer setup failed");
  }
  return () => {
    const errors: unknown[] = [];
    capture(errors, () => document.removeEventListener("focusin", onFocusIn, true));
    capture(errors, () => observer?.disconnect());
    surfaceErrors(errors, "Focus observer cleanup failed");
  };
}

/** Create an SSR-safe observer for direct or composed-descendant focus. */
export function createFocus(options: FocusOptions = {}): FocusController {
  const mode = validateOptions(options);
  const focused = shallowRef(false);
  const focusVisible = shallowRef(false);
  let active: ActiveFocus | null = null;
  let host: Element | null = null;
  let disposed = false;
  let transitionVersion = 0;

  const updateVisibility = (): boolean => {
    const current = active;
    const next = Boolean(
      current &&
      (options.autoFocus || isElementFocusVisible(current.focusedTarget, modality.modality.value)),
    );
    if (focusVisible.value === next) return false;
    focusVisible.value = next;
    return true;
  };
  const modality = createInteractionModalityTracker({
    onChange: updateVisibility,
  });

  const notify = (next: boolean, event: FocusEvent): void => {
    const errors: unknown[] = [];
    const version = ++transitionVersion;
    capture(errors, () => options.onFocusChange?.(next, event));
    if (transitionVersion === version) {
      capture(errors, () => (next ? options.onFocus?.(event) : options.onBlur?.(event)));
    }
    surfaceErrors(errors, "Focus callbacks failed");
  };

  const settle = (
    reason: FocusChangeReason,
    originalEvent: globalThis.FocusEvent | null,
    relatedTarget: Element | null,
    notifyCallbacks = true,
  ): boolean => {
    const current = active;
    if (!current) return false;
    active = null;
    focused.value = false;
    focusVisible.value = false;
    const event = createFocusEvent(
      "blur",
      mode,
      current.host,
      current.focusedTarget,
      relatedTarget,
      originalEvent,
      false,
      reason,
    );
    const errors: unknown[] = [];
    capture(errors, current.release);
    capture(errors, modality.detach);
    if (notifyCallbacks) capture(errors, () => notify(false, event));
    else transitionVersion++;
    surfaceErrors(errors, "Focus settlement failed");
    return true;
  };

  const readDisabled = (event: globalThis.FocusEvent | null): boolean => {
    try {
      return readBoolean(options.isDisabled, "isDisabled");
    } catch (error) {
      const errors: unknown[] = [error];
      capture(errors, () => settle("disabled", event, eventElement(event?.relatedTarget ?? null)));
      surfaceErrors(errors, "Focus option validation failed during teardown");
      throw error;
    }
  };

  const acquire = (
    nextHost: Element,
    focusedTarget: Element,
    reason: FocusChangeReason,
    originalEvent: globalThis.FocusEvent | null,
  ): boolean => {
    if (disposed) return false;
    if (readDisabled(originalEvent)) {
      settle("disabled", originalEvent, focusedTarget);
      return false;
    }
    if (active?.host === nextHost) {
      active.focusedTarget = focusedTarget;
      return updateVisibility();
    }
    if (active) settle(reason, originalEvent, focusedTarget);
    host = nextHost;
    modality.attach(nextHost.ownerDocument);
    let release: () => void;
    try {
      release = installSettlementObserver(mode, nextHost, () => {
        settle("focus", null, activeElementOf(nextHost));
      });
    } catch (error) {
      const errors: unknown[] = [error];
      capture(errors, modality.detach);
      surfaceErrors(errors, "Focus setup failed");
      throw error;
    }
    active = { focusedTarget, host: nextHost, release };
    focused.value = true;
    updateVisibility();
    const event = createFocusEvent(
      "focus",
      mode,
      nextHost,
      focusedTarget,
      null,
      originalEvent,
      focusVisible.value,
      reason,
    );
    notify(true, event);
    return true;
  };

  const reconcile = (nextHost: Element, reason: FocusChangeReason): boolean => {
    host = nextHost;
    const beforeFocused = focused.value;
    const beforeVisible = focusVisible.value;
    if (readDisabled(null)) {
      settle("disabled", null, activeElementOf(nextHost));
      return beforeFocused !== focused.value || beforeVisible !== focusVisible.value;
    }
    const current = activeElementOf(nextHost);
    if (ownsFocus(mode, nextHost, current) && current) acquire(nextHost, current, reason, null);
    else settle(reason, null, current);
    return beforeFocused !== focused.value || beforeVisible !== focusVisible.value;
  };

  const enter = (event: globalThis.FocusEvent): void => {
    const nextHost = eventElement(event.currentTarget);
    const eventTarget = eventElement(event.target);
    if (!nextHost || !eventTarget || (mode === "target" && eventTarget !== nextHost)) return;
    acquire(nextHost, activeElementOf(nextHost) ?? eventTarget, "focus", event);
  };
  const leave = (event: globalThis.FocusEvent): void => {
    const current = active;
    if (!current || current.host !== eventElement(event.currentTarget)) return;
    const related = eventElement(event.relatedTarget);
    if (mode === "within" && composedContains(current.host, related)) {
      if (related) current.focusedTarget = related;
      return;
    }
    settle("focus", event, related);
  };
  const focusProps: Readonly<FocusProps> = Object.freeze(
    mode === "target" ? { onBlur: leave, onFocus: enter } : { onFocusin: enter, onFocusout: leave },
  );

  let stopDisabledWatch: () => void = () => undefined;
  const disabledSource = options.isDisabled;
  if (isRef(disabledSource) || typeof disabledSource === "function") {
    stopDisabledWatch = watch(
      () => (isRef(disabledSource) ? disabledSource.value : disabledSource()),
      (value) => {
        try {
          if (validateBoolean(value, "isDisabled")) {
            settle("disabled", null, host ? activeElementOf(host) : null);
          }
        } catch (error) {
          const errors: unknown[] = [error];
          capture(errors, () => settle("disabled", null, host ? activeElementOf(host) : null));
          surfaceErrors(errors, "Focus option validation failed during teardown");
        }
      },
      { flush: "sync" },
    );
  }

  return Object.freeze({
    isFocused: shallowReadonly(focused),
    isFocusVisible: shallowReadonly(focusVisible),
    focusProps,
    refresh: (target: Element) => {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      return reconcile(validateElement(target), "refresh");
    },
    cancel: () => {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      return settle("manual", null, host ? activeElementOf(host) : null);
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      stopDisabledWatch();
      const errors: unknown[] = [];
      capture(errors, () => settle("manual", null, null, false));
      capture(errors, modality.dispose);
      surfaceErrors(errors, "Focus disposal failed");
    },
  });
}

/** Observe composed-descendant focus ownership. */
export function createFocusWithin(options: FocusWithinOptions = {}): FocusController {
  return createFocus({ ...options, mode: "within" });
}

/** Create direct focus and focus-visible state for a visible ring. */
export function createFocusRing(options: FocusRingOptions = {}): FocusRingController {
  return createFocus(options);
}

function useScopedFocus(options: FocusOptions): FocusController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createFocus(options);
  onScopeDispose(controller.dispose);
  return controller;
}

/** Create a direct focus observer disposed with the current Vue effect scope. */
export function useFocus(options: FocusOptions = {}): FocusController {
  return useScopedFocus(options);
}

/** Create a focus-within observer disposed with the current Vue effect scope. */
export function useFocusWithin(options: FocusWithinOptions = {}): FocusController {
  return useScopedFocus({ ...options, mode: "within" });
}

/** Create focus-ring state disposed with the current Vue effect scope. */
export function useFocusRing(options: FocusRingOptions = {}): FocusRingController {
  return useScopedFocus(options);
}

export type {
  FocusChangeReason,
  FocusController,
  FocusEvent,
  FocusMode,
  FocusOptions,
  FocusProps,
  FocusRingController,
  FocusRingOptions,
  FocusWithinOptions,
} from "./focus-types.ts";
