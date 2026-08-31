import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue } from "vue";

import type {
  Point,
  PointerGraceController,
  PointerGraceOptions,
  Rect,
} from "./pointer-grace-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_POINTER_GRACE_OPTION";
const disposedDiagnostic = "VIZE_UI_POINTER_GRACE_DISPOSED";
const setupDiagnostic = "VIZE_UI_POINTER_GRACE_SETUP";

function readDelay(value: PointerGraceOptions["delay"]): number {
  const resolved = toValue(value);
  if (resolved === undefined) return 300;
  if (typeof resolved !== "number" || !Number.isFinite(resolved) || resolved < 0) {
    throw new TypeError(`${invalidOptionDiagnostic}: delay must resolve to a non-negative number`);
  }
  return resolved;
}

function validateOptions(options: PointerGraceOptions): void {
  if (typeof options.delay !== "function") readDelay(options.delay);
  if (options.onGraceEnd !== undefined && typeof options.onGraceEnd !== "function") {
    throw new TypeError(`${invalidOptionDiagnostic}: onGraceEnd must be a function`);
  }
}

function cornersOf(rect: Rect): readonly Point[] {
  return [
    { x: rect.x, y: rect.y },
    { x: rect.x + rect.width, y: rect.y },
    { x: rect.x + rect.width, y: rect.y + rect.height },
    { x: rect.x, y: rect.y + rect.height },
  ];
}

function pointInRect(point: Point, rect: Rect): boolean {
  return (
    point.x >= rect.x &&
    point.x <= rect.x + rect.width &&
    point.y >= rect.y &&
    point.y <= rect.y + rect.height
  );
}

function sign(origin: Point, left: Point, right: Point): number {
  return (origin.x - right.x) * (left.y - right.y) - (left.x - right.x) * (origin.y - right.y);
}

function pointInTriangle(point: Point, a: Point, b: Point, c: Point): boolean {
  const first = sign(point, a, b) < 0;
  const second = sign(point, b, c) < 0;
  const third = sign(point, c, a) < 0;
  return first === second && second === third;
}

/** Two target corners with the extreme angles from `origin`. */
export function extremeCorners(origin: Point, target: Rect): readonly [Point, Point] {
  const corners = cornersOf(target);
  let min = corners[0] as Point;
  let max = corners[0] as Point;
  let minAngle = Number.POSITIVE_INFINITY;
  let maxAngle = Number.NEGATIVE_INFINITY;
  for (const corner of corners) {
    const angle = Math.atan2(corner.y - origin.y, corner.x - origin.x);
    if (angle < minAngle) {
      minAngle = angle;
      min = corner;
    }
    if (angle > maxAngle) {
      maxAngle = angle;
      max = corner;
    }
  }
  return [min, max];
}

/** Vertices of the origin→target safe triangle plus the target box. */
export function gracePolygon(origin: Point, target: Rect): readonly Point[] {
  const [left, right] = extremeCorners(origin, target);
  return [origin, left, right];
}

/** Whether `point` is inside the target or the safe triangle to it. */
export function pointInGraceArea(point: Point, origin: Point, target: Rect): boolean {
  if (pointInRect(point, target)) return true;
  const [left, right] = extremeCorners(origin, target);
  return pointInTriangle(point, origin, left, right);
}

/** Create an SSR-safe pointer grace tracker. */
export function createPointerGrace(options: PointerGraceOptions = {}): PointerGraceController {
  validateOptions(options);
  const isPending = shallowRef(false);
  const polygon = shallowRef<readonly Point[] | null>(null);
  let disposed = false;
  let origin: Point | null = null;
  let target: Rect | null = null;
  let timer = 0;

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  const clearTimer = (): void => {
    if (timer === 0) return;
    globalThis.clearTimeout(timer);
    timer = 0;
    isPending.value = false;
  };

  const rebuild = (): void => {
    polygon.value = origin !== null && target !== null ? gracePolygon(origin, target) : null;
  };

  const finish = (): void => {
    timer = 0;
    isPending.value = false;
    options.onGraceEnd?.();
  };

  const arm = (): void => {
    clearTimer();
    const delay = readDelay(options.delay);
    isPending.value = true;
    if (delay === 0) {
      finish();
      return;
    }
    timer = globalThis.setTimeout(finish, delay) as unknown as number;
  };

  return Object.freeze({
    contains: (point: Point) => {
      assertAlive();
      if (target === null) return false;
      if (origin === null) return pointInRect(point, target);
      return pointInGraceArea(point, origin, target);
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      clearTimer();
      origin = null;
      target = null;
      polygon.value = null;
    },
    handleMove: (point: Point) => {
      assertAlive();
      if (target === null) return;
      const inside =
        origin === null ? pointInRect(point, target) : pointInGraceArea(point, origin, target);
      if (inside) clearTimer();
      else arm();
    },
    isPending: shallowReadonly(isPending),
    polygon: shallowReadonly(polygon),
    setOrigin: (point: Point | null) => {
      assertAlive();
      origin = point;
      rebuild();
    },
    setTarget: (rect: Rect | null) => {
      assertAlive();
      target = rect;
      rebuild();
    },
  });
}

/** Create a pointer-grace tracker disposed with the current Vue effect scope. */
export function usePointerGrace(options: PointerGraceOptions = {}): PointerGraceController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createPointerGrace(options);
  onScopeDispose(controller.dispose);
  return controller;
}
