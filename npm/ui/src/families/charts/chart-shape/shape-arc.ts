// Arc and pie generators ported from d3-shape 3 (ISC licensed), including padded
// and rounded-corner annular sectors.

import { createPath } from "./shape-path.ts";
import type { PathContext } from "./shape-path.ts";

const epsilon = 1e-12;
const pi = Math.PI;
const halfPi = pi / 2;
const tau = 2 * pi;

function acos(x: number): number {
  return x > 1 ? 0 : x < -1 ? pi : Math.acos(x);
}

function asin(x: number): number {
  return x >= 1 ? halfPi : x <= -1 ? -halfPi : Math.asin(x);
}

/** Geometry of one arc, in radians clockwise from 12 o'clock. */
export interface ArcGeometry {
  /** Inner radius; `0` draws a pie slice. */
  readonly innerRadius: number;

  /** Outer radius. */
  readonly outerRadius: number;

  /** Start angle in radians. */
  readonly startAngle: number;

  /** End angle in radians. */
  readonly endAngle: number;

  /** Angular padding between adjacent arcs. @default 0 */
  readonly padAngle?: number;

  /** Rounded corner radius. @default 0 */
  readonly cornerRadius?: number;

  /** Radius at which `padAngle` is measured. @default sqrt(inner² + outer²) */
  readonly padRadius?: number;
}

function intersect(
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  x3: number,
  y3: number,
): [number, number] | undefined {
  const x10 = x1 - x0;
  const y10 = y1 - y0;
  const x32 = x3 - x2;
  const y32 = y3 - y2;
  let t = y32 * x10 - x32 * y10;
  if (t * t < epsilon) return undefined;
  t = (x32 * (y0 - y2) - y32 * (x0 - x2)) / t;
  return [x0 + t * x10, y0 + t * y10];
}

interface CornerTangents {
  readonly cx: number;
  readonly cy: number;
  readonly x01: number;
  readonly y01: number;
  readonly x11: number;
  readonly y11: number;
}

function cornerTangents(
  x0: number,
  y0: number,
  x1: number,
  y1: number,
  r1: number,
  rc: number,
  cw: boolean,
): CornerTangents {
  const x01 = x0 - x1;
  const y01 = y0 - y1;
  const lo = (cw ? rc : -rc) / Math.sqrt(x01 * x01 + y01 * y01);
  const ox = lo * y01;
  const oy = -lo * x01;
  const x11 = x0 + ox;
  const y11 = y0 + oy;
  const x10 = x1 + ox;
  const y10 = y1 + oy;
  const x00 = (x11 + x10) / 2;
  const y00 = (y11 + y10) / 2;
  const dx = x10 - x11;
  const dy = y10 - y11;
  const d2 = dx * dx + dy * dy;
  const r = r1 - rc;
  const D = x11 * y10 - x10 * y11;
  const d = (dy < 0 ? -1 : 1) * Math.sqrt(Math.max(0, r * r * d2 - D * D));
  let cx0 = (D * dy - dx * d) / d2;
  let cy0 = (-D * dx - dy * d) / d2;
  const cx1 = (D * dy + dx * d) / d2;
  const cy1 = (-D * dx + dy * d) / d2;
  const dx0 = cx0 - x00;
  const dy0 = cy0 - y00;
  const dx1 = cx1 - x00;
  const dy1 = cy1 - y00;
  if (dx0 * dx0 + dy0 * dy0 > dx1 * dx1 + dy1 * dy1) {
    cx0 = cx1;
    cy0 = cy1;
  }
  return {
    cx: cx0,
    cy: cy0,
    x01: -ox,
    y01: -oy,
    x11: cx0 * (r1 / r - 1),
    y11: cy0 * (r1 / r - 1),
  };
}

