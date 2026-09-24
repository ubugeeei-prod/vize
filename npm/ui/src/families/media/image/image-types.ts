/**
 * Loading lifecycle shared by every Image part through `data-status`.
 *
 * - `idle`: a deferred image is waiting to become visible before it requests a source.
 * - `loading`: a candidate source is attached and has not settled yet.
 * - `loaded`: the current candidate decoded successfully.
 * - `error`: every candidate failed, or no safe candidate was supplied.
 */
export type ImageStatus = "error" | "idle" | "loaded" | "loading";

/** Native image loading policies accepted by {@link ImageContent}. */
export type ImageLoading = "eager" | "lazy";

/** Native image decoding policies accepted by {@link ImageContent}. */
export type ImageDecoding = "async" | "auto" | "sync";

/** Native image fetch-priority hints accepted by {@link ImageContent}. */
export type ImageFetchPriority = "auto" | "high" | "low";

/** Native image CORS policies accepted by {@link ImageContent}. */
export type ImageCrossOrigin = "" | "anonymous" | "use-credentials";

/** Native image referrer policies accepted by {@link ImageContent}. */
export type ImageReferrerPolicy =
  | "no-referrer"
  | "no-referrer-when-downgrade"
  | "origin"
  | "origin-when-cross-origin"
  | "same-origin"
  | "strict-origin"
  | "strict-origin-when-cross-origin"
  | "unsafe-url";

/**
 * One image source or an ordered candidate chain.
 *
 * Candidates are tried in order; the next one is attached after the current one
 * fails. Unsafe or malformed candidates are skipped without being rendered.
 */
export type ImageSource = string | readonly string[];

/** Why {@link ImageRoot} moved to a new status. */
export type ImageStatusChangeReason = "error" | "load" | "reset" | "retry" | "source" | "visible";

/** State exposed to ImageRoot slots. */
export interface ImageSlotState {
  /** Current loading lifecycle state. */
  readonly status: ImageStatus;

  /** Source currently attached to the native image, or `undefined` while idle or failed. */
  readonly src: string | undefined;

  /** Zero-based position of the attached candidate in the candidate chain, or `-1`. */
  readonly candidateIndex: number;

  /** Number of safe candidates in the chain. */
  readonly candidateCount: number;
}

/** State exposed to ImageFallback and ImagePlaceholder slots. */
export type ImagePartSlotState = ImageSlotState;

/** Public instance exposed by ImageRoot. */
export interface ImageRootExpose extends ImageSlotState {
  /** Rendered root element. */
  readonly element: Element | null;

  /** Restart the candidate chain from the first safe source. Reports whether a retry started. */
  readonly retry: () => boolean;
}

/** Public instance exposed by ImageContent. */
export interface ImageContentExpose extends ImageSlotState {
  /** Rendered native image element, or `null` after every candidate failed. */
  readonly element: HTMLImageElement | null;
}

/** Public instance exposed by ImageFallback. */
export interface ImageFallbackExpose {
  /** Rendered fallback element while visible. */
  readonly element: HTMLSpanElement | null;

  /** Whether the fallback is rendered. */
  readonly visible: boolean;
}

/** Public instance exposed by ImagePlaceholder. */
export interface ImagePlaceholderExpose {
  /** Rendered placeholder element while visible. */
  readonly element: HTMLSpanElement | null;

  /** Whether the placeholder is rendered. */
  readonly visible: boolean;
}
