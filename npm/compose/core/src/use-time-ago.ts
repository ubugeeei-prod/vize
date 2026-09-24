import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import type { IntervalScheduler, PausableControls } from "./use-interval.ts";
import { useTimestamp } from "./use-now.ts";

/** Calendar units {@link formatTimeAgo} can express a distance in. */
export type TimeAgoUnit = "second" | "minute" | "hour" | "day" | "week" | "month" | "year";

/** Date input accepted by the time formatting helpers. */
export type TimeAgoInput = Date | number | string;

/** Options for {@link formatTimeAgo}. */
export interface FormatTimeAgoOptions {
  /**
   * BCP 47 locale(s) for `Intl.RelativeTimeFormat`. Defaults to a fixed
   * locale (not the host default) so server and client output agree.
   *
   * @default "en"
   */
  readonly locale?: string | readonly string[];

  /**
   * `"auto"` allows phrases such as "yesterday" and "now"; `"always"` keeps
   * numeric output such as "1 day ago".
   *
   * @default "auto"
   */
  readonly numeric?: "always" | "auto";

  /**
   * Length of the unit wording.
   *
   * @default "long"
   */
  readonly style?: "long" | "short" | "narrow";

  /**
   * Units allowed in the output, in any order. The largest allowed unit
   * that fits the distance wins; smaller distances use the smallest unit.
   *
   * @default all units from second to year
   */
  readonly units?: readonly TimeAgoUnit[];

  /**
   * How a fractional unit count becomes an integer.
   *
   * @default "round"
   */
  readonly rounding?: "round" | "floor" | "ceil";

  /**
   * Distances below this many milliseconds are reported as zero of the
   * smallest unit ("now" with `numeric: "auto"`).
   *
   * @default 0
   */
  readonly justNowMs?: number;

  /**
   * Distances of at least this many milliseconds are delegated to
   * {@link FormatTimeAgoOptions.fullDateFormatter}.
   *
   * @default Number.POSITIVE_INFINITY
   */
  readonly maxMs?: number;

  /**
   * Formats dates beyond {@link FormatTimeAgoOptions.maxMs}.
   *
   * @default medium date style in UTC for the configured locale
   */
  readonly fullDateFormatter?: (date: Date) => string;
}

/** Options for {@link useTimeAgo}. */
export interface UseTimeAgoOptions<Controls extends boolean = false> extends Omit<
  FormatTimeAgoOptions,
  "locale"
> {
  /**
   * Reactive BCP 47 locale(s). Pair with `useLocale()` to follow the user.
   *
   * @default "en"
   */
  readonly locale?: MaybeRefOrGetter<string | readonly string[]>;

  /**
   * How often the relative phrase is recomputed, in milliseconds.
   *
   * @default 30000
   */
  readonly updateIntervalMs?: MaybeRefOrGetter<number>;

  /**
   * Clock source returning Unix epoch milliseconds.
   *
   * @default Date.now
   */
  readonly now?: () => number;

  /**
   * Hydration-stable "now" used for the first render on server and client.
   *
   * @default undefined (read the clock immediately)
   */
  readonly initialNow?: number;

  /**
   * Starts host timers when no browser `window` is available.
   *
   * @default false
   */
  readonly runOnServer?: boolean;

  /**
   * Repeating timer host.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: IntervalScheduler;

  /**
   * Return the full control object instead of the bare phrase ref.
   *
   * @default false
   */
  readonly controls?: Controls;
}

/** Phrase and controls returned by `useTimeAgo(time, { controls: true })`. */
export interface TimeAgoControls extends PausableControls {
  /** Localized relative phrase such as "3 minutes ago". */
  readonly timeAgo: ComputedRef<string>;

  /** The reference "now" in Unix epoch milliseconds. */
  readonly now: Readonly<ShallowRef<number>>;
}

const unitMs: Readonly<Record<TimeAgoUnit, number>> = {
  second: 1_000,
  minute: 60_000,
  hour: 3_600_000,
  day: 86_400_000,
  week: 604_800_000,
  month: 2_592_000_000,
  year: 31_536_000_000,
};

const allUnits: readonly TimeAgoUnit[] = [
  "second",
  "minute",
  "hour",
  "day",
  "week",
  "month",
  "year",
];

function toEpochMs(input: TimeAgoInput): number {
  const value = input instanceof Date ? input.getTime() : new Date(input).getTime();
  if (Number.isNaN(value)) {
    throw new RangeError(
      `[VIZE_COMPOSE_TIME_AGO_INVALID_DATE] expected a valid date; received ${String(input)}`,
    );
  }
  return value;
}

