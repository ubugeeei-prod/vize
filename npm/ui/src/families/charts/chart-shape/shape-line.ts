// Line and area generators following d3-shape 3 (ISC licensed): the same
// `defined` gap handling, area back-line ordering, and curve protocol.

import { resolveCurve } from "./shape-curve.ts";
import type { CurveFactory, CurveName } from "./shape-curve.ts";
import { createPath } from "./shape-path.ts";

/** Accessor reading one coordinate from a datum. */
export type ShapeAccessor<T, Value = number> = (
  datum: T,
  index: number,
  data: readonly T[],
) => Value;

/** Options for {@link linePath}. */
export interface LineOptions<T> {
  /** Horizontal position of a datum. @default required */
  readonly x: ShapeAccessor<T>;

  /** Vertical position of a datum. @default required */
  readonly y: ShapeAccessor<T>;

  /**
   * Whether a datum is drawn. Undefined data split the line into segments.
   *
   * @default every datum is defined
   */
  readonly defined?: ShapeAccessor<T, boolean>;

  /**
   * Curve name or factory.
   *
   * @default "linear"
   */
  readonly curve?: CurveName | CurveFactory;

  /**
   * Coordinate rounding digits. `null` keeps full precision.
   *
   * @default 3
   */
  readonly digits?: number | null;
}

/** Options for {@link areaPath}. */
export interface AreaOptions<T> {
  /** Baseline and topline horizontal position. @default required */
  readonly x: ShapeAccessor<T>;

  /**
   * Topline horizontal position, for horizontal areas.
   *
   * @default the `x` accessor
   */
  readonly x1?: ShapeAccessor<T>;

  /**
   * Baseline vertical position.
   *
   * @default 0
   */
  readonly y0?: ShapeAccessor<T> | number;

  /** Topline vertical position. @default required */
  readonly y1: ShapeAccessor<T>;

  /**
   * Whether a datum is drawn. Undefined data split the area.
   *
   * @default every datum is defined
   */
  readonly defined?: ShapeAccessor<T, boolean>;

  /**
   * Curve name or factory.
   *
   * @default "linear"
   */
  readonly curve?: CurveName | CurveFactory;

  /**
   * Coordinate rounding digits. `null` keeps full precision.
   *
   * @default 3
   */
  readonly digits?: number | null;
}

/**
 * Generate an SVG path for a line through `data`, or `null` when nothing is drawn.
 *
 * @example
 * ```ts
 * linePath(points, { x: (d) => xScale(d.date), y: (d) => yScale(d.value), curve: "monotoneX" });
 * ```
 */
export function linePath<T>(data: readonly T[], options: LineOptions<T>): string | null {
  const path = createPath(options.digits === undefined ? 3 : options.digits);
  const output = resolveCurve(options.curve)(path);
  const defined = options.defined;
  let drawing = false;
  for (let i = 0; i <= data.length; i++) {
    const datum = i < data.length ? data[i] : undefined;
    const isDefined =
      datum !== undefined && i < data.length && (defined === undefined || defined(datum, i, data));
    if (isDefined !== drawing) {
      drawing = isDefined;
      if (drawing) output.lineStart();
      else output.lineEnd();
    }
    if (drawing && datum !== undefined)
      output.point(+options.x(datum, i, data), +options.y(datum, i, data));
  }
  return path.toString() || null;
}

/** Generate an SVG path for an area between a baseline and a topline. */
export function areaPath<T>(data: readonly T[], options: AreaOptions<T>): string | null {
  const path = createPath(options.digits === undefined ? 3 : options.digits);
  const output = resolveCurve(options.curve)(path);
  const defined = options.defined;
  const baseline = options.y0 ?? 0;
  const y0 = typeof baseline === "number" ? () => baseline : baseline;
  const x0z: number[] = [];
  const y0z: number[] = [];
  let drawing = false;
  let segmentStart = 0;
  for (let i = 0; i <= data.length; i++) {
    const datum = i < data.length ? data[i] : undefined;
    const isDefined =
      datum !== undefined && i < data.length && (defined === undefined || defined(datum, i, data));
    if (isDefined !== drawing) {
      drawing = isDefined;
      if (drawing) {
        segmentStart = i;
        output.areaStart();
        output.lineStart();
      } else {
        output.lineEnd();
        output.lineStart();
        for (let k = i - 1; k >= segmentStart; --k) output.point(x0z[k] ?? 0, y0z[k] ?? 0);
        output.lineEnd();
        output.areaEnd();
      }
    }
    if (drawing && datum !== undefined) {
      x0z[i] = +options.x(datum, i, data);
      y0z[i] = +y0(datum, i, data);
      output.point(
        options.x1 === undefined ? (x0z[i] ?? 0) : +options.x1(datum, i, data),
        +options.y1(datum, i, data),
      );
    }
  }
  return path.toString() || null;
}
