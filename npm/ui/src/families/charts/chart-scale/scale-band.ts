// Ordinal, band, and point scales follow d3-scale 4 (ISC licensed) layout rules.

import type {
  BandScale,
  BandScaleOptions,
  DiscreteValue,
  OrdinalScale,
  OrdinalScaleOptions,
  PointScale,
} from "./scale-types.ts";

type DiscreteKey = string | number;

function copyValue<Domain extends DiscreteValue>(value: Domain): Domain {
  return (value instanceof Date ? new Date(value.getTime()) : value) as Domain;
}

/** Identity key for discrete values; dates compare by timestamp like d3's InternMap. */
export function discreteKey(value: DiscreteValue): DiscreteKey {
  return value instanceof Date ? value.getTime() : value;
}

function uniqueDomain<Domain extends DiscreteValue>(
  values: readonly Domain[],
): {
  readonly domain: readonly Domain[];
  readonly index: ReadonlyMap<DiscreteKey, number>;
} {
  const domain: Domain[] = [];
  const index = new Map<DiscreteKey, number>();
  for (const value of values) {
    const key = discreteKey(value);
    if (index.has(key)) continue;
    index.set(key, domain.length);
    domain.push(copyValue(value));
  }
  return { domain: Object.freeze(domain), index };
}

/**
 * Create an immutable ordinal scale. Unlike d3, unknown values never mutate the
 * domain; without an `unknown` output they cycle through the range in the order
 * they are first seen by this scale instance.
 */
export function scaleOrdinal<Domain extends DiscreteValue, Output>(
  options: OrdinalScaleOptions<Domain, Output>,
): OrdinalScale<Domain, Output> {
  const { domain, index } = uniqueDomain(options.domain ?? []);
  const range = Object.freeze([...options.range]);
  const implicit = new Map<DiscreteKey, number>();
  const hasUnknown = "unknown" in options && options.unknown !== undefined;
  const lookup = (value: Domain): Output => {
    const key = discreteKey(value);
    let position = index.get(key);
    if (position === undefined) {
      if (hasUnknown && options.unknown !== undefined) return options.unknown;
      position = implicit.get(key);
      if (position === undefined) {
        position = domain.length + implicit.size;
        implicit.set(key, position);
      }
    }
    const output = range[position % range.length];
    if (output === undefined)
      throw new RangeError("VIZE_UI_SCALE_EMPTY_RANGE: ordinal range is empty");
    return output;
  };
  const scale = Object.assign(lookup, {
    kind: "ordinal" as const,
    range,
    with: (next: Partial<OrdinalScaleOptions<Domain, Output>>) =>
      scaleOrdinal<Domain, Output>({ ...options, domain, range, ...next }),
  }) as OrdinalScale<Domain, Output>;
  Object.defineProperty(scale, "domain", {
    enumerable: true,
    get: () => Object.freeze(domain.map(copyValue)),
  });
  return scale;
}

interface BandLayout {
  readonly step: number;
  readonly bandwidth: number;
  readonly starts: readonly number[];
}

function layoutBands(
  count: number,
  range: readonly [number, number],
  paddingInner: number,
  paddingOuter: number,
  align: number,
  round: boolean,
): BandLayout {
  const [r0, r1] = range;
  const reverse = r1 < r0;
  let start = reverse ? r1 : r0;
  const stop = reverse ? r0 : r1;
  let step = (stop - start) / Math.max(1, count - paddingInner + paddingOuter * 2);
  if (round) step = Math.floor(step);
  start += (stop - start - step * (count - paddingInner)) * align;
  let bandwidth = step * (1 - paddingInner);
  if (round) {
    start = Math.round(start);
    bandwidth = Math.round(bandwidth);
  }
  const starts = Array.from({ length: count }, (_, i) => start + step * i);
  return { bandwidth, starts: reverse ? starts.reverse() : starts, step };
}

function createBand<Domain extends DiscreteValue>(
  options: BandScaleOptions<Domain>,
  kind: "band" | "point",
): BandScale<Domain> {
  const { domain, index } = uniqueDomain(options.domain ?? []);
  const range: readonly [number, number] = Object.freeze([
    options.range?.[0] ?? 0,
    options.range?.[1] ?? 1,
  ]);
  const paddingOuter = options.padding ?? options.paddingOuter ?? 0;
  const paddingInner =
    kind === "point" ? 1 : Math.min(1, options.padding ?? options.paddingInner ?? 0);
  const align = Math.max(0, Math.min(1, options.align ?? 0.5));
  const layout = layoutBands(
    domain.length,
    range,
    paddingInner,
    paddingOuter,
    align,
    options.round ?? false,
  );
  const current: BandScaleOptions<Domain> = { ...options, domain, range };
  const scale = Object.assign(
    (value: Domain) => {
      const position = index.get(discreteKey(value));
      return position === undefined ? Number.NaN : (layout.starts[position] ?? Number.NaN);
    },
    {
      kind,
      range,
      bandwidth: layout.bandwidth,
      step: layout.step,
      ticks: () => domain.map(copyValue),
      with: (next: BandScaleOptions<Domain>) => createBand<Domain>({ ...current, ...next }, kind),
    },
  ) as BandScale<Domain>;
  Object.defineProperty(scale, "domain", {
    enumerable: true,
    get: () => Object.freeze(domain.map(copyValue)),
  });
  return scale;
}

/**
 * Create an immutable band scale for categorical positions such as bars.
 *
 * @example
 * ```ts
 * const x = scaleBand({ domain: ["a", "b", "c"], range: [0, 300], paddingInner: 0.1 });
 * x("b"); // band start
 * x.bandwidth; // band width
 * ```
 */
export function scaleBand<Domain extends DiscreteValue>(
  options: BandScaleOptions<Domain> = {},
): BandScale<Domain> {
  return createBand(options, "band");
}

/** Create an immutable point scale: evenly spaced positions with zero bandwidth. */
export function scalePoint<Domain extends DiscreteValue>(
  options: BandScaleOptions<Domain> = {},
): PointScale<Domain> {
  const { padding, ...rest } = options;
  const band = createBand({ ...rest, paddingOuter: padding ?? rest.paddingOuter ?? 0 }, "point");
  return Object.assign(band, {
    kind: "point" as const,
    with: (next: BandScaleOptions<Domain>) => scalePoint<Domain>({ ...options, ...next }),
  });
}
