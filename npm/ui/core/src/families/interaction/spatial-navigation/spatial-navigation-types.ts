import type { MaybeRefOrGetter } from "vue";

import type {
  CollectionItem,
  CollectionKey,
  CollectionRegistry,
} from "../../foundations/collection/collection.ts";

/** Physical direction requested by spatial navigation. */
export type SpatialNavigationDirection = "down" | "left" | "right" | "up";

/** Candidate ranking behavior aligned with CSS Spatial Navigation Level 1. */
export type SpatialNavigationAlgorithm = "grid" | "normal";

/** Arrow-key behavior when no candidate exists in the requested direction. */
export type SpatialNavigationBoundaryBehavior = "contain" | "exit";

/** DOM focus policy after logical state changes. */
export type SpatialNavigationFocusBehavior = "focus" | "logical";

/** Serializable, axis-aligned CSS pixel rectangle. */
export interface SpatialNavigationRect {
  readonly bottom: number;
  readonly height: number;
  readonly left: number;
  readonly right: number;
  readonly top: number;
  readonly width: number;
}

/** Immutable successful navigation snapshot. */
export interface SpatialNavigationChange<Key extends CollectionKey> {
  readonly direction: SpatialNavigationDirection;
  readonly key: Key;
  readonly previousKey: Key | null;
  readonly originalEvent: Event | null;
  readonly algorithm: SpatialNavigationAlgorithm;
  readonly score: number;
}

/** Immutable boundary snapshot emitted when no directional candidate exists. */
export interface SpatialNavigationBoundary<Key extends CollectionKey> {
  readonly direction: SpatialNavigationDirection;
  readonly key: Key | null;
  readonly originalEvent: Event | null;
}

/** Spatial navigation configuration for one caller-owned collection. */
export interface SpatialNavigationOptions<Key extends CollectionKey, Value> {
  /** Ordered logical items and active-key ownership. */
  readonly registry: CollectionRegistry<Key, Value>;

  /**
   * Candidate scoring algorithm.
   *
   * @default "normal"
   */
  readonly algorithm?: MaybeRefOrGetter<SpatialNavigationAlgorithm | undefined>;

  /**
   * Whether a boundary arrow remains owned by the composite or exits to native behavior.
   *
   * @default "contain"
   */
  readonly boundaryBehavior?: MaybeRefOrGetter<SpatialNavigationBoundaryBehavior | undefined>;

  /** Suppress keyboard and imperative navigation without discarding active state. */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Wrap to the opposite spatial edge when no forward candidate exists.
   *
   * @default false
   */
  readonly loop?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Whether successful navigation moves DOM focus or only logical active state.
   *
   * @default "focus"
   */
  readonly focusBehavior?: MaybeRefOrGetter<SpatialNavigationFocusBehavior | undefined>;

  /**
   * Avoid browser scrolling while moving DOM focus. Reveal behavior still runs.
   *
   * @default false
   */
  readonly preventScroll?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Resolve transformed viewport geometry for DOM or virtual items.
   * The default reads `element.getBoundingClientRect()` and skips unmounted candidates.
   */
  readonly getRect?: (
    item: CollectionItem<Key, Value>,
  ) => SpatialNavigationRect | DOMRectReadOnly | null | undefined;

  /** Custom visibility policy for virtualized or scroll-managed collections. */
  readonly scrollIntoView?: (item: CollectionItem<Key, Value>, originalEvent: Event | null) => void;

  /** Called after logical state, DOM focus, and reveal behavior commit. */
  readonly onNavigate?: (change: SpatialNavigationChange<Key>) => void;

  /** Called when a valid origin has no candidate in the requested direction. */
  readonly onBoundary?: (boundary: SpatialNavigationBoundary<Key>) => void;
}

/** Stable event adapter for a spatial navigation container. */
export interface SpatialNavigationProps {
  readonly onKeydown: (event: KeyboardEvent) => void;
}

/** Geometry-driven navigation controller. */
export interface SpatialNavigationController<Key extends CollectionKey> {
  readonly spatialNavigationProps: Readonly<SpatialNavigationProps>;

  /** Resolve the best candidate without mutating logical or DOM state. */
  readonly findTarget: (direction: SpatialNavigationDirection, fromKey?: Key) => Key | null;

  /** Navigate, returning the committed key or `null` at a boundary. */
  readonly navigate: (
    direction: SpatialNavigationDirection,
    originalEvent?: Event | null,
  ) => Key | null;

  /** Release scope ownership. The caller-owned registry is not disposed. */
  readonly dispose: () => void;
}
