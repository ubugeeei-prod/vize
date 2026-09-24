import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Permission outcome of the iOS-style motion/orientation permission prompt. */
export type MotionPermissionState = "granted" | "denied" | "prompt";

/** Event constructor that may expose the iOS `requestPermission` static. */
export interface MotionPermissionConstructor {
  /** Constructor prototype; present on every platform event constructor. */
  readonly prototype: object;
  /** Ask the user for sensor access (Safari on iOS/iPadOS 13+). */
  readonly requestPermission?: () => Promise<string>;
}

/** Window-like capability observed by {@link useDeviceOrientation}. */
export interface DeviceOrientationHost extends EventTarget {
  /** Constructor used to detect support and request permission. */
  readonly DeviceOrientationEvent?: MotionPermissionConstructor;
}

/** Options for {@link useDeviceOrientation}. */
export interface UseDeviceOrientationOptions {
  /**
   * Listen to `deviceorientationabsolute` (earth-frame) instead of
   * `deviceorientation`.
   *
   * @default false
   */
  readonly absolute?: boolean;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<DeviceOrientationHost | null | undefined>;
}

/** Reactive orientation state returned by {@link useDeviceOrientation}. */
export interface DeviceOrientationControls {
  /** Whether the host exposes `DeviceOrientationEvent`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Whether the latest reading is relative to the earth frame. */
  readonly isAbsolute: Readonly<Ref<boolean>>;
  /** Rotation around the Z axis in degrees `[0, 360)`. */
  readonly alpha: Readonly<Ref<number | null>>;
  /** Rotation around the X axis in degrees `[-180, 180)`. */
  readonly beta: Readonly<Ref<number | null>>;
  /** Rotation around the Y axis in degrees `[-90, 90)`. */
  readonly gamma: Readonly<Ref<number | null>>;
  /** Latest permission outcome; `"granted"` where no prompt exists. */
  readonly permission: Readonly<Ref<MotionPermissionState>>;
  /**
   * Request sensor permission where the platform requires it. Must be called
   * from a user gesture on iOS.
   *
   * @returns The resulting permission state.
   */
  readonly requestPermission: () => Promise<MotionPermissionState>;
}

/**
 * Track device orientation angles.
 *
 * Server renders expose `null` angles and `isSupported: false`. The listener
 * follows the reactive host and is removed with the owning reactive scope.
 * On platforms with a permission prompt, `permission` starts as `"prompt"`
 * and readings arrive only after `requestPermission` resolves `"granted"`.
 *
 * @param options Frame selection and window capability.
 * @default options {}
 * @returns Reactive angles, support/permission state, and a permission action.
 */
export function useDeviceOrientation(
  options: UseDeviceOrientationOptions = {},
): DeviceOrientationControls {
  const isSupported = ref(false);
  const isAbsolute = ref(false);
  const alpha = ref<number | null>(null);
  const beta = ref<number | null>(null);
  const gamma = ref<number | null>(null);
  const permission = ref<MotionPermissionState>("prompt");
  const host = (): DeviceOrientationHost | null | undefined =>
    options.host === undefined ? browserOrientationHost() : toValue(options.host);
  const eventName = options.absolute ? "deviceorientationabsolute" : "deviceorientation";

  const onOrientation = (event: Event): void => {
    if (!isOrientationEvent(event)) return;
    isAbsolute.value = event.absolute;
    alpha.value = event.alpha;
    beta.value = event.beta;
    gamma.value = event.gamma;
  };

  const stop = watch(
    host,
    (current, _previous, onCleanup) => {
      const constructor = current?.DeviceOrientationEvent;
      isSupported.value = constructor !== undefined;
      if (!current || !constructor) return;
      if (typeof constructor.requestPermission !== "function") permission.value = "granted";
      current.addEventListener(eventName, onOrientation, { passive: true });
      onCleanup(() => current.removeEventListener(eventName, onOrientation));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return {
    isSupported: readonly(isSupported),
    isAbsolute: readonly(isAbsolute),
    alpha: readonly(alpha),
    beta: readonly(beta),
    gamma: readonly(gamma),
    permission: readonly(permission),
    requestPermission: async () => {
      permission.value = await requestMotionPermission(host()?.DeviceOrientationEvent);
      return permission.value;
    },
  };
}

/**
 * Request motion/orientation sensor permission from an event constructor.
 *
 * Normalizes the platform result to {@link MotionPermissionState}. Missing
 * constructors resolve `"denied"`; constructors without a prompt resolve
 * `"granted"`; unknown results resolve `"prompt"`. Rejections (for example
 * when not called from a user gesture) resolve `"denied"` instead of throwing.
 *
 * @param constructor `DeviceOrientationEvent` or `DeviceMotionEvent`.
 * @returns The normalized permission state.
 */
export async function requestMotionPermission(
  constructor: MotionPermissionConstructor | undefined,
): Promise<MotionPermissionState> {
  if (!constructor) return "denied";
  if (typeof constructor.requestPermission !== "function") return "granted";
  try {
    const result = await constructor.requestPermission();
    return result === "granted" || result === "denied" ? result : "prompt";
  } catch {
    return "denied";
  }
}

function isOrientationEvent(event: Event): event is DeviceOrientationEvent {
  return "alpha" in event && "beta" in event && "gamma" in event;
}

function browserOrientationHost(): DeviceOrientationHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
