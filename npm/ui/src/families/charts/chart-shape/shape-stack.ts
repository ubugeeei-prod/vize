// Stack layouts ported from d3-shape 3 (ISC licensed): orders none, ascending,
// descending, reverse, insideOut, appearance; offsets none, expand, diverging,
// silhouette, wiggle.

/** One stacked point: `[lower, upper]` plus the source datum. */
export interface StackPoint<T> {
  0: number;
  1: number;
  readonly data: T;
}

/** One stacked series for a key. */
export interface StackSeries<T, Key extends string> extends Array<StackPoint<T>> {
  /** Series key. */
  readonly key: Key;

  /** Position of this series in stacking order. */
  index: number;
}

/** Stacking order. */
export type StackOrder =
  | "appearance"
  | "ascending"
  | "descending"
  | "insideOut"
  | "none"
  | "reverse";

/** Baseline strategy. */
export type StackOffset = "diverging" | "expand" | "none" | "silhouette" | "wiggle";

/** Options for {@link stackLayout}. */
export interface StackOptions<T, Key extends string> {
  /** Series keys in declaration order. @default required */
  readonly keys: readonly Key[];

  /** Value of one series for one datum. @default required */
  readonly value: (datum: T, key: Key, index: number, data: readonly T[]) => number;

  /**
   * Series stacking order.
   *
   * @default "none"
   */
  readonly order?: StackOrder;

  /**
   * Baseline offset.
   *
   * @default "none"
   */
  readonly offset?: StackOffset;
}

type MutableSeries = StackPoint<unknown>[];

function seriesSum(series: MutableSeries): number {
  let sum = 0;
  for (const point of series) {
    const value = +point[1];
    if (value) sum += value;
  }
  return sum;
}

function orderNone(count: number): number[] {
  return Array.from({ length: count }, (_, i) => i);
}

function peak(series: MutableSeries): number {
  let index = 0;
  let max = -Infinity;
  series.forEach((point, i) => {
    const value = +point[1];
    if (value > max) {
      max = value;
      index = i;
    }
  });
  return index;
}

function computeOrder(series: readonly MutableSeries[], order: StackOrder): number[] {
  const none = orderNone(series.length);
  const sums = series.map(seriesSum);
  const byValue = (values: readonly number[]) =>
    none.sort((a, b) => (values[a] ?? 0) - (values[b] ?? 0));
  switch (order) {
    case "ascending":
      return byValue(sums);
    case "descending":
      return byValue(sums).reverse();
    case "reverse":
      return none.reverse();
    case "appearance":
      return byValue(series.map(peak));
    case "insideOut": {
      const appearance = computeOrder(series, "appearance");
      let top = 0;
      let bottom = 0;
      const tops: number[] = [];
      const bottoms: number[] = [];
      for (const j of appearance) {
        if (top < bottom) {
          top += sums[j] ?? 0;
          tops.push(j);
        } else {
          bottom += sums[j] ?? 0;
          bottoms.push(j);
        }
      }
      return bottoms.reverse().concat(tops);
    }
    default:
      return none;
  }
}

function at(
  series: readonly MutableSeries[],
  i: number | undefined,
  j: number,
): StackPoint<unknown> | undefined {
  return i === undefined ? undefined : series[i]?.[j];
}

function offsetNone(series: readonly MutableSeries[], order: readonly number[]): void {
  const n = series.length;
  if (!(n > 1)) return;
  const m = series[order[0] ?? 0]?.length ?? 0;
  for (let i = 1; i < n; ++i) {
    for (let j = 0; j < m; ++j) {
      const previous = at(series, order[i - 1], j);
      const point = at(series, order[i], j);
      if (previous === undefined || point === undefined) continue;
      point[0] = Number.isNaN(previous[1]) ? previous[0] : previous[1];
      point[1] += point[0];
    }
  }
}

function offsetExpand(series: readonly MutableSeries[], order: readonly number[]): void {
  const n = series.length;
  if (!(n > 0)) return;
  const m = series[0]?.length ?? 0;
  for (let j = 0; j < m; ++j) {
    let y = 0;
    for (let i = 0; i < n; ++i) y += series[i]?.[j]?.[1] || 0;
    if (y) {
      for (let i = 0; i < n; ++i) {
        const point = series[i]?.[j];
        if (point !== undefined) point[1] /= y;
      }
    }
  }
  offsetNone(series, order);
}

