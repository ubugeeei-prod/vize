import { createPath } from "./shape-path.ts";

/** Axis along which bar values grow. */
export type BarOrientation = "horizontal" | "vertical";

/** Pixel rectangle of one bar. */
export interface BarRect<T> {
  /** Source datum. */
  readonly data: T;

  /** Index in the input data. */
  readonly index: number;

  /** Numeric value encoded by the bar. */
  readonly value: number;

  /** Left edge. */
  readonly x: number;

  /** Top edge. */
  readonly y: number;

  /** Non-negative width. */
  readonly width: number;

  /** Non-negative height. */
  readonly height: number;
}

/** Category positioning contract, satisfied by band scales. */
export interface BarCategoryScale<Category> {
  (value: Category): number;
  readonly bandwidth: number;
}

/** Options for {@link barRects}. */
export interface BarOptions<T, Category> {
  /** Category of a datum. @default required */
  readonly category: (datum: T, index: number) => Category;

  /** Value of a datum. @default required */
  readonly value: (datum: T, index: number) => number;

  /** Band scale positioning categories. @default required */
  readonly categoryScale: BarCategoryScale<Category>;

  /** Continuous scale positioning values. @default required */
  readonly valueScale: (value: number) => number;

  /**
   * Direction bars grow in.
   *
   * @default "vertical"
   */
  readonly orientation?: BarOrientation;

  /**
   * Value the bars start from.
   *
   * @default 0
   */
  readonly baseline?: number | ((datum: T, index: number) => number);
}

/**
 * Compute bar rectangles. Negative values grow away from the baseline in the
 * opposite direction, and widths and heights are always non-negative so the
 * result can be bound directly to SVG `<rect>` attributes.
 */
export function barRects<T, Category>(
  data: readonly T[],
  options: BarOptions<T, Category>,
): BarRect<T>[] {
  const vertical = (options.orientation ?? "vertical") === "vertical";
  const bandwidth = options.categoryScale.bandwidth;
  return data.map((datum, index) => {
    const value = +options.value(datum, index);
    const base =
      typeof options.baseline === "function"
        ? options.baseline(datum, index)
        : (options.baseline ?? 0);
    const band = options.categoryScale(options.category(datum, index));
    const from = options.valueScale(base);
    const to = options.valueScale(value);
    const low = Math.min(from, to);
    const extent = Math.abs(to - from);
    return vertical
      ? { data: datum, index, value, x: band, y: low, width: bandwidth, height: extent }
      : { data: datum, index, value, x: low, y: band, width: extent, height: bandwidth };
  });
}

/** Corner radii of a bar path. */
export interface BarCornerRadius {
  readonly topLeft?: number;
  readonly topRight?: number;
  readonly bottomRight?: number;
  readonly bottomLeft?: number;
}

/**
 * SVG path for a rectangle with independently rounded corners, clamped so
 * opposite radii never overlap.
 */
export function roundedRectPath(
  rect: Pick<BarRect<unknown>, "height" | "width" | "x" | "y">,
  radius: number | BarCornerRadius,
  digits: number | null = 3,
): string {
  const { x, y, width, height } = rect;
  const limit = Math.max(0, Math.min(width, height) / 2);
  const read = (value: number | undefined) => Math.max(0, Math.min(limit, value ?? 0));
  const corners =
    typeof radius === "number"
      ? { topLeft: radius, topRight: radius, bottomRight: radius, bottomLeft: radius }
      : radius;
  const tl = read(corners.topLeft);
  const tr = read(corners.topRight);
  const br = read(corners.bottomRight);
  const bl = read(corners.bottomLeft);
  const path = createPath(digits);
  const quarter = Math.PI / 2;
  path.moveTo(x + tl, y);
  path.lineTo(x + width - tr, y);
  if (tr) path.arc(x + width - tr, y + tr, tr, -quarter, 0);
  path.lineTo(x + width, y + height - br);
  if (br) path.arc(x + width - br, y + height - br, br, 0, quarter);
  path.lineTo(x + bl, y + height);
  if (bl) path.arc(x + bl, y + height - bl, bl, quarter, Math.PI);
  path.lineTo(x, y + tl);
  if (tl) path.arc(x + tl, y + tl, tl, Math.PI, Math.PI * 1.5);
  path.closePath();
  return path.toString();
}
