/** Family of a scale, exposed for discriminated handling in axes and grids. */
export type ScaleKind = "band" | "linear" | "log" | "ordinal" | "point" | "pow" | "time";

/** Options for localized number tick labels built on `Intl.NumberFormat`. */
export interface NumberTickFormatOptions {
  /**
   * BCP 47 locale. Pass one explicitly for server-rendered charts so server and
   * client produce identical labels.
   *
   * @default "en-US"
   */
  readonly locale?: string;

  /**
   * Extra `Intl.NumberFormat` options merged over the tick-derived precision,
   * for example `{ style: "percent" }` or `{ notation: "compact" }`.
   *
   * @default undefined
   */
  readonly format?: Intl.NumberFormatOptions;
}

/** Options shared by every continuous numeric scale. */
export interface ContinuousScaleOptions {
  /**
   * Input extent. More than two values create a piecewise scale.
   *
   * @default [0, 1]
   */
  readonly domain?: readonly number[];

  /**
   * Output extent, usually pixels.
   *
   * @default [0, 1]
   */
  readonly range?: readonly number[];

  /**
   * Clamp outputs (and inverted inputs) to the range (and domain).
   *
   * @default false
   */
  readonly clamp?: boolean;

  /**
   * Round outputs to integers, like d3 `rangeRound`.
   *
   * @default false
   */
  readonly round?: boolean;

  /**
   * Extend the domain to nice tick values. A number sets the tick count used.
   *
   * @default false
   */
  readonly nice?: boolean | number;

  /**
   * Output for `null`, `undefined`, and `NaN` inputs.
   *
   * @default NaN
   */
  readonly unknown?: number;
}

/** Options for {@link scalePow}. */
export interface PowScaleOptions extends ContinuousScaleOptions {
  /**
   * Exponent applied to inputs. `0.5` is a square-root scale.
   *
   * @default 1
   */
  readonly exponent?: number;
}

/** Options for {@link scaleLog}. */
export interface LogScaleOptions extends ContinuousScaleOptions {
  /**
   * Logarithm base.
   *
   * @default 10
   */
  readonly base?: number;
}

/** Shared behavior of every continuous scale over inputs of type `Input`. */
export interface ContinuousScaleBase<Input, Options> {
  /** Map a domain value to the range. */
  (value: Input | null | undefined): number;

  /** Scale family. */
  readonly kind: ScaleKind;

  /** Normalized domain. */
  readonly domain: readonly Input[];

  /** Output range. */
  readonly range: readonly number[];

  /** Whether outputs are clamped. */
  readonly clamp: boolean;

  /** Whether outputs are rounded. */
  readonly round: boolean;

  /** Map a range value back to the domain. */
  readonly invert: (value: number) => Input;

  /** Representative tick values inside the domain. */
  readonly ticks: (count?: number) => Input[];

  /** Return a copy with a nice domain. */
  readonly nice: (count?: number) => ContinuousScaleBase<Input, Options>;

  /** Return a copy with options replaced. */
  readonly with: (options: Options) => ContinuousScaleBase<Input, Options>;

  /** Return the options that rebuild this scale. */
  readonly options: () => Options;
}

/** Linear scale: `y = a·x + b` per domain segment. */
export interface LinearScale extends ContinuousScaleBase<number, ContinuousScaleOptions> {
  readonly kind: "linear";
  readonly nice: (count?: number) => LinearScale;
  readonly with: (options: ContinuousScaleOptions) => LinearScale;

  /** Localized tick label formatter with precision derived from the tick step. */
  readonly tickFormat: (
    count?: number,
    options?: NumberTickFormatOptions,
  ) => (value: number) => string;
}

/** Power scale: inputs are raised to `exponent` before linear mapping. */
export interface PowScale extends ContinuousScaleBase<number, PowScaleOptions> {
  readonly kind: "pow";
  readonly exponent: number;
  readonly nice: (count?: number) => PowScale;
  readonly with: (options: PowScaleOptions) => PowScale;

  /** Localized tick label formatter with precision derived from the tick step. */
  readonly tickFormat: (
    count?: number,
    options?: NumberTickFormatOptions,
  ) => (value: number) => string;
}

/** Logarithmic scale. Domains must be strictly positive or strictly negative. */
export interface LogScale extends ContinuousScaleBase<number, LogScaleOptions> {
  readonly kind: "log";
  readonly base: number;
  readonly nice: () => LogScale;
  readonly with: (options: LogScaleOptions) => LogScale;

  /**
   * Localized formatter that returns `""` for ticks d3 would leave unlabeled,
   * so dense logarithmic axes stay legible.
   */
  readonly tickFormat: (
    count?: number,
    options?: NumberTickFormatOptions,
  ) => (value: number) => string;
}

/** Calendar granularity chosen for a time tick label. */
export type TimeTickGranularity =
  | "day"
  | "hour"
  | "millisecond"
  | "minute"
  | "month"
  | "second"
  | "week"
  | "year";

/** Options for {@link scaleTime}. */
export interface TimeScaleOptions {
  /**
   * Input extent as `Date` values or epoch milliseconds.
   *
   * @default [2000-01-01T00:00Z, 2000-01-02T00:00Z]
   */
  readonly domain?: readonly (Date | number)[];

  /**
   * Output extent.
   *
   * @default [0, 1]
   */
  readonly range?: readonly number[];

