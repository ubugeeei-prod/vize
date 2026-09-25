import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  WebcamCaptureMessages,
  WebcamCaptureSlotState,
  WebcamMediaStreamLike,
  WebcamCapturePhotoResult,
} from "./webcam-capture-types.ts";

/** Shared state and actions for the WebcamCapture compound parts. */
export interface WebcamCaptureContextValue {
  readonly id: ComputedRef<string>;
  readonly slotState: ComputedRef<WebcamCaptureSlotState>;
  readonly messages: ComputedRef<WebcamCaptureMessages>;
  readonly stream: ComputedRef<WebcamMediaStreamLike | undefined>;
  readonly photo: Readonly<ShallowRef<WebcamCapturePhotoResult | null>>;
  readonly photoUrl: Readonly<ShallowRef<string | undefined>>;
  readonly announcement: Readonly<ShallowRef<string>>;
  readonly setVideoElement: (element: HTMLVideoElement | null) => void;
  readonly start: () => Promise<WebcamMediaStreamLike | undefined>;
  readonly stop: () => void;
  readonly switchCamera: () => Promise<void>;
  readonly selectDevice: (deviceId: string | undefined) => Promise<void>;
  readonly shoot: (countdown: number) => Promise<WebcamCapturePhotoResult | null>;
}

export const webcamCaptureContext = createContext<WebcamCaptureContextValue>("WebcamCapture");
