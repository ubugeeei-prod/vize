// SVG path serialization ported from d3-path 3 (ISC licensed). Numbers are
// written with the same formatting and optional rounding, so generated `d`
// attributes are byte-identical to d3-shape's defaults.

const pi = Math.PI;
const tau = 2 * pi;
const epsilon = 1e-6;
const tauEpsilon = tau - epsilon;

/** Drawing commands consumed by curves and shape generators. */
export interface PathContext {
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  bezierCurveTo(x1: number, y1: number, x2: number, y2: number, x: number, y: number): void;
  arc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean,
  ): void;
  closePath(): void;
}

/** A path context that serializes to an SVG `d` string. */
export interface PathBuilder extends PathContext {
  toString(): string;
}

/**
 * Create an SVG path builder. `digits` rounds coordinates (d3-shape uses 3);
 * `null` keeps full precision.
 *
 * @throws RangeError for a negative or non-finite `digits`.
 */
export function createPath(digits: number | null = 3): PathBuilder {
  let text = "";
  let x0: number | null = null;
  let y0: number | null = null;
  let x1: number | null = null;
  let y1: number | null = null;
  let format = (value: number): string => String(value);
  if (digits !== null) {
    const d = Math.floor(digits);
    if (!(d >= 0)) throw new RangeError(`VIZE_UI_PATH_DIGITS: invalid digits ${digits}`);
    if (d <= 15) {
      const k = 10 ** d;
      format = (value) => String(Math.round(value * k) / k);
    }
  }
  const n = (value: number) => format(value);

  return {
    moveTo(x, y) {
      x0 = x1 = +x;
      y0 = y1 = +y;
      text += `M${n(x0)},${n(y0)}`;
    },
    closePath() {
      if (x1 !== null) {
        x1 = x0;
        y1 = y0;
        text += "Z";
      }
    },
    lineTo(x, y) {
      x1 = +x;
      y1 = +y;
      text += `L${n(x1)},${n(y1)}`;
    },
    bezierCurveTo(cx1, cy1, cx2, cy2, x, y) {
      x1 = +x;
      y1 = +y;
      text += `C${n(+cx1)},${n(+cy1)},${n(+cx2)},${n(+cy2)},${n(x1)},${n(y1)}`;
    },
    arc(x, y, r, a0, a1, ccwInput = false) {
      const ccw = Boolean(ccwInput);
      if (r < 0) throw new RangeError(`VIZE_UI_PATH_RADIUS: negative radius ${r}`);
      const dx = r * Math.cos(a0);
      const dy = r * Math.sin(a0);
      const startX = x + dx;
      const startY = y + dy;
      const cw = ccw ? 0 : 1;
      let da = ccw ? a0 - a1 : a1 - a0;
      if (x1 === null || y1 === null) {
        text += `M${n(startX)},${n(startY)}`;
      } else if (Math.abs(x1 - startX) > epsilon || Math.abs(y1 - startY) > epsilon) {
        text += `L${n(startX)},${n(startY)}`;
      }
      if (!r) return;
      if (da < 0) da = (da % tau) + tau;
      if (da > tauEpsilon) {
        x1 = startX;
        y1 = startY;
        text += `A${n(r)},${n(r)},0,1,${cw},${n(x - dx)},${n(y - dy)}A${n(r)},${n(r)},0,1,${cw},${n(x1)},${n(y1)}`;
      } else if (da > epsilon) {
        x1 = x + r * Math.cos(a1);
        y1 = y + r * Math.sin(a1);
        text += `A${n(r)},${n(r)},0,${da >= pi ? 1 : 0},${cw},${n(x1)},${n(y1)}`;
      }
    },
    toString() {
      return text;
    },
  };
}
