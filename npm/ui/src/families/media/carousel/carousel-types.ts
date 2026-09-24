/** Scroll axis of the carousel track. */
export type CarouselOrientation = "horizontal" | "vertical";

/** Reading direction used for horizontal keys and scroll offsets. */
export type CarouselDirection = "ltr" | "rtl";

/**
 * Automatic rotation state mirrored through `data-autoplay`.
 *
 * - `playing`: rotation is on and advancing.
 * - `paused`: rotation is on but suspended by hover, focus, dragging, a hidden
 *   document, or `prefers-reduced-motion`.
 * - `stopped`: rotation is off until explicitly started.
 */
export type CarouselAutoplayState = "paused" | "playing" | "stopped";

/** How keyboard focus entering the carousel affects rotation. */
export type CarouselFocusBehavior = "none" | "pause" | "stop";

/** Why the active slide changed. */
export type CarouselChangeReason =
  | "api"
  | "autoplay"
  | "drag"
  | "indicator"
  | "keyboard"
  | "next"
  | "previous"
  | "scroll";

/** Why automatic rotation is suspended while `playing` intent is on. */
export type CarouselPauseReason = "dragging" | "focus" | "hidden" | "hover" | "reduced-motion";

/** State exposed by each slide through `data-state`. */
export type CarouselSlideState = "active" | "inactive";

/** State exposed to the CarouselRoot default slot and shared by every part. */
export interface CarouselSlotState {
  /** Zero-based active slide index. */
  readonly index: number;

  /** Total number of slides declared on the root. */
  readonly slideCount: number;

  /** Whether previous navigation is available. */
  readonly canScrollPrev: boolean;

  /** Whether next navigation is available. */
  readonly canScrollNext: boolean;

  /** Automatic rotation state. */
  readonly autoplay: CarouselAutoplayState;

  /** Whether a pointer drag is moving the track. */
  readonly dragging: boolean;

  /** Scroll axis. */
  readonly orientation: CarouselOrientation;
}

/** State exposed to CarouselSlide slots. */
export interface CarouselSlideSlotState {
  /** Zero-based slide index. */
  readonly index: number;

  /** Whether this slide is the active slide. */
  readonly active: boolean;

  /** Whether the slide is at least half visible; `null` until measured on the client. */
  readonly inView: boolean | null;

  /** Stable state token for styling and tests. */
  readonly state: CarouselSlideState;
}

/** State exposed to CarouselIndicator slots. */
export interface CarouselIndicatorSlotState {
  /** Zero-based slide index the indicator selects. */
  readonly index: number;

  /** Whether the indicator's slide is active. */
  readonly active: boolean;
}

/** Public instance exposed by CarouselRoot. */
export interface CarouselRootExpose extends CarouselSlotState {
  /** Rendered root element. */
  readonly element: HTMLElement | null;

  /** Activate one slide. Reports whether the active index changed. */
  readonly scrollTo: (index: number) => boolean;

  /** Activate the next slide (wrapping when `loop`). Reports whether the index changed. */
  readonly scrollNext: () => boolean;

  /** Activate the previous slide (wrapping when `loop`). Reports whether the index changed. */
  readonly scrollPrev: () => boolean;

  /** Turn automatic rotation on. Reports whether the intent changed. */
  readonly play: () => boolean;

  /** Turn automatic rotation off. Reports whether the intent changed. */
  readonly stop: () => boolean;
}

/** Public instance exposed by CarouselViewport. */
export interface CarouselViewportExpose {
  /** Rendered scroll container. */
  readonly element: HTMLDivElement | null;

  /** Deterministic id referenced by navigation controls. */
  readonly id: string;
}

/** Public instance exposed by CarouselSlide. */
export interface CarouselSlideExpose extends CarouselSlideSlotState {
  /** Rendered slide element. */
  readonly element: HTMLDivElement | null;

  /** Deterministic slide id. */
  readonly id: string;
}

/** Public instance exposed by navigation, indicator, and autoplay buttons. */
export interface CarouselButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Whether the button is disabled. */
  readonly disabled: boolean;
}

/** Public instance exposed by CarouselIndicatorGroup. */
export interface CarouselIndicatorGroupExpose {
  /** Rendered tablist element. */
  readonly element: HTMLDivElement | null;
}
