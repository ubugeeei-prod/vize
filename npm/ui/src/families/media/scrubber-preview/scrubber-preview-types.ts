/** Where preview images come from. */
export type ScrubberPreviewKind = "capture" | "none" | "sprite" | "vtt";

/** Thumbnail availability for the current preview time. */
export type ScrubberPreviewStatus = "error" | "idle" | "loading" | "ready";

/** Reading direction used to map pointer positions to time. */
export type ScrubberPreviewDirection = "ltr" | "rtl";

/** Typed failure codes thrown by {@link captureVideoFrame} and reported by thumbnails. */
export type ScrubberPreviewErrorCode =
  | "VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED"
  | "VIZE_UI_SCRUBBER_PREVIEW_LOAD_FAILED"
  | "VIZE_UI_SCRUBBER_PREVIEW_TAINTED"
  | "VIZE_UI_SCRUBBER_PREVIEW_UNSUPPORTED";

/**
 * Sprite-sheet thumbnails: frames of `width` x `height` pixels laid out in
 * `columns` x `rows` grids, one frame every `interval` seconds. When a sheet is
 * full the next frame starts sheet `n + 1`; pass a function to name each sheet.
 */
export interface ScrubberPreviewSprite {
  /** Sheet image URL, or a function from zero-based sheet index to URL. */
  readonly src: string | ((sheet: number) => string);

  /** Frames per sheet row. */
  readonly columns: number;

  /** Frame rows per sheet. */
  readonly rows: number;

  /** Seconds between frames. */
  readonly interval: number;

  /** Frame width in CSS pixels. */
  readonly width: number;

  /** Frame height in CSS pixels. */
  readonly height: number;

  /** Total frames across all sheets; later times clamp to the last frame. */
  readonly frameCount?: number;
}

/** One WebVTT thumbnail cue. */
export interface ScrubberPreviewCue {
  /** Cue start in seconds (inclusive). */
  readonly start: number;

  /** Cue end in seconds (exclusive). */
  readonly end: number;

  /** Image URL without the media fragment. */
  readonly src: string;

  /** Sprite region from a `#xywh=` fragment, or `null` for a whole image. */
  readonly region: ScrubberPreviewRegion | null;
}

/** A rectangle inside an image, in pixels. */
export interface ScrubberPreviewRegion {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** Resolved thumbnail frame for one time. */
export interface ScrubberPreviewFrame {
  /** Image URL (sprite sheet, VTT image, or captured object URL). */
  readonly src: string;

  /** Region inside `src`, or `null` for a whole image. */
  readonly region: ScrubberPreviewRegion | null;
}

/** State exposed to ScrubberPreviewRoot, Track, and Time slots. */
export interface ScrubberPreviewSlotState {
  /** Previewed time in seconds, or `null` while no preview is shown. */
  readonly time: number | null;

  /** Previewed position `0..1` along the track, or `null`. */
  readonly ratio: number | null;

  /** Whether a preview is shown. */
  readonly active: boolean;

  /** Media duration in seconds. */
  readonly duration: number;
}

/** State exposed to ScrubberPreviewThumbnail slots. */
export interface ScrubberPreviewThumbnailSlotState {
  /** Source of preview images. */
  readonly kind: ScrubberPreviewKind;

  /** Availability of the current frame. */
  readonly status: ScrubberPreviewStatus;

  /** Resolved frame, or `null`. */
  readonly frame: ScrubberPreviewFrame | null;

  /** Previewed time in seconds, or `null`. */
  readonly time: number | null;

  /** Failure code after an error, or `null`. */
  readonly error: ScrubberPreviewErrorCode | null;
}

/** State exposed to ScrubberPreviewTime slots. */
export interface ScrubberPreviewTimeSlotState {
  /** Previewed time in seconds, or `null`. */
  readonly time: number | null;

  /** Formatted time, e.g. `"1:05"` or `"1:02:03"`; empty while inactive. */
  readonly text: string;
}

/** Public instance exposed by ScrubberPreviewRoot. */
export interface ScrubberPreviewRootExpose extends ScrubberPreviewSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Show a preview for a time (e.g. while a seek slider has keyboard focus), or hide it. */
  readonly setTime: (time: number | null) => boolean;
}

/** Public instance exposed by ScrubberPreviewTrack. */
export interface ScrubberPreviewTrackExpose {
  /** Rendered track element. */
  readonly element: HTMLDivElement | null;

  /** Whether a pointer is scrubbing (pressed) on the track. */
  readonly scrubbing: boolean;
}

/** Public instance exposed by ScrubberPreviewThumbnail. */
export interface ScrubberPreviewThumbnailExpose extends ScrubberPreviewThumbnailSlotState {
  /** Rendered thumbnail element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by ScrubberPreviewTime. */
export interface ScrubberPreviewTimeExpose extends ScrubberPreviewTimeSlotState {
  /** Rendered time element. */
  readonly element: HTMLSpanElement | null;
}

/** Options for {@link captureVideoFrame}. */
export interface CaptureVideoFrameOptions {
  /**
   * Output width in pixels; height keeps the video aspect ratio unless given.
   *
   * @default video.videoWidth
   */
  readonly width?: number;

  /**
   * Output height in pixels.
   *
   * @default derived from width
   */
  readonly height?: number;

  /**
   * Image MIME type.
   *
   * @default "image/jpeg"
   */
  readonly type?: string;

  /**
   * Lossy encoder quality, `0..1`.
   *
   * @default 0.8
   */
  readonly quality?: number;
}
