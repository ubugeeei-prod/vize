/** Headless, dependency-free, d3-compatible SVG path generators and chart layouts. */
export { arcCentroid, arcPath, pieLayout } from "./shape-arc.ts";
export type { ArcGeometry, PieOptions, PieSlice } from "./shape-arc.ts";
export { barRects, roundedRectPath } from "./shape-bar.ts";
export type {
  BarCategoryScale,
  BarCornerRadius,
  BarOptions,
  BarOrientation,
  BarRect,
} from "./shape-bar.ts";
export {
  curveCatmullRom,
  curveLinear,
  curveMonotoneX,
  curveStep,
  curveStepAfter,
  curveStepBefore,
  resolveCurve,
} from "./shape-curve.ts";
export type { Curve, CurveFactory, CurveName } from "./shape-curve.ts";
export { areaPath, linePath } from "./shape-line.ts";
export type { AreaOptions, LineOptions, ShapeAccessor } from "./shape-line.ts";
export { createPath } from "./shape-path.ts";
export type { PathBuilder, PathContext } from "./shape-path.ts";
export { stackLayout } from "./shape-stack.ts";
export type {
  StackOffset,
  StackOptions,
  StackOrder,
  StackPoint,
  StackSeries,
} from "./shape-stack.ts";