  /**
   * IANA time zone used for tick alignment and labels. Fixed by default so
   * server and client render identical ticks regardless of host time zone.
   *
   * @default "UTC"
   */
  readonly timeZone?: string;

  /**
   * Clamp outputs to the range.
   *
   * @default false
   */
  readonly clamp?: boolean;

  /**
   * Round outputs to integers.
   *
   * @default false
   */
  readonly round?: boolean;

  /**
   * Extend the domain to calendar boundaries. A number sets the tick count used.
   *
   * @default false
   */
  readonly nice?: boolean | number;

  /**
   * Output for invalid dates and missing inputs.
   *
   * @default NaN
   */
  readonly unknown?: number;
}

/** Options for localized time tick labels built on `Intl.DateTimeFormat`. */
export interface TimeTickFormatOptions {
  /**
   * BCP 47 locale.
   *
   * @default "en-US"
   */
  readonly locale?: string;

  /**
   * Replace the `Intl.DateTimeFormat` options used for a granularity.
   *
   * @default undefined
   */
  readonly formats?: Partial<Record<TimeTickGranularity, Intl.DateTimeFormatOptions>>;
}

/** Calendar-aware time scale. */
export interface TimeScale extends ContinuousScaleBase<Date, TimeScaleOptions> {
  readonly kind: "time";
  readonly timeZone: string;
  readonly nice: (count?: number) => TimeScale;
  readonly with: (options: TimeScaleOptions) => TimeScale;

  /** Map epoch milliseconds or a `Date`. */
  (value: Date | number | null | undefined): number;

  /** Calendar granularity that best describes `date`, as d3's multi-scale format picks it. */
  readonly tickGranularity: (date: Date) => TimeTickGranularity;

  /** Localized, granularity-aware tick label formatter in the scale's time zone. */
  readonly tickFormat: (options?: TimeTickFormatOptions) => (date: Date) => string;
}

/** Domain values accepted by ordinal, band, and point scales. */
export type DiscreteValue = string | number | Date;

/** Options for {@link scaleOrdinal}. */
export interface OrdinalScaleOptions<Domain extends DiscreteValue, Output> {
  /**
   * Known domain values in order. Duplicate values are ignored.
   *
   * @default []
   */
  readonly domain?: readonly Domain[];

  /** Output values, reused cyclically when the domain is longer. @default required */
  readonly range: readonly Output[];

  /**
   * Output for values outside the domain. `undefined` cycles through the range
   * by first-seen order without mutating the scale.
   *
   * @default undefined
   */
  readonly unknown?: Output;
}

/** Discrete scale mapping domain values to range values. */
export interface OrdinalScale<Domain extends DiscreteValue, Output> {
  /** Map a domain value to its output. */
  (value: Domain): Output;
  readonly kind: "ordinal";
  readonly domain: readonly Domain[];
  readonly range: readonly Output[];

  /** Return a copy with options replaced. */
  readonly with: (
    options: Partial<OrdinalScaleOptions<Domain, Output>>,
  ) => OrdinalScale<Domain, Output>;
}

/** Options for {@link scaleBand}. */
export interface BandScaleOptions<Domain extends DiscreteValue> {
  /**
   * Categories in order.
   *
   * @default []
   */
  readonly domain?: readonly Domain[];

  /**
   * Continuous output extent.
   *
   * @default [0, 1]
   */
  readonly range?: readonly [number, number];

  /**
   * Fraction of the step reserved between bands, in `[0, 1]`.
   *
   * @default 0
   */
  readonly paddingInner?: number;

  /**
   * Padding before the first and after the last band, in steps.
   *
   * @default 0
   */
  readonly paddingOuter?: number;

  /**
   * Shorthand setting both inner and outer padding.
   *
   * @default undefined
   */
  readonly padding?: number;

  /**
   * Distribution of outer space: `0` start, `0.5` centered, `1` end.
   *
   * @default 0.5
   */
  readonly align?: number;

  /**
   * Round step, start, and bandwidth to integers.
   *
   * @default false
   */
  readonly round?: boolean;
}

/** Band scale for bar charts: each category owns a band of equal width. */
export interface BandScale<Domain extends DiscreteValue> {
  /** Band start for a category, or `NaN` for unknown categories. */
  (value: Domain): number;
  readonly kind: "band" | "point";
  readonly domain: readonly Domain[];
  readonly range: readonly [number, number];

  /** Width of each band. */
  readonly bandwidth: number;

  /** Distance between the starts of adjacent bands. */
  readonly step: number;

  /** Every category, as ticks for an axis. */
  readonly ticks: () => Domain[];

  /** Return a copy with options replaced. */
  readonly with: (options: BandScaleOptions<Domain>) => BandScale<Domain>;
}

/** Point scale: a band scale with zero bandwidth, for dot and line plots. */
export interface PointScale<Domain extends DiscreteValue> extends BandScale<Domain> {
  readonly kind: "point";
  readonly with: (options: BandScaleOptions<Domain>) => PointScale<Domain>;
}

/** Any scale that maps values to positions. */
export type PositionScale<Domain> = Domain extends Date
  ? TimeScale | BandScale<Domain>
  : Domain extends number
    ? LinearScale | PowScale | LogScale | BandScale<Domain>
    : Domain extends string
      ? BandScale<Domain>
      : never;
