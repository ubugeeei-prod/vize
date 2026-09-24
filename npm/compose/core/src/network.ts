import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Effective connection classes reported by the Network Information API. */
export type NetworkEffectiveType = "slow-2g" | "2g" | "3g" | "4g";

/** Physical connection types reported by the Network Information API. */
export type NetworkConnectionType =
  | "bluetooth"
  | "cellular"
  | "ethernet"
  | "none"
  | "wifi"
  | "wimax"
  | "other"
  | "unknown";

/** Subset of the (non-standard) `NetworkInformation` interface. */
export interface NetworkInformationLike extends EventTarget {
  /** Estimated downlink bandwidth in Mbit/s. */
  readonly downlink?: number;
  /** Maximum downlink bandwidth in Mbit/s. */
  readonly downlinkMax?: number;
  /** Effective connection class. */
  readonly effectiveType?: string;
  /** Estimated round-trip time in milliseconds. */
  readonly rtt?: number;
  /** Whether the user requested reduced data usage. */
  readonly saveData?: boolean;
  /** Physical connection type. */
  readonly type?: string;
}

/** Window-like capability observed by the network composables. */
export interface NetworkHost extends EventTarget {
  /** Navigator exposing connectivity. */
  readonly navigator: {
    /** Whether the browser believes it is online. */
    readonly onLine: boolean;
    /** Network Information API object, where supported. */
    readonly connection?: NetworkInformationLike;
  };
}

/** Options shared by {@link useOnline} and {@link useNetwork}. */
export interface UseNetworkOptions {
  /**
   * Connectivity exposed during server rendering.
   *
   * @default true
   */
  readonly ssrOnline?: boolean;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<NetworkHost | null | undefined>;
}

/** Reactive network state returned by {@link useNetwork}. */
export interface NetworkControls {
  /** Whether the Network Information API is available. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
  /** Whether the browser reports being online. */
  readonly isOnline: Readonly<Ref<boolean>>;
  /** Epoch milliseconds of the latest `online` event. */
  readonly onlineAt: Readonly<Ref<number | null>>;
  /** Epoch milliseconds of the latest `offline` event. */
  readonly offlineAt: Readonly<Ref<number | null>>;
  /** Estimated downlink in Mbit/s. */
  readonly downlink: Readonly<Ref<number | null>>;
  /** Maximum downlink in Mbit/s. */
  readonly downlinkMax: Readonly<Ref<number | null>>;
  /** Effective connection class, or `null` when unknown. */
  readonly effectiveType: Readonly<Ref<NetworkEffectiveType | null>>;
  /** Estimated round-trip time in milliseconds. */
  readonly rtt: Readonly<Ref<number | null>>;
  /** Whether the user requested reduced data usage. */
  readonly saveData: Readonly<Ref<boolean>>;
  /** Physical connection type, or `null` when unknown. */
  readonly type: Readonly<Ref<NetworkConnectionType | null>>;
}

/**
 * Track connectivity and connection quality.
 *
 * Listens for `online`/`offline` on the window and `change` on
 * `navigator.connection` where available. Unknown or future enum values
 * normalize to `null` so the public unions stay closed. Server renders
 * report `ssrOnline` with every quality metric `null`; listeners are
 * removed with the owning reactive scope.
 *
 * @param options Server fallback and window capability.
 * @default options {}
 * @returns Reactive connectivity and connection metrics.
 */
export function useNetwork(options: UseNetworkOptions = {}): NetworkControls {
  const ssrOnline = options.ssrOnline ?? true;
  const isSupported = ref(false);
  const isOnline = ref(ssrOnline);
  const onlineAt = ref<number | null>(null);
  const offlineAt = ref<number | null>(null);
  const downlink = ref<number | null>(null);
  const downlinkMax = ref<number | null>(null);
  const effectiveType = ref<NetworkEffectiveType | null>(null);
  const rtt = ref<number | null>(null);
  const saveData = ref(false);
  const type = ref<NetworkConnectionType | null>(null);

  const stop = watch(
    () => (options.host === undefined ? browserNetworkHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      if (!host) {
        isSupported.value = false;
        isOnline.value = ssrOnline;
        return;
      }
      const connection = host.navigator.connection;
      isSupported.value = connection !== undefined;
      const readConnection = (): void => {
        isOnline.value = host.navigator.onLine;
        if (!connection) return;
        downlink.value = connection.downlink ?? null;
        downlinkMax.value = connection.downlinkMax ?? null;
        effectiveType.value = toEffectiveType(connection.effectiveType);
        rtt.value = connection.rtt ?? null;
        saveData.value = connection.saveData ?? false;
        type.value = toConnectionType(connection.type);
      };
      const onOnline = (): void => {
        onlineAt.value = Date.now();
        readConnection();
      };
      const onOffline = (): void => {
        offlineAt.value = Date.now();
        readConnection();
      };
      readConnection();
      host.addEventListener("online", onOnline, { passive: true });
      host.addEventListener("offline", onOffline, { passive: true });
      connection?.addEventListener("change", readConnection, { passive: true });
      onCleanup(() => {
        host.removeEventListener("online", onOnline);
        host.removeEventListener("offline", onOffline);
        connection?.removeEventListener("change", readConnection);
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return {
    isSupported: readonly(isSupported),
    isOnline: readonly(isOnline),
    onlineAt: readonly(onlineAt),
    offlineAt: readonly(offlineAt),
    downlink: readonly(downlink),
    downlinkMax: readonly(downlinkMax),
    effectiveType: readonly(effectiveType),
    rtt: readonly(rtt),
    saveData: readonly(saveData),
    type: readonly(type),
  };
}

/**
 * Track whether the browser reports being online.
 *
 * Shorthand for `useNetwork(options).isOnline` with the same SSR fallback
 * and cleanup semantics.
 *
 * @param options Server fallback and window capability.
 * @default options {}
 * @returns Readonly online flag.
 */
export function useOnline(options: UseNetworkOptions = {}): Readonly<Ref<boolean>> {
  return useNetwork(options).isOnline;
}

function toEffectiveType(value: string | undefined): NetworkEffectiveType | null {
  return value === "slow-2g" || value === "2g" || value === "3g" || value === "4g" ? value : null;
}

function toConnectionType(value: string | undefined): NetworkConnectionType | null {
  switch (value) {
    case "bluetooth":
    case "cellular":
    case "ethernet":
    case "none":
    case "wifi":
    case "wimax":
    case "other":
    case "unknown":
      return value;
    default:
      return null;
  }
}

function browserNetworkHost(): NetworkHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