/**
 * Format the distance between two instants as a localized relative phrase.
 *
 * Pure and deterministic: the output depends only on the arguments (the
 * locale defaults to `"en"`, never the host default), which makes it safe
 * for server rendering and hydration.
 *
 * @example
 * ```ts
 * formatTimeAgo(Date.UTC(2026, 0, 1), Date.UTC(2026, 0, 2)); // "yesterday"
 * formatTimeAgo(0, 90_000, { numeric: "always" }); // "2 minutes ago"
 * ```
 *
 * @param time Instant being described.
 * @param now Reference instant.
 * @param options Locale, wording, unit, and threshold policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_TIME_AGO_INVALID_DATE` when
 * either input is not a valid date, and `RangeError` from `Intl` for invalid
 * locales.
 * @returns The localized phrase.
 */
export function formatTimeAgo(
  time: TimeAgoInput,
  now: TimeAgoInput,
  options: FormatTimeAgoOptions = {},
): string {
  const target = toEpochMs(time);
  const diff = target - toEpochMs(now);
  const distance = Math.abs(diff);
  const locales =
    typeof options.locale === "string" ? options.locale : [...(options.locale ?? ["en"])];

  if (distance >= (options.maxMs ?? Number.POSITIVE_INFINITY)) {
    const date = new Date(target);
    return options.fullDateFormatter === undefined
      ? new Intl.DateTimeFormat(locales, { dateStyle: "medium", timeZone: "UTC" }).format(date)
      : options.fullDateFormatter(date);
  }

  const allowed = allUnits.filter((unit) => (options.units ?? allUnits).includes(unit));
  const smallest = allowed[0] ?? "second";
  let unit = smallest;
  for (const candidate of allowed) if (distance >= unitMs[candidate]) unit = candidate;

  // Zero distance counts as past (`-0`), so "0 seconds ago" rather than "in 0 seconds".
  const round = Math[options.rounding ?? "round"];
  const magnitude = distance < (options.justNowMs ?? 0) ? 0 : round(distance / unitMs[unit]);
  const formatter = new Intl.RelativeTimeFormat(locales, {
    numeric: options.numeric ?? "auto",
    style: options.style ?? "long",
  });
  return formatter.format(diff <= 0 ? -magnitude : magnitude, magnitude === 0 ? smallest : unit);
}

/**
 * Reactive localized "time ago" phrase that refreshes on an interval.
 *
 * Combines {@link formatTimeAgo} with a pausable clock. For SSR, inject
 * `initialNow` (the server render time) and a fixed `locale`: both sides
 * then produce the same phrase and the client clock takes over on its first
 * tick. No timer runs on the server; the timer stops with the owning scope.
 *
 * @example
 * ```ts
 * const phrase = useTimeAgo(() => post.value.createdAt, { locale: useLocale().locale });
 * ```
 *
 * @param time Reactive instant being described.
 * @param options Formatting, cadence, clock, and SSR policy.
 * @default options {}
 * @throws `RangeError` on read for invalid dates or locales, and for invalid
 * update intervals at creation.
 * @returns The phrase ref, or the full controls when `controls` is `true`.
 */
export function useTimeAgo(
  time: MaybeRefOrGetter<TimeAgoInput>,
  options?: UseTimeAgoOptions<false>,
): ComputedRef<string>;
export function useTimeAgo(
  time: MaybeRefOrGetter<TimeAgoInput>,
  options: UseTimeAgoOptions<true>,
): TimeAgoControls;
export function useTimeAgo(
  time: MaybeRefOrGetter<TimeAgoInput>,
  options: UseTimeAgoOptions<boolean> = {},
): ComputedRef<string> | TimeAgoControls {
  const { timestamp, pause, resume, isActive } = useTimestamp({
    interval: options.updateIntervalMs ?? 30_000,
    controls: true,
    runOnServer: options.runOnServer ?? false,
    ...(options.now === undefined ? {} : { now: options.now }),
    ...(options.initialNow === undefined ? {} : { initial: options.initialNow }),
    ...(options.scheduler === undefined ? {} : { scheduler: options.scheduler }),
  });

  const timeAgo = computed(() =>
    formatTimeAgo(toValue(time), timestamp.value, {
      ...options,
      locale: toValue(options.locale) ?? "en",
    }),
  );

  if (options.controls !== true) return timeAgo;
  return { timeAgo, now: timestamp, pause, resume, isActive };
}
