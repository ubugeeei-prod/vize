/** Headless camera capture: preview, device switching, countdown shutter, and photos. */
export { default as WebcamCaptureDeviceSelect } from "./webcam-capture-device-select.vue";
export { default as WebcamCapturePhoto } from "./webcam-capture-photo.vue";
export { default as WebcamCapture, default as WebcamCaptureRoot } from "./webcam-capture-root.vue";
export { default as WebcamCaptureShutter } from "./webcam-capture-shutter.vue";
export { default as WebcamCaptureStartButton } from "./webcam-capture-start-button.vue";
export { default as WebcamCaptureStatusMessage } from "./webcam-capture-status-message.vue";
export { default as WebcamCaptureStopButton } from "./webcam-capture-stop-button.vue";
export { default as WebcamCaptureSwitchCamera } from "./webcam-capture-switch-camera.vue";
export { default as WebcamCaptureVideo } from "./webcam-capture-video.vue";
export { WebcamCaptureError, captureFrame, captureGeometry } from "./webcam-capture-frame.ts";
export {
  createWebcamConstraints,
  normalizeUserMediaError,
  resolveWebcamCaptureMessages,
  stopWebcamStream,
  toWebcamDevices,
  webcamCaptureDefaultMessages,
} from "./webcam-capture-media.ts";
export type { CreateWebcamConstraintsOptions } from "./webcam-capture-media.ts";
export type {
  CaptureFrameOptions,
  CaptureGeometry,
  WebcamCaptureButtonExpose,
  WebcamCaptureDevice,
  WebcamCaptureDeviceSelectExpose,
  WebcamCaptureErrorCode,
  WebcamCaptureFailure,
  WebcamCaptureImageType,
  WebcamCaptureMessages,
  WebcamCapturePhotoExpose,
  WebcamCapturePhotoResult,
  WebcamCaptureRootExpose,
  WebcamCaptureSlotState,
  WebcamCaptureStatus,
  WebcamCaptureStatusExpose,
  WebcamCaptureVideoExpose,
  WebcamFacingMode,
  WebcamMediaDeviceLike,
  WebcamMediaHost,
  WebcamMediaStreamLike,
  WebcamMediaTrackLike,
} from "./webcam-capture-types.ts";
