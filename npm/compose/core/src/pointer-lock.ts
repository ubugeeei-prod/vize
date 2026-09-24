import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Document-like capability used by {@link usePointerLock}. */
export interface PointerLockHost extends EventTarget {
  /** Element that currently holds the pointer lock. */
  readonly pointerLockElement: Element | null;
  /** Release the pointer lock. */
  exitPointerLock(): void;
}

/** Stable error codes rejected by {@link PointerLockControls.lock} and `unlock`. */
export type PointerLockErrorCode =
  | "VIZE_COMPOSE_POINTER_LOCK_UNSUPPORTED"
  | "VIZE_COMPOSE_POINTER_LOCK_NO_TARGET"
  | "VIZE_COMPOSE_POINTER_LOCK_FAILED";

/** Error thrown when pointer lock cannot be acquired or released. */
export class PointerLockError extends Error {
  /** Machine-readable failure code. */
  readonly code: PointerLockErrorCode;

  /**
   * @param code Machine-readable failure code.
   * @param message Human-readable explanation.
   */
  constructor(code: PointerLockErrorCode, message: string) {
    super(message);
    this.name = "PointerLockError";
    this.code = code;
  }
}

/** Options for {@link usePointerLock}. */
export interface UsePointerLockOptions {
  /**
   * Request raw (unaccelerated) movement deltas where supported.
   *
   * @default false
   */
  readonly unadjustedMovement?: boolean;

  /**
   * Reactive document capability for alternate runtimes and tests.
   *
   * @default globalThis.document when available
   */
  readonly host?: MaybeRefOrGetter<PointerLockHost | null | undefined>;
}

/** Reactive state and actions returned by {@link usePointerLock}. */
export interface PointerLockControls {
  /** Whether the host supports pointer lock. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Element currently holding the lock (any element, not only the target). */
  readonly element: Readonly<ShallowRef<Element | null>>;
  /** Whether the composable's own target holds the lock. */
  readonly isLocked: Readonly<Ref<boolean>>;
  /**
   * Lock the pointer to `target` (or the composable target).
   *
   * @returns The locked element once `pointerlockchange` confirms the lock.
   */
  readonly lock: (target?: MaybeElementTarget) => Promise<Element>;
  /**
   * Release the lock.
   *
   * @returns Whether a lock was held and released.
   */
  readonly unlock: () => Promise<boolean>;
}

/**
 * Reactive wrapper around the Pointer Lock API.
 *
 * Tracks `pointerlockchange` on the document and exposes promise-based lock
 * and unlock actions that settle on the platform confirmation events.
 * Rejections are {@link PointerLockError}s with stable codes. Nothing runs
 * during server rendering; listeners are removed with the owning scope.
 *
 * @param target Default reactive element to lock.
 * @param options Movement mode and document capability.
 * @default target undefined
 * @default options {}
 * @returns Support flag, lock state, and actions.
 */
export function usePointerLock(
  target?: MaybeElementTarget,
  options: UsePointerLockOptions = {},
): PointerLockControls {
  const isSupported = ref(false);
  const element = shallowRef<Element | null>(null);
  const host = (): PointerLockHost | null | undefined =>
    options.host === undefined ? browserPointerLockHost() : toValue(options.host);

  const stop = watch(
    host,
    (current, _previous, onCleanup) => {
      isSupported.value = Boolean(current && "pointerLockElement" in current);
      element.value = current?.pointerLockElement ?? null;
      if (!current) return;
      const onChange = (): void => {
        element.value = current.pointerLockElement;
      };
      current.addEventListener("pointerlockchange", onChange);
      onCleanup(() => current.removeEventListener("pointerlockchange", onChange));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  const isLocked = ref(false);
  watch(
    [element, () => (target === undefined ? null : resolveElement(target))],
    ([locked, own]) => {
      isLocked.value = locked !== null && locked === own;
    },
    { immediate: true, flush: "sync" },
  );

  const lock = async (override?: MaybeElementTarget): Promise<Element> => {
    const current = host();
    if (!current || !isSupported.value) {
      throw new PointerLockError(
        "VIZE_COMPOSE_POINTER_LOCK_UNSUPPORTED",
        "Pointer lock is not supported.",
      );
    }
    const candidate = resolveElement(override ?? target);
    if (!candidate || !isPointerLockable(candidate)) {
      throw new PointerLockError(
        "VIZE_COMPOSE_POINTER_LOCK_NO_TARGET",
        "No lockable element resolved.",
      );
    }
    const confirmed = waitFor(current, () => current.pointerLockElement === candidate);
    try {
      await candidate.requestPointerLock(
        options.unadjustedMovement ? { unadjustedMovement: true } : undefined,
      );
    } catch (cause) {
      confirmed.cancel();
      throw new PointerLockError("VIZE_COMPOSE_POINTER_LOCK_FAILED", describe(cause));
    }
    await confirmed.promise;
    return candidate;
  };

  const unlock = async (): Promise<boolean> => {
    const current = host();
    if (!current?.pointerLockElement) return false;
    const confirmed = waitFor(current, () => current.pointerLockElement === null);
    current.exitPointerLock();
    await confirmed.promise;
    return true;
  };

  return {
    isSupported: readonly(isSupported),
    element,
    isLocked: readonly(isLocked),
    lock,
    unlock,
  };
}

function waitFor(
  host: PointerLockHost,
  done: () => boolean,
): { readonly promise: Promise<void>; readonly cancel: () => void } {
  let cleanup = (): void => undefined;
  const promise = new Promise<void>((resolve, reject) => {
    const onChange = (): void => {
      if (!done()) return;
      cleanup();
      resolve();
    };
    const onError = (): void => {
      cleanup();
      reject(
        new PointerLockError("VIZE_COMPOSE_POINTER_LOCK_FAILED", "The pointer lock was refused."),
      );
    };
    host.addEventListener("pointerlockchange", onChange);
    host.addEventListener("pointerlockerror", onError);
    cleanup = () => {
      host.removeEventListener("pointerlockchange", onChange);
      host.removeEventListener("pointerlockerror", onError);
    };
  });
  return { promise, cancel: () => cleanup() };
}

function isPointerLockable(element: Element): boolean {
  return typeof element.requestPointerLock === "function";
}

function describe(cause: unknown): string {
  return cause instanceof Error ? cause.message : "The pointer lock request failed.";
}

function browserPointerLockHost(): PointerLockHost | undefined {
  return typeof document !== "undefined" ? document : undefined;
}
