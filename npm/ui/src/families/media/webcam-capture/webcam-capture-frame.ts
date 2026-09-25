import type {
  CaptureFrameOptions,
  CaptureGeometry,
  WebcamCapturePhotoResult,
} from "./webcam-capture-types.ts";

/** Stable diagnostic thrown by {@link captureFrame}. */
export class WebcamCaptureError extends Error {
  /** Diagnostic code. */
  readonly code: "VIZE_UI_WEBCAM_NO_CANVAS" | "VIZE_UI_WEBCAM_NO_FRAME" | "VIZE_UI_WEBCAM_ENCODE";

  constructor(code: WebcamCaptureError["code"], message: string) {
    super(`${code}: ${message}`);
    this.name = "WebcamCaptureError";
    this.code = code;
  }
}

/**
 * Compute the center crop and output size for one frame. Pure: a zero-sized
 * source yields a zero-sized geometry.
 */
export function captureGeometry(
  sourceWidth: number,
  sourceHeight: number,
  options: Pick<CaptureFrameOptions, "aspectRatio" | "maxWidth"> = {},
): CaptureGeometry {
  const width = Number.isFinite(sourceWidth) && sourceWidth > 0 ? sourceWidth : 0;
  const height = Number.isFinite(sourceHeight) && sourceHeight > 0 ? sourceHeight : 0;
  let sw = width;
  let sh = height;
  const ratio = options.aspectRatio;
  if (ratio !== undefined && Number.isFinite(ratio) && ratio > 0 && width > 0 && height > 0) {
    if (width / height > ratio) sw = Math.round(height * ratio);
    else sh = Math.round(width / ratio);
  }
  const sx = Math.round((width - sw) / 2);
  const sy = Math.round((height - sh) / 2);
  const maxWidth = options.maxWidth;
  const scale =
    maxWidth !== undefined && Number.isFinite(maxWidth) && maxWidth > 0 && sw > maxWidth
      ? maxWidth / sw
      : 1;
  return Object.freeze({
    sx,
    sy,
    sw,
    sh,
    width: Math.round(sw * scale),
    height: Math.round(sh * scale),
  });
}

/**
 * Draw the current video frame to a canvas and encode it.
 *
 * Client-only: throws `VIZE_UI_WEBCAM_NO_CANVAS` without a DOM canvas,
 * `VIZE_UI_WEBCAM_NO_FRAME` before the video has dimensions, and
 * `VIZE_UI_WEBCAM_ENCODE` when the browser cannot encode the image.
 */
export async function captureFrame(
  video: HTMLVideoElement,
  options: CaptureFrameOptions = {},
): Promise<WebcamCapturePhotoResult> {
  if (typeof document === "undefined") {
    throw new WebcamCaptureError("VIZE_UI_WEBCAM_NO_CANVAS", "capture requires a DOM canvas");
  }
  const geometry = captureGeometry(video.videoWidth, video.videoHeight, options);
  if (geometry.width === 0 || geometry.height === 0) {
    throw new WebcamCaptureError("VIZE_UI_WEBCAM_NO_FRAME", "the video has no frame yet");
  }
  const canvas = document.createElement("canvas");
  canvas.width = geometry.width;
  canvas.height = geometry.height;
  const context = canvas.getContext("2d");
  if (context === null) {
    throw new WebcamCaptureError("VIZE_UI_WEBCAM_NO_CANVAS", "2D canvas context is unavailable");
  }
  const mirrored = options.mirrored === true;
  if (mirrored) {
    context.translate(geometry.width, 0);
    context.scale(-1, 1);
  }
  context.drawImage(
    video,
    geometry.sx,
    geometry.sy,
    geometry.sw,
    geometry.sh,
    0,
    0,
    geometry.width,
    geometry.height,
  );
  const type = options.type ?? "image/png";
  const blob = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob(resolve, type, options.quality);
  });
  if (blob === null) {
    throw new WebcamCaptureError("VIZE_UI_WEBCAM_ENCODE", `could not encode ${type}`);
  }
  return Object.freeze({
    blob,
    height: geometry.height,
    mirrored,
    type: blob.type.length > 0 ? blob.type : type,
    width: geometry.width,
  });
}
