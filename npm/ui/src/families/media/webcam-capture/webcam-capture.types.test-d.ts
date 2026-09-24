/** Compile-only assertions for the public WebcamCapture contract. */

import {
  WebcamCapture,
  WebcamCaptureRoot,
  captureFrame,
  normalizeUserMediaError,
  type CaptureFrameOptions,
  type WebcamCaptureErrorCode,
  type WebcamCaptureFailure,
  type WebcamCaptureMessages,
  type WebcamCapturePhotoResult,
  type WebcamCaptureRootExpose,
  type WebcamCaptureSlotState,
  type WebcamCaptureStatus,
  type WebcamFacingMode,
  type WebcamMediaHost,
  type WebcamMediaStreamLike,
} from "./webcam-capture.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: WebcamCaptureRootExpose;
declare const nativeStream: MediaStream;
declare const nativeDevices: MediaDevices;

type _StatusMatchesComposable = Expect<
  Equal<WebcamCaptureStatus, "active" | "error" | "idle" | "requesting">
>;
type _ErrorCodesMatchComposable = Expect<
  Equal<
    WebcamCaptureErrorCode,
    | "aborted"
    | "failed"
    | "invalid-constraints"
    | "not-found"
    | "not-readable"
    | "overconstrained"
    | "permission-denied"
    | "unsupported"
  >
>;
type _FacingIsLiteral = Expect<Equal<WebcamFacingMode, "environment" | "user">>;
type _NormalizeReturnsCode = Expect<
  Equal<ReturnType<typeof normalizeUserMediaError>, WebcamCaptureErrorCode>
>;
type _CaptureResolvesPhoto = Expect<
  Equal<ReturnType<typeof captureFrame>, Promise<WebcamCapturePhotoResult>>
>;
type _FailureShape = Expect<
  Equal<WebcamCaptureFailure, { readonly code: WebcamCaptureErrorCode; readonly cause: unknown }>
>;
type _RootCapture = Expect<
  Equal<typeof root.capture, () => Promise<WebcamCapturePhotoResult | null>>
>;
type _SlotStatus = Expect<Equal<WebcamCaptureSlotState["status"], WebcamCaptureStatus>>;
type _CountdownMessage = Expect<
  Equal<WebcamCaptureMessages["countdown"], (seconds: number) => string>
>;
type _AliasIsRoot = Expect<Equal<typeof WebcamCapture, typeof WebcamCaptureRoot>>;

// Native streams and devices are structurally accepted, e.g. from useUserMedia().
const _stream: WebcamMediaStreamLike = nativeStream;
const _host: WebcamMediaHost = nativeDevices;
const _options: CaptureFrameOptions = { mirrored: true, type: "image/webp", quality: 0.9 };

// @ts-expect-error unknown image types are rejected.
const _badType: CaptureFrameOptions = { type: "image/gif" };
// @ts-expect-error facing modes are a closed union.
const _badFacing: WebcamFacingMode = "left";
// @ts-expect-error exposed state is read-only.
root.status = "active";
