// Continuous scales follow d3-scale 4 (ISC licensed): the same normalization,
// interpolation, piecewise bisection, clamping, and nice/tick rules, so outputs
// match d3 exactly. Scales here are immutable: `with()` and `nice()` return copies.

import { numberTickFormat } from "./scale-format.ts";
import { bisectRight, niceExtent, ticks as linearTicks } from "./scale-ticks.ts";
import type {
  ContinuousScaleOptions,
  LinearScale,
  LogScale,
  LogScaleOptions,
  NumberTickFormatOptions,
  PowScale,
  PowScaleOptions,
} from "./scale-types.ts";

type Unary = (value: number) => number;

const unitInterval: readonly number[] = Object.freeze([0, 1]);

function normalize(a: number, b: number): Unary {
  const span = b - a;
  if (span) return (x) => (x - a) / span;
  const constant = Number.isNaN(span) ? Number.NaN : 0.5;
  return () => constant;
}

function interpolateNumber(a: number, b: number): Unary {
  return (t) => a * (1 - t) + b * t;
}

function interpolateRound(a: number, b: number): Unary {
  return (t) => Math.round(a * (1 - t) + b * t);
}

function clamper(a: number, b: number): Unary {
  const lo = a > b ? b : a;
  const hi = a > b ? a : b;
  return (x) => Math.max(lo, Math.min(hi, x));
}

function identity(x: number): number {
  return x;
}

type Interpolator = (a: number, b: number) => Unary;

function bimap(
  domain: readonly number[],
  range: readonly number[],
  interpolate: Interpolator,
): Unary {
  const d0 = domain[0] ?? 0;
  const d1 = domain[1] ?? 0;
  const r0 = range[0] ?? 0;
  const r1 = range[1] ?? 0;
  const [toUnit, fromUnit] =
    d1 < d0 ? [normalize(d1, d0), interpolate(r1, r0)] : [normalize(d0, d1), interpolate(r0, r1)];
  return (x) => fromUnit(toUnit(x));
}

function polymap(
  domainInput: readonly number[],
  rangeInput: readonly number[],
  interpolate: Interpolator,
): Unary {
  const j = Math.min(domainInput.length, rangeInput.length) - 1;
  const descending = (domainInput[j] ?? 0) < (domainInput[0] ?? 0);
  const domain = descending ? [...domainInput].reverse() : domainInput;
  const range = descending ? [...rangeInput].reverse() : rangeInput;
  const toUnit: Unary[] = [];
  const fromUnit: Unary[] = [];
  for (let i = 0; i < j; i++) {
    toUnit.push(normalize(domain[i] ?? 0, domain[i + 1] ?? 0));
    fromUnit.push(interpolate(range[i] ?? 0, range[i + 1] ?? 0));
  }
  return (x) => {
    const i = bisectRight(domain, x, 1, j) - 1;
    const forward = toUnit[i];
    const back = fromUnit[i];
    return forward === undefined || back === undefined ? Number.NaN : back(forward(x));
  };
}

function piecewise(
  domain: readonly number[],
  range: readonly number[],
  interpolate: Interpolator,
): Unary {
  return Math.min(domain.length, range.length) > 2
    ? polymap(domain, range, interpolate)
    : bimap(domain, range, interpolate);
}

/** Resolved mapping functions for one continuous scale configuration. */
export interface ContinuousCore {
  readonly map: (value: number) => number;
  readonly invert: (value: number) => number;
}

/** Build the forward and inverse mappings shared by every continuous scale. */
export function createContinuousCore(config: {
  readonly domain: readonly number[];
  readonly range: readonly number[];
  readonly clamp: boolean;
  readonly round: boolean;
  readonly unknown: number;
  readonly transform: Unary;
  readonly untransform: Unary;
}): ContinuousCore {
  const n = Math.min(config.domain.length, config.range.length);
  const clamp = config.clamp ? clamper(config.domain[0] ?? 0, config.domain[n - 1] ?? 0) : identity;
  const transformed = config.domain.map(config.transform);
  const output = piecewise(
    transformed,
    config.range,
    config.round ? interpolateRound : interpolateNumber,
  );
  const input = piecewise(config.range, transformed, interpolateNumber);
  return {
    map: (value) => (Number.isNaN(value) ? config.unknown : output(config.transform(clamp(value)))),
    invert: (value) => clamp(config.untransform(input(value))),
  };
}

function toNumber(value: number | null | undefined): number {
  return value === null || value === undefined ? Number.NaN : +value;
}

