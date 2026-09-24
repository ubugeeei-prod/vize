/** Compile-only assertions for the public SignaturePad contract. */

import {
  SignaturePad,
  SignaturePadCanvas,
  SignaturePadClear,
  SignaturePadError,
  SignaturePadGuide,
  SignaturePadRedo,
  SignaturePadRoot,
  SignaturePadUndo,
  parseSignature,
  signatureToDataUrl,
  strokeOutlinePath,
  type SignatureImageType,
  type SignaturePadButtonExpose,
  type SignaturePadCanvasExpose,
  type SignaturePadChangeReason,
  type SignaturePadErrorCode,
  type SignaturePadPressureMode,
  type SignaturePadRootExpose,
  type SignaturePadSlotState,
  type SignaturePadState,
  type SignaturePadValueFormat,
  type SignaturePoint,
  type SignatureValue,
} from "./signature-pad.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: SignaturePadRootExpose;
declare const canvas: SignaturePadCanvasExpose;
declare const button: SignaturePadButtonExpose;
declare const error: SignaturePadError;

type _PointIsExact = Expect<
  Equal<
    SignaturePoint,
    { readonly x: number; readonly y: number; readonly pressure: number; readonly time: number }
  >
>;
type _ValueIsReadonly = Expect<
  Equal<SignatureValue, readonly { readonly points: readonly SignaturePoint[] }[]>
>;
type _StateIsLiteral = Expect<Equal<SignaturePadState, "empty" | "filled">>;
type _FormatIsLiteral = Expect<Equal<SignaturePadValueFormat, "json" | "svg">>;
type _PressureIsLiteral = Expect<Equal<SignaturePadPressureMode, "auto" | "pointer" | "simulate">>;
type _ReasonIsLiteral = Expect<
  Equal<SignaturePadChangeReason, "api" | "clear" | "redo" | "stroke" | "undo">
>;
type _ImageTypeIsLiteral = Expect<
  Equal<SignatureImageType, "image/jpeg" | "image/png" | "image/svg+xml" | "image/webp">
>;
type _ErrorCode = Expect<Equal<typeof error.code, SignaturePadErrorCode>>;
type _SlotValue = Expect<Equal<SignaturePadSlotState["value"], SignatureValue>>;
type _RootElement = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _CanvasElement = Expect<Equal<typeof canvas.element, SVGSVGElement | null>>;
type _ButtonElement = Expect<Equal<typeof button.element, HTMLButtonElement | null>>;
type _DataUrlIsAsync = Expect<Equal<ReturnType<typeof signatureToDataUrl>, Promise<string>>>;
type _ExposeDataUrlIsAsync = Expect<Equal<ReturnType<typeof root.toDataUrl>, Promise<string>>>;
type _OutlineIsString = Expect<Equal<ReturnType<typeof strokeOutlinePath>, string>>;
type _ParseReturnsValue = Expect<Equal<ReturnType<typeof parseSignature>, SignatureValue>>;
type _AliasIsRoot = Expect<Equal<typeof SignaturePad, typeof SignaturePadRoot>>;

void SignaturePadCanvas;
void SignaturePadClear;
void SignaturePadGuide;
void SignaturePadRedo;
void SignaturePadUndo;

// @ts-expect-error value formats are a closed union.
const _unknownFormat: SignaturePadValueFormat = "png";
// @ts-expect-error points are read-only.
root.value[0]!.points[0]!.x = 1;
// @ts-expect-error raster types are a closed union.
void signatureToDataUrl([], { width: 1, height: 1, type: "image/gif" });
