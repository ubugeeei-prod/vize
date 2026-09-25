/** Compile-only assertions for the public QrCode contract. */

import {
  QrCode,
  QrCodeEncodeError,
  createQrCodeByteSegment,
  createQrCodeEciSegment,
  createQrCodeNumericSegment,
  encodeQrCode,
  isQrCodeModuleDark,
  qrCodeCapacity,
  qrCodeToSvgPath,
  type QrCodeEncodeErrorCode,
  type QrCodeEncodeErrorLike,
  type QrCodeEncodeOptions,
  type QrCodeErrorCorrection,
  type QrCodeExpose,
  type QrCodeKanjiEncoder,
  type QrCodeMask,
  type QrCodeMatrix,
  type QrCodeMode,
  type QrCodeSegment,
  type QrCodeSegmentation,
  type QrCodeSegmentMode,
  type QrCodeSegmentSummary,
  type QrCodeSlotState,
  type QrCodeState,
  type QrCodeValue,
  type QrCodeVersion,
} from "./qr-code.ts";
import { qrCodeKanjiEncoder } from "./qr-code-kanji.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: QrCodeExpose;
declare const slot: QrCodeSlotState;
declare const error: QrCodeEncodeError;

const matrix = encodeQrCode("https://vizejs.dev", { errorCorrection: "H", mask: "auto" });

type _LevelIsClosed = Expect<Equal<QrCodeErrorCorrection, "L" | "M" | "Q" | "H">>;
type _MaskIsClosed = Expect<Equal<QrCodeMask, 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7>>;
type _ModeIsClosed = Expect<Equal<QrCodeMode, "numeric" | "alphanumeric" | "byte" | "kanji">>;
type _SegmentModeAddsEci = Expect<Equal<QrCodeSegmentMode, QrCodeMode | "eci">>;
type _SegmentationIsClosed = Expect<Equal<QrCodeSegmentation, "optimal" | "single">>;
type _SegmentIsReadonly = Expect<
  Equal<
    QrCodeSegment,
    {
      readonly mode: QrCodeSegmentMode;
      readonly count: number;
      readonly bits: readonly number[];
    }
  >
>;
type _KanjiEncoderShape = Expect<
  Equal<QrCodeKanjiEncoder["toShiftJis"], (codePoint: number) => number | undefined>
>;
type _KanjiTableIsAnEncoder = Expect<Equal<typeof qrCodeKanjiEncoder, QrCodeKanjiEncoder>>;
type _SegmentHelpersReturnSegments = Expect<
  Equal<ReturnType<typeof createQrCodeNumericSegment>, QrCodeSegment>
>;
type _MatrixSegments = Expect<Equal<QrCodeMatrix["segments"], readonly QrCodeSegmentSummary[]>>;
type _MatrixEci = Expect<Equal<QrCodeMatrix["eci"], number | null>>;
type _StateIsClosed = Expect<Equal<QrCodeState, "error" | "ready">>;
type _ValueIsTextBytesOrSegments = Expect<
  Equal<QrCodeValue, string | Uint8Array | readonly QrCodeSegment[]>
>;
type _VersionSpansStandard = Expect<Equal<Extract<QrCodeVersion, 1 | 40 | 41 | 0>, 1 | 40>>;
type _ErrorCodesAreClosed = Expect<
  Equal<
    QrCodeEncodeErrorCode,
    "VIZE_UI_QR_DATA_TOO_LONG" | "VIZE_UI_QR_INVALID_MODE" | "VIZE_UI_QR_INVALID_OPTION"
  >
>;
type _EncodeReturnsMatrix = Expect<Equal<typeof matrix, QrCodeMatrix>>;
type _MatrixModulesAreReadonly = Expect<Equal<QrCodeMatrix["modules"], readonly boolean[]>>;
type _MatrixModeIsNullable = Expect<Equal<QrCodeMatrix["mode"], QrCodeMode | "mixed" | null>>;
type _ErrorCodeIsTyped = Expect<Equal<typeof error.code, QrCodeEncodeErrorCode>>;
type _ErrorIsStructurallyExposed = Expect<
  typeof error extends QrCodeEncodeErrorLike ? true : false
>;
type _ExposeElementIsSvg = Expect<Equal<typeof exposed.element, SVGSVGElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slot,
    {
      readonly state: QrCodeState;
      readonly matrix: QrCodeMatrix | null;
      readonly error: QrCodeEncodeErrorLike | null;
      readonly dimension: number;
      readonly quietZone: number;
    }
  >
>;
type _PathIsString = Expect<Equal<ReturnType<typeof qrCodeToSvgPath>, string>>;
type _CapacityIsNumber = Expect<Equal<ReturnType<typeof qrCodeCapacity>, number>>;
type _ModuleLookupIsBoolean = Expect<Equal<ReturnType<typeof isQrCodeModuleDark>, boolean>>;

const options = {
  boostErrorCorrection: true,
  errorCorrection: "Q",
  mask: 4,
  maxVersion: 10,
  minVersion: 2,
  mode: "alphanumeric",
  segmentation: "single",
  kanji: qrCodeKanjiEncoder,
  eci: 26,
  utf8Eci: true,
  version: "auto",
} satisfies QrCodeEncodeOptions;

const segmentValue = encodeQrCode([
  createQrCodeEciSegment(26),
  createQrCodeNumericSegment("123"),
  createQrCodeByteSegment("é"),
]);

const componentProps: InstanceType<typeof QrCode>["$props"] = {
  background: "white",
  errorCorrection: "H",
  label: "Download the app",
  mask: 2,
  quietZone: 4,
  value: new Uint8Array([1, 2, 3]),
  version: 7,
  segmentation: "optimal",
  kanji: qrCodeKanjiEncoder,
  utf8Eci: true,
};

// @ts-expect-error versions stop at 40.
const tooLarge: QrCodeVersion = 41;

// @ts-expect-error masks are the eight standard patterns.
const badMask: QrCodeMask = 8;

// @ts-expect-error ECI is a segment kind, not a data mode.
const badMode: QrCodeMode = "eci";

// @ts-expect-error segmentation is optimal or single.
encodeQrCode("x", { segmentation: "greedy" });

// @ts-expect-error a kanji encoder maps code points, not strings.
encodeQrCode("x", { kanji: { toShiftJis: (character: string) => character.length } });

// @ts-expect-error error correction levels are L, M, Q, or H.
encodeQrCode("x", { errorCorrection: "X" });

// @ts-expect-error values are text, bytes, or segments.
encodeQrCode(42);

// @ts-expect-error the component requires a value.
const missingValue: InstanceType<typeof QrCode>["$props"] = { label: "Empty" };

void badMask;
void badMode;
void componentProps;
void missingValue;
void options;
void segmentValue;
void tooLarge;