function niceCount(nice: boolean | number | undefined): number | null {
  if (nice === undefined || nice === false) return null;
  return nice === true ? 10 : nice;
}

function niceDomain(domain: readonly number[], count: number): number[] {
  const next = [...domain];
  const last = next.length - 1;
  const [start, stop] = niceExtent(next[0] ?? 0, next[last] ?? 0, count);
  next[0] = start;
  next[last] = stop;
  return next;
}

interface ResolvedContinuous {
  readonly domain: readonly number[];
  readonly range: readonly number[];
  readonly clamp: boolean;
  readonly round: boolean;
  readonly unknown: number;
}

function resolveContinuous(
  options: ContinuousScaleOptions,
  applyNice: (domain: readonly number[], count: number) => readonly number[],
): ResolvedContinuous {
  const domain = Object.freeze([...(options.domain ?? unitInterval)].map(Number));
  const count = niceCount(options.nice);
  return {
    clamp: options.clamp ?? false,
    domain: count === null ? domain : Object.freeze([...applyNice(domain, count)]),
    range: Object.freeze([...(options.range ?? unitInterval)]),
    round: options.round ?? false,
    unknown: options.unknown ?? Number.NaN,
  };
}

function withoutNice<Options extends ContinuousScaleOptions>(
  options: Options,
  resolved: ResolvedContinuous,
): Options {
  return { ...options, domain: resolved.domain, range: resolved.range, nice: false };
}

/**
 * Create an immutable linear scale.
 *
 * @example
 * ```ts
 * const y = scaleLinear({ domain: [0, 120], range: [300, 0], nice: true });
 * y(60); // 150
 * ```
 */
export function scaleLinear(options: ContinuousScaleOptions = {}): LinearScale {
  const resolved = resolveContinuous(options, niceDomain);
  const core = createContinuousCore({ ...resolved, transform: identity, untransform: identity });
  const current: ContinuousScaleOptions = withoutNice(options, resolved);
  const first = resolved.domain[0] ?? 0;
  const last = resolved.domain.at(-1) ?? 0;
  return Object.assign((value: number | null | undefined) => core.map(toNumber(value)), {
    kind: "linear" as const,
    domain: resolved.domain,
    range: resolved.range,
    clamp: resolved.clamp,
    round: resolved.round,
    invert: core.invert,
    ticks: (count = 10) => linearTicks(first, last, count),
    tickFormat: (count = 10, format: NumberTickFormatOptions = {}) =>
      numberTickFormat(first, last, count, format),
    nice: (count = 10) => scaleLinear({ ...current, nice: count }),
    with: (next: ContinuousScaleOptions) => scaleLinear({ ...current, ...next }),
    options: () => ({ ...current }),
  });
}

function powTransform(exponent: number): [Unary, Unary] {
  if (exponent === 1) return [identity, identity];
  if (exponent === 0.5) {
    return [(x) => (x < 0 ? -Math.sqrt(-x) : Math.sqrt(x)), (x) => (x < 0 ? -x * x : x * x)];
  }
  const forward =
    (e: number): Unary =>
    (x) =>
      x < 0 ? -((-x) ** e) : x ** e;
  return [forward(exponent), forward(1 / exponent)];
}

/**
 * Create an immutable power scale. Use `exponent: 0.5` for area-proportional
 * encodings such as bubble radii.
 */
export function scalePow(options: PowScaleOptions = {}): PowScale {
  const exponent = options.exponent ?? 1;
  const resolved = resolveContinuous(options, niceDomain);
  const [transform, untransform] = powTransform(exponent);
  const core = createContinuousCore({ ...resolved, transform, untransform });
  const current: PowScaleOptions = { ...withoutNice(options, resolved), exponent };
  const first = resolved.domain[0] ?? 0;
  const last = resolved.domain.at(-1) ?? 0;
  return Object.assign((value: number | null | undefined) => core.map(toNumber(value)), {
    kind: "pow" as const,
    exponent,
    domain: resolved.domain,
    range: resolved.range,
    clamp: resolved.clamp,
    round: resolved.round,
    invert: core.invert,
    ticks: (count = 10) => linearTicks(first, last, count),
    tickFormat: (count = 10, format: NumberTickFormatOptions = {}) =>
      numberTickFormat(first, last, count, format),
    nice: (count = 10) => scalePow({ ...current, nice: count }),
    with: (next: PowScaleOptions) => scalePow({ ...current, ...next }),
    options: () => ({ ...current }),
  });
}

