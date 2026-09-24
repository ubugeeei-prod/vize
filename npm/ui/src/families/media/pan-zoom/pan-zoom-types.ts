/** Content-to-viewport transform: `viewportPoint = contentPoint * scale + (x, y)`. */
export interface PanZoomTransform {
  /** Horizontal translation in viewport pixels. */
  readonly x: number;

  /** Vertical translation in viewport pixels. */
  readonly y: number;

  /** Uniform zoom factor; `1` renders content at its natural size. */
  readonly scale: number;
}

/** A point in viewport pixels, relative to the viewport's top-left corner. */
export interface PanZoomPoint {
  readonly x: number;
  readonly y: number;
}

/** A width and height in CSS pixels. */
export interface PanZoomSize {
  readonly width: number;
  readonly height: number;
}

/** A rectangle in content pixels. */
export interface PanZoomRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/**
 * How far content may be panned and zoomed.
 *
 * - `none`: unconstrained.
 * - `contain`: an axis where the scaled content is smaller than the viewport is
 *   centered; a larger axis may not reveal empty space.
 * - `cover`: the scale never drops below the scale that covers the viewport, and
 *   no empty space is revealed on either axis.
 * - a {@link PanZoomRect}: the viewport center stays inside that content region.
 */
export type PanZoomBounds = "contain" | "cover" | "none" | PanZoomRect;

/** How wheel input is interpreted. Trackpad pinches (`ctrlKey` wheels) always zoom. */
export type PanZoomWheelMode = "pan" | "zoom" | "zoom-with-ctrl";

/** What produced a transform change. */
export type PanZoomChangeSource =
  | "api"
  | "button"
  | "double-click"
  | "keyboard"
  | "pinch"
  | "pointer"
  | "wheel";

/** Interaction state mirrored through `data-state`. */
export type PanZoomState = "disabled" | "idle" | "panning" | "pinching";

/** Localizable strings. Omitted entries use English defaults. */
export interface PanZoomMessages {
  /** `aria-roledescription` of the viewport. @default "pan and zoom area" */
  readonly roleDescription?: string;

  /** Zoom-in button fallback label. @default "Zoom in" */
  readonly zoomIn?: string;

  /** Zoom-out button fallback label. @default "Zoom out" */
  readonly zoomOut?: string;

  /** Reset button fallback label. @default "Reset zoom" */
  readonly reset?: string;

  /** Fit button fallback label. @default "Fit to view" */
  readonly fit?: string;

  /** Settled zoom announcement from a rounded percentage. @default `${percent}%` */
  readonly zoomLevel?: (percent: number) => string;
}

/** State shared with every PanZoom slot. */
export interface PanZoomSlotState {
  /** Current transform. */
  readonly transform: PanZoomTransform;

  /** Current zoom factor. */
  readonly scale: number;

  /** Current interaction state. */
  readonly state: PanZoomState;

  /** Whether zooming in is possible. */
  readonly canZoomIn: boolean;

  /** Whether zooming out is possible. */
  readonly canZoomOut: boolean;
}

/** Public instance exposed by PanZoomRoot. */
export interface PanZoomRootExpose extends PanZoomSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Request a transform (constrained by scale limits and bounds). Reports a change. */
  readonly setTransform: (transform: PanZoomTransform) => boolean;

  /** Zoom to `scale` around a viewport point (default: the viewport center). */
  readonly zoomTo: (scale: number, point?: PanZoomPoint) => boolean;

  /** Zoom in by one `zoomStep` around a viewport point (default: center). */
  readonly zoomIn: (point?: PanZoomPoint) => boolean;

  /** Zoom out by one `zoomStep` around a viewport point (default: center). */
  readonly zoomOut: (point?: PanZoomPoint) => boolean;

  /** Translate by a viewport-pixel delta. */
  readonly panBy: (dx: number, dy: number) => boolean;

  /** Restore `defaultValue`. */
  readonly reset: () => boolean;

  /** Scale and center the content inside the viewport. */
  readonly fit: () => boolean;
}

/** Public instance exposed by PanZoomViewport. */
export interface PanZoomViewportExpose {
  /** Rendered viewport element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by PanZoomContent. */
export interface PanZoomContentExpose {
  /** Rendered content element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by PanZoom control buttons. */
export interface PanZoomButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Whether the control is disabled. */
  readonly disabled: boolean;
}

/** Public instance exposed by PanZoomStatus. */
export interface PanZoomStatusExpose {
  /** Rendered live region. */
  readonly element: HTMLDivElement | null;

  /** Announced text. */
  readonly text: string;
}