function drawArc(context: PathContext, geometry: ArcGeometry): void {
  let r0 = +geometry.innerRadius;
  let r1 = +geometry.outerRadius;
  const a0 = geometry.startAngle - halfPi;
  const a1 = geometry.endAngle - halfPi;
  const da = Math.abs(a1 - a0);
  const cw = a1 > a0;
  if (r1 < r0) [r0, r1] = [r1, r0];

  if (!(r1 > epsilon)) {
    context.moveTo(0, 0);
  } else if (da > tau - epsilon) {
    context.moveTo(r1 * Math.cos(a0), r1 * Math.sin(a0));
    context.arc(0, 0, r1, a0, a1, !cw);
    if (r0 > epsilon) {
      context.moveTo(r0 * Math.cos(a1), r0 * Math.sin(a1));
      context.arc(0, 0, r0, a1, a0, cw);
    }
  } else {
    let a01 = a0;
    let a11 = a1;
    let a00 = a0;
    let a10 = a1;
    let da0 = da;
    let da1 = da;
    const ap = (geometry.padAngle ?? 0) / 2;
    const rp = ap > epsilon ? (geometry.padRadius ?? Math.sqrt(r0 * r0 + r1 * r1)) : 0;
    const rc = Math.min(Math.abs(r1 - r0) / 2, geometry.cornerRadius ?? 0);
    let rc0 = rc;
    let rc1 = rc;

    if (rp > epsilon) {
      let p0 = asin((rp / r0) * Math.sin(ap));
      let p1 = asin((rp / r1) * Math.sin(ap));
      if ((da0 -= p0 * 2) > epsilon) {
        p0 *= cw ? 1 : -1;
        a00 += p0;
        a10 -= p0;
      } else {
        da0 = 0;
        a00 = a10 = (a0 + a1) / 2;
      }
      if ((da1 -= p1 * 2) > epsilon) {
        p1 *= cw ? 1 : -1;
        a01 += p1;
        a11 -= p1;
      } else {
        da1 = 0;
        a01 = a11 = (a0 + a1) / 2;
      }
    }

    const x01 = r1 * Math.cos(a01);
    const y01 = r1 * Math.sin(a01);
    const x10 = r0 * Math.cos(a10);
    const y10 = r0 * Math.sin(a10);
    const x11 = r1 * Math.cos(a11);
    const y11 = r1 * Math.sin(a11);
    const x00 = r0 * Math.cos(a00);
    const y00 = r0 * Math.sin(a00);

    if (rc > epsilon && da < pi) {
      const oc = intersect(x01, y01, x00, y00, x11, y11, x10, y10);
      if (oc !== undefined) {
        const ax = x01 - oc[0];
        const ay = y01 - oc[1];
        const bx = x11 - oc[0];
        const by = y11 - oc[1];
        const kc =
          1 /
          Math.sin(
            acos(
              (ax * bx + ay * by) / (Math.sqrt(ax * ax + ay * ay) * Math.sqrt(bx * bx + by * by)),
            ) / 2,
          );
        const lc = Math.sqrt(oc[0] * oc[0] + oc[1] * oc[1]);
        rc0 = Math.min(rc, (r0 - lc) / (kc - 1));
        rc1 = Math.min(rc, (r1 - lc) / (kc + 1));
      } else {
        rc0 = rc1 = 0;
      }
    }

    if (!(da1 > epsilon)) {
      context.moveTo(x01, y01);
    } else if (rc1 > epsilon) {
      const t0 = cornerTangents(x00, y00, x01, y01, r1, rc1, cw);
      const t1 = cornerTangents(x11, y11, x10, y10, r1, rc1, cw);
      context.moveTo(t0.cx + t0.x01, t0.cy + t0.y01);
      if (rc1 < rc) {
        context.arc(t0.cx, t0.cy, rc1, Math.atan2(t0.y01, t0.x01), Math.atan2(t1.y01, t1.x01), !cw);
      } else {
        context.arc(t0.cx, t0.cy, rc1, Math.atan2(t0.y01, t0.x01), Math.atan2(t0.y11, t0.x11), !cw);
        context.arc(
          0,
          0,
          r1,
          Math.atan2(t0.cy + t0.y11, t0.cx + t0.x11),
          Math.atan2(t1.cy + t1.y11, t1.cx + t1.x11),
          !cw,
        );
        context.arc(t1.cx, t1.cy, rc1, Math.atan2(t1.y11, t1.x11), Math.atan2(t1.y01, t1.x01), !cw);
      }
    } else {
      context.moveTo(x01, y01);
      context.arc(0, 0, r1, a01, a11, !cw);
    }

    if (!(r0 > epsilon) || !(da0 > epsilon)) {
      context.lineTo(x10, y10);
    } else if (rc0 > epsilon) {
      const t0 = cornerTangents(x10, y10, x11, y11, r0, -rc0, cw);
      const t1 = cornerTangents(x01, y01, x00, y00, r0, -rc0, cw);
      context.lineTo(t0.cx + t0.x01, t0.cy + t0.y01);
      if (rc0 < rc) {
        context.arc(t0.cx, t0.cy, rc0, Math.atan2(t0.y01, t0.x01), Math.atan2(t1.y01, t1.x01), !cw);
      } else {
        context.arc(t0.cx, t0.cy, rc0, Math.atan2(t0.y01, t0.x01), Math.atan2(t0.y11, t0.x11), !cw);
        context.arc(
          0,
          0,
          r0,
          Math.atan2(t0.cy + t0.y11, t0.cx + t0.x11),
          Math.atan2(t1.cy + t1.y11, t1.cx + t1.x11),
          cw,
        );
        context.arc(t1.cx, t1.cy, rc0, Math.atan2(t1.y11, t1.x11), Math.atan2(t1.y01, t1.x01), !cw);
      }
    } else {
      context.arc(0, 0, r0, a10, a00, cw);
    }
  }
  context.closePath();
}