/** Create a square-root scale, `scalePow` with `exponent: 0.5`. */
export function scaleSqrt(options: ContinuousScaleOptions = {}): PowScale {
  return scalePow({ ...options, exponent: 0.5 });
}

function pow10(x: number): number {
  return Number.isFinite(x) ? Number(`1e${x}`) : x < 0 ? 0 : x;
}

function powp(base: number): Unary {
  if (base === 10) return pow10;
  if (base === Math.E) return Math.exp;
  return (x) => base ** x;
}

function logp(base: number): Unary {
  if (base === Math.E) return Math.log;
  if (base === 10) return Math.log10;
  if (base === 2) return Math.log2;
  const divisor = Math.log(base);
  return (x) => Math.log(x) / divisor;
}

function reflect(f: Unary): Unary {
  return (x) => -f(-x);
}

function logTicks(domain: readonly number[], base: number, count: number): number[] {
  let u = domain[0] ?? 1;
  let v = domain.at(-1) ?? 1;
  const reversed = v < u;
  if (reversed) [u, v] = [v, u];
  const negative = (domain[0] ?? 1) < 0;
  const logs = negative ? reflect(logp(base)) : logp(base);
  const pows = negative ? reflect(powp(base)) : powp(base);
  let i = logs(u);
  let j = logs(v);
  let z: number[] = [];
  if (!(base % 1) && j - i < count) {
    i = Math.floor(i);
    j = Math.ceil(j);
    if (u > 0) {
      for (; i <= j; ++i) {
        for (let k = 1; k < base; ++k) {
          const t = i < 0 ? k / pows(-i) : k * pows(i);
          if (t < u) continue;
          if (t > v) break;
          z.push(t);
        }
      }
    } else {
      for (; i <= j; ++i) {
        for (let k = base - 1; k >= 1; --k) {
          const t = i > 0 ? k / pows(-i) : k * pows(i);
          if (t < u) continue;
          if (t > v) break;
          z.push(t);
        }
      }
    }
    if (z.length * 2 < count) z = linearTicks(u, v, count);
  } else {
    z = linearTicks(i, j, Math.min(j - i, count)).map(pows);
  }
  return reversed ? z.reverse() : z;
}

/**
 * Create an immutable logarithmic scale. Ticks and nice domains snap to powers
 * of the base, and `tickFormat` blanks minor ticks on dense axes like d3.
 */
export function scaleLog(options: LogScaleOptions = {}): LogScale {
  const base = options.base ?? 10;
  const sourceDomain = options.domain ?? [1, 10];
  const negative = (sourceDomain[0] ?? 1) < 0;
  const logs = negative ? reflect(logp(base)) : logp(base);
  const pows = negative ? reflect(powp(base)) : powp(base);
  const resolved = resolveContinuous({ ...options, domain: sourceDomain }, (domain) => {
    const next = [...domain];
    const lastIndex = next.length - 1;
    const x0 = next[0] ?? 1;
    const x1 = next[lastIndex] ?? 1;
    const [lo, hi, i0, i1] = x1 < x0 ? [x1, x0, lastIndex, 0] : [x0, x1, 0, lastIndex];
    next[i0] = pows(Math.floor(logs(lo)));
    next[i1] = pows(Math.ceil(logs(hi)));
    return next;
  });
  const core = createContinuousCore({
    ...resolved,
    transform: negative ? (x) => -Math.log(-x) : Math.log,
    untransform: negative ? (x) => -Math.exp(-x) : Math.exp,
  });
  const current: LogScaleOptions = { ...withoutNice(options, resolved), base };
  const ticks = (count = 10) => logTicks(resolved.domain, base, count);
  return Object.assign((value: number | null | undefined) => core.map(toNumber(value)), {
    kind: "log" as const,
    base,
    domain: resolved.domain,
    range: resolved.range,
    clamp: resolved.clamp,
    round: resolved.round,
    invert: core.invert,
    ticks,
    tickFormat: (count = 10, format: NumberTickFormatOptions = {}) => {
      const formatter = new Intl.NumberFormat(format.locale ?? "en-US", {
        notation: "compact",
        ...format.format,
      });
      const k = Math.max(1, (base * count) / ticks().length);
      return (value: number) => {
        let i = value / pows(Math.round(logs(value)));
        if (i * base < base - 0.5) i *= base;
        return i <= k ? formatter.format(value) : "";
      };
    },
    nice: () => scaleLog({ ...current, nice: true }),
    with: (next: LogScaleOptions) => scaleLog({ ...current, ...next }),
    options: () => ({ ...current }),
  });
}
