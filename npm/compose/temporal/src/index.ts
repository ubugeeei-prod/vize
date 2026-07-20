import { computed, readonly, shallowRef, toValue, watchEffect } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";
import { Intl as TemporalIntl, Temporal } from "temporal-polyfill-lite";

/**
 * Timer host used by {@link useTemporalNow}.
 *
 * Implement this interface to integrate a deterministic clock, a native
 * runtime timer, or an application-owned scheduler.
 */
export interface TemporalScheduler {
  /** Starts a repeating callback and returns its opaque cancellation handle. */
  readonly setInterval: (callback: () => void, intervalMs: number) => unknown;

  /** Cancels a handle previously returned by {@link TemporalScheduler.setInterval}. */
  readonly clearInterval: (handle: unknown) => void;
}

/** Options for {@link useTemporalNow}. */
export interface UseTemporalNowOptions {
  /**
   * Clock update interval in milliseconds.
   *
   * Reactive changes replace the active timer. Values must be finite and
   * greater than zero.
   *
   * @default 1000
   */
  readonly intervalMs?: MaybeRefOrGetter<number>;

  /**
   * Pauses periodic updates while preserving the current instant.
   * Manual calls to {@link TemporalClock.refresh} continue to work.
   *
   * @default false
   */
  readonly paused?: MaybeRefOrGetter<boolean>;

  /**
   * Starts the timer when no browser `window` is available.
   *
   * Keep this disabled during server rendering. Enable it for native,
   * desktop, worker, and test runtimes whose scheduler is lifecycle-bound.
   *
   * @default false
   */
  readonly runOnServer?: MaybeRefOrGetter<boolean>;

  /**
   * Produces the current instant.
   *
   * @default Temporal.Now.instant
   */
  readonly now?: () => Temporal.Instant;

  /**
   * Owns the repeating timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TemporalScheduler;
}

/** Reactive controls returned by {@link useTemporalNow}. */
export interface TemporalClock {
  /** The latest instant. This ref is readonly to consumers. */
  readonly instant: Readonly<Ref<Temporal.Instant>>;

  /** Reads the clock source immediately, stores the value, and returns it. */
  readonly refresh: () => Temporal.Instant;
}

/** Options for {@link useTemporalZonedDateTime}. */
export interface UseTemporalZonedDateTimeOptions extends UseTemporalNowOptions {
  /**
   * Time-zone identifier or zoned date-time accepted by Temporal.
   *
   * @default Temporal.Now.timeZoneId()
   */
  readonly timeZone?: MaybeRefOrGetter<Temporal.TimeZoneLike>;
}

const defaultScheduler: TemporalScheduler = {
  setInterval: (callback, intervalMs) => globalThis.setInterval(callback, intervalMs),
  clearInterval: (handle) => {
    globalThis.clearInterval(handle as ReturnType<typeof setInterval>);
  },
};

function resolveIntervalMs(value: number): number {
  if (!Number.isFinite(value) || value <= 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_TEMPORAL_INVALID_INTERVAL] intervalMs must be finite and greater than zero; received ${String(value)}`,
    );
  }

  return Math.max(1, Math.trunc(value));
}

function shouldSchedule(options: UseTemporalNowOptions): boolean {
  if (toValue(options.paused ?? false)) return false;
  return typeof window !== "undefined" || toValue(options.runOnServer ?? false);
}

/**
 * Creates a pauseable Temporal clock whose timer follows the current Vue
 * effect scope.
 *
 * The timer is replaced when reactive options change and is always cancelled
 * when the owning scope stops. During server rendering, no timer starts unless
 * {@link UseTemporalNowOptions.runOnServer} is explicitly enabled.
 */
export function useTemporalNow(options: UseTemporalNowOptions = {}): TemporalClock {
  const readNow = options.now ?? Temporal.Now.instant;
  const instant = shallowRef(readNow());

  // Fail synchronously for invalid initial input. Reactive updates are still
  // validated inside the watcher before a replacement timer is created.
  if (shouldSchedule(options)) resolveIntervalMs(toValue(options.intervalMs ?? 1_000));

  const refresh = (): Temporal.Instant => {
    const nextInstant = readNow();
    instant.value = nextInstant;
    return nextInstant;
  };

  watchEffect((onCleanup) => {
    if (!shouldSchedule(options)) return;

    const intervalMs = resolveIntervalMs(toValue(options.intervalMs ?? 1_000));
    const scheduler = options.scheduler ?? defaultScheduler;
    const handle = scheduler.setInterval(refresh, intervalMs);

    onCleanup(() => scheduler.clearInterval(handle));
  });

  return {
    instant: readonly(instant),
    refresh,
  };
}

/**
 * Creates a reactive zoned date-time derived from a scoped Temporal clock.
 *
 * Changes to {@link UseTemporalZonedDateTimeOptions.timeZone} are reflected
 * without replacing the underlying timer.
 */
export function useTemporalZonedDateTime(
  options: UseTemporalZonedDateTimeOptions = {},
): ComputedRef<Temporal.ZonedDateTime> {
  const clock = useTemporalNow(options);

  return computed(() => {
    const timeZone =
      options.timeZone === undefined ? Temporal.Now.timeZoneId() : toValue(options.timeZone);

    return clock.instant.value.toZonedDateTimeISO(timeZone);
  });
}

export { Temporal, TemporalIntl };
