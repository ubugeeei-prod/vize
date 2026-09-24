import { computed, readonly, ref, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** A single duration or an alternating vibrate/pause pattern in milliseconds. */
export type VibrationPattern = number | readonly number[];

/** Minimal Vibration API (`navigator`). */
export interface VibrationHost {
  /** Start (or with `0`/`[]`, cancel) a vibration. Returns whether it was accepted. */
  vibrate(pattern: VibrationPattern): boolean;
}

/** Options for {@link useVibrate}. */
export interface UseVibrateOptions {
  /**
   * Default pattern used by `vibrate()`.
   *
   * @default []
   */
  readonly pattern?: MaybeRefOrGetter<VibrationPattern>;

  /**
   * Repeat the pattern every `interval` milliseconds until `stop`; `0`
   * vibrates once.
   *
   * @default 0
   */
  readonly interval?: number;

  /**
   * Vibration capability for alternate runtimes and tests.
   *
   * @default window.navigator when it implements `vibrate`
   */
  readonly host?: MaybeRefOrGetter<VibrationHost | null | undefined>;

  /**
   * Timer host used for repetition.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

/** Reactive state and actions returned by {@link useVibrate}. */
export interface VibrateControls {
  /** Whether the Vibration API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Whether a (possibly repeating) vibration started by this composable is active. */
  readonly vibrating: Readonly<Ref<boolean>>;

  /**
   * Vibrate with `pattern` or the default pattern.
   *
   * @param pattern Pattern overriding the default.
   * @returns Whether the platform accepted the pattern.
   * @throws {RangeError} `[VIZE_COMPOSE_VIBRATE_INVALID_PATTERN]` for negative,
   * fractional, or non-finite durations.
   */
  readonly vibrate: (pattern?: VibrationPattern) => boolean;

  /** Cancel the vibration and any repetition. */
  readonly stop: () => void;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

function browserVibrationHost(): VibrationHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  return typeof navigator.vibrate === "function" ? navigator : undefined;
}

function validatePattern(pattern: VibrationPattern): readonly number[] {
  const durations = typeof pattern === "number" ? [pattern] : pattern;
  for (const duration of durations) {
    if (!Number.isSafeInteger(duration) || duration < 0) {
      throw new RangeError(
        `[VIZE_COMPOSE_VIBRATE_INVALID_PATTERN] vibration durations must be non-negative integers; received ${String(duration)}`,
      );
    }
  }
  return durations;
}

/**
 * Drive the Vibration API with validated, optionally repeating patterns.
 *
 * Patterns alternate vibration and pause durations. With `interval > 0`
 * the pattern repeats until `stop` is called or the owning reactive scope
 * stops, which also cancels an ongoing vibration.
 *
 * Server rendering: nothing vibrates and no timer starts; `supported` is false.
 *
 * @example
 * ```ts
 * const { vibrate, stop } = useVibrate({ pattern: [200, 100, 200] });
 * vibrate();
 * ```
 *
 * @param options Default pattern, repetition, and capability.
 * @default options {}
 * @returns Vibration state and actions.
 * @throws {RangeError} `[VIZE_COMPOSE_VIBRATE_INVALID_INTERVAL]` for a negative
 * or non-finite interval.
 */
export function useVibrate(options: UseVibrateOptions = {}): VibrateControls {
  const interval = options.interval ?? 0;
  if (!Number.isFinite(interval) || interval < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_VIBRATE_INVALID_INTERVAL] interval must be a non-negative finite number; received ${String(interval)}`,
    );
  }
  const scheduler = options.scheduler ?? defaultScheduler;
  const vibrating = ref(false);
  let handle: unknown;
  let scheduled = false;

  const resolveHost = (): VibrationHost | undefined =>
    options.host === undefined ? browserVibrationHost() : (toValue(options.host) ?? undefined);

  const clearRepeat = (): void => {
    if (scheduled) scheduler.clearTimeout(handle);
    scheduled = false;
  };

  const stop = (): void => {
    clearRepeat();
    if (vibrating.value) resolveHost()?.vibrate(0);
    vibrating.value = false;
  };

  const vibrate = (pattern?: VibrationPattern): boolean => {
    const durations = validatePattern(pattern ?? toValue(options.pattern) ?? []);
    const host = resolveHost();
    clearRepeat();
    if (!host) return false;
    const accepted = host.vibrate(durations);
    const total = durations.reduce((sum, duration) => sum + duration, 0);
    vibrating.value = accepted && total > 0;
    if (vibrating.value && interval > 0) {
      const repeat = (): void => {
        scheduled = false;
        if (!vibrating.value) return;
        vibrating.value = host.vibrate(durations);
        if (vibrating.value) {
          scheduled = true;
          handle = scheduler.setTimeout(repeat, interval);
        }
      };
      scheduled = true;
      handle = scheduler.setTimeout(repeat, interval);
    }
    return accepted;
  };

  tryOnScopeDispose(stop);

  return {
    supported: computed(() => resolveHost() !== undefined),
    vibrating: readonly(vibrating),
    vibrate,
    stop,
  };
}
