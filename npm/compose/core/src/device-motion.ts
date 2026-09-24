import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { requestMotionPermission } from "./device-orientation.ts";
import type { MotionPermissionConstructor, MotionPermissionState } from "./device-orientation.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Window-like capability observed by {@link useDeviceMotion}. */
export interface DeviceMotionHost extends EventTarget {
  /** Constructor used to detect support and request permission. */
  readonly DeviceMotionEvent?: MotionPermissionConstructor;
}

/** Acceleration along the device axes in m/s². */
export interface MotionVector {
  /** X axis (left to right). */
  readonly x: number | null;
  /** Y axis (bottom to top). */
  readonly y: number | null;
  /** Z axis (back to front). */
  readonly z: number | null;
}

/** Rotation rate around the device axes in degrees per second. */
export interface MotionRotationRate {
  /** Around the Z axis. */
  readonly alpha: number | null;
  /** Around the X axis. */
  readonly beta: number | null;
  /** Around the Y axis. */
  readonly gamma: number | null;
}

/** Options for {@link useDeviceMotion}. */
export interface UseDeviceMotionOptions {
  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<DeviceMotionHost | null | undefined>;
}

/** Reactive motion state returned by {@link useDeviceMotion}. */
export interface DeviceMotionControls {
  /** Whether the host exposes `DeviceMotionEvent`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Acceleration without gravity; `null` before the first reading. */
  readonly acceleration: Readonly<ShallowRef<MotionVector | null>>;
  /** Acceleration including gravity; `null` before the first reading. */
  readonly accelerationIncludingGravity: Readonly<ShallowRef<MotionVector | null>>;
  /** Rotation rate; `null` before the first reading. */
  readonly rotationRate: Readonly<ShallowRef<MotionRotationRate | null>>;
  /** Sampling interval in milliseconds. */
  readonly interval: Readonly<Ref<number>>;
  /** Latest permission outcome; `"granted"` where no prompt exists. */
  readonly permission: Readonly<Ref<MotionPermissionState>>;
  /**
   * Request sensor permission where the platform requires it.
   *
   * @returns The resulting permission state.
   */
  readonly requestPermission: () => Promise<MotionPermissionState>;
}

/**
 * Track device acceleration and rotation rate.
 *
 * Readings are copied into plain snapshots. Server renders expose `null`
 * readings and `isSupported: false`; the listener is removed with the owning
 * reactive scope. Shares the permission model of `useDeviceOrientation`.
 *
 * @param options Window capability.
 * @default options {}
 * @returns Reactive readings, support/permission state, and a permission action.
 */
export function useDeviceMotion(options: UseDeviceMotionOptions = {}): DeviceMotionControls {
  const isSupported = ref(false);
  const acceleration = shallowRef<MotionVector | null>(null);
  const accelerationIncludingGravity = shallowRef<MotionVector | null>(null);
  const rotationRate = shallowRef<MotionRotationRate | null>(null);
  const interval = ref(0);
  const permission = ref<MotionPermissionState>("prompt");
  const host = (): DeviceMotionHost | null | undefined =>
    options.host === undefined ? browserMotionHost() : toValue(options.host);

  const onMotion = (event: Event): void => {
    if (!isMotionEvent(event)) return;
    acceleration.value = vector(event.acceleration);
    accelerationIncludingGravity.value = vector(event.accelerationIncludingGravity);
    rotationRate.value = event.rotationRate
      ? {
          alpha: event.rotationRate.alpha,
          beta: event.rotationRate.beta,
          gamma: event.rotationRate.gamma,
        }
      : null;
    interval.value = event.interval;
  };

  const stop = watch(
    host,
    (current, _previous, onCleanup) => {
      const constructor = current?.DeviceMotionEvent;
      isSupported.value = constructor !== undefined;
      if (!current || !constructor) return;
      if (typeof constructor.requestPermission !== "function") permission.value = "granted";
      current.addEventListener("devicemotion", onMotion, { passive: true });
      onCleanup(() => current.removeEventListener("devicemotion", onMotion));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return {
    isSupported: readonly(isSupported),
    acceleration,
    accelerationIncludingGravity,
    rotationRate,
    interval: readonly(interval),
    permission: readonly(permission),
    requestPermission: async () => {
      permission.value = await requestMotionPermission(host()?.DeviceMotionEvent);
      return permission.value;
    },
  };
}

function vector(value: DeviceMotionEventAcceleration | null): MotionVector | null {
  return value ? { x: value.x, y: value.y, z: value.z } : null;
}

function isMotionEvent(event: Event): event is DeviceMotionEvent {
  return "accelerationIncludingGravity" in event && "interval" in event;
}

function browserMotionHost(): DeviceMotionHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
