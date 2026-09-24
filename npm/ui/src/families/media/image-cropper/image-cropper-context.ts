import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  CropArea,
  CropPoint,
  CropSize,
  ImageCropperChangeReason,
  ImageCropperHandlePosition,
  ImageCropperInteraction,
  ImageCropperMessages,
  ImageCropperSlotState,
} from "./image-cropper-types.ts";

/** Shared state and actions for the ImageCropper compound parts. */
export interface ImageCropperContextValue {
  readonly id: ComputedRef<string>;
  readonly slotState: ComputedRef<ImageCropperSlotState>;
  readonly crop: ComputedRef<CropArea | null>;
  readonly bounds: ComputedRef<CropSize | null>;
  readonly naturalSize: Readonly<ShallowRef<CropSize | null>>;
  readonly viewportSize: Readonly<ShallowRef<CropSize>>;
  readonly scale: ComputedRef<number>;
  readonly center: ComputedRef<CropPoint>;
  readonly zoom: ComputedRef<number>;
  readonly rotation: ComputedRef<number>;
  readonly ready: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly messages: ComputedRef<ImageCropperMessages>;
  readonly setNaturalSize: (size: CropSize | null) => void;
  readonly setViewportSize: (size: CropSize) => void;
  readonly setInteraction: (interaction: ImageCropperInteraction) => void;
  readonly setCrop: (crop: CropArea, reason: ImageCropperChangeReason) => boolean;
  readonly moveFrom: (
    start: CropArea,
    dx: number,
    dy: number,
    reason: ImageCropperChangeReason,
  ) => boolean;
  readonly resizeFrom: (
    start: CropArea,
    handle: ImageCropperHandlePosition,
    dx: number,
    dy: number,
    reason: ImageCropperChangeReason,
  ) => boolean;
  readonly panTo: (center: CropPoint) => void;
  readonly zoomTo: (zoom: number, anchor?: CropPoint) => boolean;
  readonly zoomBy: (steps: number, anchor?: CropPoint) => boolean;
  readonly rotateBy: (steps: number) => boolean;
  readonly commit: () => void;
  readonly nudgeStep: ComputedRef<number>;
}

export const imageCropperContext = createContext<ImageCropperContextValue>("ImageCropper");
