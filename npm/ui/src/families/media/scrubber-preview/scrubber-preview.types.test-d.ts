/** Compile-only assertions for the public ScrubberPreview contract. */

import {
  captureVideoFrame,
  parseThumbnailVtt,
  ScrubberPreview,
  ScrubberPreviewError,
  ScrubberPreviewRoot,
  ScrubberPreviewThumbnail,
  ScrubberPreviewTime,
  ScrubberPreviewTrack,
  spriteFrame,
  type ScrubberPreviewCue,
  type ScrubberPreviewErrorCode,
  type ScrubberPreviewFrame,
  type ScrubberPreviewKind,
  type ScrubberPreviewRootExpose,
  type ScrubberPreviewSprite,
  type ScrubberPreviewStatus,
  type ScrubberPreviewThumbnailSlotState,
} from "./scrubber-preview.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const video: HTMLVideoElement;
declare const root: ScrubberPreviewRootExpose;

type _KindIsLiteral = Expect<Equal<ScrubberPreviewKind, "capture" | "none" | "sprite" | "vtt">>;
type _StatusIsLiteral = Expect<
  Equal<ScrubberPreviewStatus, "error" | "idle" | "loading" | "ready">
>;
type _ErrorCodes = Expect<
  Equal<
    ScrubberPreviewErrorCode,
    | "VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED"
    | "VIZE_UI_SCRUBBER_PREVIEW_LOAD_FAILED"
    | "VIZE_UI_SCRUBBER_PREVIEW_TAINTED"
    | "VIZE_UI_SCRUBBER_PREVIEW_UNSUPPORTED"
  >
>;
const blob = captureVideoFrame(video, 1);
type _BlobByDefault = Expect<Equal<typeof blob, Promise<Blob>>>;
const dataUrl = captureVideoFrame(video, 1, { output: "data-url" });
type _DataUrlOutput = Expect<Equal<typeof dataUrl, Promise<string>>>;
type _Cues = Expect<Equal<ReturnType<typeof parseThumbnailVtt>, readonly ScrubberPreviewCue[]>>;
type _SpriteFrame = Expect<Equal<ReturnType<typeof spriteFrame>, ScrubberPreviewFrame>>;
type _SetTime = Expect<
  Equal<ScrubberPreviewRootExpose["setTime"], (time: number | null) => boolean>
>;
type _ThumbnailFrame = Expect<
  Equal<ScrubberPreviewThumbnailSlotState["frame"], ScrubberPreviewFrame | null>
>;
type _ErrorHasCode = Expect<Equal<ScrubberPreviewError["code"], ScrubberPreviewErrorCode>>;
type _AliasIsRoot = Expect<Equal<typeof ScrubberPreview, typeof ScrubberPreviewRoot>>;

void ScrubberPreviewThumbnail;
void ScrubberPreviewTime;
void ScrubberPreviewTrack;

// @ts-expect-error output is a closed union.
void captureVideoFrame(video, 1, { output: "canvas" });
// @ts-expect-error sprites need a frame grid.
const _badSprite: ScrubberPreviewSprite = { src: "a.jpg", interval: 1 };
// @ts-expect-error exposed state is read-only.
root.time = 3;
