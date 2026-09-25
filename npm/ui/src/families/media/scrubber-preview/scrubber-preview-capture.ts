import type {
  CaptureVideoFrameOptions,
  ScrubberPreviewErrorCode,
} from "./scrubber-preview-types.ts";

const MESSAGES: Readonly<Record<ScrubberPreviewErrorCode, string>> = Object.freeze({
  VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED: "the frame could not be encoded",
  VIZE_UI_SCRUBBER_PREVIEW_LOAD_FAILED: "the video could not be loaded or seeked",
  VIZE_UI_SCRUBBER_PREVIEW_TAINTED: "the video is cross-origin without CORS; frames cannot be read",
  VIZE_UI_SCRUBBER_PREVIEW_UNSUPPORTED: "frame capture needs a DOM with 2D canvas support",
});

/** Typed failure from {@link captureVideoFrame}. */
export class ScrubberPreviewError extends Error {
  /** Stable failure code. */
  readonly code: ScrubberPreviewErrorCode;

  constructor(code: ScrubberPreviewErrorCode, options?: { readonly cause?: unknown }) {
    super(`${code}: ${MESSAGES[code]}`, options);
    this.name = "ScrubberPreviewError";
    this.code = code;
  }
}

function once(target: EventTarget, success: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      target.removeEventListener(success, onSuccess);
      target.removeEventListener("error", onError);
    };
    const onSuccess = () => {
      cleanup();
      resolve();
    };
    const onError = (event: Event) => {
      cleanup();
      reject(new ScrubberPreviewError("VIZE_UI_SCRUBBER_PREVIEW_LOAD_FAILED", { cause: event }));
    };
    target.addEventListener(success, onSuccess);
    target.addEventListener("error", onError);
  });
}

function encode(canvas: HTMLCanvasElement, type: string, quality: number): Promise<Blob> {
  return new Promise((resolve, reject) => {
    try {
      canvas.toBlob(
        (blob) => {
          if (blob === null)
            reject(new ScrubberPreviewError("VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED"));
          else resolve(blob);
        },
        type,
        quality,
      );
    } catch (cause) {
      reject(new ScrubberPreviewError("VIZE_UI_SCRUBBER_PREVIEW_TAINTED", { cause }));
    }
  });
}

/**
 * Seek a video to `time` (clamped to its duration), draw that frame to a
 * canvas, and encode it as a `Blob` (or a data URL with `output: "data-url"`). Waits for metadata when needed.
 * Client-only; cross-origin videos need `crossorigin` plus CORS headers.
 *
 * @throws {ScrubberPreviewError} `…_UNSUPPORTED` without a DOM or 2D canvas,
 * `…_LOAD_FAILED` when loading or seeking fails, `…_TAINTED` for cross-origin
 * frames, and `…_CAPTURE_FAILED` when encoding produces no image.
 */
export function captureVideoFrame(
  video: HTMLVideoElement,
  time: number,
  options: CaptureVideoFrameOptions & { readonly output: "data-url" },
): Promise<string>;
export function captureVideoFrame(
  video: HTMLVideoElement,
  time: number,
  options?: CaptureVideoFrameOptions & { readonly output?: "blob" },
): Promise<Blob>;
export async function captureVideoFrame(
  video: HTMLVideoElement,
  time: number,
  options: CaptureVideoFrameOptions & { readonly output?: "blob" | "data-url" } = {},
): Promise<Blob | string> {
  if (typeof document === "undefined") {
    throw new ScrubberPreviewError("VIZE_UI_SCRUBBER_PREVIEW_UNSUPPORTED");
  }
  if (video.readyState < 1) await once(video, "loadedmetadata");
  const duration = Number.isFinite(video.duration) ? video.duration : Number.POSITIVE_INFINITY;
  const target = Math.min(Math.max(0, time), duration);
  if (Math.abs(video.currentTime - target) > 0.001) {
    const seeked = once(video, "seeked");
    video.currentTime = target;
    await seeked;
  }
  const sourceWidth = video.videoWidth || options.width || 0;
  const sourceHeight = video.videoHeight || options.height || 0;
  const width = Math.max(1, Math.round(options.width ?? sourceWidth));
  const height = Math.max(
    1,
    Math.round(
      options.height ?? (sourceWidth > 0 ? (width * sourceHeight) / sourceWidth : sourceHeight),
    ),
  );
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d");
  if (context === null) throw new ScrubberPreviewError("VIZE_UI_SCRUBBER_PREVIEW_UNSUPPORTED");
  try {
    context.drawImage(video, 0, 0, width, height);
  } catch (cause) {
    throw new ScrubberPreviewError("VIZE_UI_SCRUBBER_PREVIEW_CAPTURE_FAILED", { cause });
  }
  const type = options.type ?? "image/jpeg";
  const quality = options.quality ?? 0.8;
  if (options.output !== "data-url") return encode(canvas, type, quality);
  try {
    return canvas.toDataURL(type, quality);
  } catch (cause) {
    throw new ScrubberPreviewError("VIZE_UI_SCRUBBER_PREVIEW_TAINTED", { cause });
  }
}
