import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Subset of the Battery Status API `BatteryManager`. */
export interface BatteryManagerLike extends EventTarget {
  /** Whether the battery is charging. */
  readonly charging: boolean;
  /** Seconds until fully charged (`Infinity` when discharging). */
  readonly chargingTime: number;
  /** Seconds until empty (`Infinity` when charging). */
  readonly dischargingTime: number;
  /** Charge level in `[0, 1]`. */
  readonly level: number;
}

/** Navigator-like capability used by {@link useBattery}. */
export interface BatteryHost {
  /** Resolve the battery manager, where supported. */
  readonly getBattery?: () => Promise<BatteryManagerLike>;
}

/** Options for {@link useBattery}. */
export interface UseBatteryOptions {
  /**
   * Reactive navigator capability for alternate runtimes and tests.
   *
   * @default globalThis.window.navigator when available
   */
  readonly host?: MaybeRefOrGetter<BatteryHost | null | undefined>;
}

/** Reactive battery state returned by {@link useBattery}. */
export interface BatteryControls {
  /** Whether the host exposes `getBattery`. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Whether the battery manager has resolved. */
  readonly isReady: Readonly<Ref<boolean>>;
  /** Whether the battery is charging. */
  readonly charging: Readonly<Ref<boolean>>;
  /** Seconds until fully charged. */
  readonly chargingTime: Readonly<Ref<number>>;
  /** Seconds until empty. */
  readonly dischargingTime: Readonly<Ref<number>>;
  /** Charge level in `[0, 1]`. */
  readonly level: Readonly<Ref<number>>;
  /** Rejection from `getBattery`, if any (for example a permissions policy). */
  readonly error: Readonly<ShallowRef<unknown>>;
}

const batteryEvents = [
  "chargingchange",
  "chargingtimechange",
  "dischargingtimechange",
  "levelchange",
] as const;

/**
 * Track the device battery with the Battery Status API.
 *
 * Server renders (and unsupported browsers) expose a full, charging battery
 * (`level: 1`, `charging: true`, times `0`/`Infinity`), which is the
 * least alarming deterministic default. Late resolutions after the host
 * changed or the scope stopped are ignored, and listeners are removed with
 * the owning reactive scope.
 *
 * @param options Navigator capability.
 * @default options {}
 * @returns Reactive battery state.
 */
export function useBattery(options: UseBatteryOptions = {}): BatteryControls {
  const isSupported = ref(false);
  const isReady = ref(false);
  const charging = ref(true);
  const chargingTime = ref(0);
  const dischargingTime = ref(Number.POSITIVE_INFINITY);
  const level = ref(1);
  const error = shallowRef<unknown>(undefined);

  const stop = watch(
    () => (options.host === undefined ? browserBatteryHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      const getBattery = host?.getBattery;
      isSupported.value = typeof getBattery === "function";
      isReady.value = false;
      if (!host || typeof getBattery !== "function") return;
      let cancelled = false;
      let release = (): void => undefined;
      onCleanup(() => {
        cancelled = true;
        release();
      });
      getBattery.call(host).then(
        (battery) => {
          if (cancelled) return;
          const update = (): void => {
            charging.value = battery.charging;
            chargingTime.value = battery.chargingTime;
            dischargingTime.value = battery.dischargingTime;
            level.value = battery.level;
          };
          update();
          isReady.value = true;
          for (const type of batteryEvents) battery.addEventListener(type, update);
          release = () => {
            for (const type of batteryEvents) battery.removeEventListener(type, update);
          };
        },
        (cause: unknown) => {
          if (!cancelled) error.value = cause;
        },
      );
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return {
    isSupported: readonly(isSupported),
    isReady: readonly(isReady),
    charging: readonly(charging),
    chargingTime: readonly(chargingTime),
    dischargingTime: readonly(dischargingTime),
    level: readonly(level),
    error,
  };
}

function browserBatteryHost(): BatteryHost | undefined {
  if (typeof window === "undefined") return undefined;
  const navigator: object = window.navigator;
  if (!("getBattery" in navigator) || typeof navigator.getBattery !== "function") return {};
  const request = navigator.getBattery;
  return {
    getBattery: async () => {
      const battery: unknown = await Reflect.apply(request, navigator, []);
      if (isBatteryManager(battery)) return battery;
      throw new TypeError("navigator.getBattery() resolved to an unexpected value.");
    },
  };
}

function isBatteryManager(value: unknown): value is BatteryManagerLike {
  return (
    typeof value === "object" &&
    value !== null &&
    "level" in value &&
    "charging" in value &&
    "addEventListener" in value
  );
}
