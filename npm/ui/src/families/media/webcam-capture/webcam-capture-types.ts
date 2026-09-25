/**
 * Lifecycle of the camera stream, identical to `MediaStreamStatus` from
 * `@vizejs/composable` so either side can drive the other.
 */
export type WebcamCaptureStatus = "active" | "error" | "idle" | "requesting";

/**
 * Normalized acquisition failure, identical to `MediaStreamErrorCode` from
 * `@vizejs/composable`.
 */
export type WebcamCaptureErrorCode =
  | "aborted"
  | "failed"
  | "invalid-constraints"
  | "not-found"
  | "not-readable"
  | "overconstrained"
  | "permission-denied"
  | "unsupported";

/** Camera facing direction requested through `facingMode`. */
export type WebcamFacingMode = "environment" | "user";

/** Encoded image types accepted by {@link captureFrame}. */
export type WebcamCaptureImageType = "image/jpeg" | "image/png" | "image/webp";

/** Acquisition failure with the exact thrown value. */
export interface WebcamCaptureFailure {
  /** Normalized reason. */
  readonly code: WebcamCaptureErrorCode;

  /** Value thrown by `getUserMedia`, or `undefined` when the API is missing. */
  readonly cause: unknown;
}

/** Minimal `MediaStreamTrack` (structurally compatible with the DOM and composable types). */
export interface WebcamMediaTrackLike extends EventTarget {
  /** `"audio"` or `"video"`. */
  readonly kind: string;

  /** `"live"` or `"ended"`. */
  readonly readyState: string;

  /** Stop the track and release its device. */
  stop(): void;
}

/** Minimal `MediaStream`, e.g. `useUserMedia().stream.value` or a native `MediaStream`. */
export interface WebcamMediaStreamLike {
  /** Every track of the stream. */
  getTracks(): readonly WebcamMediaTrackLike[];
}

/** Minimal `MediaDeviceInfo`. */
export interface WebcamMediaDeviceLike {
  /** Device identifier. */
  readonly deviceId: string;

  /** `"audioinput"`, `"audiooutput"`, or `"videoinput"`. */
  readonly kind: string;

  /** Human-readable label (empty until permission is granted). */
  readonly label: string;

  /** Identifier shared by devices of one physical unit. */
  readonly groupId: string;
}

/** Minimal `MediaDevices` used when the root acquires its own stream. */
export interface WebcamMediaHost extends EventTarget {
  /** Request camera access. */
  getUserMedia(constraints?: MediaStreamConstraints): Promise<WebcamMediaStreamLike>;

  /** List media devices. */
  enumerateDevices?(): Promise<readonly WebcamMediaDeviceLike[]>;
}

/** One selectable camera. */
export interface WebcamCaptureDevice {
  /** Device identifier. */
  readonly deviceId: string;

  /** Label, or a positional fallback from `messages.deviceFallback` before permission. */
  readonly label: string;

  /** Identifier shared by devices of one physical unit. */
  readonly groupId: string;
}

/** Options for {@link captureFrame}. */
export interface CaptureFrameOptions {
  /**
   * Flip the image horizontally, matching a mirrored selfie preview.
   *
   * @default false
   */
  readonly mirrored?: boolean | undefined;

  /**
   * Center-crop to this width / height ratio. `undefined` keeps the source ratio.
   *
   * @default undefined
   */
  readonly aspectRatio?: number | undefined;

  /**
   * Encoded image type.
   *
   * @default "image/png"
   */
  readonly type?: WebcamCaptureImageType | undefined;

  /**
   * Encoder quality for lossy types, from `0` to `1`.
   *
   * @default undefined
   */
  readonly quality?: number | undefined;

  /**
   * Downscale so the output is at most this many pixels wide.
   *
   * @default undefined
   */
  readonly maxWidth?: number | undefined;
}

/** Source and destination rectangles computed by {@link captureGeometry}. */
export interface CaptureGeometry {
  /** Source x offset in video pixels. */
  readonly sx: number;
  /** Source y offset in video pixels. */
  readonly sy: number;
  /** Source width in video pixels. */
  readonly sw: number;
  /** Source height in video pixels. */
  readonly sh: number;
  /** Output width in pixels. */
  readonly width: number;
  /** Output height in pixels. */
  readonly height: number;
}

/** One captured photo. */
export interface WebcamCapturePhotoResult {
  /** Encoded image. */
  readonly blob: Blob;

  /** Encoded image type actually produced by the browser. */
  readonly type: string;

