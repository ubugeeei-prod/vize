import { computed, shallowRef } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { useIntervalFn } from "./use-interval.ts";
import type { IntervalScheduler, PausableControls } from "./use-interval.ts";
import { useRafFn } from "./use-raf-fn.ts";
import type { FrameScheduler } from "./use-raf-fn.ts";

/** Options for {@link useTimestamp}. */
export interface UseTimestampOptions<Controls extends boolean = false> {
  /**
   * Update cadence: a period in milliseconds (reactive), or
   * `"requestAnimationFrame"` to update on every animation frame.
   *
   * @default 1000
   */
  readonly interval?: MaybeRefOrGetter<number> | "requestAnimationFrame";

  /**
   * Milliseconds added to every reading, for example a server clock skew.
   *
   * @default 0
   */
  readonly offset?: number;

  /**
   * Clock source returning Unix epoch milliseconds.
   *
   * @default Date.now
   */
  readonly now?: () => number;

  /**
   * Hydration-stable starting value. When set, the first reading is this
   * value on both server and client instead of the live clock, and the live
   * clock takes over on the first tick. Pass the server's render time here
   * (for example through a payload) to avoid hydration mismatches.
   *
   * @default undefined (read the clock immediately)
   */
  readonly initial?: number;

  /**
   * Start updating as soon as the composable is created.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Starts host timers or frames when no browser `window` is available.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Repeating timer host for millisecond cadences.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: IntervalScheduler;

  /**
   * Frame host for the `"requestAnimationFrame"` cadence.
   *
   * @default globalThis animation-frame functions when available
   */
  readonly frameScheduler?: FrameScheduler;

  /**
   * Invoked after every update with the new timestamp.
   *
   * @default undefined
   */
  readonly callback?: (timestamp: number) => void;

  /**
   * Return the full control object instead of the bare ref.
   *
   * @default false
   */
  readonly controls?: Controls;
}

/** Options for {@link useNow}; identical to {@link UseTimestampOptions}. */
export type UseNowOptions<Controls extends boolean = false> = UseTimestampOptions<Controls>;

/** Timestamp and controls returned by `useTimestamp({ controls: true })`. */
export interface TimestampControls extends PausableControls {
  /** Latest reading in Unix epoch milliseconds. */
  readonly timestamp: Readonly<ShallowRef<number>>;

  /**
   * Read the clock immediately, store the value, and return it.
   *
   * @returns The new timestamp.
   */
  readonly refresh: () => number;
}

/** Date and controls returned by `useNow({ controls: true })`. */
export interface NowControls extends PausableControls {
  /** Latest reading as a `Date`; a fresh instance per update. */
  readonly now: ComputedRef<Date>;

  /**
   * Read the clock immediately, store the value, and return it.
   *
   * @returns The new date.
   */
  readonly refresh: () => Date;
}

function createTimestamp(options: UseTimestampOptions<boolean>): TimestampControls {
  const offset = options.offset ?? 0;
  const read = options.now ?? Date.now;
  const timestamp = shallowRef(options.initial ?? read() + offset);

  const refresh = (): number => {
    timestamp.value = read() + offset;
    options.callback?.(timestamp.value);
    return timestamp.value;
  };

  const interval = options.interval ?? 1_000;
  const controls =
    interval === "requestAnimationFrame"
      ? useRafFn(refresh, {
          immediate: options.immediate ?? true,
          runOnServer: options.runOnServer ?? false,
          ...(options.frameScheduler === undefined ? {} : { scheduler: options.frameScheduler }),
        })
      : useIntervalFn(refresh, interval, {
          immediate: options.immediate ?? true,
          runOnServer: options.runOnServer ?? false,
          ...(options.scheduler === undefined ? {} : { scheduler: options.scheduler }),
        });

  return { ...controls, timestamp, refresh };
}

/**
 * Reactive Unix timestamp in milliseconds.
 *
 * Updates on a millisecond interval or every animation frame. For SSR,
 * supply {@link UseTimestampOptions.initial} (or an injected `now`) so the
 * server and the hydrating client render the same value; no timer runs on
 * the server. Timers stop with the owning reactive scope.
 *
 * @example
 * ```ts
 * const timestamp = useTimestamp({ interval: 250 });
 * const { timestamp: t, pause } = useTimestamp({ controls: true });
 * ```
 *
 * @param options Cadence, clock, SSR, and scheduler policy.
 * @default options {}
 * @throws `RangeError` for invalid millisecond cadences (see `useIntervalFn`).
 * @returns The timestamp ref, or the full controls when `controls` is `true`.
 */
export function useTimestamp(options?: UseTimestampOptions<false>): Readonly<ShallowRef<number>>;
export function useTimestamp(options: UseTimestampOptions<true>): TimestampControls;
export function useTimestamp(
  options: UseTimestampOptions<boolean> = {},
): Readonly<ShallowRef<number>> | TimestampControls {
  const controls = createTimestamp(options);
  return options.controls === true ? controls : controls.timestamp;
}

/**
 * Reactive current `Date`.
 *
 * Same cadence, SSR, and cleanup rules as {@link useTimestamp}; the value is
 * a fresh `Date` derived from the latest timestamp.
 *
 * @example
 * ```ts
 * const now = useNow({ initial: serverRenderedAt });
 * ```
 *
 * @param options Cadence, clock, SSR, and scheduler policy.
 * @default options {}
 * @throws `RangeError` for invalid millisecond cadences (see `useIntervalFn`).
 * @returns The date ref, or the full controls when `controls` is `true`.
 */
export function useNow(options?: UseNowOptions<false>): ComputedRef<Date>;
export function useNow(options: UseNowOptions<true>): NowControls;
export function useNow(options: UseNowOptions<boolean> = {}): ComputedRef<Date> | NowControls {
  const { timestamp, refresh, ...pausable } = createTimestamp(options);
  const now = computed(() => new Date(timestamp.value));
  if (options.controls !== true) return now;
  return { ...pausable, now, refresh: () => new Date(refresh()) };
}
