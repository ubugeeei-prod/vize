import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Consumer styling tones mirrored by {@link EmptyState} through `data-tone`. */
export type EmptyStateTone = "danger" | "info" | "neutral" | "success" | "warning";

/** Density tokens mirrored by {@link EmptyState} through `data-density`. */
export type EmptyStateDensity = "compact" | "comfortable";

/** Layout orientation tokens mirrored by {@link EmptyState} through `data-orientation`. */
export type EmptyStateOrientation = "block" | "inline";

/** Stable state token mirrored by {@link EmptyState} through `data-state`. */
export type EmptyStateState = "empty";

/** Rendered value exposed by {@link EmptyState}. */
export type EmptyStateElement = PrimitiveElement;

/** State exposed to the default EmptyState slot. */
export interface EmptyStateSlotState {
  /** Consumer styling tone mirrored to `data-tone`. */
  readonly tone: EmptyStateTone;

  /** Density token mirrored to `data-density`. */
  readonly density: EmptyStateDensity;

  /** Layout orientation mirrored to `data-orientation`. */
  readonly orientation: EmptyStateOrientation;

  /** Stable empty-state token mirrored to `data-state`. */
  readonly state: EmptyStateState;
}

/** Public component instance state exposed by the EmptyState primitive. */
export interface EmptyStateExpose extends EmptyStateSlotState {
  /** Rendered host element or component instance. */
  readonly element: EmptyStateElement | null;
}
