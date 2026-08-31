import { toValue } from "vue";

import type {
  SpatialNavigationAlgorithm,
  SpatialNavigationBoundaryBehavior,
  SpatialNavigationDirection,
  SpatialNavigationFocusBehavior,
  SpatialNavigationOptions,
  SpatialNavigationRect,
} from "./spatial-navigation-types.ts";
import type { CollectionItem, CollectionKey } from "../../foundations/collection/collection.ts";

const optionDiagnostic = "VIZE_UI_SPATIAL_NAVIGATION_OPTION";
const rectDiagnostic = "VIZE_UI_SPATIAL_NAVIGATION_RECT";

export interface RankedSpatialItem<Key extends CollectionKey, Value> {
  readonly item: CollectionItem<Key, Value>;
  readonly rect: SpatialNavigationRect;
  readonly index: number;
  readonly primary: number;
  readonly orthogonal: number;
  readonly overlap: number;
  readonly score: number;
}

export function capture(errors: unknown[], callback: () => void): void {
  try {
    callback();
  } catch (error) {
    errors.push(error);
  }
}

export function surfaceErrors(errors: readonly unknown[], message: string): void {
  if (errors.length === 1) throw errors[0];
  if (errors.length < 2) return;
  const Aggregate = globalThis.AggregateError as typeof AggregateError | undefined;
  if (typeof Aggregate === "function") throw new Aggregate(errors, message);
  const fallback = Object.assign(new Error(message), { errors: [...errors] });
  fallback.name = "AggregateError";
  throw fallback;
}

export function readBoolean(source: unknown, name: string): boolean {
  const value = toValue(source);
  if (value === undefined) return false;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return value;
}

