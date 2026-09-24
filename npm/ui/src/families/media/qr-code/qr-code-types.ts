/**
 * QR Code symbol version. Version `n` renders a `(17 + 4n)` module square,
 * from 21×21 (version 1) to 177×177 (version 40).
 */
export type QrCodeVersion =
  | 1
  | 2
  | 3
  | 4
  | 5
  | 6
  | 7
  | 8
  | 9
  | 10
  | 11
  | 12
  | 13
  | 14
  | 15
  | 16
  | 17
  | 18
  | 19
  | 20
  | 21
  | 22
  | 23
  | 24
  | 25
  | 26
  | 27
  | 28
  | 29
  | 30
  | 31
  | 32
  | 33
  | 34
  | 35
  | 36
  | 37
  | 38
  | 39
  | 40;

/**
 * Error correction level. Roughly recovers 7% (`L`), 15% (`M`), 25% (`Q`),
 * or 30% (`H`) of damaged or covered codewords.
 */
export type QrCodeErrorCorrection = "L" | "M" | "Q" | "H";

/** One of the eight standard data mask patterns. */
export type QrCodeMask = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7;

/** Data encoding mode. Kanji and ECI segments are intentionally not emitted. */
export type QrCodeMode = "numeric" | "alphanumeric" | "byte";

/** Data accepted by the encoder: text is UTF-8 encoded, bytes are used as-is. */
export type QrCodeValue = string | Uint8Array;

/** Stable diagnostic codes thrown by {@link QrCodeEncodeError}. */
export type QrCodeEncodeErrorCode =
  | "VIZE_UI_QR_DATA_TOO_LONG"
  | "VIZE_UI_QR_INVALID_MODE"
  | "VIZE_UI_QR_INVALID_OPTION";

/** Options for `encodeQrCode`. */
export interface QrCodeEncodeOptions {
  /**
   * Minimum error correction level.
   *
   * @default "M"
   */
  readonly errorCorrection?: QrCodeErrorCorrection;

  /**
   * Raise the error correction level while the data still fits the chosen version.
   *
   * @default false
   */
  readonly boostErrorCorrection?: boolean;

  /**
   * Fixed symbol version, or `"auto"` to select the smallest version in
   * `minVersion..maxVersion` that holds the data.
   *
   * @default "auto"
   */
  readonly version?: QrCodeVersion | "auto";

  /**
   * Smallest version considered by automatic selection.
   *
   * @default 1
   */
  readonly minVersion?: QrCodeVersion;

  /**
   * Largest version considered by automatic selection.
   *
   * @default 40
   */
  readonly maxVersion?: QrCodeVersion;

  /**
   * Fixed data mask, or `"auto"` to pick the mask with the lowest standard penalty score.
   *
   * @default "auto"
   */
  readonly mask?: QrCodeMask | "auto";

  /**
   * Forced encoding mode, or `"auto"` for the most compact single mode that
   * represents the whole value. Byte values always use byte mode.
   *
   * @default "auto"
   */
  readonly mode?: QrCodeMode | "auto";
}

/** Immutable encoded QR Code symbol, excluding the quiet zone. */
export interface QrCodeMatrix {
  /** Symbol version that was encoded. */
  readonly version: QrCodeVersion;

  /** Modules per side, `17 + 4 * version`. */
  readonly size: number;

  /** Final error correction level, after any boost. */
  readonly errorCorrection: QrCodeErrorCorrection;

  /** Data mask applied to the symbol. */
  readonly mask: QrCodeMask;

  /** Encoding mode used for the data segment, or `null` for an empty value. */
  readonly mode: QrCodeMode | null;

  /** Row-major module colors: `true` is dark, index is `y * size + x`. */
  readonly modules: readonly boolean[];
}

/** Options for `qrCodeToSvgPath`. */
export interface QrCodeSvgPathOptions {
  /**
   * Light modules added around the symbol, offsetting every path coordinate.
   *
   * @default 0
   */
  readonly quietZone?: number;
}

/** Render state exposed by the QrCode data contract. */
export type QrCodeState = "error" | "ready";

/** State exposed to QrCode slots. */
export interface QrCodeSlotState {
  /** Whether the value encoded successfully. */
  readonly state: QrCodeState;

  /** Encoded symbol, or `null` when encoding failed. */
  readonly matrix: QrCodeMatrix | null;

  /** Encoding diagnostic, or `null` when the value encoded successfully. */
  readonly error: QrCodeEncodeErrorLike | null;

  /** Rendered viewBox side length in modules, including the quiet zone. */
  readonly dimension: number;

  /** Quiet zone width in modules. */
  readonly quietZone: number;
}

/** Structural view of a {@link QrCodeEncodeError} exposed to slots and refs. */
export interface QrCodeEncodeErrorLike {
  /** Stable machine-readable diagnostic code. */
  readonly code: QrCodeEncodeErrorCode;

  /** Human-readable diagnostic message prefixed with the code. */
  readonly message: string;
}

/** Public instance exposed by QrCode. */
export interface QrCodeExpose extends QrCodeSlotState {
  /** Rendered `<svg>` element, or `null` while encoding fails. */
  readonly element: SVGSVGElement | null;
}
