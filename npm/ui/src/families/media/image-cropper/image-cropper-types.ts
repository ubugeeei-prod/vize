/** A crop rectangle in rotated-image bounding-box pixels. */
export interface CropArea {
  /** Left edge. */
  readonly x: number;

  /** Top edge. */
  readonly y: number;

  /** Width. */
  readonly width: number;

  /** Height. */
  readonly height: number;
}

/** A width and height pair. */
export interface CropSize {
  readonly width: number;
  readonly height: number;
}

/** A point. */
export interface CropPoint {
  readonly x: number;
  readonly y: number;
}

/** Size and shape limits applied to every crop change. */
export interface CropConstraints {
  /**
   * Locked width / height ratio. `undefined` allows free resizing.
   *
   * @default undefined
   */
  readonly aspectRatio?: number;

  /**
   * Minimum crop width in image pixels.
   *
   * @default 1
   */
  readonly minWidth?: number;

  /**
   * Minimum crop height in image pixels.
   *
   * @default 1
   */
  readonly minHeight?: number;

  /**
   * Maximum crop width in image pixels. The image bounds always apply.
   *
   * @default undefined
   */
  readonly maxWidth?: number;

  /**
   * Maximum crop height in image pixels. The image bounds always apply.
   *
   * @default undefined
   */
  readonly maxHeight?: number;
}

/** Resize handle position: an edge or a corner, named by compass direction. */
export type ImageCropperHandlePosition = "e" | "n" | "ne" | "nw" | "s" | "se" | "sw" | "w";

/** Why the crop, zoom, or rotation changed. */
export type ImageCropperChangeReason =
  | "api"
  | "init"
  | "keyboard"
  | "move"
  | "resize"
  | "rotate"
  | "zoom";

/** Pointer or keyboard interaction in progress. */
export type ImageCropperInteraction = "idle" | "moving" | "panning" | "resizing";

/** Output encodings produced by {@link cropImage}. */
export type CropImageType = "image/jpeg" | "image/png" | "image/webp";

/** Stable diagnostic codes thrown by the crop helper. */
export type ImageCropperErrorCode =
  | "VIZE_UI_IMAGE_CROPPER_CANVAS_UNAVAILABLE"
  | "VIZE_UI_IMAGE_CROPPER_EMPTY_CROP"
  | "VIZE_UI_IMAGE_CROPPER_ENCODE_FAILED"
  | "VIZE_UI_IMAGE_CROPPER_IMAGE_LOAD";

/** Options for {@link cropImage}. */
export interface CropImageOptions {
  /**
   * Rotation in degrees that the crop area was measured against.
   *
   * @default 0
   */
  readonly rotation?: number;

  /**
   * Encoded image type.
   *
   * @default "image/png"
   */
  readonly type?: CropImageType;

  /**
   * Encoder quality for lossy types, from `0` to `1`.
   *
   * @default undefined
   */
  readonly quality?: number;

  /**
   * Downscale the output so it is at most this wide.
   *
   * @default undefined
   */
  readonly maxWidth?: number;

  /**
   * Downscale the output so it is at most this tall.
   *
   * @default undefined
   */
  readonly maxHeight?: number;

  /**
   * CORS mode used when `source` is a URL.
   *
   * @default "anonymous"
   */
  readonly crossOrigin?: "" | "anonymous" | "use-credentials";
}

/** Result encoding selected by {@link CropImageOptions}. */
export type CropImageOutput = "blob" | "data-url";

/** Localizable accessible text. Functions receive the rounded crop. */
export interface ImageCropperMessages {
  /** Role description of the crop area. */
  readonly areaRoleDescription: string;

  /** Accessible name of the crop area, describing its position and size. */
  readonly area: (crop: CropArea) => string;
}

/** State exposed to every ImageCropper slot. */
export interface ImageCropperSlotState {
  /** Current crop, or `null` until the image size is known. */
  readonly crop: CropArea | null;

  /** Current zoom factor (`1` fits the whole image). */
  readonly zoom: number;

  /** Current rotation in degrees, normalized to `[0, 360)`. */
  readonly rotation: number;

  /** Natural image size, or `null` until the image loads. */
  readonly naturalSize: CropSize | null;

  /** Whether image and viewport are measured and geometry is published. */
  readonly ready: boolean;

  /** Pointer or keyboard interaction in progress. */
  readonly interaction: ImageCropperInteraction;

  /** Whether every interaction is suppressed. */
  readonly disabled: boolean;
}

/** Public instance exposed by ImageCropperRoot. */
export interface ImageCropperRootExpose extends ImageCropperSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Request a crop (clamped to the constraints). Reports whether it changed. */
  readonly setCrop: (crop: CropArea) => boolean;

  /** Request a zoom factor (clamped). Reports whether it changed. */
  readonly setZoom: (zoom: number) => boolean;

  /** Request a rotation in degrees. Reports whether it changed. */
  readonly setRotation: (degrees: number) => boolean;

  /** Restore the centered default crop, zoom `1`, and rotation `0`. */
  readonly reset: () => void;

  /** Crop the loaded image into a Blob with the current crop and rotation (client-only). */
  readonly toBlob: (options?: Omit<CropImageOptions, "rotation">) => Promise<Blob>;

  /** Crop the loaded image into a data URL with the current crop and rotation (client-only). */
  readonly toDataUrl: (options?: Omit<CropImageOptions, "rotation">) => Promise<string>;
}

/** Public instance exposed by ImageCropper parts. */
export interface ImageCropperPartExpose<Element_> {
  /** Rendered element. */
  readonly element: Element_ | null;
}
