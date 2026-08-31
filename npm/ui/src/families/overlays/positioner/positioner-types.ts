import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Axis a floating element prefers relative to its reference. */
export type PlacementSide = "bottom" | "left" | "right" | "top";

/** Alignment along the cross axis. */
export type PlacementAlign = "center" | "end" | "start";

/**
 * Named placement used by the positioner.
 *
 * Spelled as an explicit union (rather than a template-literal cross of
 * side and align) so SFC prop-default checking can evaluate membership.
 */
export type Placement =
  | PlacementSide
  | "bottom-center"
  | "bottom-end"
  | "bottom-start"
  | "left-center"
  | "left-end"
  | "left-start"
  | "right-center"
  | "right-end"
  | "right-start"
  | "top-center"
  | "top-end"
  | "top-start";

/** How floating coordinates are published to CSS. */
export type PositionerStrategy = "absolute" | "fixed";

/** Axis-aligned box in viewport coordinates. */
export interface Rect {
  readonly height: number;
  readonly width: number;
  readonly x: number;
  readonly y: number;
}

/** Per-edge insets applied to the active viewport. */
export interface SafeAreaInsets {
  readonly bottom: number;
  readonly left: number;
  readonly right: number;
  readonly top: number;
}

/** Space a floating element may occupy at a resolved placement. */
export interface AvailableSize {
  readonly height: number;
  readonly width: number;
}

/** Inputs for the pure available-space measurement. */
export interface AvailableSizeInput {
  /**
   * Viewport padding the floating element should not cross.
   *
   * @default 0
   */
  readonly collisionPadding?: number;

  /**
   * Gap on the main axis between reference and floating.
   *
   * @default 0
   */
  readonly offset?: number;

  /** Placement after collision handling. */
  readonly placement: Placement;

  /** Reference box. */
  readonly reference: Rect;

  /** Visible viewport the floating element must stay inside. */
  readonly viewport: Rect;
}

/** Measurement source accepted in place of a live element. */
export interface VirtualElement {
  readonly getBoundingClientRect: () => Rect | DOMRect;
}

/** Reference or floating node the positioner can measure. */
export type PositionerElement = Element | VirtualElement;

/** Inputs for the pure collision solver. */
export interface ComputePositionInput {
  /**
   * Arrow box used to clamp the arrow along the facing edge.
   *
   * @default null
   */
  readonly arrow?: Rect | null;

  /**
   * Inset kept between the arrow and floating edges.
   *
   * @default 0
   */
  readonly arrowPadding?: number;

  /**
   * Viewport padding the floating element should not cross.
   *
   * @default 0
   */
  readonly collisionPadding?: number;

  /**
   * Flip to the opposite side when the preferred side overflows more.
   *
   * @default true
   */
  readonly flip?: boolean;

  /** Measured floating size. */
  readonly floating: Pick<Rect, "height" | "width">;

  /**
   * Hide when the reference no longer intersects the viewport.
   *
   * @default true
   */
  readonly hide?: boolean;

  /**
   * Gap on the main axis between reference and floating.
   *
   * @default 0
   */
  readonly offset?: number;

  /**
   * Preferred placement before collision handling.
   *
   * @default "bottom"
   */
  readonly placement?: Placement;

  /** Reference box. */
  readonly reference: Rect;

  /**
   * Mirror start/end on horizontal placements.
   *
   * @default false
   */
  readonly rtl?: boolean;

  /**
   * Shift the floating box back into the viewport after flip.
   *
   * @default true
   */
  readonly shift?: boolean;

  /** Visible viewport the floating element must stay inside. */
  readonly viewport: Rect;
}

/** Result of {@link computePosition}. */
export interface ComputePositionResult {
  readonly arrowX: number | null;
  readonly arrowY: number | null;
  readonly hidden: boolean;
  readonly overflow: Readonly<{
    readonly bottom: number;
    readonly left: number;
    readonly right: number;
    readonly top: number;
  }>;
  readonly placement: Placement;
  readonly x: number;
  readonly y: number;
}

/** Options shared by {@link createPositioner} and {@link usePositioner}. */
export interface PositionerOptions {
  /**
   * Inset kept between the arrow and floating edges.
   *
   * @default 0
   */
  readonly arrowPadding?: MaybeRefOrGetter<number | undefined>;

