import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";
import type { ShallowRef } from "vue";

import type { StartViewTransitionOptions, ViewTransitionHandle } from "./motion-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_MOTION_OPTION";
const setupDiagnostic = "VIZE_UI_MOTION_SETUP";
const reducedMotionQuery = "(prefers-reduced-motion: reduce)";

interface NativeViewTransition {
  readonly finished: Promise<unknown>;
  readonly ready: Promise<unknown>;
  readonly updateCallbackDone: Promise<unknown>;
  skipTransition(): void;
}

type ViewTransitionDocument = Document & {
  readonly startViewTransition?: (update: () => Promise<void>) => NativeViewTransition;
};

/** Whether the current environment matches `prefers-reduced-motion: reduce`. */
export function prefersReducedMotion(): boolean {
  if (typeof globalThis.matchMedia !== "function") return false;
  return globalThis.matchMedia(reducedMotionQuery).matches;
}

/** Whether this environment can run native document view transitions. */
export function supportsViewTransitions(): boolean {
  if (typeof document === "undefined") return false;
  return typeof (document as ViewTransitionDocument).startViewTransition === "function";
}

/** Whether this environment supports `@starting-style` first-render transitions. */
export function supportsStartingStyle(): boolean {
  return "CSSStartingStyleRule" in globalThis;
}

/** Whether this environment supports scroll-driven animation timelines. */
export function supportsScrollDrivenAnimations(): boolean {
  if (typeof CSS === "undefined" || typeof CSS.supports !== "function") return false;
  return CSS.supports("animation-timeline: scroll()");
}

function readBoolean(value: boolean | undefined, fallback: boolean): boolean {
  if (value === undefined) return fallback;
  if (typeof value !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: respectReducedMotion must be a boolean`);
  }
  return value;
}

function toHandledVoid(promise: Promise<unknown>): Promise<void> {
  const result = promise.then(() => undefined);
  // Pre-handle so callers that ignore this phase never see an unhandled rejection.
  result.catch(() => undefined);
  return result;
}

/**
 * Run a DOM update through the View Transitions API when available.
 *
 * SSR-safe: without a document, without native support, or under reduced
 * motion (unless opted out) the update runs directly and the handle resolves
 * once it completes, so callers write one code path for every platform.
 */
export function startViewTransition(
  update: () => void | Promise<void>,
  options: StartViewTransitionOptions = {},
): ViewTransitionHandle {
  if (typeof update !== "function") {
    throw new TypeError(`${invalidOptionDiagnostic}: update must be a function`);
  }
  const respectReducedMotion = readBoolean(options.respectReducedMotion, true);

  if (!supportsViewTransitions() || (respectReducedMotion && prefersReducedMotion())) {
    const updateCallbackDone = toHandledVoid(Promise.resolve().then(update));
    return Object.freeze({
      native: false,
      finished: updateCallbackDone,
      ready: updateCallbackDone,
      updateCallbackDone,
      skipTransition: () => undefined,
    });
  }

  const transition = (document as ViewTransitionDocument).startViewTransition?.(() =>
    Promise.resolve(update()).then(() => undefined),
  );
  if (transition === undefined) {
    throw new Error(`${invalidOptionDiagnostic}: the platform revoked view transition support`);
  }
  return Object.freeze({
    native: true,
    finished: toHandledVoid(transition.finished),
    ready: toHandledVoid(transition.ready),
    updateCallbackDone: toHandledVoid(transition.updateCallbackDone),
    skipTransition: () => {
      transition.skipTransition();
    },
  });
}

/**
 * Reactive `prefers-reduced-motion` state for motion policy decisions.
 *
 * SSR-safe: without `matchMedia` the ref stays `false`. The media listener is
 * released with the current Vue effect scope.
 */
export function useReducedMotion(): Readonly<ShallowRef<boolean>> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const reduced = shallowRef(false);
  if (typeof globalThis.matchMedia === "function") {
    const query = globalThis.matchMedia(reducedMotionQuery);
    reduced.value = query.matches;
    const onChange = (event: MediaQueryListEvent): void => {
      reduced.value = event.matches;
    };
    query.addEventListener("change", onChange);
    onScopeDispose(() => {
      query.removeEventListener("change", onChange);
    });
  }
  return shallowReadonly(reduced);
}
