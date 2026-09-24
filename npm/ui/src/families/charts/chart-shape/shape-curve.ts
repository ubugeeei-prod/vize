// Curve interpolators ported from d3-shape 3 (ISC licensed): linear, monotoneX
// (Steffen 1990), step/stepBefore/stepAfter, and centripetal Catmull–Rom.

import type { PathContext } from "./shape-path.ts";

const curveEpsilon = 1e-12;

/** Streaming interpolator that turns points into path commands. */
export interface Curve {
  areaStart(): void;
  areaEnd(): void;
  lineStart(): void;
  lineEnd(): void;
  point(x: number, y: number): void;
}

/** Creates a curve that draws into a path context. */
export type CurveFactory = (context: PathContext) => Curve;

/** Built-in curve names accepted wherever a curve can be configured. */
export type CurveName = "catmullRom" | "linear" | "monotoneX" | "step" | "stepAfter" | "stepBefore";

/** Shared close-path bookkeeping: `line` is 0 inside an area, NaN after, undefined for lines. */
interface LineState {
  line: number;
  point: number;
}

function closeIfNeeded(state: LineState, context: PathContext): void {
  if (state.line || (state.line !== 0 && state.point === 1)) context.closePath();
  state.line = 1 - state.line;
}

/** Straight segments between points. */
export const curveLinear: CurveFactory = (context) => {
  const state: LineState = { line: Number.NaN, point: 0 };
  return {
    areaStart: () => {
      state.line = 0;
    },
    areaEnd: () => {
      state.line = Number.NaN;
    },
    lineStart: () => {
      state.point = 0;
    },
    lineEnd: () => closeIfNeeded(state, context),
    point: (x, y) => {
      if (state.point === 0) {
        state.point = 1;
        if (state.line) context.lineTo(x, y);
        else context.moveTo(x, y);
      } else {
        state.point = 2;
        context.lineTo(x, y);
      }
    },
  };
};

function sign(x: number): number {
  return x < 0 ? -1 : 1;
}

/** Cubic curve that preserves monotonicity in y for x-sorted data. */
export const curveMonotoneX: CurveFactory = (context) => {
  const state: LineState = { line: Number.NaN, point: 0 };
  let x0 = Number.NaN;
  let y0 = Number.NaN;
  let x1 = Number.NaN;
  let y1 = Number.NaN;
  let t0 = Number.NaN;

  const slope3 = (x2: number, y2: number): number => {
    const h0 = x1 - x0;
    const h1 = x2 - x1;
    const s0 = (y1 - y0) / (h0 || (h1 < 0 ? -0 : 0));
    const s1 = (y2 - y1) / (h1 || (h0 < 0 ? -0 : 0));
    const p = (s0 * h1 + s1 * h0) / (h0 + h1);
    return (sign(s0) + sign(s1)) * Math.min(Math.abs(s0), Math.abs(s1), 0.5 * Math.abs(p)) || 0;
  };
  const slope2 = (t: number): number => {
    const h = x1 - x0;
    return h ? ((3 * (y1 - y0)) / h - t) / 2 : t;
  };
  const hermite = (startTangent: number, endTangent: number): void => {
    const dx = (x1 - x0) / 3;
    context.bezierCurveTo(x0 + dx, y0 + dx * startTangent, x1 - dx, y1 - dx * endTangent, x1, y1);
  };

  return {
    areaStart: () => {
      state.line = 0;
    },
    areaEnd: () => {
      state.line = Number.NaN;
    },
    lineStart: () => {
      x0 = x1 = y0 = y1 = t0 = Number.NaN;
      state.point = 0;
    },
    lineEnd: () => {
      if (state.point === 2) context.lineTo(x1, y1);
      else if (state.point === 3) hermite(t0, slope2(t0));
      closeIfNeeded(state, context);
    },
    point: (x, y) => {
      let t1 = Number.NaN;
      if (x === x1 && y === y1) return;
      switch (state.point) {
        case 0:
          state.point = 1;
          if (state.line) context.lineTo(x, y);
          else context.moveTo(x, y);
          break;
        case 1:
          state.point = 2;
          break;
        case 2:
          state.point = 3;
          t1 = slope3(x, y);
          hermite(slope2(t1), t1);
          break;
        default:
          t1 = slope3(x, y);
          hermite(t0, t1);
          break;
      }
      x0 = x1;
      x1 = x;
      y0 = y1;
      y1 = y;
      t0 = t1;
    },
  };
};

function stepCurve(initialT: number): CurveFactory {
  return (context) => {
    const state: LineState = { line: Number.NaN, point: 0 };
    let t = initialT;
    let px = Number.NaN;
    let py = Number.NaN;
    return {
      areaStart: () => {
        state.line = 0;
      },
      areaEnd: () => {
        state.line = Number.NaN;
      },
      lineStart: () => {
        px = py = Number.NaN;
        state.point = 0;
      },
      lineEnd: () => {
        if (0 < t && t < 1 && state.point === 2) context.lineTo(px, py);
        if (state.line || (state.line !== 0 && state.point === 1)) context.closePath();
        if (state.line >= 0) {
          t = 1 - t;
          state.line = 1 - state.line;
        }
      },
      point: (x, y) => {
        if (state.point === 0) {
          state.point = 1;
          if (state.line) context.lineTo(x, y);
          else context.moveTo(x, y);
        } else {
          state.point = 2;
          if (t <= 0) {
            context.lineTo(px, y);
            context.lineTo(x, y);
          } else {
            const x1 = px * (1 - t) + x * t;
            context.lineTo(x1, py);
            context.lineTo(x1, y);
          }
        }
        px = x;
        py = y;
      },
    };
  };
}

