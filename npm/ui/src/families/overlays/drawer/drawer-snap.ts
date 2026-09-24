import type { DrawerSide, DrawerSnapPoint } from "./drawer-types.ts";

const snapPointDiagnostic = "VIZE_UI_DRAWER_SNAP_POINT";
const pixelPattern = /^(\d+(?:\.\d+)?)px$/u;

/** Milliseconds of release velocity projected forward when choosing a snap point. */
export const drawerVelocityProjection = 180;

/** Whether a value is a valid {@link DrawerSnapPoint}. */
export function isDrawerSnapPoint(value: unknown): value is DrawerSnapPoint {
  if (typeof value === "number") return Number.isFinite(value) && value > 0 && value <= 1;
  return typeof value === "string" && pixelPattern.test(value);
}

/** Throw a stable diagnostic when a snap point list contains an invalid entry. */
export function assertDrawerSnapPoints(values: readonly DrawerSnapPoint[]): void {
  for (const value of values) {
    if (!isDrawerSnapPoint(value)) {
      throw new TypeError(
        `${snapPointDiagnostic}: snap points must be fractions in (0, 1] or "<number>px" strings`,
      );
    }
  }
}

/** Visible size in px for one snap point, clamped to the drawer size. */
export function drawerSnapVisibleSize(snapPoint: DrawerSnapPoint, size: number): number {
  if (typeof snapPoint === "number") return Math.min(size, Math.max(0, snapPoint * size));
  const match = pixelPattern.exec(snapPoint);
  const pixels = match?.[1] === undefined ? 0 : Number(match[1]);
  return Math.min(size, Math.max(0, pixels));
}

/** Offset in px toward the dismiss direction that leaves `snapPoint` visible. */
export function drawerSnapOffset(snapPoint: DrawerSnapPoint | null, size: number): number {
  if (snapPoint === null || size <= 0) return 0;
  return size - drawerSnapVisibleSize(snapPoint, size);
}

/** Whether two snap points are the same resting position token. */
export function drawerSnapPointEquals(
  left: DrawerSnapPoint | null,
  right: DrawerSnapPoint | null,
): boolean {
  return Object.is(left, right);
}

/** Main drag axis for a side. */
export function drawerAxis(side: DrawerSide): "x" | "y" {
  return side === "left" || side === "right" ? "x" : "y";
}

/** +1 when the dismiss direction follows the positive axis, otherwise -1. */
export function drawerDismissSign(side: DrawerSide): 1 | -1 {
  return side === "bottom" || side === "right" ? 1 : -1;
}

/** Inputs for {@link resolveDrawerRelease}. */
export interface DrawerReleaseInput {
  /** Snap points in consumer order; empty means a single fully open position. */
  readonly snapPoints: readonly DrawerSnapPoint[];

  /** Drawer size along the drag axis in px. */
  readonly size: number;

  /** Offset in px of the snap point the drag started from. */
  readonly startOffset: number;

  /** Signed travel in px toward the dismiss direction. */
  readonly distance: number;

  /** Release velocity in px/ms toward the dismiss direction. */
  readonly velocity: number;

  /** Whether drag gestures may dismiss the drawer. */
  readonly dismissible: boolean;

  /** Fraction of the remaining visible size past the lowest snap that dismisses. */
  readonly closeThreshold: number;

  /** Velocity in px/ms toward the dismiss direction that flicks the drawer closed. */
  readonly velocityThreshold: number;
}

/** Result of {@link resolveDrawerRelease}. */
export type DrawerReleaseResult =
  | { readonly outcome: "dismiss" }
  | { readonly outcome: "snap"; readonly snapPoint: DrawerSnapPoint | null };

/**
 * Pick the resting position after a drag is released.
 *
 * Dismissal wins when the drawer travelled past the lowest snap point by more
 * than `closeThreshold` of what remained visible there, or when it was flicked
 * toward the dismiss direction faster than `velocityThreshold` while already
 * past the lowest snap point. Otherwise the release position is projected by
 * {@link drawerVelocityProjection} ms of velocity and the nearest snap point wins.
 */
export function resolveDrawerRelease(input: DrawerReleaseInput): DrawerReleaseResult {
  const points = input.snapPoints;
  const offsets = points.map((point) => drawerSnapOffset(point, input.size));
  const lowestOffset = offsets.length === 0 ? 0 : Math.max(...offsets);
  const current = input.startOffset + input.distance;
  const remaining = Math.max(1, input.size - lowestOffset);
  if (input.dismissible && current > lowestOffset) {
    const pastLowest = current - lowestOffset;
    if (pastLowest > remaining * input.closeThreshold) return { outcome: "dismiss" };
    if (input.velocity > input.velocityThreshold) return { outcome: "dismiss" };
  }
  if (points.length === 0) return { outcome: "snap", snapPoint: null };
  const projected = current + input.velocity * drawerVelocityProjection;
  let bestIndex = 0;
  let bestDistance = Number.POSITIVE_INFINITY;
  offsets.forEach((offset, index) => {
    const distance = Math.abs(offset - projected);
    if (distance < bestDistance) {
      bestDistance = distance;
      bestIndex = index;
    }
  });
  return { outcome: "snap", snapPoint: points[bestIndex] ?? null };
}

/**
 * Next snap point for a keyboard or click step.
 *
 * `direction` `1` moves toward a more visible snap point, `-1` toward a less
 * visible one; `wrap` cycles past either end. Visibility is compared through a
 * nominal size so pixel and fractional snap points order consistently.
 */
export function stepDrawerSnapPoint(
  snapPoints: readonly DrawerSnapPoint[],
  current: DrawerSnapPoint | null,
  direction: 1 | -1,
  size: number,
  wrap: boolean,
): DrawerSnapPoint | null {
  if (snapPoints.length === 0) return null;
  const nominal = size > 0 ? size : 1000;
  const ordered = [...snapPoints].sort(
    (left, right) => drawerSnapVisibleSize(left, nominal) - drawerSnapVisibleSize(right, nominal),
  );
  const index = current === null ? -1 : ordered.findIndex((point) => Object.is(point, current));
  let next = index + direction;
  if (index === -1) next = direction === 1 ? ordered.length - 1 : 0;
  if (next >= ordered.length) next = wrap ? 0 : ordered.length - 1;
  if (next < 0) next = wrap ? ordered.length - 1 : 0;
  return ordered[next] ?? null;
}
