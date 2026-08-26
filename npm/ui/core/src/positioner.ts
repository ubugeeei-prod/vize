export { computePosition, parsePlacement, readRect } from "./positioner-geometry.ts";
export { createPositioner, usePositioner } from "./positioner-runtime.ts";

/** Accessible, unstyled floating host with collision-aware placement. */
export { default as Positioner } from "./positioner.vue";

/** Arrow aligned to the facing edge of {@link Positioner}. */
export { default as PositionerArrow } from "./positioner-arrow.vue";

export type {
  ComputePositionInput,
  ComputePositionResult,
  Placement,
  PlacementAlign,
  PlacementSide,
  PositionerArrowStyle,
  PositionerController,
  PositionerElement,
  PositionerOptions,
  PositionerStrategy,
  PositionerStyle,
  Rect,
  VirtualElement,
} from "./positioner-types.ts";
