/** Element size in CSS pixels. */
export interface ResizableSize {
  /** Width in CSS pixels. */
  readonly width: number;

  /** Height in CSS pixels. */
  readonly height: number;
}

/** Physical edge or corner a handle resizes from. */
export type ResizablePhysicalEdge = "e" | "n" | "ne" | "nw" | "s" | "se" | "sw" | "w";

/**
 * Edge or corner a handle resizes from. `start` and `end` are logical inline
 * edges resolved through the root `dir`.
 */
export type ResizableEdge = ResizablePhysicalEdge | "end" | "start";

/** Reading direction used to resolve logical edges. */
export type ResizableDirection = "ltr" | "rtl";

/** Interaction state mirrored to `data-state`. */
export type ResizableState = "idle" | "resizing";

/** Input family that produced a resize. */
export type ResizableSource = "keyboard" | "pointer";

/** Immutable resize lifecycle payload. */
export interface ResizableResizeEvent {
  /** Size after this step. */
  readonly size: ResizableSize;

  /** Size when the interaction started. */
  readonly initialSize: ResizableSize;

  /** Physical edge that owns the interaction. */
  readonly edge: ResizablePhysicalEdge;

  /** Input family that produced the resize. */
  readonly source: ResizableSource;

  /** Native event responsible for this step, or `null` for manual cancellation. */
  readonly originalEvent: Event | null;
}

/** State exposed to Resizable slots. */
export interface ResizableSlotState {
  /** Current size. */
  readonly size: ResizableSize;

  /** Interaction state. */
  readonly state: ResizableState;

  /** Whether resizing is disabled. */
  readonly disabled: boolean;
}

/** Public instance exposed by ResizableRoot. */
export interface ResizableRootExpose extends ResizableSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Request a size (clamped to the constraints) and report whether it changed. */
  readonly setSize: (size: ResizableSize) => boolean;
}

/** Public instance exposed by ResizableHandle. */
export interface ResizableHandleExpose {
  /** Rendered separator element. */
  readonly element: HTMLDivElement | null;

  /** Physical edge resolved from `edge` and the root `dir`. */
  readonly physicalEdge: ResizablePhysicalEdge;

  /** Move focus to the handle. */
  readonly focus: (options?: FocusOptions) => void;
}