/** Horizontal-then-vertical steps centered between points. */
export const curveStep: CurveFactory = stepCurve(0.5);

/** Vertical step at the start of each segment. */
export const curveStepBefore: CurveFactory = stepCurve(0);

/** Vertical step at the end of each segment. */
export const curveStepAfter: CurveFactory = stepCurve(1);

function cardinal(tension: number): CurveFactory {
  const k = (1 - tension) / 6;
  return (context) => {
    const state: LineState = { line: Number.NaN, point: 0 };
    let x0 = Number.NaN;
    let x1 = Number.NaN;
    let x2 = Number.NaN;
    let y0 = Number.NaN;
    let y1 = Number.NaN;
    let y2 = Number.NaN;
    const segment = (x: number, y: number) =>
      context.bezierCurveTo(
        x1 + k * (x2 - x0),
        y1 + k * (y2 - y0),
        x2 + k * (x1 - x),
        y2 + k * (y1 - y),
        x2,
        y2,
      );
    return {
      areaStart: () => {
        state.line = 0;
      },
      areaEnd: () => {
        state.line = Number.NaN;
      },
      lineStart: () => {
        x0 = x1 = x2 = y0 = y1 = y2 = Number.NaN;
        state.point = 0;
      },
      lineEnd: () => {
        if (state.point === 2) context.lineTo(x2, y2);
        else if (state.point === 3) segment(x1, y1);
        closeIfNeeded(state, context);
      },
      point: (x, y) => {
        switch (state.point) {
          case 0:
            state.point = 1;
            if (state.line) context.lineTo(x, y);
            else context.moveTo(x, y);
            break;
          case 1:
            state.point = 2;
            x1 = x;
            y1 = y;
            break;
          case 2:
            state.point = 3;
            segment(x, y);
            break;
          default:
            segment(x, y);
            break;
        }
        x0 = x1;
        x1 = x2;
        x2 = x;
        y0 = y1;
        y1 = y2;
        y2 = y;
      },
    };
  };
}

/**
 * Catmull–Rom spline through every point. `alpha` 0.5 (centripetal, the
 * default) avoids cusps and self-intersections; 0 is uniform, 1 chordal.
 */
export function curveCatmullRom(alpha = 0.5): CurveFactory {
  if (!alpha) return cardinal(0);
  return (context) => {
    const state: LineState = { line: Number.NaN, point: 0 };
    let x0 = Number.NaN;
    let x1 = Number.NaN;
    let x2 = Number.NaN;
    let y0 = Number.NaN;
    let y1 = Number.NaN;
    let y2 = Number.NaN;
    let l01a = 0;
    let l12a = 0;
    let l23a = 0;
    let l01_2a = 0;
    let l12_2a = 0;
    let l23_2a = 0;

    const segment = (x: number, y: number) => {
      let cx1 = x1;
      let cy1 = y1;
      let cx2 = x2;
      let cy2 = y2;
      if (l01a > curveEpsilon) {
        const a = 2 * l01_2a + 3 * l01a * l12a + l12_2a;
        const n = 3 * l01a * (l01a + l12a);
        cx1 = (cx1 * a - x0 * l12_2a + x2 * l01_2a) / n;
        cy1 = (cy1 * a - y0 * l12_2a + y2 * l01_2a) / n;
      }
      if (l23a > curveEpsilon) {
        const b = 2 * l23_2a + 3 * l23a * l12a + l12_2a;
        const m = 3 * l23a * (l23a + l12a);
        cx2 = (cx2 * b + x1 * l23_2a - x * l12_2a) / m;
        cy2 = (cy2 * b + y1 * l23_2a - y * l12_2a) / m;
      }
      context.bezierCurveTo(cx1, cy1, cx2, cy2, x2, y2);
    };

    const push = (x: number, y: number) => {
      if (state.point) {
        const x23 = x2 - x;
        const y23 = y2 - y;
        l23_2a = (x23 * x23 + y23 * y23) ** alpha;
        l23a = Math.sqrt(l23_2a);
      }
      switch (state.point) {
        case 0:
          state.point = 1;
          if (state.line) context.lineTo(x, y);
          else context.moveTo(x, y);
          break;
        case 1:
          state.point = 2;
          break;
        case 2:
          state.point = 3;
          segment(x, y);
          break;
        default:
          segment(x, y);
          break;
      }
      l01a = l12a;
      l12a = l23a;
      l01_2a = l12_2a;
      l12_2a = l23_2a;
      x0 = x1;
      x1 = x2;
      x2 = x;
      y0 = y1;
      y1 = y2;
      y2 = y;
    };

    return {
      areaStart: () => {
        state.line = 0;
      },
      areaEnd: () => {
        state.line = Number.NaN;
      },
      lineStart: () => {
        x0 = x1 = x2 = y0 = y1 = y2 = Number.NaN;
        l01a = l12a = l23a = l01_2a = l12_2a = l23_2a = 0;
        state.point = 0;
      },
      lineEnd: () => {
        if (state.point === 2) context.lineTo(x2, y2);
        else if (state.point === 3) push(x2, y2);
        closeIfNeeded(state, context);
      },
      point: push,
    };
  };
}

const namedCurves: Readonly<Record<CurveName, CurveFactory>> = Object.freeze({
  catmullRom: curveCatmullRom(0.5),
  linear: curveLinear,
  monotoneX: curveMonotoneX,
  step: curveStep,
  stepAfter: curveStepAfter,
  stepBefore: curveStepBefore,
});

/** Resolve a curve name or factory. */
export function resolveCurve(curve: CurveName | CurveFactory | undefined): CurveFactory {
  if (curve === undefined) return curveLinear;
  return typeof curve === "function" ? curve : namedCurves[curve];
}