  /**
   * Viewport padding the floating element should not cross.
   *
   * @default 0
   */
  readonly collisionPadding?: MaybeRefOrGetter<number | undefined>;

  /**
   * Writing direction used to resolve start/end alignment.
   *
   * @default "ltr"
   */
  readonly direction?: MaybeRefOrGetter<"ltr" | "rtl" | undefined>;

  /**
   * Flip to the opposite side when the preferred side overflows more.
   *
   * @default true
   */
  readonly flip?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Hide when the reference no longer intersects the viewport.
   *
   * @default true
   */
  readonly hide?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Gap on the main axis between reference and floating.
   *
   * @default 0
   */
  readonly offset?: MaybeRefOrGetter<number | undefined>;

  /**
   * Preferred placement before collision handling.
   *
   * @default "bottom"
   */
  readonly placement?: MaybeRefOrGetter<Placement | undefined>;

  /**
   * Inset the active viewport by `env(safe-area-inset-*)` before collision
   * handling, keeping floating content clear of notches and rounded corners.
   *
   * @default false
   */
  readonly safeArea?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Shift the floating box back into the viewport after flip.
   *
   * @default true
   */
  readonly shift?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Constrain the floating host to the available space with
   * `max-width`/`max-height` and publish
   * `--vize-ui-positioner-available-width` and
   * `--vize-ui-positioner-available-height` custom properties.
   *
   * @default false
   */
  readonly size?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * CSS positioning mode published on the floating host.
   *
   * @default "fixed"
   */
  readonly strategy?: MaybeRefOrGetter<PositionerStrategy | undefined>;

  /**
   * Recalculate when the document or visual viewport resizes.
   *
   * @default true
   */
  readonly updateOnResize?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Recalculate while ancestors scroll.
   *
   * @default true
   */
  readonly updateOnScroll?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Viewport used for flip, shift, and hide. Defaults to the visual viewport.
   *
   * @default undefined
   */
  readonly viewport?: MaybeRefOrGetter<Rect | undefined>;
}

/** Inline style string spread onto the floating host. */
export type PositionerStyle = string;

/** Inline style string spread onto the arrow. */
export type PositionerArrowStyle = string;

/** Stateful collision-aware floating placement controller. */
export interface PositionerController {
  /** Arrow coordinates relative to the floating origin, when measured. */
  readonly arrowX: Readonly<ShallowRef<number | null>>;

  /** Available height at the resolved placement, once measured. */
  readonly availableHeight: Readonly<ShallowRef<number | null>>;

  /** Available width at the resolved placement, once measured. */
  readonly availableWidth: Readonly<ShallowRef<number | null>>;

  /** Arrow coordinates relative to the floating origin, when measured. */
  readonly arrowY: Readonly<ShallowRef<number | null>>;

  /** Stable inline styles for the arrow node. */
  readonly arrowStyle: Readonly<ShallowRef<PositionerArrowStyle>>;

  /** Release listeners and freeze the controller. */
  readonly dispose: () => void;

  /** Whether the reference is considered off-screen. */
  readonly hidden: Readonly<ShallowRef<boolean>>;

  /** Whether coordinates have been measured at least once. */
  readonly ready: Readonly<ShallowRef<boolean>>;

  /** Placement after flip. */
  readonly resolvedPlacement: Readonly<ShallowRef<Placement>>;

  /** Bind the arrow node used for clamping. */
  readonly setArrow: (element: PositionerElement | null) => void;

  /** Bind the floating node that occupies the positioned box. */
  readonly setFloating: (element: PositionerElement | null) => void;

  /** Bind the reference node or virtual element. */
  readonly setReference: (element: PositionerElement | null) => void;

  /** CSS strategy currently applied. */
  readonly strategy: Readonly<ShallowRef<PositionerStrategy>>;

  /** Stable inline styles for the floating host. */
  readonly style: Readonly<ShallowRef<PositionerStyle>>;

  /** Recompute from the latest measurements. */
  readonly update: () => void;

  /** Viewport-relative (or offset-parent-relative) x. */
  readonly x: Readonly<ShallowRef<number>>;

  /** Viewport-relative (or offset-parent-relative) y. */
  readonly y: Readonly<ShallowRef<number>>;
}
