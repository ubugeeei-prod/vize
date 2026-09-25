/** One captured pointer sample in drawing-surface (viewBox) coordinates. */
export interface SignaturePoint {
  /** Horizontal position in viewBox units. */
  readonly x: number;

  /** Vertical position in viewBox units. */
  readonly y: number;

  /** Normalized pressure from `0` to `1`; simulated from velocity for non-pen pointers. */
  readonly pressure: number;

  /** Milliseconds since the stroke started. */
  readonly time: number;
}

/** One continuous pen-down to pen-up stroke. */
export interface SignatureStroke {
  /** Samples in capture order. */
  readonly points: readonly SignaturePoint[];
}

/** Complete signature value: strokes in drawing order. */
export type SignatureValue = readonly SignatureStroke[];

/** Serialization used for the hidden form input. */
export type SignaturePadValueFormat = "json" | "svg";

/** When pointer pressure is simulated from drawing velocity. */
export type SignaturePadPressureMode = "auto" | "pointer" | "simulate";

/** State mirrored through `data-state`. */
export type SignaturePadState = "empty" | "filled";

/** Why the signature value changed. */
export type SignaturePadChangeReason = "api" | "clear" | "redo" | "stroke" | "undo";

/** Geometry options shared by rendering and export. */
export interface SignatureStrokeOptions {
  /**
   * Stroke diameter at full pressure, in viewBox units.
   *
   * @default 3
   */
  readonly size?: number;

  /**
   * How strongly pressure thins the stroke, from `0` (constant width) to `1`.
   *
   * @default 0.6
   */
  readonly thinning?: number;

  /**
   * Fraction of each segment smoothed into a quadratic curve, from `0` to `1`.
   *
   * @default 0.5
   */
  readonly smoothing?: number;
}

/** Options for {@link signatureToSvg}. */
export interface SignatureSvgOptions extends SignatureStrokeOptions {
  /** Drawing-surface width in viewBox units. */
  readonly width: number;

  /** Drawing-surface height in viewBox units. */
  readonly height: number;

  /**
   * Stroke fill color.
   *
   * @default "#000"
   */
  readonly color?: string;

  /**
   * Background fill. `undefined` keeps the background transparent.
   *
   * @default undefined
   */
  readonly background?: string;
}

/** Raster or vector type produced by {@link signatureToDataUrl}. */
export type SignatureImageType = "image/jpeg" | "image/png" | "image/svg+xml" | "image/webp";

/** Options for {@link signatureToDataUrl}. */
export interface SignatureDataUrlOptions extends SignatureSvgOptions {
  /**
   * Output type. `image/svg+xml` needs no canvas and also works on the server.
   *
   * @default "image/png"
   */
  readonly type?: SignatureImageType;

  /**
   * Device-pixel multiplier for raster output.
   *
   * @default 1
   */
  readonly scale?: number;

  /**
   * Encoder quality for lossy raster types, from `0` to `1`.
   *
   * @default undefined
   */
  readonly quality?: number;
}

/** Stable diagnostic codes thrown by signature helpers. */
export type SignaturePadErrorCode =
  | "VIZE_UI_SIGNATURE_PAD_CANVAS_UNAVAILABLE"
  | "VIZE_UI_SIGNATURE_PAD_INVALID_VALUE";

/** State exposed to every SignaturePad slot. */
export interface SignaturePadSlotState {
  /** Current strokes. */
  readonly value: SignatureValue;

  /** Whether no stroke has been drawn. */
  readonly empty: boolean;

  /** Whether a stroke is in progress. */
  readonly drawing: boolean;

  /** Whether drawing and editing are suppressed. */
  readonly disabled: boolean;

  /** Whether the value is shown but cannot change. */
  readonly readOnly: boolean;

  /** Whether an undo step is available. */
  readonly canUndo: boolean;

  /** Whether a redo step is available. */
  readonly canRedo: boolean;

  /** Stable state token for styling and tests. */
  readonly state: SignaturePadState;
}

/** Public instance exposed by SignaturePadRoot. */
export interface SignaturePadRootExpose extends SignaturePadSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Remove every stroke. Reports whether the value changed. */
  readonly clear: () => boolean;

  /** Restore the previous value. Reports whether a step was undone. */
  readonly undo: () => boolean;

  /** Re-apply the last undone value. Reports whether a step was redone. */
  readonly redo: () => boolean;

  /** Replace the value. Reports whether it changed. */
  readonly setValue: (value: SignatureValue) => boolean;

  /** Serialize the current value as standalone SVG markup. */
  readonly toSvg: (options?: Partial<SignatureSvgOptions>) => string;

  /** Export the current value as a data URL (client-only for raster types). */
  readonly toDataUrl: (options?: Partial<SignatureDataUrlOptions>) => Promise<string>;
}

/** Public instance exposed by SignaturePadCanvas. */
export interface SignaturePadCanvasExpose {
  /** Rendered drawing surface. */
  readonly element: SVGSVGElement | null;

  /** Whether a stroke is in progress. */
  readonly drawing: boolean;
}

/** Public instance exposed by SignaturePad buttons. */
export interface SignaturePadButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Whether the button is disabled. */
  readonly disabled: boolean;
}

/** Public instance exposed by SignaturePadGuide. */
export interface SignaturePadGuideExpose {
  /** Rendered guide element. */
  readonly element: HTMLDivElement | null;
}
