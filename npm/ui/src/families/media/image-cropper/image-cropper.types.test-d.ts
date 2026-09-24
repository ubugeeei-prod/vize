/** Compile-only assertions for the public ImageCropper contract. */

import {
  ImageCropper,
  ImageCropperArea,
  ImageCropperError,
  ImageCropperGrid,
  ImageCropperHandle,
  ImageCropperImage,
  ImageCropperRoot,
  ImageCropperViewport,
  cropImage,
  resizeCrop,
  rotatedBounds,
  type CropArea,
  type CropImageType,
  type ImageCropperChangeReason,
  type ImageCropperErrorCode,
  type ImageCropperHandlePosition,
  type ImageCropperInteraction,
  type ImageCropperMessages,
  type ImageCropperPartExpose,
  type ImageCropperRootExpose,
  type ImageCropperSlotState,
} from "./image-cropper.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: ImageCropperRootExpose;
declare const area: ImageCropperPartExpose<HTMLDivElement>;
declare const error: ImageCropperError;
declare const image: HTMLImageElement;
declare const crop: CropArea;

type _CropIsExact = Expect<
  Equal<
    CropArea,
    { readonly x: number; readonly y: number; readonly width: number; readonly height: number }
  >
>;
type _HandleIsLiteral = Expect<
  Equal<ImageCropperHandlePosition, "e" | "n" | "ne" | "nw" | "s" | "se" | "sw" | "w">
>;
type _ReasonIsLiteral = Expect<
  Equal<
    ImageCropperChangeReason,
    "api" | "init" | "keyboard" | "move" | "resize" | "rotate" | "zoom"
  >
>;
type _InteractionIsLiteral = Expect<
  Equal<ImageCropperInteraction, "idle" | "moving" | "panning" | "resizing">
>;
type _TypeIsLiteral = Expect<Equal<CropImageType, "image/jpeg" | "image/png" | "image/webp">>;
type _ErrorCode = Expect<Equal<typeof error.code, ImageCropperErrorCode>>;
type _AreaMessageTakesCrop = Expect<
  Equal<Parameters<ImageCropperMessages["area"]>, [crop: CropArea]>
>;
type _SlotCropIsNullable = Expect<Equal<ImageCropperSlotState["crop"], CropArea | null>>;
type _RootElement = Expect<Equal<typeof root.element, HTMLDivElement | null>>;
type _AreaElement = Expect<Equal<typeof area.element, HTMLDivElement | null>>;
type _BlobExport = Expect<Equal<ReturnType<typeof root.toBlob>, Promise<Blob>>>;
type _DataUrlExport = Expect<Equal<ReturnType<typeof root.toDataUrl>, Promise<string>>>;
type _ResizeReturnsCrop = Expect<Equal<ReturnType<typeof resizeCrop>, CropArea>>;
type _BoundsHaveSize = Expect<
  Equal<ReturnType<typeof rotatedBounds>, { readonly width: number; readonly height: number }>
>;
type _AliasIsRoot = Expect<Equal<typeof ImageCropper, typeof ImageCropperRoot>>;

const blob: Promise<Blob> = cropImage(image, crop);
const url: Promise<string> = cropImage(image, crop, { output: "data-url" });
void blob;
void url;
void ImageCropperArea;
void ImageCropperGrid;
void ImageCropperHandle;
void ImageCropperImage;
void ImageCropperViewport;

// @ts-expect-error data-url output does not produce a Blob.
const _wrongOutput: Promise<Blob> = cropImage(image, crop, { output: "data-url" });
// @ts-expect-error handle positions are a closed union.
const _unknownHandle: ImageCropperHandlePosition = "center";
// @ts-expect-error crops are read-only.
crop.x = 1;
// @ts-expect-error rotation is fixed by the cropper for exports.
void root.toBlob({ rotation: 90 });
