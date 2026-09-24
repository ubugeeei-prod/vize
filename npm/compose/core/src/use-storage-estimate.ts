import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { IntervalScheduler } from "./use-interval.ts";

/** Result of `navigator.storage.estimate()`. */
export interface StorageEstimateLike {
  /** Bytes used by the origin. */
  readonly usage?: number | undefined;

  /** Bytes available to the origin. */
  readonly quota?: number | undefined;

  /** Per-storage-system usage in bytes (Chromium only). */
  readonly usageDetails?: Readonly<Record<string, number>> | undefined;
}

/** Minimal `StorageManager` (`navigator.storage`) used by this module. */
export interface StorageManagerLike {
  /** Estimate usage and quota. */
  estimate?(): Promise<StorageEstimateLike>;

  /** Whether storage is already persistent. */
  persisted?(): Promise<boolean>;

  /** Request persistent storage. */
  persist?(): Promise<boolean>;
}

/** Options for {@link useStorageEstimate}. */
export interface UseStorageEstimateOptions {
  /**
   * Storage manager for alternate runtimes and tests.
   *
   * @default window.navigator.storage when it exists
   */
  readonly storage?: MaybeRefOrGetter<StorageManagerLike | null | undefined>;

  /**
   * Estimate as soon as the composable is created (browser only).
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Re-estimate every `interval` milliseconds; `0` disables polling.
   *
   * @default 0
   */
  readonly interval?: number;

  /**
   * Owns the polling timer.
   *
   * @default window timer functions (no timer without a browser window)
   */
  readonly scheduler?: IntervalScheduler;
}

/** Reactive state and actions returned by {@link useStorageEstimate}. */
export interface StorageEstimateControls {
  /** Whether `navigator.storage.estimate` is available. */
  readonly supported: ComputedRef<boolean>;

  /** Bytes used, or `null` before the first estimate. */
  readonly usage: Readonly<Ref<number | null>>;

  /** Bytes available, or `null` before the first estimate. */
  readonly quota: Readonly<Ref<number | null>>;

  /** Per-storage-system usage, or `null` when the browser does not report it. */
  readonly usageDetails: Readonly<ShallowRef<Readonly<Record<string, number>> | null>>;

  /** Usage as a percentage (0..100) of quota, or `null` when unknown. */
  readonly percentUsed: ComputedRef<number | null>;

  /** Most recent estimate failure, cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Re-read the estimate.
   *
   * @returns Whether a fresh estimate was stored.
   */
  readonly refresh: () => Promise<boolean>;
}

/** Options for {@link usePersistentStorage}. */
export interface UsePersistentStorageOptions {
  /**
   * Storage manager for alternate runtimes and tests.
   *
   * @default window.navigator.storage when it exists
   */
  readonly storage?: MaybeRefOrGetter<StorageManagerLike | null | undefined>;

  /**
   * Read `persisted()` as soon as the composable is created (browser only).
   *
   * @default true
   */
  readonly immediate?: boolean;
}

/** Reactive state and actions returned by {@link usePersistentStorage}. */
export interface PersistentStorageControls {
  /** Whether `navigator.storage.persist` is available. */
  readonly supported: ComputedRef<boolean>;

  /** Whether the origin's storage is persistent (`false` until known). */
  readonly persisted: Readonly<Ref<boolean>>;

  /** Most recent failure, cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Request persistent storage.
   *
   * @returns Whether storage is persistent afterwards; `false` when unsupported.
   */
  readonly persist: () => Promise<boolean>;

  /**
   * Re-read `persisted()`.
   *
   * @returns The current persistence state.
   */
  readonly check: () => Promise<boolean>;
}

function browserStorage(): StorageManagerLike | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  return "storage" in navigator && navigator.storage ? navigator.storage : undefined;
}

function browserScheduler(): IntervalScheduler | undefined {
  if (typeof window === "undefined") return undefined;
  return {
    setInterval: (callback, intervalMs) => window.setInterval(callback, intervalMs),
    clearInterval: (handle) => {
      if (typeof handle === "number") window.clearInterval(handle);
    },
  };
}

function resolver(
  source: MaybeRefOrGetter<StorageManagerLike | null | undefined> | undefined,
): () => StorageManagerLike | undefined {
  return () => (source === undefined ? browserStorage() : (toValue(source) ?? undefined));
}

