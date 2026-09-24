/** Compile-only assertions for the public chart scale contract. */

import {
  scaleBand,
  scaleLinear,
  scaleLog,
  scaleOrdinal,
  scalePoint,
  scalePow,
  scaleTime,
  type BandScale,
  type LinearScale,
  type LogScale,
  type OrdinalScale,
  type PointScale,
  type PowScale,
  type ScaleKind,
  type TimeScale,
  type TimeTickGranularity,
} from "./chart-scale.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const linear = scaleLinear({ domain: [0, 1], range: [0, 100] });
const pow = scalePow({ exponent: 2 });
const log = scaleLog({ base: 2 });
const time = scaleTime({ domain: [new Date(0), 86_400_000], timeZone: "Europe/Berlin" });
const band = scaleBand({ domain: ["north", "south"] as const, range: [0, 100] });
const point = scalePoint({ domain: [1, 2, 3] });
const ordinal = scaleOrdinal({ domain: ["a", "b"] as const, range: ["#111", "#222"] as const });

type _Kinds = Expect<
  Equal<ScaleKind, "band" | "linear" | "log" | "ordinal" | "point" | "pow" | "time">
>;
type _Granularity = Expect<
  Equal<
    TimeTickGranularity,
    "day" | "hour" | "millisecond" | "minute" | "month" | "second" | "week" | "year"
  >
>;
type _LinearIsLinear = Expect<Equal<typeof linear, LinearScale>>;
type _PowIsPow = Expect<Equal<typeof pow, PowScale>>;
type _LogIsLog = Expect<Equal<typeof log, LogScale>>;
type _TimeIsTime = Expect<Equal<typeof time, TimeScale>>;
type _BandInfersCategories = Expect<Equal<typeof band, BandScale<"north" | "south">>>;
type _PointInfersNumbers = Expect<Equal<typeof point, PointScale<1 | 2 | 3>>>;
type _OrdinalInfersBoth = Expect<Equal<typeof ordinal, OrdinalScale<"a" | "b", "#111" | "#222">>>;
type _LinearMaps = Expect<Equal<ReturnType<typeof linear>, number>>;
type _TimeInverts = Expect<Equal<ReturnType<typeof time.invert>, Date>>;
type _TimeTicks = Expect<Equal<ReturnType<typeof time.ticks>, Date[]>>;
type _NiceKeepsType = Expect<Equal<ReturnType<typeof linear.nice>, LinearScale>>;
type _OrdinalOutput = Expect<Equal<ReturnType<typeof ordinal>, "#111" | "#222">>;

linear(null);
time(new Date());
time(0);
band("north");

// @ts-expect-error band scales only accept declared categories.
band("east");

// @ts-expect-error linear scales map numbers.
linear("1");

// @ts-expect-error domains are numeric.
scaleLinear({ domain: ["0", "1"] });

// @ts-expect-error scales are immutable.
linear.domain = [0, 2];

// @ts-expect-error ordinal ranges are required.
scaleOrdinal({ domain: ["a"] });
