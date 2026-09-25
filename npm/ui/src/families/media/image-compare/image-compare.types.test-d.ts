/** Compile-only assertions for the public ImageCompare contract. */

import {
  ImageCompare,
  ImageCompareAfter,
  ImageCompareBefore,
  ImageCompareHandle,
  ImageCompareLabel,
  ImageCompareRoot,
  imageComparePositionForKey,
  imageComparePositionFromPoint,
  type ImageCompareChangeSource,
  type ImageCompareDirection,
  type ImageCompareHandleExpose,
  type ImageCompareMessages,
  type ImageCompareMode,
  type ImageCompareOrientation,
  type ImageComparePartExpose,
  type ImageCompareRootExpose,
  type ImageCompareSide,
  type ImageCompareSlotState,
  type ImageCompareState,
} from "./image-compare.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: ImageCompareRootExpose;
declare const handle: ImageCompareHandleExpose;
declare const part: ImageComparePartExpose;

type _Orientation = Expect<Equal<ImageCompareOrientation, "horizontal" | "vertical">>;
type _Direction = Expect<Equal<ImageCompareDirection, "ltr" | "rtl">>;
type _Mode = Expect<Equal<ImageCompareMode, "drag" | "hover">>;
type _State = Expect<Equal<ImageCompareState, "disabled" | "dragging" | "idle">>;
type _Side = Expect<Equal<ImageCompareSide, "after" | "before">>;
type _Source = Expect<Equal<ImageCompareChangeSource, "api" | "keyboard" | "pointer">>;
type _SlotState = Expect<
  Equal<
    ImageCompareSlotState,
    {
      readonly position: number;
      readonly orientation: ImageCompareOrientation;
      readonly state: ImageCompareState;
    }
  >
>;
type _Messages = Expect<
  Equal<
    ImageCompareMessages,
    { readonly handleLabel?: string; readonly valueText?: (position: number) => string }
  >
>;
type _RootElement = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _SetPosition = Expect<Equal<typeof root.setPosition, (position: number) => boolean>>;
type _HandleElement = Expect<Equal<typeof handle.element, HTMLDivElement | null>>;
type _PartElement = Expect<Equal<typeof part.element, HTMLElement | null>>;
type _Alias = Expect<Equal<typeof ImageCompare, typeof ImageCompareRoot>>;
type _KeyResult = Expect<Equal<ReturnType<typeof imageComparePositionForKey>, number | null>>;
type _PointResult = Expect<Equal<ReturnType<typeof imageComparePositionFromPoint>, number | null>>;

void ImageCompareAfter;
void ImageCompareBefore;
void ImageCompareHandle;
void ImageCompareLabel;

// @ts-expect-error modes are a closed union.
const _badMode: ImageCompareMode = "click";
// @ts-expect-error value text formats numbers into strings.
const _badMessages: ImageCompareMessages = { valueText: (position: number) => position };
// @ts-expect-error exposed state is read-only.
root.position = 10;