function readEnum<Value extends string>(
  source: unknown,
  fallback: Value,
  values: readonly Value[],
  name: string,
): Value {
  const value = toValue(source) ?? fallback;
  if (!values.includes(value as Value)) {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to ${values.join(" or ")}`);
  }
  return value as Value;
}

export const readAlgorithm = (source: unknown): SpatialNavigationAlgorithm =>
  readEnum(source, "normal", ["normal", "grid"], "algorithm");

export const readBoundary = (source: unknown): SpatialNavigationBoundaryBehavior =>
  readEnum(source, "contain", ["contain", "exit"], "boundaryBehavior");

export const readFocus = (source: unknown): SpatialNavigationFocusBehavior =>
  readEnum(source, "focus", ["focus", "logical"], "focusBehavior");

export function readDirection(value: unknown): SpatialNavigationDirection {
  if (value !== "down" && value !== "left" && value !== "right" && value !== "up") {
    throw new TypeError(`${optionDiagnostic}: direction is invalid`);
  }
  return value;
}

export function validateOptions<Key extends CollectionKey, Value>(
  options: SpatialNavigationOptions<Key, Value>,
): void {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: options must be an object`);
  }
  const registry = options.registry as Partial<typeof options.registry> | null;
  if (!registry || typeof registry.getItem !== "function" || !registry.navigableItems) {
    throw new TypeError(`${optionDiagnostic}: registry must be a CollectionRegistry`);
  }
  for (const name of ["getRect", "scrollIntoView", "onNavigate", "onBoundary"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a function`);
    }
  }
  readAlgorithm(options.algorithm);
  readBoundary(options.boundaryBehavior);
  readFocus(options.focusBehavior);
  readBoolean(options.isDisabled, "isDisabled");
  readBoolean(options.loop, "loop");
  readBoolean(options.preventScroll, "preventScroll");
}

export function normalizeRect(value: unknown): SpatialNavigationRect {
  if (!value || typeof value !== "object") {
    throw new TypeError(`${rectDiagnostic}: geometry must be a rectangle object`);
  }
  const input = value as Partial<SpatialNavigationRect>;
  const numbers = [input.bottom, input.height, input.left, input.right, input.top, input.width];
  if (numbers.some((number) => typeof number !== "number" || !Number.isFinite(number))) {
    throw new TypeError(`${rectDiagnostic}: rectangle coordinates must be finite numbers`);
  }
  if (
    input.width! < 0 ||
    input.height! < 0 ||
    input.right! < input.left! ||
    input.bottom! < input.top!
  ) {
    throw new TypeError(`${rectDiagnostic}: rectangle dimensions must be non-negative and ordered`);
  }
  return Object.freeze({
    bottom: input.bottom!,
    height: input.height!,
    left: input.left!,
    right: input.right!,
    top: input.top!,
    width: input.width!,
  });
}

function intervalGap(startA: number, endA: number, startB: number, endB: number): number {
  return Math.max(0, startA - endB, startB - endA);
}

function intervalOverlap(startA: number, endA: number, startB: number, endB: number): number {
  return Math.max(0, Math.min(endA, endB) - Math.max(startA, startB));
}

function directionalGap(
  origin: SpatialNavigationRect,
  candidate: SpatialNavigationRect,
  direction: SpatialNavigationDirection,
): number | null {
  if (direction === "down")
    return candidate.top >= origin.bottom ? candidate.top - origin.bottom : null;
  if (direction === "up")
    return candidate.bottom <= origin.top ? origin.top - candidate.bottom : null;
  if (direction === "right")
    return candidate.left >= origin.right ? candidate.left - origin.right : null;
  return candidate.right <= origin.left ? origin.left - candidate.right : null;
}

export function rankItem<Key extends CollectionKey, Value>(
  item: CollectionItem<Key, Value>,
  rect: SpatialNavigationRect,
  index: number,
  origin: SpatialNavigationRect,
  direction: SpatialNavigationDirection,
): RankedSpatialItem<Key, Value> | null {
  const primary = directionalGap(origin, rect, direction);
  if (primary === null) return null;
  const horizontal = direction === "left" || direction === "right";
  const orthogonal = horizontal
    ? intervalGap(origin.top, origin.bottom, rect.top, rect.bottom)
    : intervalGap(origin.left, origin.right, rect.left, rect.right);
  const overlap = horizontal
    ? intervalOverlap(origin.top, origin.bottom, rect.top, rect.bottom)
    : intervalOverlap(origin.left, origin.right, rect.left, rect.right);
  const orthogonalSize = horizontal ? origin.height : origin.width;
  const orthogonalBias = orthogonalSize / 2;
  const orthogonalWeight = horizontal ? 30 : 2;
  const alignment = orthogonalSize > 0 ? (overlap / orthogonalSize) * 5 : 0;
  const score =
    Math.hypot(primary, orthogonal) + (orthogonal + orthogonalBias) * orthogonalWeight - alignment;
  return { item, rect, index, primary, orthogonal, overlap, score };
}

export function selectCandidate<Key extends CollectionKey, Value>(
  candidates: readonly RankedSpatialItem<Key, Value>[],
  algorithm: SpatialNavigationAlgorithm,
): RankedSpatialItem<Key, Value> | undefined {
  let ranked = [...candidates];
  if (algorithm === "grid" && ranked.some(({ overlap }) => overlap > 0)) {
    ranked = ranked.filter(({ overlap }) => overlap > 0);
  }
  ranked.sort((left, right) =>
    algorithm === "normal"
      ? left.score - right.score || left.index - right.index
      : left.primary - right.primary ||
        right.overlap - left.overlap ||
        left.orthogonal - right.orthogonal ||
        left.index - right.index,
  );
  return ranked[0];
}

export function selectWrappedCandidate<Key extends CollectionKey, Value>(
  items: readonly {
    item: CollectionItem<Key, Value>;
    rect: SpatialNavigationRect;
    index: number;
  }[],
  origin: SpatialNavigationRect,
  direction: SpatialNavigationDirection,
): RankedSpatialItem<Key, Value> | undefined {
  const horizontal = direction === "left" || direction === "right";
  const candidates = items.map(({ item, rect, index }) => {
    const orthogonal = horizontal
      ? intervalGap(origin.top, origin.bottom, rect.top, rect.bottom)
      : intervalGap(origin.left, origin.right, rect.left, rect.right);
    const overlap = horizontal
      ? intervalOverlap(origin.top, origin.bottom, rect.top, rect.bottom)
      : intervalOverlap(origin.left, origin.right, rect.left, rect.right);
    const primary =
      direction === "right"
        ? rect.left
        : direction === "left"
          ? -rect.right
          : direction === "down"
            ? rect.top
            : -rect.bottom;
    return { item, rect, index, primary, orthogonal, overlap, score: primary + orthogonal };
  });
  const aligned = candidates.some(({ overlap }) => overlap > 0)
    ? candidates.filter(({ overlap }) => overlap > 0)
    : candidates;
  aligned.sort(
    (left, right) =>
      left.primary - right.primary ||
      left.orthogonal - right.orthogonal ||
      right.overlap - left.overlap ||
      left.index - right.index,
  );
  return aligned[0];
}

export function keyDirection(event: KeyboardEvent): SpatialNavigationDirection | null {
  if (event.isComposing || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) {
    return null;
  }
  if (event.key === "ArrowDown") return "down";
  if (event.key === "ArrowLeft") return "left";
  if (event.key === "ArrowRight") return "right";
  if (event.key === "ArrowUp") return "up";
  return null;
}

export function isEditableDescendant(event: KeyboardEvent): boolean {
  const path = typeof event.composedPath === "function" ? event.composedPath() : [event.target];
  for (const candidate of path) {
    if (candidate === event.currentTarget) break;
    const target = candidate as Partial<HTMLElement> | null;
    if (
      target?.localName === "input" ||
      target?.localName === "select" ||
      target?.localName === "textarea" ||
      target?.isContentEditable === true
    )
      return true;
  }
  return false;
}
