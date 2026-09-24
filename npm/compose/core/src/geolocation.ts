import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { availableCapability, unavailableCapability } from "./capability.ts";
import type { CapabilityResult } from "./capability.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Navigator-like capability used by {@link useGeolocation}. */
export interface GeolocationHost {
  /** Geolocation service, absent in unsupported runtimes. */
  readonly geolocation?: Pick<Geolocation, "watchPosition" | "clearWatch">;
}

/** Serializable snapshot of `GeolocationCoordinates`. */
export interface GeolocationCoordinatesSnapshot {
  /** Latitude in decimal degrees. */
  readonly latitude: number;
  /** Longitude in decimal degrees. */
  readonly longitude: number;
  /** Accuracy radius of latitude/longitude in meters. */
  readonly accuracy: number;
  /** Altitude in meters, when available. */
  readonly altitude: number | null;
  /** Altitude accuracy in meters, when available. */
  readonly altitudeAccuracy: number | null;
  /** Heading in degrees clockwise from true north, when available. */
  readonly heading: number | null;
  /** Ground speed in meters per second, when available. */
  readonly speed: number | null;
}

/** Why geolocation cannot currently provide a position. */
export type GeolocationUnavailableReason =
  | "unsupported"
  | "permission-denied"
  | "unavailable"
  | "not-ready";

/** Options for {@link useGeolocation}. */
export interface UseGeolocationOptions {
  /**
   * Ask for the most accurate position the device can provide.
   *
   * @default false
   */
  readonly enableHighAccuracy?: boolean;

  /**
   * Maximum age in milliseconds of a cached position.
   *
   * @default 30000
   */
  readonly maximumAge?: number;

  /**
   * Maximum time in milliseconds to wait for a position.
   *
   * @default 27000
   */
  readonly timeout?: number;

  /**
   * Start watching during composable creation. When `false`, call `resume`
   * (typically from a user gesture, which some browsers require).
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Reactive navigator capability for alternate runtimes and tests.
   *
   * @default globalThis.window.navigator when available
   */
  readonly host?: MaybeRefOrGetter<GeolocationHost | null | undefined>;
}

/** Reactive position state returned by {@link useGeolocation}. */
export interface GeolocationControls {
  /** Whether the host exposes geolocation. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Latest coordinates; `null` until the first fix and during server rendering. */
  readonly coords: Readonly<ShallowRef<GeolocationCoordinatesSnapshot | null>>;
  /** Epoch milliseconds of the latest fix; `null` before the first fix. */
  readonly locatedAt: Readonly<Ref<number | null>>;
  /** Latest platform error; cleared by the next successful fix. */
  readonly error: Readonly<ShallowRef<GeolocationPositionError | null>>;
  /** Capability result combining support, permission, and readiness. */
  readonly capability: ComputedRef<
    CapabilityResult<GeolocationCoordinatesSnapshot, GeolocationUnavailableReason>
  >;
  /** Whether a position watch is active. */
  readonly isActive: Readonly<Ref<boolean>>;
  /** Start (or restart) watching. */
  readonly resume: () => void;
  /** Stop watching. Idempotent. */
  readonly pause: () => void;
}

const PERMISSION_DENIED = 1;
const POSITION_UNAVAILABLE = 2;

/**
 * Watch the device position with the Geolocation API.
 *
 * The watch follows the reactive host and is cleared when paused or when the
 * owning reactive scope stops. Coordinates are copied into a plain snapshot
 * so they are serializable. The `capability` result reports
 * `"unsupported"`, `"permission-denied"`, `"unavailable"`, or `"not-ready"`
 * (before the first fix) as data instead of throwing. Server renders expose
 * `null` coordinates.
 *
 * @param options Accuracy, caching, start mode, and navigator capability.
 * @default options {}
 * @returns Reactive position, error, capability, and controls.
 */
export function useGeolocation(options: UseGeolocationOptions = {}): GeolocationControls {
  const isSupported = ref(false);
  const coords = shallowRef<GeolocationCoordinatesSnapshot | null>(null);
  const locatedAt = ref<number | null>(null);
  const error = shallowRef<GeolocationPositionError | null>(null);
  const isActive = ref(options.immediate ?? true);
  const host = (): GeolocationHost | null | undefined =>
    options.host === undefined ? browserGeolocationHost() : toValue(options.host);

  const stopWatch = watch(
    [host, isActive],
    ([current, active], _previous, onCleanup) => {
      const geolocation = current?.geolocation;
      isSupported.value = geolocation !== undefined;
      if (!geolocation || !active) return;
      const id = geolocation.watchPosition(
        (position) => {
          coords.value = snapshotCoordinates(position.coords);
          locatedAt.value = position.timestamp;
          error.value = null;
        },
        (failure) => {
          error.value = failure;
        },
        {
          enableHighAccuracy: options.enableHighAccuracy ?? false,
          maximumAge: options.maximumAge ?? 30_000,
          timeout: options.timeout ?? 27_000,
        },
      );
      onCleanup(() => geolocation.clearWatch(id));
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stopWatch.stop());

  const capability = computed<
    CapabilityResult<GeolocationCoordinatesSnapshot, GeolocationUnavailableReason>
  >(() => {
    if (!isSupported.value) return unavailableCapability("unsupported");
    if (error.value?.code === PERMISSION_DENIED) return unavailableCapability("permission-denied");
    if (error.value?.code === POSITION_UNAVAILABLE) return unavailableCapability("unavailable");
    return coords.value ? availableCapability(coords.value) : unavailableCapability("not-ready");
  });

  return {
    isSupported: readonly(isSupported),
    coords,
    locatedAt: readonly(locatedAt),
    error,
    capability,
    isActive: readonly(isActive),
    resume: () => {
      isActive.value = true;
    },
    pause: () => {
      isActive.value = false;
    },
  };
}

function snapshotCoordinates(coords: GeolocationCoordinates): GeolocationCoordinatesSnapshot {
  return {
    latitude: coords.latitude,
    longitude: coords.longitude,
    accuracy: coords.accuracy,
    altitude: coords.altitude,
    altitudeAccuracy: coords.altitudeAccuracy,
    heading: coords.heading,
    speed: coords.speed,
  };
}

function browserGeolocationHost(): GeolocationHost | undefined {
  return typeof window !== "undefined" ? window.navigator : undefined;
}