/**
 * Generate the SVG path of a circular or annular sector centered at the origin.
 *
 * @example
 * ```ts
 * arcPath({ innerRadius: 40, outerRadius: 80, startAngle: 0, endAngle: Math.PI / 2 });
 * ```
 */
export function arcPath(geometry: ArcGeometry, digits: number | null = 3): string {
  const path = createPath(digits);
  drawArc(path, geometry);
  return path.toString();
}

/** Midpoint of an arc, useful for labels and tooltips. */
export function arcCentroid(geometry: ArcGeometry): [number, number] {
  const r = (geometry.innerRadius + geometry.outerRadius) / 2;
  const a = (geometry.startAngle + geometry.endAngle) / 2 - pi / 2;
  return [Math.cos(a) * r, Math.sin(a) * r];
}

/** One slice produced by {@link pieLayout}. */
export interface PieSlice<T> {
  /** Source datum. */
  readonly data: T;

  /** Position in drawing order (after sorting). */
  readonly index: number;

  /** Numeric value used for the slice. */
  readonly value: number;

  /** Start angle in radians. */
  readonly startAngle: number;

  /** End angle in radians. */
  readonly endAngle: number;

  /** Padding angle applied between slices. */
  readonly padAngle: number;
}

/** Options for {@link pieLayout}. */
export interface PieOptions<T> {
  /** Slice value. Non-positive values produce zero-width slices. @default required */
  readonly value: (datum: T, index: number, data: readonly T[]) => number;

  /**
   * Drawing order: `"descending"` by value (d3's default), `"none"` for input
   * order, or a comparator over data.
   *
   * @default "descending"
   */
  readonly sort?: "descending" | "none" | ((left: T, right: T) => number);

  /**
   * Start angle in radians.
   *
   * @default 0
   */
  readonly startAngle?: number;

  /**
   * End angle in radians.
   *
   * @default 2π
   */
  readonly endAngle?: number;

  /**
   * Padding between slices in radians.
   *
   * @default 0
   */
  readonly padAngle?: number;
}

/** Compute pie or donut slice angles. Slices are returned in input order. */
export function pieLayout<T>(data: readonly T[], options: PieOptions<T>): PieSlice<T>[] {
  const n = data.length;
  const a0 = options.startAngle ?? 0;
  const da = Math.min(tau, Math.max(-tau, (options.endAngle ?? tau) - a0));
  const p = Math.min(Math.abs(da) / n, options.padAngle ?? 0);
  const pa = p * (da < 0 ? -1 : 1);
  const values = data.map((datum, i) => +options.value(datum, i, data));
  let sum = 0;
  for (const value of values) if (value > 0) sum += value;
  const order = values.map((_, i) => i);
  const sort = options.sort ?? "descending";
  if (sort === "descending") {
    order.sort((i, j) => {
      const left = values[i] ?? 0;
      const right = values[j] ?? 0;
      return right < left ? -1 : right > left ? 1 : right >= left ? 0 : Number.NaN;
    });
  } else if (typeof sort === "function") {
    order.sort((i, j) => {
      const left = data[i];
      const right = data[j];
      return left === undefined || right === undefined ? 0 : sort(left, right);
    });
  }
  const k = sum ? (da - n * pa) / sum : 0;
  const slices = Array.from<PieSlice<T> | undefined>({ length: n });
  let angle = a0;
  order.forEach((j, index) => {
    const value = values[j] ?? 0;
    const datum = data[j];
    if (datum === undefined) return;
    const end = angle + (value > 0 ? value * k : 0) + pa;
    slices[j] = { data: datum, index, value, startAngle: angle, endAngle: end, padAngle: p };
    angle = end;
  });
  return slices.flatMap((slice) => (slice === undefined ? [] : [slice]));
}
