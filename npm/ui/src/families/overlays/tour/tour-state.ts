import { toValue } from "vue";

import type { PlacementAlign, PlacementSide } from "../positioner/positioner.ts";
import type {
  TourContentPlacement,
  TourSpotlightRect,
  TourStepDefinition,
  TourTarget,
} from "./tour-types.ts";

/** Box accepted by {@link padTourRect}; `DOMRect` satisfies it. */
export interface TourRectLike {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/**
 * Resolve a step target on the client.
 *
 * Selectors are looked up in `root` and invalid selectors resolve to `null`
 * instead of throwing, so a typo degrades to the missing-target behavior.
 */
export function resolveTourTarget(
  target: TourTarget | undefined,
  root: ParentNode | null,
): Element | null {
  if (target === undefined) return null;
  if (typeof target === "string") {
    if (root === null || target.trim().length === 0) return null;
    try {
      return root.querySelector(target);
    } catch {
      return null;
    }
  }
  const value = toValue(target);
  return value instanceof Element ? value : null;
}

/** Enabled steps in declaration order. */
export function enabledTourSteps<Step extends TourStepDefinition>(
  steps: readonly Step[],
): readonly Step[] {
  return steps.filter((step) => step.disabled !== true);
}

/**
 * Find the first step at or after `start` in `direction` accepted by `canEnter`.
 *
 * Returns `null` when the walk leaves the sequence; tours never wrap.
 */
export function findTourStep<Step extends TourStepDefinition>(
  steps: readonly Step[],
  start: number,
  direction: 1 | -1,
  canEnter: (step: Step) => boolean,
): Step | null {
  for (let index = start; index >= 0 && index < steps.length; index += direction) {
    const step = steps[index];
    if (step !== undefined && canEnter(step)) return step;
  }
  return null;
}

/** Grow a measured box by `padding` on every side, clamping the size at zero. */
export function padTourRect(rect: TourRectLike, padding: number): TourSpotlightRect {
  const inset = Number.isFinite(padding) ? padding : 0;
  return {
    x: rect.x - inset,
    y: rect.y - inset,
    width: Math.max(0, rect.width + inset * 2),
    height: Math.max(0, rect.height + inset * 2),
  };
}

/** Side token for a content placement. */
export function tourSideFromPlacement(value: TourContentPlacement): PlacementSide | "center" {
  if (value === "center") return "center";
  const [side] = value.split("-", 1);
  return side === "top" || side === "right" || side === "left" ? side : "bottom";
}

/** Alignment token for a content placement. */
export function tourAlignFromPlacement(value: TourContentPlacement): PlacementAlign {
  if (value === "center") return "center";
  const [, align] = value.split("-");
  return align === "start" || align === "end" ? align : "center";
}

/** Whether a keyboard event originates in a control that owns arrow keys. */
export function isTourEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}
