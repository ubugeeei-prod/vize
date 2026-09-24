import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Lock targets accepted by `ScreenOrientation.lock()`. */
export type ScreenOrientationLockType =
  | "any"
  | "natural"
  | "landscape"
  | "portrait"
  | "portrait-primary"
  | "portrait-secondary"
  | "landscape-primary"
  | "landscape-secondary";

/** Subset of `ScreenOrientation`, including the lock methods absent from some DOM typings. */
export interface ScreenOrientationLike extends EventTarget {
  /** Current orientation type. */
  readonly type: OrientationType;
  /** Current orientation angle in degrees. */
  readonly angle: number;
  /** Lock the orientation (fullscreen or installed apps only in most browsers). */
  readonly lock?: (orientation: ScreenOrientationLockType) => Promise<void>;
  /** Release an orientation lock. */
  readonly unlock?: () => void;
}

/** Window-like capability observed by {@link useScreenOrientation}. */
export interface ScreenOrientationHost {
  /** Screen exposing the orientation object. */
  readonly screen: { readonly orientation?: ScreenOrientationLike };
}

/** Options for {@link useScreenOrientation}. */
export interface UseScreenOrientationOptions {
  /**
   * Orientation exposed during server rendering.
   *
   * @default "portrait-primary"
   */
  readonly ssrOrientation?: OrientationType;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<ScreenOrientationHost | null | undefined>;
}

/** Reactive orientation state returned by {@link useScreenOrientation}. */
export interface ScreenOrientationControls {
  /** Whether the host exposes `screen.orientation`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Current orientation type. */
  readonly orientation: Readonly<Ref<OrientationType>>;
  /** Current angle in degrees. */
  readonly angle: Readonly<Ref<number>>;
  /**
   * Lock the orientation.
   *
   * @returns Whether a lock was requested and accepted.
   */
  readonly lockOrientation: (type: ScreenOrientationLockType) => Promise<boolean>;
  /**
   * Release a lock.
   *
   * @returns Whether an unlock method was available.
   */
  readonly unlockOrientation: () => boolean;
}

/**
 * Track the screen orientation.
 *
 * Server renders expose `ssrOrientation` with angle `0`. The `change`
 * listener follows the reactive host and is removed with the owning scope.
 * Lock failures (not fullscreen, unsupported) resolve `false` instead of
 * throwing.
 *
 * @param options Server fallback and window capability.
 * @default options {}
 * @returns Reactive orientation plus lock actions.
 */
export function useScreenOrientation(
  options: UseScreenOrientationOptions = {},
): ScreenOrientationControls {
  const fallback = options.ssrOrientation ?? "portrait-primary";
  const isSupported = ref(false);
  const orientation = ref<OrientationType>(fallback);
  const angle = ref(0);
  const target = (): ScreenOrientationLike | undefined => {
    const host = options.host === undefined ? browserScreenHost() : toValue(options.host);
    return host?.screen.orientation;
  };

  const stop = watch(
    target,
    (current, _previous, onCleanup) => {
      isSupported.value = current !== undefined;
      if (!current) {
        orientation.value = fallback;
        angle.value = 0;
        return;
      }
      const update = (): void => {
        orientation.value = current.type;
        angle.value = current.angle;
      };
      update();
      current.addEventListener("change", update, { passive: true });
      onCleanup(() => current.removeEventListener("change", update));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return {
    isSupported: readonly(isSupported),
    orientation: readonly(orientation),
    angle: readonly(angle),
    lockOrientation: async (type) => {
      const current = target();
      if (typeof current?.lock !== "function") return false;
      try {
        await current.lock(type);
        return true;
      } catch {
        return false;
      }
    },
    unlockOrientation: () => {
      const current = target();
      if (typeof current?.unlock !== "function") return false;
      current.unlock();
      return true;
    },
  };
}

function browserScreenHost(): ScreenOrientationHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
