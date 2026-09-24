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

/**
 * Data encoding mode. `kanji` packs Shift_JIS double-byte characters into 13
 * bits each and requires a {@link QrCodeKanjiEncoder}.
 */
export type QrCodeMode = "numeric" | "alphanumeric" | "byte" | "kanji";

/** Segment kind: a data mode, or an ECI header that switches the character set. */
export type QrCodeSegmentMode = QrCodeMode | "eci";

/**
 * One encoded segment. Create segments with the `createQrCode*Segment`
 * helpers and pass an array of them as the value for full manual control.
 */
export interface QrCodeSegment {
  /** Segment kind. */
  readonly mode: QrCodeSegmentMode;

  /** Character (or byte) count written to the count indicator; `0` for ECI. */
  readonly count: number;

  /** Encoded payload bits (0 or 1), excluding the mode indicator and count. */
  readonly bits: readonly number[];
}

/**
 * Data accepted by the encoder: text (segmented automatically, byte runs are
 * UTF-8), raw bytes (one byte segment), or explicit segments.
 */
export type QrCodeValue = string | Uint8Array | readonly QrCodeSegment[];

/**
 * How text is split into segments.
 *
 * - `optimal`: mixes numeric, alphanumeric, byte, and kanji segments to
 *   minimize the bit length for each version range.
 * - `single`: the most compact single mode that represents the whole text.
 */
export type QrCodeSegmentation = "optimal" | "single";

/**
 * Maps Unicode characters to Shift_JIS for Kanji mode. The full JIS X 0208
 * table ships separately as `qrCodeKanjiEncoder` so it is only bundled on demand.
 */
export interface QrCodeKanjiEncoder {
  /**
   * Shift_JIS double-byte code (`0x8140..0x9FFC` or `0xE040..0xEBBF`) for a
   * Unicode code point, or `undefined` when the character has none.
   */
  readonly toShiftJis: (codePoint: number) => number | undefined;
}

/** Mode and count of one encoded segment, reported on {@link QrCodeMatrix}. */
export interface QrCodeSegmentSummary {
  /** Segment kind. */
  readonly mode: QrCodeSegmentMode;

  /** Character or byte count; the ECI designator for ECI segments. */
  readonly count: number;
}

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
   * Forced single encoding mode for text, or `"auto"` to follow
   * {@link segmentation}. Byte values always use byte mode; explicit segment
   * arrays ignore this option.
   *
   * @default "auto"
   */
  readonly mode?: QrCodeMode | "auto";

  /**
   * How automatic mode splits text into segments.
   *
   * @default "optimal"
   */
  readonly segmentation?: QrCodeSegmentation;

  /**
   * Shift_JIS mapping that enables Kanji mode, typically `qrCodeKanjiEncoder`
   * from `qr-code-kanji.ts`. Without it, Japanese text is encoded as UTF-8 bytes.
   *
   * @default undefined
   */
  readonly kanji?: QrCodeKanjiEncoder | undefined;

  /**
   * Extended Channel Interpretation designator (`0..999999`) written before the
   * data, e.g. `26` for UTF-8 or `20` for Shift_JIS.
   *
   * @default undefined
   */
  readonly eci?: number | undefined;

  /**
   * Declare UTF-8 byte data with ECI 26 so strict scanners do not assume ISO-8859-1.
   * Must agree with {@link eci} when both are set.
   *
   * @default false
   */
  readonly utf8Eci?: boolean;
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

  /**
   * Mode shared by every data segment, `"mixed"` when data segments use
   * different modes, or `null` for an empty value.
   */
  readonly mode: QrCodeMode | "mixed" | null;

  /** Encoded segments in order, including any ECI header. */
  readonly segments: readonly QrCodeSegmentSummary[];

  /** ECI designator written before the data, or `null`. */
  readonly eci: number | null;

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
