import type {
  WebcamCaptureDevice,
  WebcamCaptureErrorCode,
  WebcamCaptureMessages,
  WebcamFacingMode,
  WebcamMediaDeviceLike,
  WebcamMediaStreamLike,
} from "./webcam-capture-types.ts";

/** English defaults for {@link WebcamCaptureMessages}. */
export const webcamCaptureDefaultMessages: WebcamCaptureMessages = Object.freeze({
  start: "Start camera",
  stop: "Stop camera",
  switchCamera: "Switch camera",
  deviceSelect: "Camera",
  deviceFallback: (position: number) => `Camera ${position}`,
  shutter: "Take photo",
  preview: "Camera preview",
  photo: "Captured photo",
  countdown: (seconds: number) => `Taking photo in ${seconds}`,
  captured: "Photo taken",
  requesting: "Requesting camera access",
  active: "Camera on",
  error: (code: WebcamCaptureErrorCode) => ERROR_TEXT[code],
});

const ERROR_TEXT: Readonly<Record<WebcamCaptureErrorCode, string>> = Object.freeze({
  aborted: "Camera request was interrupted",
  failed: "Camera could not be started",
  "invalid-constraints": "Camera settings are invalid",
  "not-found": "No camera was found",
  "not-readable": "Camera is in use by another application",
  overconstrained: "No camera matches the requested settings",
  "permission-denied": "Camera permission was denied",
  unsupported: "Camera is not supported in this browser",
});

/** Merge partial consumer messages over the English defaults. */
export function resolveWebcamCaptureMessages(
  messages: Partial<WebcamCaptureMessages> | undefined,
): WebcamCaptureMessages {
  return messages === undefined
    ? webcamCaptureDefaultMessages
    : Object.freeze({ ...webcamCaptureDefaultMessages, ...messages });
}

function errorName(error: unknown): string | undefined {
  return typeof error === "object" && error !== null && "name" in error
    ? String(error.name)
    : undefined;
}

/**
 * Classify a `getUserMedia` rejection exactly like `useUserMedia` from
 * `@vizejs/composable`, so both report the same codes.
 */
export function normalizeUserMediaError(error: unknown): WebcamCaptureErrorCode {
  switch (errorName(error)) {
    case "NotAllowedError":
    case "SecurityError":
      return "permission-denied";
    case "NotFoundError":
      return "not-found";
    case "NotReadableError":
      return "not-readable";
    case "OverconstrainedError":
      return "overconstrained";
    case "AbortError":
      return "aborted";
    case "TypeError":
      return "invalid-constraints";
    default:
      return "failed";
  }
}

/** Stop every track of a stream. */
export function stopWebcamStream(stream: WebcamMediaStreamLike): void {
  for (const track of stream.getTracks()) track.stop();
}

/** Options for {@link createWebcamConstraints}. */
export interface CreateWebcamConstraintsOptions {
  /** Extra video constraints, merged under the facing mode or device. */
  readonly video?: MediaTrackConstraints | undefined;
  /** Requested facing direction when no device is selected. */
  readonly facingMode: WebcamFacingMode;
  /** Exact device to open; wins over `facingMode`. */
  readonly deviceId?: string | undefined;
  /** Also request the microphone. */
  readonly audio?: boolean | undefined;
}

/** Build `getUserMedia` constraints for one camera request. */
export function createWebcamConstraints(
  options: CreateWebcamConstraintsOptions,
): MediaStreamConstraints {
  const video: MediaTrackConstraints =
    options.deviceId === undefined
      ? { ...options.video, facingMode: options.facingMode }
      : { ...options.video, deviceId: { exact: options.deviceId } };
  return { audio: options.audio === true, video };
}

/** Keep video inputs, with positional labels for devices listed before permission. */
export function toWebcamDevices(
  devices: readonly WebcamMediaDeviceLike[],
  fallback: (position: number) => string,
): readonly WebcamCaptureDevice[] {
  return Object.freeze(
    devices
      .filter((device) => device.kind === "videoinput")
      .map((device, index) =>
        Object.freeze({
          deviceId: device.deviceId,
          groupId: device.groupId,
          label: device.label.length > 0 ? device.label : fallback(index + 1),
        }),
      ),
  );
}