  /** Output width in pixels. */
  readonly width: number;

  /** Output height in pixels. */
  readonly height: number;

  /** Whether the image was mirrored horizontally. */
  readonly mirrored: boolean;
}

/**
 * Localizable strings for every WebcamCapture part. Omitted entries use the
 * English defaults in `webcamCaptureDefaultMessages`.
 */
export interface WebcamCaptureMessages {
  /** Start button label. */
  readonly start: string;
  /** Stop button label. */
  readonly stop: string;
  /** Switch-camera button label. */
  readonly switchCamera: string;
  /** Device select accessible name. */
  readonly deviceSelect: string;
  /** Option label for a device without a label (before permission). */
  readonly deviceFallback: (position: number) => string;
  /** Shutter button label. */
  readonly shutter: string;
  /** Live preview accessible name. */
  readonly preview: string;
  /** Captured photo alternative text. */
  readonly photo: string;
  /** Countdown announcement. */
  readonly countdown: (seconds: number) => string;
  /** Announcement after a capture. */
  readonly captured: string;
  /** Status announcement while permission is requested. */
  readonly requesting: string;
  /** Status announcement once the camera is live. */
  readonly active: string;
  /** Status announcement for each failure. */
  readonly error: (code: WebcamCaptureErrorCode) => string;
}

/** State shared with every WebcamCapture slot. */
export interface WebcamCaptureSlotState {
  /** Stream lifecycle. */
  readonly status: WebcamCaptureStatus;

  /** Last acquisition failure, or `undefined`. */
  readonly error: WebcamCaptureFailure | undefined;

  /** Whether the stream is supplied by the consumer through the `stream` prop. */
  readonly external: boolean;

  /** Requested facing direction. */
  readonly facingMode: WebcamFacingMode;

  /** Selected device id, or `undefined` for the facing-mode default. */
  readonly deviceId: string | undefined;

  /** Known cameras (filled after permission is granted). */
  readonly devices: readonly WebcamCaptureDevice[];

  /** Whether the preview and captures are mirrored. */
  readonly mirrored: boolean;

  /** Seconds left in a running shutter countdown, or `0`. */
  readonly countdown: number;

  /** Whether a capture is being encoded. */
  readonly capturing: boolean;
}

/** Public instance exposed by WebcamCaptureRoot. */
export interface WebcamCaptureRootExpose extends WebcamCaptureSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Current stream (owned or external), or `undefined`. */
  readonly stream: WebcamMediaStreamLike | undefined;

  /** Last captured photo, or `null`. */
  readonly photo: WebcamCapturePhotoResult | null;

  /** Object URL of the last captured photo, owned and revoked by the root. */
  readonly photoUrl: string | undefined;

  /** Acquire the camera. Resolves with the stream, or `undefined` on failure or external mode. */
  readonly start: () => Promise<WebcamMediaStreamLike | undefined>;

  /** Stop and release an owned stream. External streams are never stopped. */
  readonly stop: () => void;

  /** Toggle between `user` and `environment`, restarting an active owned stream. */
  readonly switchCamera: () => Promise<void>;

  /** Select one camera, restarting an active owned stream. */
  readonly selectDevice: (deviceId: string | undefined) => Promise<void>;

  /** Capture one frame from the preview. Resolves with `null` when no frame is available. */
  readonly capture: () => Promise<WebcamCapturePhotoResult | null>;

  /** Forget the last photo and revoke its object URL. */
  readonly clearPhoto: () => void;
}

/** Public instance exposed by WebcamCaptureVideo. */
export interface WebcamCaptureVideoExpose {
  /** Rendered native video element. */
  readonly element: HTMLVideoElement | null;
}

/** Public instance exposed by WebcamCapture buttons. */
export interface WebcamCaptureButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Whether the button is disabled. */
  readonly disabled: boolean;
}

/** Public instance exposed by WebcamCaptureDeviceSelect. */
export interface WebcamCaptureDeviceSelectExpose {
  /** Rendered native select. */
  readonly element: HTMLSelectElement | null;
}

/** Public instance exposed by WebcamCapturePhoto. */
export interface WebcamCapturePhotoExpose {
  /** Rendered image, or `null` without a photo. */
  readonly element: HTMLImageElement | null;
}

/** Public instance exposed by WebcamCaptureStatusMessage. */
export interface WebcamCaptureStatusExpose {
  /** Rendered live region. */
  readonly element: HTMLDivElement | null;

  /** Current announcement text. */
  readonly message: string;
}