/**
 * Track the origin's storage usage and quota with `navigator.storage.estimate()`.
 *
 * The estimate is read on creation (unless `immediate` is false), whenever a
 * reactive `storage` host changes, on `refresh()`, and every `interval`
 * milliseconds through the injectable `scheduler`. The polling timer stops
 * with the owning reactive scope; outside a scope, polling runs until the
 * page unloads, so leave `interval` at `0` there.
 *
 * Server rendering: nothing is read, no timer starts, `supported` is false
 * and every value is `null`.
 *
 * @example
 * ```ts
 * const { usage, quota, percentUsed } = useStorageEstimate({ interval: 60_000 });
 * ```
 *
 * @param options Storage host, polling interval and scheduler.
 * @default options {}
 * @returns Storage usage state and a refresh action.
 */
export function useStorageEstimate(
  options: UseStorageEstimateOptions = {},
): StorageEstimateControls {
  const { immediate = true, interval = 0 } = options;
  if (!Number.isFinite(interval) || interval < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_STORAGE_ESTIMATE_INVALID_INTERVAL] interval must be a non-negative finite number; received ${interval}.`,
    );
  }
  const resolveStorage = resolver(options.storage);
  const usage = ref<number | null>(null);
  const quota = ref<number | null>(null);
  const usageDetails = shallowRef<Readonly<Record<string, number>> | null>(null);
  const error = shallowRef<unknown>(undefined);
  let generation = 0;

  const refresh = async (): Promise<boolean> => {
    const storage = resolveStorage();
    if (!storage?.estimate) return false;
    const current = ++generation;
    try {
      const estimate = await storage.estimate();
      if (current !== generation) return false;
      usage.value = estimate.usage ?? null;
      quota.value = estimate.quota ?? null;
      usageDetails.value = estimate.usageDetails ?? null;
      error.value = undefined;
      return true;
    } catch (cause) {
      if (current === generation) error.value = cause;
      return false;
    }
  };

  watch(
    resolveStorage,
    (storage, _previous, onCleanup) => {
      // A previous host may still have an estimate in flight. Its result must
      // not populate the next host's state, even when the next host is absent.
      generation += 1;
      usage.value = null;
      quota.value = null;
      usageDetails.value = null;
      error.value = undefined;
      if (!storage?.estimate) return;
      if (immediate) void refresh();
      const scheduler = options.scheduler ?? browserScheduler();
      if (interval === 0 || !scheduler) return;
      const handle = scheduler.setInterval(() => void refresh(), interval);
      onCleanup(() => scheduler.clearInterval(handle));
    },
    { immediate: true },
  );

  tryOnScopeDispose(() => {
    generation += 1;
  });

  return {
    supported: computed(() => resolveStorage()?.estimate !== undefined),
    usage: readonly(usage),
    quota: readonly(quota),
    usageDetails: readonly(usageDetails),
    percentUsed: computed(() =>
      usage.value === null || quota.value === null || quota.value <= 0
        ? null
        : (usage.value / quota.value) * 100,
    ),
    error: readonly(error),
    refresh,
  };
}

/**
 * Read and request persistent storage with `navigator.storage.persist()`.
 *
 * Persistent storage is not evicted under storage pressure. `persist()` may
 * show a prompt or be decided silently by the browser; the result is stored
 * in `persisted`. Nothing needs cleanup; a pending read is ignored after the
 * owning reactive scope stops.
 *
 * Server rendering: nothing is read, `supported` and `persisted` are false.
 *
 * @example
 * ```ts
 * const storage = usePersistentStorage();
 * const keepOfflineData = () => storage.persist();
 * ```
 *
 * @param options Storage host and initial read policy.
 * @default options {}
 * @returns Persistence state and actions.
 */
export function usePersistentStorage(
  options: UsePersistentStorageOptions = {},
): PersistentStorageControls {
  const resolveStorage = resolver(options.storage);
  const persisted = ref(false);
  const error = shallowRef<unknown>(undefined);
  let active = true;

  const call = async (method: "persisted" | "persist"): Promise<boolean> => {
    const storage = resolveStorage();
    const read = storage?.[method];
    if (!read) return false;
    try {
      const value = await read.call(storage);
      if (active) {
        persisted.value = value;
        error.value = undefined;
      }
      return value;
    } catch (cause) {
      if (active) error.value = cause;
      return false;
    }
  };
  const check = (): Promise<boolean> => call("persisted");
  const persist = (): Promise<boolean> => call("persist");

  if (options.immediate ?? true) void check();

  tryOnScopeDispose(() => {
    active = false;
  });

  return {
    supported: computed(() => resolveStorage()?.persist !== undefined),
    persisted: readonly(persisted),
    error: readonly(error),
    persist,
    check,
  };
}
