import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue, watch } from "vue";

import type {
  PresenceController,
  PresenceOptions,
  PresenceProps,
  PresenceStatus,
} from "./presence-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_PRESENCE_OPTION";
const disposedDiagnostic = "VIZE_UI_PRESENCE_DISPOSED";
const setupDiagnostic = "VIZE_UI_PRESENCE_SETUP";

function readBoolean(value: PresenceOptions["respectReducedMotion"], fallback: boolean): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return fallback;
  if (typeof resolved !== "boolean") {
    throw new TypeError(
      `${invalidOptionDiagnostic}: respectReducedMotion must resolve to a boolean`,
    );
  }
  return resolved;
}

function readPresent(value: PresenceOptions["present"]): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return false;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: present must resolve to a boolean`);
  }
  return resolved;
}

function prefersReducedMotion(): boolean {
  if (typeof globalThis.matchMedia !== "function") return false;
  return globalThis.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function validateOptions(options: PresenceOptions): void {
  for (const name of ["onEnterComplete", "onExitComplete"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.present !== "function") readPresent(options.present);
  if (typeof options.respectReducedMotion !== "function") {
    readBoolean(options.respectReducedMotion, true);
  }
}

function notify(callback: (() => void) | undefined): void {
  callback?.();
}

/** Create an SSR-safe enter/exit presence controller. */
export function createPresence(options: PresenceOptions = {}): PresenceController {
  validateOptions(options);
  const initiallyPresent = readPresent(options.present);
  const status = shallowRef<PresenceStatus>(initiallyPresent ? "present" : "unmounted");
  const isPresent = shallowRef(initiallyPresent);
  let disposed = false;

  const skipMotion = (): boolean =>
    readBoolean(options.respectReducedMotion, true) && prefersReducedMotion();

  const finishEnter = (): void => {
    status.value = "present";
    notify(options.onEnterComplete);
  };

  const finishExit = (): void => {
    isPresent.value = false;
    status.value = "unmounted";
    notify(options.onExitComplete);
  };

  const enter = (): void => {
    if (status.value === "present" || status.value === "entering") return;
    isPresent.value = true;
    if (skipMotion()) {
      finishEnter();
      return;
    }
    status.value = "entering";
  };

  const exit = (): void => {
    if (status.value === "unmounted") return;
    if (skipMotion() || status.value === "entering") {
      finishExit();
      return;
    }
    status.value = "exiting";
  };

  const apply = (): void => {
    if (readPresent(options.present)) enter();
    else exit();
  };

  const completeAnimation = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
    if (status.value === "entering") finishEnter();
    else if (status.value === "exiting") finishExit();
  };

  const onComplete = (event: AnimationEvent | TransitionEvent): void => {
    if (event.target !== event.currentTarget) return;
    completeAnimation();
  };

  const stopWatch = watch(
    () => [toValue(options.present), toValue(options.respectReducedMotion)],
    apply,
    { flush: "sync" },
  );

  const presenceProps: Readonly<PresenceProps> = Object.freeze({
    onAnimationend: onComplete,
    onTransitionend: onComplete,
  });

  return Object.freeze({
    isPresent: shallowReadonly(isPresent),
    status: shallowReadonly(status),
    presenceProps,
    completeAnimation,
    dispose: () => {
      if (disposed) return;
      disposed = true;
      stopWatch();
    },
  });
}

/** Create a presence controller disposed with the current Vue effect scope. */
export function usePresence(options: PresenceOptions = {}): PresenceController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createPresence(options);
  onScopeDispose(controller.dispose);
  return controller;
}
