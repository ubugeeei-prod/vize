/** Compile-only assertions for the public Image contract. */

import {
  ImageContent,
  ImageFallback,
  ImagePlaceholder,
  ImageRoot,
  resolveImageCandidates,
  type ImageContentExpose,
  type ImageFallbackExpose,
  type ImagePartSlotState,
  type ImagePlaceholderExpose,
  type ImageRootExpose,
  type ImageSlotState,
  type ImageSource,
  type ImageStatus,
  type ImageStatusChangeReason,
} from "./image.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: ImageRootExpose;
declare const content: ImageContentExpose;
declare const fallback: ImageFallbackExpose;
declare const placeholder: ImagePlaceholderExpose;

type _StatusIsLiteral = Expect<Equal<ImageStatus, "error" | "idle" | "loaded" | "loading">>;
type _ReasonIsLiteral = Expect<
  Equal<ImageStatusChangeReason, "error" | "load" | "reset" | "retry" | "source" | "visible">
>;
type _SourceAcceptsChains = Expect<Equal<ImageSource, string | readonly string[]>>;
type _PartSlotIsRootSlot = Expect<Equal<ImagePartSlotState, ImageSlotState>>;
type _SlotStateIsExact = Expect<
  Equal<
    ImageSlotState,
    {
      readonly status: ImageStatus;
      readonly src: string | undefined;
      readonly candidateIndex: number;
      readonly candidateCount: number;
    }
  >
>;
type _RetryReportsStart = Expect<Equal<typeof root.retry, () => boolean>>;
type _ContentElementIsImage = Expect<Equal<typeof content.element, HTMLImageElement | null>>;
type _FallbackElementIsSpan = Expect<Equal<typeof fallback.element, HTMLSpanElement | null>>;
type _PlaceholderVisibleIsBoolean = Expect<Equal<typeof placeholder.visible, boolean>>;
type _CandidatesAreReadonly = Expect<
  Equal<ReturnType<typeof resolveImageCandidates>, readonly string[]>
>;

void ImageRoot;
void ImageContent;
void ImageFallback;
void ImagePlaceholder;

// @ts-expect-error statuses are a closed union.
const _unknownStatus: ImageStatus = "decoding";
// @ts-expect-error candidate chains contain strings only.
const _numericSource: ImageSource = [1, 2];
// @ts-expect-error slot state is read-only.
root.status = "loaded";