function offsetDiverging(series: readonly MutableSeries[], order: readonly number[]): void {
  const n = series.length;
  if (!(n > 0)) return;
  const m = series[order[0] ?? 0]?.length ?? 0;
  for (let j = 0; j < m; ++j) {
    let positive = 0;
    let negative = 0;
    for (let i = 0; i < n; ++i) {
      const point = at(series, order[i], j);
      if (point === undefined) continue;
      const dy = point[1] - point[0];
      if (dy > 0) {
        point[0] = positive;
        point[1] = positive += dy;
      } else if (dy < 0) {
        point[1] = negative;
        point[0] = negative += dy;
      } else {
        point[0] = 0;
        point[1] = dy;
      }
    }
  }
}

function offsetSilhouette(series: readonly MutableSeries[], order: readonly number[]): void {
  const n = series.length;
  if (!(n > 0)) return;
  const first = series[order[0] ?? 0];
  const m = first?.length ?? 0;
  for (let j = 0; j < m; ++j) {
    let y = 0;
    for (let i = 0; i < n; ++i) y += series[i]?.[j]?.[1] || 0;
    const point = first?.[j];
    if (point !== undefined) {
      point[0] = -y / 2;
      point[1] += point[0];
    }
  }
  offsetNone(series, order);
}

function offsetWiggle(series: readonly MutableSeries[], order: readonly number[]): void {
  const n = series.length;
  const first = series[order[0] ?? 0];
  const m = first?.length ?? 0;
  if (!(n > 0) || !(m > 0) || first === undefined) return;
  let y = 0;
  let j = 1;
  for (; j < m; ++j) {
    let s1 = 0;
    let s2 = 0;
    for (let i = 0; i < n; ++i) {
      const si = series[order[i] ?? 0];
      const sij0 = si?.[j]?.[1] || 0;
      const sij1 = si?.[j - 1]?.[1] || 0;
      let s3 = (sij0 - sij1) / 2;
      for (let k = 0; k < i; ++k) {
        const sk = series[order[k] ?? 0];
        s3 += (sk?.[j]?.[1] || 0) - (sk?.[j - 1]?.[1] || 0);
      }
      s1 += sij0;
      s2 += s3 * sij0;
    }
    const point = first[j - 1];
    if (point !== undefined) {
      point[0] = y;
      point[1] += y;
    }
    if (s1) y -= s2 / s1;
  }
  const last = first[j - 1];
  if (last !== undefined) {
    last[0] = y;
    last[1] += y;
  }
  offsetNone(series, order);
}

/**
 * Stack series for stacked bar and area charts. Series are returned in `keys`
 * order; `series.index` holds the stacking position.
 *
 * @example
 * ```ts
 * const series = stackLayout(rows, { keys: ["apples", "pears"], value: (row, key) => row[key] });
 * ```
 */
export function stackLayout<T, Key extends string>(
  data: readonly T[],
  options: StackOptions<T, Key>,
): StackSeries<T, Key>[] {
  const series = options.keys.map((key) => {
    const points = data.map((datum, j) => {
      const point: StackPoint<T> = { 0: 0, 1: +options.value(datum, key, j, data), data: datum };
      return point;
    });
    return Object.assign(points, { key, index: 0 });
  });
  const erased: MutableSeries[] = series;
  const order = computeOrder(erased, options.order ?? "none");
  order.forEach((seriesIndex, position) => {
    const target = series[seriesIndex];
    if (target !== undefined) target.index = position;
  });
  switch (options.offset ?? "none") {
    case "expand":
      offsetExpand(erased, order);
      break;
    case "diverging":
      offsetDiverging(erased, order);
      break;
    case "silhouette":
      offsetSilhouette(erased, order);
      break;
    case "wiggle":
      offsetWiggle(erased, order);
      break;
    default:
      offsetNone(erased, order);
  }
  return series;
}
