import type { ComponentPublicInstance } from "vue";

/** Accessibility state derived from Skeleton props. */
export type SkeletonAriaState = "decorative" | "status";

/** Visibility and loading state mirrored to `data-state`. */
export type SkeletonState = "hidden" | "loaded" | "loading";

/** Rendered value exposed by {@link Skeleton}. */
export type SkeletonElement = Element | ComponentPublicInstance;

/** State exposed to the default Skeleton slot. */
export interface SkeletonSlotState {
  /** Whether the placeholder represents work still loading. */
  readonly loading: boolean;

  /** Whether the placeholder remains visible in layout. */
  readonly visible: boolean;

  /** Stable state token for styling and tests. */
  readonly state: SkeletonState;

  /** Whether accessibility semantics are decorative or status-like. */
  readonly ariaState: SkeletonAriaState;
}

/** Public component instance exposed by the Skeleton primitive. */
export interface SkeletonExpose extends SkeletonSlotState {
  /** Rendered host element or component instance. */
  readonly element: SkeletonElement | null;
}

/** Inline style hooks applied to the rendered host. */
export interface SkeletonStyle {
  /** Consumer-overridable block size hook for placeholder styling. */
  readonly "--vize-ui-skeleton-block-size": string;

  /** Consumer-overridable inline size hook for placeholder styling. */
  readonly "--vize-ui-skeleton-inline-size": string;
}
